//! Sketch management with Z3 integration
//!
//! The sketch module provides the main interface for creating and managing
//! geometric entities and constraints using Z3 as the underlying solver.

use generational_arena::Arena;
use z3::{Context, SatResult, Solver};

use crate::entities::{Point2D, PointId};
use crate::error::{Result, TextCadError};

/// Main sketch structure that manages geometric entities and constraints
///
/// A sketch wraps a Z3 context and solver, providing the foundation for
/// constraint-based geometric modeling.
pub struct Sketch<'ctx> {
    ctx: &'ctx Context,
    solver: Solver<'ctx>,
    /// Arena for managing Point2D entities
    points: Arena<Point2D<'ctx>>,
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
        Self {
            ctx,
            solver,
            points,
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
}
