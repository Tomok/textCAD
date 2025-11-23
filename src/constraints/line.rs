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

    #[test]
    fn test_parallel_lines_constraint_creation() {
        let line1_id = LineId(Index::from_raw_parts(0, 0));
        let line2_id = LineId(Index::from_raw_parts(1, 0));

        let constraint = ParallelLinesConstraint::new(line1_id, line2_id);

        assert_eq!(constraint.line1, line1_id);
        assert_eq!(constraint.line2, line2_id);
        assert!(constraint.description().contains("parallel"));
    }

    #[test]
    fn test_parallel_lines_constraint_apply() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        // Create two lines with 4 points
        let p1 = PointId(Index::from_raw_parts(0, 0));
        let p2 = PointId(Index::from_raw_parts(1, 0));
        let p3 = PointId(Index::from_raw_parts(2, 0));
        let p4 = PointId(Index::from_raw_parts(3, 0));
        let line1_id = LineId(Index::from_raw_parts(0, 0));
        let line2_id = LineId(Index::from_raw_parts(1, 0));

        // Create Z3 variables for each point
        let x1 = Real::new_const(&ctx, "x1");
        let y1 = Real::new_const(&ctx, "y1");
        let x2 = Real::new_const(&ctx, "x2");
        let y2 = Real::new_const(&ctx, "y2");
        let x3 = Real::new_const(&ctx, "x3");
        let y3 = Real::new_const(&ctx, "y3");
        let x4 = Real::new_const(&ctx, "x4");
        let y4 = Real::new_const(&ctx, "y4");

        let mut mock_sketch = MockLineSketch::new();
        mock_sketch.add_point(p1, x1, y1);
        mock_sketch.add_point(p2, x2, y2);
        mock_sketch.add_point(p3, x3, y3);
        mock_sketch.add_point(p4, x4, y4);
        mock_sketch.add_line(line1_id, p1, p2);
        mock_sketch.add_line(line2_id, p3, p4);

        let constraint = ParallelLinesConstraint::new(line1_id, line2_id);

        // Apply the constraint
        constraint.apply(&ctx, &solver, &mock_sketch).unwrap();

        // Check that we have exactly 1 assertion (cross product = 0)
        assert_eq!(solver.get_assertions().len(), 1);
    }

    #[test]
    fn test_perpendicular_lines_constraint_creation() {
        let line1_id = LineId(Index::from_raw_parts(0, 0));
        let line2_id = LineId(Index::from_raw_parts(1, 0));

        let constraint = PerpendicularLinesConstraint::new(line1_id, line2_id);

        assert_eq!(constraint.line1, line1_id);
        assert_eq!(constraint.line2, line2_id);
        assert!(constraint.description().contains("perpendicular"));
    }

    #[test]
    fn test_perpendicular_lines_constraint_apply() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        // Create two lines with 4 points
        let p1 = PointId(Index::from_raw_parts(0, 0));
        let p2 = PointId(Index::from_raw_parts(1, 0));
        let p3 = PointId(Index::from_raw_parts(2, 0));
        let p4 = PointId(Index::from_raw_parts(3, 0));
        let line1_id = LineId(Index::from_raw_parts(0, 0));
        let line2_id = LineId(Index::from_raw_parts(1, 0));

        // Create Z3 variables for each point
        let x1 = Real::new_const(&ctx, "x1");
        let y1 = Real::new_const(&ctx, "y1");
        let x2 = Real::new_const(&ctx, "x2");
        let y2 = Real::new_const(&ctx, "y2");
        let x3 = Real::new_const(&ctx, "x3");
        let y3 = Real::new_const(&ctx, "y3");
        let x4 = Real::new_const(&ctx, "x4");
        let y4 = Real::new_const(&ctx, "y4");

        let mut mock_sketch = MockLineSketch::new();
        mock_sketch.add_point(p1, x1, y1);
        mock_sketch.add_point(p2, x2, y2);
        mock_sketch.add_point(p3, x3, y3);
        mock_sketch.add_point(p4, x4, y4);
        mock_sketch.add_line(line1_id, p1, p2);
        mock_sketch.add_line(line2_id, p3, p4);

        let constraint = PerpendicularLinesConstraint::new(line1_id, line2_id);

        // Apply the constraint
        constraint.apply(&ctx, &solver, &mock_sketch).unwrap();

        // Check that we have exactly 1 assertion (dot product = 0)
        assert_eq!(solver.get_assertions().len(), 1);
    }

    #[test]
    fn test_parallel_lines_constraint_with_invalid_line() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        let line1_id = LineId(Index::from_raw_parts(0, 0));
        let line2_id = LineId(Index::from_raw_parts(999, 999)); // Non-existent line

        let mock_sketch = MockLineSketch::new();
        let constraint = ParallelLinesConstraint::new(line1_id, line2_id);

        // Should fail because line2 doesn't exist
        let result = constraint.apply(&ctx, &solver, &mock_sketch);
        assert!(result.is_err());
        assert!(matches!(result, Err(TextCadError::EntityError(_))));
    }

    #[test]
    fn test_perpendicular_lines_constraint_with_invalid_line() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        let line1_id = LineId(Index::from_raw_parts(0, 0));
        let line2_id = LineId(Index::from_raw_parts(999, 999)); // Non-existent line

        let mock_sketch = MockLineSketch::new();
        let constraint = PerpendicularLinesConstraint::new(line1_id, line2_id);

        // Should fail because line2 doesn't exist
        let result = constraint.apply(&ctx, &solver, &mock_sketch);
        assert!(result.is_err());
        assert!(matches!(result, Err(TextCadError::EntityError(_))));
    }

    #[test]
    fn test_parallel_lines_constraint_with_missing_point() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        let p1 = PointId(Index::from_raw_parts(0, 0));
        let p2 = PointId(Index::from_raw_parts(1, 0));
        let p3 = PointId(Index::from_raw_parts(2, 0));
        let p4 = PointId(Index::from_raw_parts(999, 999)); // Non-existent point
        let line1_id = LineId(Index::from_raw_parts(0, 0));
        let line2_id = LineId(Index::from_raw_parts(1, 0));

        let x1 = Real::new_const(&ctx, "x1");
        let y1 = Real::new_const(&ctx, "y1");
        let x2 = Real::new_const(&ctx, "x2");
        let y2 = Real::new_const(&ctx, "y2");
        let x3 = Real::new_const(&ctx, "x3");
        let y3 = Real::new_const(&ctx, "y3");

        let mut mock_sketch = MockLineSketch::new();
        mock_sketch.add_point(p1, x1, y1);
        mock_sketch.add_point(p2, x2, y2);
        mock_sketch.add_point(p3, x3, y3);
        // Not adding p4
        mock_sketch.add_line(line1_id, p1, p2);
        mock_sketch.add_line(line2_id, p3, p4); // p4 doesn't exist

        let constraint = ParallelLinesConstraint::new(line1_id, line2_id);

        let result = constraint.apply(&ctx, &solver, &mock_sketch);
        assert!(result.is_err());
        assert!(matches!(result, Err(TextCadError::EntityError(_))));
    }

    #[test]
    fn test_perpendicular_lines_constraint_with_missing_point() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        let p1 = PointId(Index::from_raw_parts(0, 0));
        let p2 = PointId(Index::from_raw_parts(1, 0));
        let p3 = PointId(Index::from_raw_parts(2, 0));
        let p4 = PointId(Index::from_raw_parts(999, 999)); // Non-existent point
        let line1_id = LineId(Index::from_raw_parts(0, 0));
        let line2_id = LineId(Index::from_raw_parts(1, 0));

        let x1 = Real::new_const(&ctx, "x1");
        let y1 = Real::new_const(&ctx, "y1");
        let x2 = Real::new_const(&ctx, "x2");
        let y2 = Real::new_const(&ctx, "y2");
        let x3 = Real::new_const(&ctx, "x3");
        let y3 = Real::new_const(&ctx, "y3");

        let mut mock_sketch = MockLineSketch::new();
        mock_sketch.add_point(p1, x1, y1);
        mock_sketch.add_point(p2, x2, y2);
        mock_sketch.add_point(p3, x3, y3);
        // Not adding p4
        mock_sketch.add_line(line1_id, p1, p2);
        mock_sketch.add_line(line2_id, p3, p4); // p4 doesn't exist

        let constraint = PerpendicularLinesConstraint::new(line1_id, line2_id);

        let result = constraint.apply(&ctx, &solver, &mock_sketch);
        assert!(result.is_err());
        assert!(matches!(result, Err(TextCadError::EntityError(_))));
    }

    #[test]
    fn test_parallel_lines_constraint_description() {
        let line1_id = LineId(Index::from_raw_parts(0, 0));
        let line2_id = LineId(Index::from_raw_parts(1, 0));
        let constraint = ParallelLinesConstraint::new(line1_id, line2_id);

        let description = constraint.description();
        assert!(description.contains("parallel"));
        assert!(description.contains("LineId"));
    }

    #[test]
    fn test_perpendicular_lines_constraint_description() {
        let line1_id = LineId(Index::from_raw_parts(0, 0));
        let line2_id = LineId(Index::from_raw_parts(1, 0));
        let constraint = PerpendicularLinesConstraint::new(line1_id, line2_id);

        let description = constraint.description();
        assert!(description.contains("perpendicular"));
        assert!(description.contains("LineId"));
    }

    #[test]
    fn test_parallel_lines_constraint_clone() {
        let line1_id = LineId(Index::from_raw_parts(0, 0));
        let line2_id = LineId(Index::from_raw_parts(1, 0));
        let constraint1 = ParallelLinesConstraint::new(line1_id, line2_id);
        let constraint2 = constraint1.clone();

        assert_eq!(constraint1.line1, constraint2.line1);
        assert_eq!(constraint1.line2, constraint2.line2);
        assert_eq!(constraint1.description(), constraint2.description());
    }

    #[test]
    fn test_perpendicular_lines_constraint_clone() {
        let line1_id = LineId(Index::from_raw_parts(0, 0));
        let line2_id = LineId(Index::from_raw_parts(1, 0));
        let constraint1 = PerpendicularLinesConstraint::new(line1_id, line2_id);
        let constraint2 = constraint1.clone();

        assert_eq!(constraint1.line1, constraint2.line1);
        assert_eq!(constraint1.line2, constraint2.line2);
        assert_eq!(constraint1.description(), constraint2.description());
    }

    #[test]
    fn test_constraint_debug_format() {
        let line1_id = LineId(Index::from_raw_parts(0, 0));
        let line2_id = LineId(Index::from_raw_parts(1, 0));

        let parallel_constraint = ParallelLinesConstraint::new(line1_id, line2_id);
        let perpendicular_constraint = PerpendicularLinesConstraint::new(line1_id, line2_id);

        // Test that Debug format works (doesn't panic)
        let _parallel_debug = format!("{:?}", parallel_constraint);
        let _perpendicular_debug = format!("{:?}", perpendicular_constraint);
    }

    #[test]
    fn test_constraint_same_line_ids() {
        // Test creating constraints where both lines are the same (edge case)
        let line_id = LineId(Index::from_raw_parts(0, 0));

        let parallel_constraint = ParallelLinesConstraint::new(line_id, line_id);
        let perpendicular_constraint = PerpendicularLinesConstraint::new(line_id, line_id);

        assert_eq!(parallel_constraint.line1, parallel_constraint.line2);
        assert_eq!(
            perpendicular_constraint.line1,
            perpendicular_constraint.line2
        );
    }
}

/// Constraint that forces two lines to be parallel
///
/// Uses the cross product method: two lines are parallel if their direction vectors
/// have a cross product of zero (u1 × u2 = 0, where u1·u2x - u1y·u2x = 0).
#[derive(Debug, Clone)]
pub struct ParallelLinesConstraint {
    /// First line to constrain
    pub line1: LineId,
    /// Second line to constrain  
    pub line2: LineId,
}

impl ParallelLinesConstraint {
    /// Create a new parallel lines constraint
    pub fn new(line1: LineId, line2: LineId) -> Self {
        Self { line1, line2 }
    }
}

impl Constraint for ParallelLinesConstraint {
    fn apply(
        &self,
        context: &z3::Context,
        solver: &z3::Solver,
        sketch: &dyn SketchQuery,
    ) -> Result<()> {
        // Get both line endpoints
        let (start1, end1) = sketch
            .line_endpoints(self.line1)
            .map_err(|_| TextCadError::EntityError(format!("Line {:?} not found", self.line1)))?;
        let (start2, end2) = sketch
            .line_endpoints(self.line2)
            .map_err(|_| TextCadError::EntityError(format!("Line {:?} not found", self.line2)))?;

        // Get point coordinates for line1
        let (x1_start, y1_start) = sketch.point_variables(start1).map_err(|_| {
            TextCadError::EntityError(format!("Start point {:?} of line1 not found", start1))
        })?;
        let (x1_end, y1_end) = sketch.point_variables(end1).map_err(|_| {
            TextCadError::EntityError(format!("End point {:?} of line1 not found", end1))
        })?;

        // Get point coordinates for line2
        let (x2_start, y2_start) = sketch.point_variables(start2).map_err(|_| {
            TextCadError::EntityError(format!("Start point {:?} of line2 not found", start2))
        })?;
        let (x2_end, y2_end) = sketch.point_variables(end2).map_err(|_| {
            TextCadError::EntityError(format!("End point {:?} of line2 not found", end2))
        })?;

        // Calculate direction vectors
        // v1 = (dx1, dy1) = (x1_end - x1_start, y1_end - y1_start)
        let dx1 = (&x1_end).sub(&x1_start);
        let dy1 = (&y1_end).sub(&y1_start);

        // v2 = (dx2, dy2) = (x2_end - x2_start, y2_end - y2_start)
        let dx2 = (&x2_end).sub(&x2_start);
        let dy2 = (&y2_end).sub(&y2_start);

        // For parallel lines: v1 × v2 = 0
        // Cross product in 2D: dx1 * dy2 - dy1 * dx2 = 0
        let cross_product = (&dx1).mul(&dy2).sub(&(&dy1).mul(&dx2));

        // Zero for comparison
        let zero = Real::from_real(context, 0, 1);

        // Assert that cross product equals zero
        solver.assert(&cross_product._eq(&zero));

        Ok(())
    }

    fn description(&self) -> String {
        format!("Lines {:?} and {:?} are parallel", self.line1, self.line2)
    }
}

/// Constraint that forces two lines to be perpendicular
///
/// Uses the dot product method: two lines are perpendicular if their direction vectors
/// have a dot product of zero (u1 · u2 = 0, where u1x·u2x + u1y·u2y = 0).
#[derive(Debug, Clone)]
pub struct PerpendicularLinesConstraint {
    /// First line to constrain
    pub line1: LineId,
    /// Second line to constrain
    pub line2: LineId,
}

impl PerpendicularLinesConstraint {
    /// Create a new perpendicular lines constraint
    pub fn new(line1: LineId, line2: LineId) -> Self {
        Self { line1, line2 }
    }
}

impl Constraint for PerpendicularLinesConstraint {
    fn apply(
        &self,
        context: &z3::Context,
        solver: &z3::Solver,
        sketch: &dyn SketchQuery,
    ) -> Result<()> {
        // Get both line endpoints
        let (start1, end1) = sketch
            .line_endpoints(self.line1)
            .map_err(|_| TextCadError::EntityError(format!("Line {:?} not found", self.line1)))?;
        let (start2, end2) = sketch
            .line_endpoints(self.line2)
            .map_err(|_| TextCadError::EntityError(format!("Line {:?} not found", self.line2)))?;

        // Get point coordinates for line1
        let (x1_start, y1_start) = sketch.point_variables(start1).map_err(|_| {
            TextCadError::EntityError(format!("Start point {:?} of line1 not found", start1))
        })?;
        let (x1_end, y1_end) = sketch.point_variables(end1).map_err(|_| {
            TextCadError::EntityError(format!("End point {:?} of line1 not found", end1))
        })?;

        // Get point coordinates for line2
        let (x2_start, y2_start) = sketch.point_variables(start2).map_err(|_| {
            TextCadError::EntityError(format!("Start point {:?} of line2 not found", start2))
        })?;
        let (x2_end, y2_end) = sketch.point_variables(end2).map_err(|_| {
            TextCadError::EntityError(format!("End point {:?} of line2 not found", end2))
        })?;

        // Calculate direction vectors
        // v1 = (dx1, dy1) = (x1_end - x1_start, y1_end - y1_start)
        let dx1 = (&x1_end).sub(&x1_start);
        let dy1 = (&y1_end).sub(&y1_start);

        // v2 = (dx2, dy2) = (x2_end - x2_start, y2_end - y2_start)
        let dx2 = (&x2_end).sub(&x2_start);
        let dy2 = (&y2_end).sub(&y2_start);

        // For perpendicular lines: v1 · v2 = 0
        // Dot product in 2D: dx1 * dx2 + dy1 * dy2 = 0
        let dot_product = (&dx1).mul(&dx2).add(&(&dy1).mul(&dy2));

        // Zero for comparison
        let zero = Real::from_real(context, 0, 1);

        // Assert that dot product equals zero
        solver.assert(&dot_product._eq(&zero));

        Ok(())
    }

    fn description(&self) -> String {
        format!(
            "Lines {:?} and {:?} are perpendicular",
            self.line1, self.line2
        )
    }
}
