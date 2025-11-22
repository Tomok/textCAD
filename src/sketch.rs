//! Sketch management with Z3 integration
//!
//! The sketch module provides the main interface for creating and managing
//! geometric entities and constraints using Z3 as the underlying solver.

use generational_arena::Arena;
use z3::{Context, SatResult, Solver};

use crate::constraint::{Constraint, SketchQuery};
use crate::entities::{Line, Point2D, PointId};
use crate::entity::LineId;
use crate::error::{Result, TextCadError};
use crate::solution::Solution;

/// Main sketch structure that manages geometric entities and constraints
///
/// A sketch wraps a Z3 context and solver, providing the foundation for
/// constraint-based geometric modeling.
pub struct Sketch<'ctx> {
    ctx: &'ctx Context,
    solver: Solver<'ctx>,
    /// Arena for managing Point2D entities
    points: Arena<Point2D<'ctx>>,
    /// Arena for managing Line entities
    lines: Arena<Line>,
    /// Vector of constraints that have been added to the sketch
    constraints: Vec<Box<dyn Constraint>>,
}

impl<'ctx> Sketch<'ctx> {
    /// Create a new sketch using the provided Z3 context
    ///
    /// # Arguments
    /// * `ctx` - Z3 context to use for constraint solving
    ///
    /// # Example
    /// ```
    /// use z3::{Config, Context};
    /// use textcad::sketch::Sketch;
    ///
    /// let cfg = Config::new();
    /// let ctx = Context::new(&cfg);
    /// let sketch = Sketch::new(&ctx);
    /// ```
    pub fn new(ctx: &'ctx Context) -> Self {
        let solver = Solver::new(ctx);
        let points = Arena::new();
        let lines = Arena::new();
        let constraints = Vec::new();
        Self {
            ctx,
            solver,
            points,
            lines,
            constraints,
        }
    }

    /// Get a reference to the underlying Z3 context
    pub fn context(&self) -> &'ctx Context {
        self.ctx
    }

    /// Get a reference to the underlying Z3 solver
    pub fn solver(&self) -> &Solver<'ctx> {
        &self.solver
    }

    /// Get a mutable reference to the underlying Z3 solver
    ///
    /// This allows adding assertions directly to the solver
    pub fn solver_mut(&mut self) -> &mut Solver<'ctx> {
        &mut self.solver
    }

    /// Check if the current constraint system is satisfiable
    ///
    /// Returns the satisfiability result from Z3
    pub fn check(&mut self) -> Result<SatResult> {
        Ok(self.solver.check())
    }

    /// Solve the current constraint system and return satisfiability result
    ///
    /// This is a convenience method that wraps `check()` and provides
    /// better error reporting for common failure cases.
    pub fn solve(&mut self) -> Result<SatResult> {
        let result = self.solver.check();
        match result {
            SatResult::Sat => Ok(result),
            SatResult::Unsat => Err(TextCadError::OverConstrained),
            SatResult::Unknown => Err(TextCadError::SolverError(
                "Z3 solver returned unknown result".to_string(),
            )),
        }
    }

    /// Add a new point to the sketch
    ///
    /// Creates a new Point2D with Z3 symbolic variables for its coordinates
    /// and adds it to the points arena.
    ///
    /// # Arguments
    /// * `name` - Optional name for debugging and display
    ///
    /// # Returns
    /// PointId that can be used to reference this point
    ///
    /// # Example
    /// ```
    /// use z3::{Config, Context};
    /// use textcad::sketch::Sketch;
    ///
    /// let cfg = Config::new();
    /// let ctx = Context::new(&cfg);
    /// let mut sketch = Sketch::new(&ctx);
    /// let p1 = sketch.add_point(Some("p1".to_string()));
    /// let p2 = sketch.add_point(None);
    /// ```
    pub fn add_point(&mut self, name: Option<String>) -> PointId {
        let idx = self.points.insert_with(|idx| {
            let id = PointId::from(idx);
            Point2D::new(id, self.ctx, name)
        });
        PointId::from(idx)
    }

    /// Get a reference to a point by its ID
    ///
    /// # Arguments  
    /// * `id` - The PointId to look up
    ///
    /// # Returns
    /// Option containing a reference to the Point2D, or None if not found
    ///
    /// # Example
    /// ```
    /// use z3::{Config, Context};
    /// use textcad::sketch::Sketch;
    ///
    /// let cfg = Config::new();
    /// let ctx = Context::new(&cfg);
    /// let mut sketch = Sketch::new(&ctx);
    /// let id = sketch.add_point(Some("test".to_string()));
    /// let point = sketch.get_point(id).unwrap();
    /// ```
    pub fn get_point(&self, id: PointId) -> Option<&Point2D<'ctx>> {
        self.points.get(id.into())
    }

    /// Add a new line to the sketch
    ///
    /// Creates a new Line that connects two existing points and adds it to the lines arena.
    ///
    /// # Arguments
    /// * `start` - PointId of the starting point  
    /// * `end` - PointId of the ending point
    /// * `name` - Optional name for debugging and display
    ///
    /// # Returns
    /// LineId that can be used to reference this line
    ///
    /// # Example
    /// ```
    /// use z3::{Config, Context};
    /// use textcad::sketch::Sketch;
    ///
    /// let cfg = Config::new();
    /// let ctx = Context::new(&cfg);
    /// let mut sketch = Sketch::new(&ctx);
    /// let p1 = sketch.add_point(Some("p1".to_string()));
    /// let p2 = sketch.add_point(Some("p2".to_string()));
    /// let line = sketch.add_line(p1, p2, Some("line1".to_string()));
    /// ```
    pub fn add_line(&mut self, start: PointId, end: PointId, name: Option<String>) -> LineId {
        let idx = self.lines.insert_with(|idx| {
            let id = LineId::from(idx);
            Line::new(id, start, end, name)
        });
        LineId::from(idx)
    }

    /// Get a reference to a line by its ID
    ///
    /// # Arguments  
    /// * `id` - The LineId to look up
    ///
    /// # Returns
    /// Option containing a reference to the Line, or None if not found
    ///
    /// # Example
    /// ```
    /// use z3::{Config, Context};
    /// use textcad::sketch::Sketch;
    ///
    /// let cfg = Config::new();
    /// let ctx = Context::new(&cfg);
    /// let mut sketch = Sketch::new(&ctx);
    /// let p1 = sketch.add_point(None);
    /// let p2 = sketch.add_point(None);
    /// let id = sketch.add_line(p1, p2, Some("test".to_string()));
    /// let line = sketch.get_line(id).unwrap();
    /// ```
    pub fn get_line(&self, id: LineId) -> Option<&Line> {
        self.lines.get(id.into())
    }

    /// Add a constraint to the sketch
    pub fn add_constraint(&mut self, constraint: impl Constraint + 'static) {
        self.constraints.push(Box::new(constraint));
    }

    /// Apply all constraints and solve the system
    pub fn solve_constraints(&mut self) -> Result<SatResult> {
        // Apply all constraints
        for constraint in &self.constraints {
            constraint.apply(self.ctx, &self.solver, self)?;
        }

        // Solve the constraint system
        self.solve()
    }

    /// Apply all constraints, solve, and return a Solution with extracted coordinates
    pub fn solve_and_extract(&mut self) -> Result<Solution<'ctx>> {
        // Apply all constraints and solve
        self.solve_constraints()?;

        // Extract the model
        let model = self.solver.get_model().ok_or_else(|| {
            TextCadError::SolverError("No model available after solving".to_string())
        })?;

        // Create solution and extract all point coordinates
        let mut solution = Solution::new(model);

        // Extract coordinates for all points
        for (idx, point) in self.points.iter() {
            let point_id = PointId::from(idx);
            solution.extract_point_coordinates(point_id, &point.x, &point.y)?;
        }

        // Extract parameters for all lines
        for (idx, line) in self.lines.iter() {
            let line_id = LineId::from(idx);

            // Get start and end point coordinates
            let start_coords = solution.get_point_coordinates(line.start)?;
            let end_coords = solution.get_point_coordinates(line.end)?;

            // Extract line parameters
            solution.extract_line_parameters(line_id, start_coords, end_coords)?;
        }

        Ok(solution)
    }
}

impl<'ctx> SketchQuery for Sketch<'ctx> {
    fn point_variables(&self, point_id: PointId) -> Result<(z3::ast::Real<'_>, z3::ast::Real<'_>)> {
        if let Some(point) = self.get_point(point_id) {
            Ok((point.x.clone(), point.y.clone()))
        } else {
            Err(TextCadError::EntityError(format!(
                "Point {:?} not found",
                point_id
            )))
        }
    }

    fn line_endpoints(&self, line_id: LineId) -> Result<(PointId, PointId)> {
        if let Some(line) = self.get_line(line_id) {
            Ok((line.start, line.end))
        } else {
            Err(TextCadError::EntityError(format!(
                "Line {:?} not found",
                line_id
            )))
        }
    }

    fn length_variable(&self, name: &str) -> Result<z3::ast::Real<'_>> {
        // For now, create a new length variable on demand
        Ok(z3::ast::Real::new_const(
            self.ctx,
            format!("length_{}", name),
        ))
    }

    fn angle_variable(&self, name: &str) -> Result<z3::ast::Real<'_>> {
        // For now, create a new angle variable on demand
        Ok(z3::ast::Real::new_const(
            self.ctx,
            format!("angle_{}", name),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::{Add, Sub};
    use z3::{
        Config,
        ast::{Ast, Real},
    };

    #[test]
    fn test_sketch_creation() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let sketch = Sketch::new(&ctx);

        // Verify we can access the context
        let _context = sketch.context();

        // Verify initial state
        assert_eq!(sketch.solver().get_assertions().len(), 0);
    }

    #[test]
    fn test_simple_equation() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        // Create equation: x + 2 = 5
        let x = Real::new_const(sketch.context(), "x");
        let two = Real::from_real(sketch.context(), 2, 1);
        let five = Real::from_real(sketch.context(), 5, 1);

        let equation = (&x).add(&two)._eq(&five);
        sketch.solver_mut().assert(&equation);

        let result = sketch.check().unwrap();
        assert_eq!(result, SatResult::Sat);

        // Extract solution and verify x = 3
        let model = sketch.solver().get_model().unwrap();
        let x_value = model.eval(&x, true).unwrap();
        let (num, den) = x_value.as_real().unwrap();
        assert_eq!((num, den), (3, 1)); // x = 3/1 = 3
    }

    #[test]
    fn test_unsatisfiable_constraint() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        // Create unsatisfiable constraints: x > 5 AND x < 3
        let x = Real::new_const(sketch.context(), "x");
        let three = Real::from_real(sketch.context(), 3, 1);
        let five = Real::from_real(sketch.context(), 5, 1);

        sketch.solver_mut().assert(&x.gt(&five)); // x > 5
        sketch.solver_mut().assert(&x.lt(&three)); // x < 3

        let result = sketch.solve();
        assert!(matches!(result, Err(TextCadError::OverConstrained)));
    }

    #[test]
    fn test_multiple_variables() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        // System: x + y = 10, x - y = 2
        // Solution: x = 6, y = 4
        let x = Real::new_const(sketch.context(), "x");
        let y = Real::new_const(sketch.context(), "y");
        let ten = Real::from_real(sketch.context(), 10, 1);
        let two = Real::from_real(sketch.context(), 2, 1);

        let eq1 = (&x).add(&y)._eq(&ten);
        let eq2 = (&x).sub(&y)._eq(&two);
        sketch.solver_mut().assert(&eq1);
        sketch.solver_mut().assert(&eq2);

        let result = sketch.check().unwrap();
        assert_eq!(result, SatResult::Sat);

        let model = sketch.solver().get_model().unwrap();
        let x_value = model.eval(&x, true).unwrap().as_real().unwrap();
        let y_value = model.eval(&y, true).unwrap().as_real().unwrap();

        assert_eq!(x_value, (6, 1)); // x = 6
        assert_eq!(y_value, (4, 1)); // y = 4
    }

    #[test]
    fn test_point_creation() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        let p1 = sketch.add_point(Some("p1".to_string()));
        let p2 = sketch.add_point(Some("p2".to_string()));

        assert_ne!(p1, p2);
        assert!(sketch.get_point(p1).is_some());
        assert!(sketch.get_point(p2).is_some());

        let point1 = sketch.get_point(p1).unwrap();
        let point2 = sketch.get_point(p2).unwrap();

        assert_eq!(point1.id, p1);
        assert_eq!(point2.id, p2);
        assert_eq!(point1.name, Some("p1".to_string()));
        assert_eq!(point2.name, Some("p2".to_string()));
    }

    #[test]
    fn test_point_creation_without_name() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        let p = sketch.add_point(None);
        let point = sketch.get_point(p).unwrap();

        assert_eq!(point.id, p);
        assert_eq!(point.name, None);
        assert!(point.display_name().starts_with("Point"));
    }

    #[test]
    fn test_multiple_points_distinct_ids() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        let p1 = sketch.add_point(Some("p1".to_string()));
        let p2 = sketch.add_point(Some("p2".to_string()));
        let p3 = sketch.add_point(Some("p3".to_string()));

        // All IDs should be different
        assert_ne!(p1, p2);
        assert_ne!(p2, p3);
        assert_ne!(p1, p3);

        // All points should be retrievable
        assert!(sketch.get_point(p1).is_some());
        assert!(sketch.get_point(p2).is_some());
        assert!(sketch.get_point(p3).is_some());

        // Z3 variables should have different names
        let point1 = sketch.get_point(p1).unwrap();
        let point2 = sketch.get_point(p2).unwrap();
        let point3 = sketch.get_point(p3).unwrap();

        assert!(point1.x.to_string().contains("p1_x"));
        assert!(point2.x.to_string().contains("p2_x"));
        assert!(point3.x.to_string().contains("p3_x"));
    }

    #[test]
    fn test_point_z3_variable_names() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        let p1 = sketch.add_point(Some("test_point".to_string()));
        let point = sketch.get_point(p1).unwrap();

        // Verify Z3 variables have correct names
        let x_str = point.x.to_string();
        let y_str = point.y.to_string();

        assert!(x_str.contains("test_point_x"));
        assert!(y_str.contains("test_point_y"));
    }

    #[test]
    fn test_get_nonexistent_point() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let sketch = Sketch::new(&ctx);

        // Create a fake PointId that doesn't exist
        use crate::entities::PointId;
        use generational_arena::Index;
        let fake_id = PointId::from(Index::from_raw_parts(999, 999));

        assert!(sketch.get_point(fake_id).is_none());
    }

    // Integration tests for constraint solving workflow
    #[test]
    fn test_single_point_fixed_position() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        // Add a point and fix it at a specific position
        let p1 = sketch.add_point(Some("p1".to_string()));
        let constraint = crate::constraints::FixedPositionConstraint::new(
            p1,
            crate::units::Length::meters(3.0),
            crate::units::Length::meters(4.0),
        );
        sketch.add_constraint(constraint);

        // Solve and extract solution
        let solution = sketch.solve_and_extract().unwrap();
        let (x, y) = solution.get_point_coordinates(p1).unwrap();

        assert!((x - 3.0).abs() < 1e-6);
        assert!((y - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_coincident_points_constraint() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        // Add two points and make them coincident
        let p1 = sketch.add_point(Some("p1".to_string()));
        let p2 = sketch.add_point(Some("p2".to_string()));

        // Fix one point's position
        let fix_constraint = crate::constraints::FixedPositionConstraint::new(
            p1,
            crate::units::Length::meters(1.0),
            crate::units::Length::meters(2.0),
        );
        sketch.add_constraint(fix_constraint);

        // Make the second point coincident with the first
        let coincident_constraint = crate::constraints::CoincidentPointsConstraint::new(p1, p2);
        sketch.add_constraint(coincident_constraint);

        // Solve and verify both points have the same coordinates
        let solution = sketch.solve_and_extract().unwrap();
        let (x1, y1) = solution.get_point_coordinates(p1).unwrap();
        let (x2, y2) = solution.get_point_coordinates(p2).unwrap();

        assert!((x1 - 1.0).abs() < 1e-6);
        assert!((y1 - 2.0).abs() < 1e-6);
        assert!((x1 - x2).abs() < 1e-6);
        assert!((y1 - y2).abs() < 1e-6);
    }

    #[test]
    fn test_overconstrainted_system() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        // Add a point
        let p1 = sketch.add_point(Some("p1".to_string()));

        // Try to fix it at two different positions (overconstraint)
        sketch.add_constraint(crate::constraints::FixedPositionConstraint::new(
            p1,
            crate::units::Length::meters(1.0),
            crate::units::Length::meters(1.0),
        ));
        sketch.add_constraint(crate::constraints::FixedPositionConstraint::new(
            p1,
            crate::units::Length::meters(2.0),
            crate::units::Length::meters(2.0),
        ));

        // This should fail as the system is overconstrained
        let result = sketch.solve_and_extract();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TextCadError::OverConstrained));
    }

    // Tests for Line entity functionality
    #[test]
    fn test_line_creation() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        let p1 = sketch.add_point(Some("p1".to_string()));
        let p2 = sketch.add_point(Some("p2".to_string()));
        let line = sketch.add_line(p1, p2, Some("line1".to_string()));

        assert!(sketch.get_line(line).is_some());

        let line_obj = sketch.get_line(line).unwrap();
        assert_eq!(line_obj.id, line);
        assert_eq!(line_obj.start, p1);
        assert_eq!(line_obj.end, p2);
        assert_eq!(line_obj.name, Some("line1".to_string()));
    }

    #[test]
    fn test_line_creation_without_name() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        let p1 = sketch.add_point(None);
        let p2 = sketch.add_point(None);
        let line = sketch.add_line(p1, p2, None);

        let line_obj = sketch.get_line(line).unwrap();
        assert_eq!(line_obj.id, line);
        assert_eq!(line_obj.start, p1);
        assert_eq!(line_obj.end, p2);
        assert_eq!(line_obj.name, None);
        assert!(line_obj.display_name().starts_with("Line"));
    }

    #[test]
    fn test_multiple_lines_distinct_ids() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        let p1 = sketch.add_point(Some("p1".to_string()));
        let p2 = sketch.add_point(Some("p2".to_string()));
        let p3 = sketch.add_point(Some("p3".to_string()));

        let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
        let line2 = sketch.add_line(p2, p3, Some("line2".to_string()));
        let line3 = sketch.add_line(p1, p3, Some("line3".to_string()));

        // All IDs should be different
        assert_ne!(line1, line2);
        assert_ne!(line2, line3);
        assert_ne!(line1, line3);

        // All lines should be retrievable
        assert!(sketch.get_line(line1).is_some());
        assert!(sketch.get_line(line2).is_some());
        assert!(sketch.get_line(line3).is_some());

        // Lines should have correct endpoints
        let line1_obj = sketch.get_line(line1).unwrap();
        let line2_obj = sketch.get_line(line2).unwrap();
        let line3_obj = sketch.get_line(line3).unwrap();

        assert_eq!(line1_obj.endpoints(), (p1, p2));
        assert_eq!(line2_obj.endpoints(), (p2, p3));
        assert_eq!(line3_obj.endpoints(), (p1, p3));
    }

    #[test]
    fn test_get_nonexistent_line() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let sketch = Sketch::new(&ctx);

        // Create a fake LineId that doesn't exist
        use generational_arena::Index;
        let fake_id = LineId::from(Index::from_raw_parts(999, 999));

        assert!(sketch.get_line(fake_id).is_none());
    }

    #[test]
    fn test_line_endpoints_query() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        let p1 = sketch.add_point(Some("p1".to_string()));
        let p2 = sketch.add_point(Some("p2".to_string()));
        let line = sketch.add_line(p1, p2, Some("test_line".to_string()));

        // Test SketchQuery trait implementation
        let endpoints = sketch.line_endpoints(line).unwrap();
        assert_eq!(endpoints, (p1, p2));
    }

    #[test]
    fn test_line_endpoints_query_invalid_line() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let sketch = Sketch::new(&ctx);

        // Try to query a non-existent line
        use generational_arena::Index;
        let fake_line_id = LineId::from(Index::from_raw_parts(999, 999));

        let result = sketch.line_endpoints(fake_line_id);
        assert!(result.is_err());
        assert!(matches!(result, Err(TextCadError::EntityError(_))));
    }

    #[test]
    fn test_line_contains_point() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        let p1 = sketch.add_point(Some("p1".to_string()));
        let p2 = sketch.add_point(Some("p2".to_string()));
        let p3 = sketch.add_point(Some("p3".to_string()));

        let line = sketch.add_line(p1, p2, Some("test_line".to_string()));
        let line_obj = sketch.get_line(line).unwrap();

        assert!(line_obj.contains_point(p1));
        assert!(line_obj.contains_point(p2));
        assert!(!line_obj.contains_point(p3));
    }

    // Integration tests for Line entity with constraints
    #[test]
    fn test_line_with_fixed_endpoints() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        // Create two points and fix their positions
        let p1 = sketch.add_point(Some("p1".to_string()));
        let p2 = sketch.add_point(Some("p2".to_string()));

        // Fix p1 at origin (0, 0)
        sketch.add_constraint(crate::constraints::FixedPositionConstraint::new(
            p1,
            crate::units::Length::meters(0.0),
            crate::units::Length::meters(0.0),
        ));

        // Fix p2 at (3, 4) - this creates a 3-4-5 right triangle
        sketch.add_constraint(crate::constraints::FixedPositionConstraint::new(
            p2,
            crate::units::Length::meters(3.0),
            crate::units::Length::meters(4.0),
        ));

        // Create a line connecting these points
        let line = sketch.add_line(p1, p2, Some("line1".to_string()));

        // Verify line was created properly
        let line_obj = sketch.get_line(line).unwrap();
        assert_eq!(line_obj.endpoints(), (p1, p2));
        assert_eq!(line_obj.name, Some("line1".to_string()));

        // Solve and extract solution
        let solution = sketch.solve_and_extract().unwrap();

        // Verify point coordinates
        let (x1, y1) = solution.get_point_coordinates(p1).unwrap();
        let (x2, y2) = solution.get_point_coordinates(p2).unwrap();

        assert!((x1 - 0.0).abs() < 1e-6);
        assert!((y1 - 0.0).abs() < 1e-6);
        assert!((x2 - 3.0).abs() < 1e-6);
        assert!((y2 - 4.0).abs() < 1e-6);

        // Calculate line length using Pythagorean theorem
        let line_length = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
        assert!((line_length - 5.0).abs() < 1e-6); // 3-4-5 triangle
    }

    #[test]
    fn test_triangle_with_three_lines() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        // Create three points for a triangle
        let p1 = sketch.add_point(Some("A".to_string()));
        let p2 = sketch.add_point(Some("B".to_string()));
        let p3 = sketch.add_point(Some("C".to_string()));

        // Fix triangle vertices at specific positions
        sketch.add_constraint(crate::constraints::FixedPositionConstraint::new(
            p1,
            crate::units::Length::meters(0.0),
            crate::units::Length::meters(0.0),
        ));
        sketch.add_constraint(crate::constraints::FixedPositionConstraint::new(
            p2,
            crate::units::Length::meters(6.0),
            crate::units::Length::meters(0.0),
        ));
        sketch.add_constraint(crate::constraints::FixedPositionConstraint::new(
            p3,
            crate::units::Length::meters(3.0),
            crate::units::Length::meters(4.0),
        ));

        // Create three lines forming the triangle
        let line_ab = sketch.add_line(p1, p2, Some("AB".to_string()));
        let line_bc = sketch.add_line(p2, p3, Some("BC".to_string()));
        let line_ca = sketch.add_line(p3, p1, Some("CA".to_string()));

        // Verify lines have correct endpoints
        let line_ab_obj = sketch.get_line(line_ab).unwrap();
        let line_bc_obj = sketch.get_line(line_bc).unwrap();
        let line_ca_obj = sketch.get_line(line_ca).unwrap();

        assert_eq!(line_ab_obj.endpoints(), (p1, p2));
        assert_eq!(line_bc_obj.endpoints(), (p2, p3));
        assert_eq!(line_ca_obj.endpoints(), (p3, p1));

        // Solve the system
        let solution = sketch.solve_and_extract().unwrap();

        // Verify all points have correct coordinates
        let (ax, ay) = solution.get_point_coordinates(p1).unwrap();
        let (bx, by) = solution.get_point_coordinates(p2).unwrap();
        let (cx, cy) = solution.get_point_coordinates(p3).unwrap();

        assert!((ax - 0.0).abs() < 1e-6 && (ay - 0.0).abs() < 1e-6);
        assert!((bx - 6.0).abs() < 1e-6 && (by - 0.0).abs() < 1e-6);
        assert!((cx - 3.0).abs() < 1e-6 && (cy - 4.0).abs() < 1e-6);

        // Calculate and verify triangle side lengths
        let ab_length = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt();
        let bc_length = ((cx - bx).powi(2) + (cy - by).powi(2)).sqrt();
        let ca_length = ((ax - cx).powi(2) + (ay - cy).powi(2)).sqrt();

        assert!((ab_length - 6.0).abs() < 1e-6); // Base of triangle
        assert!((bc_length - 5.0).abs() < 1e-6); // 3-4-5 triangle side
        assert!((ca_length - 5.0).abs() < 1e-6); // 3-4-5 triangle side
    }

    #[test]
    fn test_line_endpoint_query_integration() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        let p1 = sketch.add_point(Some("start".to_string()));
        let p2 = sketch.add_point(Some("end".to_string()));
        let line = sketch.add_line(p1, p2, Some("test_line".to_string()));

        // Test the SketchQuery trait implementation
        let endpoints = sketch.line_endpoints(line).unwrap();
        assert_eq!(endpoints.0, p1);
        assert_eq!(endpoints.1, p2);

        // Verify this matches the line object's endpoints method
        let line_obj = sketch.get_line(line).unwrap();
        assert_eq!(endpoints, line_obj.endpoints());
    }

    #[test]
    fn test_line_length_constraint_with_entity_factory() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        // Create two points
        let p1 = sketch.add_point(Some("start".to_string()));
        let p2 = sketch.add_point(Some("end".to_string()));

        // Fix one point at the origin
        sketch.add_constraint(crate::constraints::FixedPositionConstraint::new(
            p1,
            crate::units::Length::meters(0.0),
            crate::units::Length::meters(0.0),
        ));

        // Create a line
        let line_id = sketch.add_line(p1, p2, Some("test_line".to_string()));

        // Use the entity-as-constraint-factory pattern to create length constraint
        let length_constraint = {
            let line_obj = sketch.get_line(line_id).unwrap();
            line_obj.length_equals(crate::units::Length::meters(10.0))
        };
        sketch.add_constraint(length_constraint);

        // Solve the system
        let solution = sketch.solve_and_extract().unwrap();

        // Verify point positions
        let (x1, y1) = solution.get_point_coordinates(p1).unwrap();
        let (x2, y2) = solution.get_point_coordinates(p2).unwrap();

        assert!((x1 - 0.0).abs() < 1e-6);
        assert!((y1 - 0.0).abs() < 1e-6);

        // Calculate actual line length
        let actual_length = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
        assert!((actual_length - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_multiple_line_constraints() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        // Create points for two lines forming an L-shape
        let origin = sketch.add_point(Some("origin".to_string()));
        let end1 = sketch.add_point(Some("end1".to_string()));
        let end2 = sketch.add_point(Some("end2".to_string()));

        // Fix origin
        sketch.add_constraint(crate::constraints::FixedPositionConstraint::new(
            origin,
            crate::units::Length::meters(0.0),
            crate::units::Length::meters(0.0),
        ));

        // Fix end1 on x-axis
        sketch.add_constraint(crate::constraints::FixedPositionConstraint::new(
            end1,
            crate::units::Length::meters(3.0),
            crate::units::Length::meters(0.0),
        ));

        // Fix end2 on y-axis
        sketch.add_constraint(crate::constraints::FixedPositionConstraint::new(
            end2,
            crate::units::Length::meters(0.0),
            crate::units::Length::meters(4.0),
        ));

        // Create two lines
        let line1_id = sketch.add_line(origin, end1, Some("horizontal".to_string()));
        let line2_id = sketch.add_line(origin, end2, Some("vertical".to_string()));

        // Use entity-as-constraint-factory to set line lengths
        let length1_constraint = {
            let line1 = sketch.get_line(line1_id).unwrap();
            line1.length_equals(crate::units::Length::meters(3.0))
        };
        let length2_constraint = {
            let line2 = sketch.get_line(line2_id).unwrap();
            line2.length_equals(crate::units::Length::meters(4.0))
        };

        sketch.add_constraint(length1_constraint);
        sketch.add_constraint(length2_constraint);

        // Solve and verify
        let solution = sketch.solve_and_extract().unwrap();

        let (ox, oy) = solution.get_point_coordinates(origin).unwrap();
        let (x1, y1) = solution.get_point_coordinates(end1).unwrap();
        let (x2, y2) = solution.get_point_coordinates(end2).unwrap();

        // Verify fixed positions
        assert!((ox - 0.0).abs() < 1e-6 && (oy - 0.0).abs() < 1e-6);
        assert!((x1 - 3.0).abs() < 1e-6 && (y1 - 0.0).abs() < 1e-6);
        assert!((x2 - 0.0).abs() < 1e-6 && (y2 - 4.0).abs() < 1e-6);

        // Verify line lengths
        let len1 = ((x1 - ox).powi(2) + (y1 - oy).powi(2)).sqrt();
        let len2 = ((x2 - ox).powi(2) + (y2 - oy).powi(2)).sqrt();

        assert!((len1 - 3.0).abs() < 1e-6);
        assert!((len2 - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_line_parameter_extraction() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        // Create a right triangle with known angles
        let origin = sketch.add_point(Some("origin".to_string()));
        let right = sketch.add_point(Some("right".to_string()));
        let top = sketch.add_point(Some("top".to_string()));

        // Fix points for a 3-4-5 right triangle
        sketch.add_constraint(crate::constraints::FixedPositionConstraint::new(
            origin,
            crate::units::Length::meters(0.0),
            crate::units::Length::meters(0.0),
        ));
        sketch.add_constraint(crate::constraints::FixedPositionConstraint::new(
            right,
            crate::units::Length::meters(3.0),
            crate::units::Length::meters(0.0),
        ));
        sketch.add_constraint(crate::constraints::FixedPositionConstraint::new(
            top,
            crate::units::Length::meters(0.0),
            crate::units::Length::meters(4.0),
        ));

        // Create lines
        let horizontal_line = sketch.add_line(origin, right, Some("horizontal".to_string()));
        let vertical_line = sketch.add_line(origin, top, Some("vertical".to_string()));
        let hypotenuse_line = sketch.add_line(right, top, Some("hypotenuse".to_string()));

        // Solve and extract
        let solution = sketch.solve_and_extract().unwrap();

        // Check horizontal line parameters
        let h_params = solution.get_line_parameters(horizontal_line).unwrap();
        assert!((h_params.start.0 - 0.0).abs() < 1e-6);
        assert!((h_params.start.1 - 0.0).abs() < 1e-6);
        assert!((h_params.end.0 - 3.0).abs() < 1e-6);
        assert!((h_params.end.1 - 0.0).abs() < 1e-6);
        assert!((h_params.length - 3.0).abs() < 1e-6);
        assert!((h_params.angle - 0.0).abs() < 1e-6); // 0 radians (horizontal)

        // Check vertical line parameters
        let v_params = solution.get_line_parameters(vertical_line).unwrap();
        assert!((v_params.length - 4.0).abs() < 1e-6);
        assert!((v_params.angle - std::f64::consts::FRAC_PI_2).abs() < 1e-6); // π/2 radians (vertical)

        // Check hypotenuse line parameters
        let hyp_params = solution.get_line_parameters(hypotenuse_line).unwrap();
        assert!((hyp_params.length - 5.0).abs() < 1e-6); // 3-4-5 triangle

        // Check angle is correct (from (3,0) to (0,4))
        let expected_angle = (4.0_f64 - 0.0_f64).atan2(0.0_f64 - 3.0_f64); // atan2(4, -3)
        assert!((hyp_params.angle - expected_angle).abs() < 1e-6);
    }
}
