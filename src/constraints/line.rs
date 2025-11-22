//! Line-related constraints for geometric modeling
//!
//! Implements constraints that apply to Line entities, including length constraints
//! and future constraints like parallel/perpendicular relationships.

use crate::constraint::{Constraint, SketchQuery};
use crate::entity::LineId;
use crate::error::{Result, TextCadError};
use crate::units::Length;
use std::ops::{Add, Mul, Sub};
use z3::ast::{Ast, Real};

/// Constraint that sets the length of a line to a specific value
#[derive(Debug, Clone)]
pub struct LineLengthConstraint {
    /// Line to constrain
    pub line: LineId,
    /// Target length for the line
    pub length: Length,
}

impl LineLengthConstraint {
    /// Create a new line length constraint
    pub fn new(line: LineId, length: Length) -> Self {
        Self { line, length }
    }
}

impl Constraint for LineLengthConstraint {
    fn apply(
        &self,
        context: &z3::Context,
        solver: &z3::Solver,
        sketch: &dyn SketchQuery,
    ) -> Result<()> {
        // Get the line endpoints
        let (start_id, end_id) = sketch
            .line_endpoints(self.line)
            .map_err(|_| TextCadError::EntityError(format!("Line {:?} not found", self.line)))?;

        // Get the point coordinates for both endpoints
        let (x1, y1) = sketch.point_variables(start_id).map_err(|_| {
            TextCadError::EntityError(format!("Start point {:?} not found", start_id))
        })?;
        let (x2, y2) = sketch
            .point_variables(end_id)
            .map_err(|_| TextCadError::EntityError(format!("End point {:?} not found", end_id)))?;

        // Calculate distance squared: (x2-x1)² + (y2-y1)²
        let dx = (&x2).sub(&x1);
        let dy = (&y2).sub(&y1);
        let dist_sq = (&dx).mul(&dx).add(&(&dy).mul(&dy));

        // Convert target length to Z3 rational value
        // Use high precision by multiplying by 1_000_000
        let target_meters = self.length.to_meters();
        let target_sq = target_meters * target_meters;
        let target_rational = Real::from_real(context, (target_sq * 1_000_000.0) as i32, 1_000_000);

        // Assert that distance squared equals target squared
        solver.assert(&dist_sq._eq(&target_rational));

        Ok(())
    }

    fn description(&self) -> String {
        format!(
            "Line {:?} has length {:.3}m",
            self.line,
            self.length.to_meters()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::PointId;
    use crate::entity::LineId;
    use generational_arena::Index;
    use std::collections::HashMap;
    use z3::ast::Real;
    use z3::{Config, Context, Solver};

    // Mock implementation of SketchQuery for testing line constraints
    struct MockLineSketch<'ctx> {
        points: HashMap<PointId, (Real<'ctx>, Real<'ctx>)>,
        lines: HashMap<LineId, (PointId, PointId)>,
    }

    impl<'ctx> MockLineSketch<'ctx> {
        fn new() -> Self {
            Self {
                points: HashMap::new(),
                lines: HashMap::new(),
            }
        }

        fn add_point(&mut self, id: PointId, x: Real<'ctx>, y: Real<'ctx>) {
            self.points.insert(id, (x, y));
        }

        fn add_line(&mut self, line_id: LineId, start: PointId, end: PointId) {
            self.lines.insert(line_id, (start, end));
        }
    }

    impl<'ctx> SketchQuery for MockLineSketch<'ctx> {
        fn point_variables(&self, point_id: PointId) -> Result<(Real<'_>, Real<'_>)> {
            self.points
                .get(&point_id)
                .map(|(x, y)| (x.clone(), y.clone()))
                .ok_or_else(|| TextCadError::EntityError("Point not found".to_string()))
        }

        fn line_endpoints(&self, line_id: LineId) -> Result<(PointId, PointId)> {
            self.lines
                .get(&line_id)
                .copied()
                .ok_or_else(|| TextCadError::EntityError("Line not found".to_string()))
        }

        fn length_variable(&self, _name: &str) -> Result<Real<'_>> {
            Err(TextCadError::InvalidConstraint(
                "Not implemented".to_string(),
            ))
        }

        fn angle_variable(&self, _name: &str) -> Result<Real<'_>> {
            Err(TextCadError::InvalidConstraint(
                "Not implemented".to_string(),
            ))
        }
    }

    #[test]
    fn test_line_length_constraint_creation() {
        let line_id = LineId(Index::from_raw_parts(0, 0));
        let length = Length::meters(5.0);

        let constraint = LineLengthConstraint::new(line_id, length);

        assert_eq!(constraint.line, line_id);
        assert_eq!(constraint.length, length);
        assert!(constraint.description().contains("5.000m"));
    }

    #[test]
    fn test_line_length_constraint_apply() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        let p1 = PointId(Index::from_raw_parts(0, 0));
        let p2 = PointId(Index::from_raw_parts(1, 0));
        let line_id = LineId(Index::from_raw_parts(0, 0));

        let x1 = Real::new_const(&ctx, "x1");
        let y1 = Real::new_const(&ctx, "y1");
        let x2 = Real::new_const(&ctx, "x2");
        let y2 = Real::new_const(&ctx, "y2");

        let mut mock_sketch = MockLineSketch::new();
        mock_sketch.add_point(p1, x1, y1);
        mock_sketch.add_point(p2, x2, y2);
        mock_sketch.add_line(line_id, p1, p2);

        let constraint = LineLengthConstraint::new(line_id, Length::meters(3.0));

        // Apply the constraint
        constraint.apply(&ctx, &solver, &mock_sketch).unwrap();

        // Check that we have exactly 1 assertion (distance_squared = 9)
        assert_eq!(solver.get_assertions().len(), 1);
    }

    #[test]
    fn test_line_length_constraint_with_invalid_line() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        let line_id = LineId(Index::from_raw_parts(999, 999)); // Non-existent line

        let mock_sketch = MockLineSketch::new();
        let constraint = LineLengthConstraint::new(line_id, Length::meters(1.0));

        // Should fail because line doesn't exist
        let result = constraint.apply(&ctx, &solver, &mock_sketch);
        assert!(result.is_err());
        assert!(matches!(result, Err(TextCadError::EntityError(_))));
    }

    #[test]
    fn test_line_length_constraint_with_invalid_points() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        let p1 = PointId(Index::from_raw_parts(0, 0));
        let p2 = PointId(Index::from_raw_parts(999, 999)); // Non-existent point
        let line_id = LineId(Index::from_raw_parts(0, 0));

        let x1 = Real::new_const(&ctx, "x1");
        let y1 = Real::new_const(&ctx, "y1");

        let mut mock_sketch = MockLineSketch::new();
        mock_sketch.add_point(p1, x1, y1);
        // Not adding p2
        mock_sketch.add_line(line_id, p1, p2);

        let constraint = LineLengthConstraint::new(line_id, Length::meters(1.0));

        // Should fail because p2 doesn't exist
        let result = constraint.apply(&ctx, &solver, &mock_sketch);
        assert!(result.is_err());
        assert!(matches!(result, Err(TextCadError::EntityError(_))));
    }
}
