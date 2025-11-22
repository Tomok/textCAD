//! Basic geometric constraints for points
//!
//! Implements fundamental constraints for point positioning and coincidence.

use crate::constraint::{Constraint, SketchQuery};
use crate::entities::PointId;
use crate::error::{Result, TextCadError};
use crate::units::Length;
use z3::ast::{Ast, Real};

/// Constraint that makes two points coincident (same coordinates)
#[derive(Debug, Clone)]
pub struct CoincidentPointsConstraint {
    /// First point to make coincident
    pub point1: PointId,
    /// Second point to make coincident  
    pub point2: PointId,
}

impl CoincidentPointsConstraint {
    /// Create a new coincident points constraint
    pub fn new(point1: PointId, point2: PointId) -> Self {
        Self { point1, point2 }
    }
}

impl Constraint for CoincidentPointsConstraint {
    fn apply(
        &self,
        _context: &z3::Context,
        solver: &z3::Solver,
        sketch: &dyn SketchQuery,
    ) -> Result<()> {
        // Get the coordinates for both points
        let (x1, y1) = sketch.point_variables(self.point1)
            .map_err(|_| TextCadError::EntityError(format!("Point {:?} not found", self.point1)))?;
        let (x2, y2) = sketch.point_variables(self.point2)
            .map_err(|_| TextCadError::EntityError(format!("Point {:?} not found", self.point2)))?;

        // Assert that both coordinates are equal
        solver.assert(&x1._eq(&x2));
        solver.assert(&y1._eq(&y2));

        Ok(())
    }

    fn description(&self) -> String {
        format!("Points {:?} and {:?} are coincident", self.point1, self.point2)
    }
}

/// Constraint that fixes a point at specific coordinates
#[derive(Debug, Clone)]
pub struct FixedPositionConstraint {
    /// Point to fix in position
    pub point: PointId,
    /// X coordinate to fix the point at
    pub x: Length,
    /// Y coordinate to fix the point at
    pub y: Length,
}

impl FixedPositionConstraint {
    /// Create a new fixed position constraint
    pub fn new(point: PointId, x: Length, y: Length) -> Self {
        Self { point, x, y }
    }
}

impl Constraint for FixedPositionConstraint {
    fn apply(
        &self,
        context: &z3::Context,
        solver: &z3::Solver,
        sketch: &dyn SketchQuery,
    ) -> Result<()> {
        // Get the point's coordinate variables
        let (px, py) = sketch.point_variables(self.point)
            .map_err(|_| TextCadError::EntityError(format!("Point {:?} not found", self.point)))?;

        // Convert coordinates to Z3 rational values
        // Use high precision by multiplying by 1_000_000 and using as denominator
        let x_meters = self.x.to_meters();
        let y_meters = self.y.to_meters();
        
        // Convert to rational with high precision (6 decimal places)
        let x_val = Real::from_real(context, 
            (x_meters * 1_000_000.0) as i32, 
            1_000_000);
        let y_val = Real::from_real(context,
            (y_meters * 1_000_000.0) as i32,
            1_000_000);

        // Assert that the point coordinates equal the fixed values
        solver.assert(&px._eq(&x_val));
        solver.assert(&py._eq(&y_val));

        Ok(())
    }

    fn description(&self) -> String {
        format!(
            "Point {:?} is fixed at position ({:.3}m, {:.3}m)",
            self.point,
            self.x.to_meters(),
            self.y.to_meters()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::PointId;
    use generational_arena::Index;
    use std::collections::HashMap;
    use z3::{Config, Context, Solver};
    use z3::ast::Real;

    // Mock implementation of SketchQuery for testing
    struct MockSketch<'ctx> {
        points: HashMap<PointId, (Real<'ctx>, Real<'ctx>)>,
    }

    impl<'ctx> MockSketch<'ctx> {
        fn new() -> Self {
            Self {
                points: HashMap::new(),
            }
        }

        fn add_point(&mut self, id: PointId, x: Real<'ctx>, y: Real<'ctx>) {
            self.points.insert(id, (x, y));
        }
    }

    impl<'ctx> SketchQuery for MockSketch<'ctx> {
        fn point_variables(&self, point_id: PointId) -> Result<(Real<'_>, Real<'_>)> {
            self.points.get(&point_id)
                .map(|(x, y)| (x.clone(), y.clone()))
                .ok_or_else(|| TextCadError::EntityError("Point not found".to_string()))
        }

        fn length_variable(&self, _name: &str) -> Result<Real<'_>> {
            Err(TextCadError::InvalidConstraint("Not implemented".to_string()))
        }

        fn angle_variable(&self, _name: &str) -> Result<Real<'_>> {
            Err(TextCadError::InvalidConstraint("Not implemented".to_string()))
        }
    }

    #[test]
    fn test_coincident_points_constraint_creation() {
        let p1 = PointId(Index::from_raw_parts(0, 0));
        let p2 = PointId(Index::from_raw_parts(1, 0));
        
        let constraint = CoincidentPointsConstraint::new(p1, p2);
        
        assert_eq!(constraint.point1, p1);
        assert_eq!(constraint.point2, p2);
        assert!(constraint.description().contains("coincident"));
    }

    #[test]
    fn test_fixed_position_constraint_creation() {
        let p = PointId(Index::from_raw_parts(0, 0));
        let x = Length::meters(1.0);
        let y = Length::meters(2.0);
        
        let constraint = FixedPositionConstraint::new(p, x, y);
        
        assert_eq!(constraint.point, p);
        assert_eq!(constraint.x, x);
        assert_eq!(constraint.y, y);
        assert!(constraint.description().contains("fixed"));
        assert!(constraint.description().contains("1.000m"));
        assert!(constraint.description().contains("2.000m"));
    }

    #[test]
    fn test_coincident_points_constraint_apply() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        let p1 = PointId(Index::from_raw_parts(0, 0));
        let p2 = PointId(Index::from_raw_parts(1, 0));

        let x1 = Real::new_const(&ctx, "x1");
        let y1 = Real::new_const(&ctx, "y1");
        let x2 = Real::new_const(&ctx, "x2");
        let y2 = Real::new_const(&ctx, "y2");

        let mut mock_sketch = MockSketch::new();
        mock_sketch.add_point(p1, x1, y1);
        mock_sketch.add_point(p2, x2, y2);

        let constraint = CoincidentPointsConstraint::new(p1, p2);
        
        // Apply the constraint
        constraint.apply(&ctx, &solver, &mock_sketch).unwrap();

        // Check that we have 2 assertions (x1 = x2 and y1 = y2)
        assert_eq!(solver.get_assertions().len(), 2);
    }

    #[test]
    fn test_fixed_position_constraint_apply() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        let p = PointId(Index::from_raw_parts(0, 0));
        let x = Real::new_const(&ctx, "x");
        let y = Real::new_const(&ctx, "y");

        let mut mock_sketch = MockSketch::new();
        mock_sketch.add_point(p, x, y);

        let constraint = FixedPositionConstraint::new(
            p,
            Length::meters(3.0),
            Length::meters(4.0),
        );

        // Apply the constraint
        constraint.apply(&ctx, &solver, &mock_sketch).unwrap();

        // Check that we have 2 assertions (x = 3.0 and y = 4.0)
        assert_eq!(solver.get_assertions().len(), 2);
    }

    #[test]
    fn test_constraint_with_invalid_point() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        let p1 = PointId(Index::from_raw_parts(0, 0));
        let p2 = PointId(Index::from_raw_parts(999, 999)); // Non-existent point

        let x1 = Real::new_const(&ctx, "x1");
        let y1 = Real::new_const(&ctx, "y1");

        let mut mock_sketch = MockSketch::new();
        mock_sketch.add_point(p1, x1, y1);

        let constraint = CoincidentPointsConstraint::new(p1, p2);
        
        // Should fail because p2 doesn't exist
        let result = constraint.apply(&ctx, &solver, &mock_sketch);
        assert!(result.is_err());
        assert!(matches!(result, Err(TextCadError::EntityError(_))));
    }
}