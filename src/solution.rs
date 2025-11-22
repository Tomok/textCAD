//! Solution extraction from Z3 models
//!
//! This module provides functionality for extracting concrete geometric
//! coordinates from Z3 models after constraint solving.

use std::collections::HashMap;
use z3::{Model, ast::Real};

use crate::entities::PointId;
use crate::error::{Result, TextCadError};

/// Solution containing extracted coordinates from a Z3 model
///
/// The Solution struct caches extracted coordinate values and provides
/// methods for accessing them by entity ID.
#[derive(Debug)]
pub struct Solution<'ctx> {
    /// Z3 model containing the satisfying variable assignments
    model: Model<'ctx>,
    /// Cached point coordinates extracted from the model
    point_coords: HashMap<PointId, (f64, f64)>,
}

impl<'ctx> Solution<'ctx> {
    /// Create a new solution from a Z3 model
    ///
    /// # Arguments
    /// * `model` - Z3 model containing variable assignments
    ///
    /// # Example
    /// ```
    /// use z3::{Config, Context, SatResult};
    /// use textcad::sketch::Sketch;
    /// use textcad::solution::Solution;
    /// use textcad::constraints::FixedPositionConstraint;
    /// use textcad::units::Length;
    ///
    /// let cfg = Config::new();
    /// let ctx = Context::new(&cfg);
    /// let mut sketch = Sketch::new(&ctx);
    /// let p1 = sketch.add_point(Some("p1".to_string()));
    /// 
    /// let constraint = FixedPositionConstraint::new(
    ///     p1,
    ///     Length::meters(1.0),
    ///     Length::meters(2.0),
    /// );
    /// sketch.add_constraint(constraint);
    /// 
    /// if let SatResult::Sat = sketch.solve_constraints().unwrap() {
    ///     let model = sketch.solver().get_model().unwrap();
    ///     let solution = Solution::new(model);
    /// }
    /// ```
    pub fn new(model: Model<'ctx>) -> Self {
        Self {
            model,
            point_coords: HashMap::new(),
        }
    }

    /// Extract point coordinates from the Z3 model
    ///
    /// This method evaluates the point's x and y variables in the Z3 model
    /// and converts them to floating-point coordinates.
    ///
    /// # Arguments
    /// * `point_id` - ID of the point to extract coordinates for
    /// * `x_var` - Z3 Real variable representing the x coordinate
    /// * `y_var` - Z3 Real variable representing the y coordinate
    ///
    /// # Returns
    /// Tuple of (x, y) coordinates in meters
    ///
    /// # Example
    /// ```no_run
    /// use textcad::solution::Solution;
    /// use textcad::entities::PointId;
    /// use generational_arena::Index;
    /// 
    /// # let model = todo!(); // Get Z3 model from somewhere
    /// # let x_var = todo!(); // Get x variable
    /// # let y_var = todo!(); // Get y variable
    /// let mut solution = Solution::new(model);
    /// let point_id = PointId(Index::from_raw_parts(0, 0));
    /// let (x, y) = solution.extract_point_coordinates(point_id, &x_var, &y_var).unwrap();
    /// println!("Point is at ({:.3}, {:.3})", x, y);
    /// ```
    pub fn extract_point_coordinates(
        &mut self, 
        point_id: PointId, 
        x_var: &Real<'ctx>, 
        y_var: &Real<'ctx>
    ) -> Result<(f64, f64)> {
        // Check if we've already cached this point's coordinates
        if let Some(&coords) = self.point_coords.get(&point_id) {
            return Ok(coords);
        }

        // Evaluate the variables in the model
        let x_value = self.model.eval(x_var, true)
            .ok_or_else(|| TextCadError::SolutionError("Failed to evaluate x coordinate".to_string()))?;
        let y_value = self.model.eval(y_var, true)
            .ok_or_else(|| TextCadError::SolutionError("Failed to evaluate y coordinate".to_string()))?;

        // Convert rational values to floating point
        let x = rational_to_f64(x_value.into())?;
        let y = rational_to_f64(y_value.into())?;

        // Cache the result
        self.point_coords.insert(point_id, (x, y));

        Ok((x, y))
    }

    /// Get cached point coordinates by ID
    ///
    /// Returns the coordinates if they have been previously extracted,
    /// otherwise returns an error.
    ///
    /// # Arguments
    /// * `point_id` - ID of the point to get coordinates for
    ///
    /// # Returns
    /// Tuple of (x, y) coordinates in meters
    pub fn get_point_coordinates(&self, point_id: PointId) -> Result<(f64, f64)> {
        self.point_coords.get(&point_id)
            .copied()
            .ok_or_else(|| TextCadError::SolutionError(
                format!("Point {:?} coordinates not extracted", point_id)
            ))
    }

    /// Get all cached point coordinates
    ///
    /// Returns a reference to the internal HashMap containing all
    /// extracted point coordinates.
    pub fn all_point_coordinates(&self) -> &HashMap<PointId, (f64, f64)> {
        &self.point_coords
    }

    /// Get the underlying Z3 model
    ///
    /// Provides access to the raw Z3 model for advanced use cases.
    pub fn model(&self) -> &Model<'ctx> {
        &self.model
    }
}

/// Convert a Z3 Real AST node to an f64 value
///
/// This function extracts the rational number from a Z3 Real and converts
/// it to a floating-point value.
///
/// # Arguments
/// * `ast` - Z3 Dynamic AST node to convert
///
/// # Returns
/// Floating-point value corresponding to the rational
fn rational_to_f64(ast: z3::ast::Dynamic) -> Result<f64> {
    // Try to interpret as a real/rational number
    if let Some(real_ast) = ast.as_real() {
        if let Some((numerator, denominator)) = real_ast.as_real() {
            if denominator == 0 {
                return Err(TextCadError::SolutionError("Division by zero in rational".to_string()));
            }
            Ok(numerator as f64 / denominator as f64)
        } else {
            Err(TextCadError::SolutionError("Failed to extract rational value".to_string()))
        }
    } else {
        Err(TextCadError::SolutionError("AST is not a real number".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::PointId;
    use generational_arena::Index;
    use z3::{Config, Context, SatResult, Solver};
    use z3::ast::{Ast, Real};

    #[test]
    fn test_solution_creation() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        // Create a simple equation: x = 5
        let x = Real::new_const(&ctx, "x");
        let five = Real::from_real(&ctx, 5, 1);
        solver.assert(&x._eq(&five));

        assert_eq!(solver.check(), SatResult::Sat);
        let model = solver.get_model().unwrap();
        
        let solution = Solution::new(model);
        assert_eq!(solution.point_coords.len(), 0); // No points extracted yet
    }

    #[test]
    fn test_point_coordinate_extraction() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        // Create point variables and fix them
        let x = Real::new_const(&ctx, "x");
        let y = Real::new_const(&ctx, "y");
        let three = Real::from_real(&ctx, 3, 1);
        let four = Real::from_real(&ctx, 4, 1);
        
        solver.assert(&x._eq(&three));
        solver.assert(&y._eq(&four));

        assert_eq!(solver.check(), SatResult::Sat);
        let model = solver.get_model().unwrap();
        
        let mut solution = Solution::new(model);
        let point_id = PointId(Index::from_raw_parts(0, 0));
        
        let (px, py) = solution.extract_point_coordinates(point_id, &x, &y).unwrap();
        assert!((px - 3.0).abs() < 1e-6);
        assert!((py - 4.0).abs() < 1e-6);

        // Test cached access
        let (px2, py2) = solution.get_point_coordinates(point_id).unwrap();
        assert_eq!(px, px2);
        assert_eq!(py, py2);
    }

    #[test]
    fn test_rational_to_f64_conversion() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);

        // Test simple integer
        let three = Real::from_real(&ctx, 3, 1);
        let three_eval = three.simplify();
        let value = rational_to_f64(three_eval.into()).unwrap();
        assert!((value - 3.0).abs() < 1e-10);

        // Test proper fraction
        let half = Real::from_real(&ctx, 1, 2);
        let half_eval = half.simplify();
        let value = rational_to_f64(half_eval.into()).unwrap();
        assert!((value - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_get_nonexistent_point() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        let x = Real::new_const(&ctx, "x");
        let five = Real::from_real(&ctx, 5, 1);
        solver.assert(&x._eq(&five));

        assert_eq!(solver.check(), SatResult::Sat);
        let model = solver.get_model().unwrap();
        let solution = Solution::new(model);
        
        let point_id = PointId(Index::from_raw_parts(999, 999));
        let result = solution.get_point_coordinates(point_id);
        
        assert!(result.is_err());
        assert!(matches!(result, Err(TextCadError::SolutionError(_))));
    }
}