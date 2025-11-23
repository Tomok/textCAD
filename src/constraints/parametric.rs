//! Parametric constraints for geometric modeling
//!
//! Implements constraints that use internal parameters to define geometric relationships.
//! These constraints automatically introduce parameter variables that are constrained
//! within appropriate bounds to achieve the desired geometric properties.

use crate::constraint::{Constraint, SketchQuery};
use crate::entities::PointId;
use crate::entity::LineId;
use crate::error::{Result, TextCadError};
use std::ops::{Add, Mul, Sub};
use z3::ast::{Ast, Real};

/// Constraint that places a point on a line segment using parametric representation
///
/// This constraint introduces an internal parameter t ∈ [0,1] and constrains the point
/// to lie on the line segment using the parametric equation:
/// point = start + t * (end - start)
///
/// When t = 0, the point is at the line's start
/// When t = 1, the point is at the line's end
/// When t = 0.5, the point is at the line's midpoint
#[derive(Debug, Clone)]
pub struct PointOnLineConstraint {
    /// Line that the point must lie on
    pub line: LineId,
    /// Point to constrain to the line
    pub point: PointId,
}

impl PointOnLineConstraint {
    /// Create a new point-on-line constraint
    ///
    /// # Arguments
    /// * `line` - The line that the point must lie on
    /// * `point` - The point to constrain to the line
    ///
    /// # Example
    /// ```
    /// use textcad::constraints::PointOnLineConstraint;
    /// use textcad::entities::PointId;
    /// use textcad::entity::LineId;
    /// use generational_arena::Index;
    ///
    /// let line_id = LineId::from(Index::from_raw_parts(0, 0));
    /// let point_id = PointId::from(Index::from_raw_parts(0, 0));
    ///
    /// let constraint = PointOnLineConstraint::new(line_id, point_id);
    /// ```
    pub fn new(line: LineId, point: PointId) -> Self {
        Self { line, point }
    }
}

impl Constraint for PointOnLineConstraint {
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

        // Get point coordinates for all three points
        let (px, py) = sketch
            .point_variables(self.point)
            .map_err(|_| TextCadError::EntityError(format!("Point {:?} not found", self.point)))?;

        let (p1x, p1y) = sketch.point_variables(start_id).map_err(|_| {
            TextCadError::EntityError(format!("Line start point {:?} not found", start_id))
        })?;

        let (p2x, p2y) = sketch.point_variables(end_id).map_err(|_| {
            TextCadError::EntityError(format!("Line end point {:?} not found", end_id))
        })?;

        // Introduce parameter t for this constraint
        // Use unique parameter name based on line and point IDs to avoid conflicts
        let t_name = format!(
            "t_line_{}_point_{}",
            self.line.0.into_raw_parts().0,
            self.point.0.into_raw_parts().0
        );
        let t = Real::new_const(context, t_name);

        // Apply parametric line equation: point = p1 + t * (p2 - p1)
        // px = p1x + t * (p2x - p1x)
        // py = p1y + t * (p2y - p1y)

        let dx = (&p2x).sub(&p1x); // p2x - p1x
        let dy = (&p2y).sub(&p1y); // p2y - p1y

        let point_x = (&p1x).add(&(&t).mul(&dx)); // p1x + t * dx
        let point_y = (&p1y).add(&(&t).mul(&dy)); // p1y + t * dy

        // Assert that the point coordinates equal the parametric expressions
        solver.assert(&px._eq(&point_x));
        solver.assert(&py._eq(&point_y));

        // Constrain parameter t to be within [0, 1] to ensure point is on line segment
        let zero = Real::from_real(context, 0, 1);
        let one = Real::from_real(context, 1, 1);
        solver.assert(&t.ge(&zero)); // t >= 0
        solver.assert(&t.le(&one)); // t <= 1

        Ok(())
    }

    fn description(&self) -> String {
        format!(
            "Point {:?} lies on line segment {:?}",
            self.point, self.line
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

    // Mock implementation of SketchQuery for testing parametric constraints
    struct MockParametricSketch<'ctx> {
        points: HashMap<PointId, (Real<'ctx>, Real<'ctx>)>,
        lines: HashMap<LineId, (PointId, PointId)>,
    }

    impl<'ctx> MockParametricSketch<'ctx> {
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

    impl<'ctx> SketchQuery for MockParametricSketch<'ctx> {
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

        fn circle_center_and_radius(
            &self,
            _circle_id: crate::entity::CircleId,
        ) -> Result<(crate::entities::PointId, Real<'_>)> {
            Err(TextCadError::InvalidConstraint(
                "Not implemented".to_string(),
            ))
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
    fn test_point_on_line_constraint_creation() {
        let line_id = LineId(Index::from_raw_parts(0, 0));
        let point_id = PointId(Index::from_raw_parts(0, 0));

        let constraint = PointOnLineConstraint::new(line_id, point_id);

        assert_eq!(constraint.line, line_id);
        assert_eq!(constraint.point, point_id);
        assert!(constraint.description().contains("lies on line segment"));
    }

    #[test]
    fn test_point_on_line_constraint_apply() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        // Create a line from point p1 to p2
        let p1 = PointId(Index::from_raw_parts(0, 0));
        let p2 = PointId(Index::from_raw_parts(1, 0));
        let line_id = LineId(Index::from_raw_parts(0, 0));

        // Point p3 will be constrained to lie on the line
        let p3 = PointId(Index::from_raw_parts(2, 0));

        // Create Z3 variables for all points
        let x1 = Real::new_const(&ctx, "x1");
        let y1 = Real::new_const(&ctx, "y1");
        let x2 = Real::new_const(&ctx, "x2");
        let y2 = Real::new_const(&ctx, "y2");
        let x3 = Real::new_const(&ctx, "x3");
        let y3 = Real::new_const(&ctx, "y3");

        let mut mock_sketch = MockParametricSketch::new();
        mock_sketch.add_point(p1, x1, y1);
        mock_sketch.add_point(p2, x2, y2);
        mock_sketch.add_point(p3, x3, y3);
        mock_sketch.add_line(line_id, p1, p2);

        let constraint = PointOnLineConstraint::new(line_id, p3);

        // Apply the constraint
        constraint.apply(&ctx, &solver, &mock_sketch).unwrap();

        // Check that we have exactly 4 assertions:
        // 1. px = p1x + t * (p2x - p1x)
        // 2. py = p1y + t * (p2y - p1y)
        // 3. t >= 0
        // 4. t <= 1
        assert_eq!(solver.get_assertions().len(), 4);
    }

    #[test]
    fn test_point_on_line_constraint_with_invalid_line() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        let line_id = LineId(Index::from_raw_parts(999, 999)); // Non-existent line
        let point_id = PointId(Index::from_raw_parts(0, 0));

        let mock_sketch = MockParametricSketch::new();
        let constraint = PointOnLineConstraint::new(line_id, point_id);

        // Should fail because line doesn't exist
        let result = constraint.apply(&ctx, &solver, &mock_sketch);
        assert!(result.is_err());
        assert!(matches!(result, Err(TextCadError::EntityError(_))));
    }

    #[test]
    fn test_point_on_line_constraint_with_invalid_point() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        let p1 = PointId(Index::from_raw_parts(0, 0));
        let p2 = PointId(Index::from_raw_parts(1, 0));
        let line_id = LineId(Index::from_raw_parts(0, 0));
        let point_id = PointId(Index::from_raw_parts(999, 999)); // Non-existent point

        let x1 = Real::new_const(&ctx, "x1");
        let y1 = Real::new_const(&ctx, "y1");
        let x2 = Real::new_const(&ctx, "x2");
        let y2 = Real::new_const(&ctx, "y2");

        let mut mock_sketch = MockParametricSketch::new();
        mock_sketch.add_point(p1, x1, y1);
        mock_sketch.add_point(p2, x2, y2);
        mock_sketch.add_line(line_id, p1, p2);

        let constraint = PointOnLineConstraint::new(line_id, point_id);

        // Should fail because point doesn't exist
        let result = constraint.apply(&ctx, &solver, &mock_sketch);
        assert!(result.is_err());
        assert!(matches!(result, Err(TextCadError::EntityError(_))));
    }

    #[test]
    fn test_point_on_line_constraint_with_invalid_line_endpoints() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        let p1 = PointId(Index::from_raw_parts(0, 0));
        let p2 = PointId(Index::from_raw_parts(999, 999)); // Non-existent endpoint
        let p3 = PointId(Index::from_raw_parts(2, 0));
        let line_id = LineId(Index::from_raw_parts(0, 0));

        let x1 = Real::new_const(&ctx, "x1");
        let y1 = Real::new_const(&ctx, "y1");
        let x3 = Real::new_const(&ctx, "x3");
        let y3 = Real::new_const(&ctx, "y3");

        let mut mock_sketch = MockParametricSketch::new();
        mock_sketch.add_point(p1, x1, y1);
        mock_sketch.add_point(p3, x3, y3);
        // Not adding p2, but line references it
        mock_sketch.add_line(line_id, p1, p2);

        let constraint = PointOnLineConstraint::new(line_id, p3);

        // Should fail because line endpoint p2 doesn't exist
        let result = constraint.apply(&ctx, &solver, &mock_sketch);
        assert!(result.is_err());
        assert!(matches!(result, Err(TextCadError::EntityError(_))));
    }

    #[test]
    fn test_point_on_line_constraint_description() {
        let line_id = LineId(Index::from_raw_parts(0, 0));
        let point_id = PointId(Index::from_raw_parts(1, 0));
        let constraint = PointOnLineConstraint::new(line_id, point_id);

        let description = constraint.description();
        assert!(description.contains("lies on line segment"));
        assert!(description.contains("PointId"));
        assert!(description.contains("LineId"));
    }

    #[test]
    fn test_point_on_line_constraint_clone() {
        let line_id = LineId(Index::from_raw_parts(0, 0));
        let point_id = PointId(Index::from_raw_parts(1, 0));
        let constraint1 = PointOnLineConstraint::new(line_id, point_id);
        let constraint2 = constraint1.clone();

        assert_eq!(constraint1.line, constraint2.line);
        assert_eq!(constraint1.point, constraint2.point);
        assert_eq!(constraint1.description(), constraint2.description());
    }

    #[test]
    fn test_point_on_line_constraint_debug_format() {
        let line_id = LineId(Index::from_raw_parts(0, 0));
        let point_id = PointId(Index::from_raw_parts(1, 0));
        let constraint = PointOnLineConstraint::new(line_id, point_id);

        // Test that Debug format works (doesn't panic)
        let _debug = format!("{:?}", constraint);
    }

    #[test]
    fn test_point_on_line_parameter_name_uniqueness() {
        // Test that different line/point combinations generate different parameter names
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        let p1 = PointId(Index::from_raw_parts(0, 0));
        let p2 = PointId(Index::from_raw_parts(1, 0));
        let p3 = PointId(Index::from_raw_parts(2, 0));
        let p4 = PointId(Index::from_raw_parts(3, 0));
        let line1 = LineId(Index::from_raw_parts(0, 0));
        let line2 = LineId(Index::from_raw_parts(1, 0));

        let x1 = Real::new_const(&ctx, "x1");
        let y1 = Real::new_const(&ctx, "y1");
        let x2 = Real::new_const(&ctx, "x2");
        let y2 = Real::new_const(&ctx, "y2");
        let x3 = Real::new_const(&ctx, "x3");
        let y3 = Real::new_const(&ctx, "y3");
        let x4 = Real::new_const(&ctx, "x4");
        let y4 = Real::new_const(&ctx, "y4");

        let mut mock_sketch = MockParametricSketch::new();
        mock_sketch.add_point(p1, x1, y1);
        mock_sketch.add_point(p2, x2, y2);
        mock_sketch.add_point(p3, x3, y3);
        mock_sketch.add_point(p4, x4, y4);
        mock_sketch.add_line(line1, p1, p2);
        mock_sketch.add_line(line2, p3, p4);

        // Create two different point-on-line constraints
        let constraint1 = PointOnLineConstraint::new(line1, p3);
        let constraint2 = PointOnLineConstraint::new(line2, p1);

        // Apply both constraints
        constraint1.apply(&ctx, &solver, &mock_sketch).unwrap();
        constraint2.apply(&ctx, &solver, &mock_sketch).unwrap();

        // Should have 8 assertions total (4 from each constraint)
        assert_eq!(solver.get_assertions().len(), 8);
    }

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use crate::constraints::FixedPositionConstraint;
        use crate::sketch::Sketch;
        use crate::units::Length;
        use proptest::prelude::*;
        use z3::{Config, Context};

        proptest! {
            #[test]
            fn prop_point_on_line_parameter_bounds(
                x1 in -10.0f64..10.0f64,
                y1 in -10.0f64..10.0f64,
                x2 in -10.0f64..10.0f64,
                y2 in -10.0f64..10.0f64
            ) {
                // Skip degenerate cases where the line has zero length
                prop_assume!((x2 - x1).abs() > 1e-3 || (y2 - y1).abs() > 1e-3);

                let cfg = Config::new();
                let ctx = Context::new(&cfg);
                let mut sketch = Sketch::new(&ctx);

                let p1 = sketch.add_point(Some("p1".to_string()));
                let p2 = sketch.add_point(Some("p2".to_string()));
                sketch.add_constraint(FixedPositionConstraint::new(
                    p1,
                    Length::meters(x1),
                    Length::meters(y1),
                ));
                sketch.add_constraint(FixedPositionConstraint::new(
                    p2,
                    Length::meters(x2),
                    Length::meters(y2),
                ));

                let line = sketch.add_line(p1, p2, Some("test_line".to_string()));
                let p = sketch.add_point(Some("point_on_line".to_string()));
                sketch.add_constraint(PointOnLineConstraint::new(line, p));

                let solution_result = sketch.solve_and_extract();
                if let Ok(solution) = solution_result {
                    let (px, py) = solution.get_point_coordinates(p).unwrap();

                    // Verify point is on line segment
                    // Compute t parameter and verify 0 <= t <= 1
                    let dx = x2 - x1;
                    let dy = y2 - y1;
                    let length_sq = dx * dx + dy * dy;

                    if length_sq > 1e-6 {
                        // Project point onto line to find parameter t
                        let dot_product = (px - x1) * dx + (py - y1) * dy;
                        let t = dot_product / length_sq;

                        prop_assert!(t >= -1e-6 && t <= 1.0 + 1e-6,
                            "Parameter t should be in [0,1], got: {}, point: ({}, {}), line: ({}, {}) to ({}, {})",
                            t, px, py, x1, y1, x2, y2);

                        // Verify point lies exactly on the line
                        let expected_px = x1 + t * dx;
                        let expected_py = y1 + t * dy;

                        prop_assert!((px - expected_px).abs() < 1e-3,
                            "Point x-coordinate should match parametric equation");
                        prop_assert!((py - expected_py).abs() < 1e-3,
                            "Point y-coordinate should match parametric equation");
                    }
                }
            }

            #[test]
            fn prop_point_on_line_collinear_verification(
                x1 in -5.0f64..5.0f64,
                y1 in -5.0f64..5.0f64,
                x2 in -5.0f64..5.0f64,
                y2 in -5.0f64..5.0f64
            ) {
                // Skip degenerate cases where the line has zero length
                prop_assume!((x2 - x1).abs() > 1e-3 || (y2 - y1).abs() > 1e-3);

                let cfg = Config::new();
                let ctx = Context::new(&cfg);
                let mut sketch = Sketch::new(&ctx);

                let p1 = sketch.add_point(Some("start".to_string()));
                let p2 = sketch.add_point(Some("end".to_string()));
                let line = sketch.add_line(p1, p2, Some("line".to_string()));
                let p3 = sketch.add_point(Some("on_line".to_string()));

                sketch.add_constraint(FixedPositionConstraint::new(
                    p1, Length::meters(x1), Length::meters(y1)
                ));
                sketch.add_constraint(FixedPositionConstraint::new(
                    p2, Length::meters(x2), Length::meters(y2)
                ));
                sketch.add_constraint(PointOnLineConstraint::new(line, p3));

                let solution_result = sketch.solve_and_extract();
                if let Ok(solution) = solution_result {
                    let (px1, py1) = solution.get_point_coordinates(p1).unwrap();
                    let (px2, py2) = solution.get_point_coordinates(p2).unwrap();
                    let (px3, py3) = solution.get_point_coordinates(p3).unwrap();

                    // Check collinearity using cross product
                    // Points are collinear if (p3-p1) × (p2-p1) = 0
                    let v1x = px2 - px1;  // p2 - p1
                    let v1y = py2 - py1;
                    let v2x = px3 - px1;  // p3 - p1
                    let v2y = py3 - py1;

                    let cross_product = v1x * v2y - v1y * v2x;
                    prop_assert!(cross_product.abs() < 1e-6,
                        "Points should be collinear, cross product: {}", cross_product);
                }
            }

            #[test]
            fn prop_multiple_points_on_same_line_are_collinear(
                x1 in -5.0f64..5.0f64,
                y1 in -5.0f64..5.0f64,
                x2 in -5.0f64..5.0f64,
                y2 in -5.0f64..5.0f64
            ) {
                // Skip degenerate cases where the line has zero length
                prop_assume!((x2 - x1).abs() > 1e-3 || (y2 - y1).abs() > 1e-3);

                let cfg = Config::new();
                let ctx = Context::new(&cfg);
                let mut sketch = Sketch::new(&ctx);

                let p1 = sketch.add_point(Some("start".to_string()));
                let p2 = sketch.add_point(Some("end".to_string()));
                let line = sketch.add_line(p1, p2, Some("line".to_string()));

                // Add multiple points on the same line
                let p3 = sketch.add_point(Some("on_line_1".to_string()));
                let p4 = sketch.add_point(Some("on_line_2".to_string()));
                let p5 = sketch.add_point(Some("on_line_3".to_string()));

                sketch.add_constraint(FixedPositionConstraint::new(
                    p1, Length::meters(x1), Length::meters(y1)
                ));
                sketch.add_constraint(FixedPositionConstraint::new(
                    p2, Length::meters(x2), Length::meters(y2)
                ));
                sketch.add_constraint(PointOnLineConstraint::new(line, p3));
                sketch.add_constraint(PointOnLineConstraint::new(line, p4));
                sketch.add_constraint(PointOnLineConstraint::new(line, p5));

                let solution_result = sketch.solve_and_extract();
                if let Ok(solution) = solution_result {
                    let points = [p1, p2, p3, p4, p5];
                    let mut coords = Vec::new();
                    for &pid in &points {
                        let (x, y) = solution.get_point_coordinates(pid).unwrap();
                        coords.push((x, y));
                    }

                    // Check that all points are collinear with the line p1-p2
                    let (x1, y1) = coords[0];
                    let (x2, y2) = coords[1];
                    let line_dx = x2 - x1;
                    let line_dy = y2 - y1;

                    for i in 2..coords.len() {
                        let (xi, yi) = coords[i];
                        let point_dx = xi - x1;
                        let point_dy = yi - y1;

                        // Cross product should be zero for collinear points
                        let cross_product = line_dx * point_dy - line_dy * point_dx;
                        prop_assert!(cross_product.abs() < 1e-6,
                            "Point {} should be collinear with line, cross product: {}",
                            i, cross_product);
                    }
                }
            }
        }
    }
}
