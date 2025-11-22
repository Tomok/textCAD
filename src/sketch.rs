//! Sketch management with Z3 integration
//!
//! The sketch module provides the main interface for creating and managing
//! geometric entities and constraints using Z3 as the underlying solver.

use z3::{Context, SatResult, Solver};

use crate::error::{Result, TextCadError};

/// Main sketch structure that manages geometric entities and constraints
///
/// A sketch wraps a Z3 context and solver, providing the foundation for
/// constraint-based geometric modeling.
pub struct Sketch<'ctx> {
    ctx: &'ctx Context,
    solver: Solver<'ctx>,
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
        Self { ctx, solver }
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
}
