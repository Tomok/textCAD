//! Solution extraction from Z3 models
//!
//! This module provides functionality for extracting concrete geometric
//! coordinates from Z3 models after constraint solving.

use std::collections::HashMap;
use z3::{Model, ast::Real};

use crate::entities::PointId;
use crate::entity::{CircleId, LineId};
use crate::error::{Result, TextCadError};

/// Solution containing extracted coordinates and parameters from a Z3 model
///
/// The Solution struct caches extracted coordinate values and provides
/// methods for accessing them by entity ID. It supports extensible entity
/// extraction for points, lines, circles, and parametric variables.
#[derive(Debug)]
pub struct Solution<'ctx> {
    /// Z3 model containing the satisfying variable assignments
    model: Model<'ctx>,
    /// Cached point coordinates extracted from the model (x, y in meters)
    point_coords: HashMap<PointId, (f64, f64)>,
    /// Cached line parameters extracted from the model
    line_params: HashMap<LineId, LineParameters>,
    /// Cached circle parameters extracted from the model
    circle_params: HashMap<CircleId, CircleParameters>,
    /// Cached parameter variables for parametric constraints
    parameter_vars: HashMap<String, f64>,
}

/// Parameters extracted for a line entity
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineParameters {
    /// Start point coordinates (x, y in meters)
    pub start: (f64, f64),
    /// End point coordinates (x, y in meters)
    pub end: (f64, f64),
    /// Computed length in meters
    pub length: f64,
    /// Computed angle in radians (from start to end)
    pub angle: f64,
}

/// Parameters extracted for a circle entity
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CircleParameters {
    /// Center point coordinates (x, y in meters)
    pub center: (f64, f64),
    /// Radius in meters
    pub radius: f64,
    /// Computed circumference in meters
    pub circumference: f64,
    /// Computed area in square meters
    pub area: f64,
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
            line_params: HashMap::new(),
            circle_params: HashMap::new(),
            parameter_vars: HashMap::new(),
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
        y_var: &Real<'ctx>,
    ) -> Result<(f64, f64)> {
        // Check if we've already cached this point's coordinates
        if let Some(&coords) = self.point_coords.get(&point_id) {
            return Ok(coords);
        }

        // Evaluate the variables in the model
        let x_value = self.model.eval(x_var, true).ok_or_else(|| {
            TextCadError::SolutionError("Failed to evaluate x coordinate".to_string())
        })?;
        let y_value = self.model.eval(y_var, true).ok_or_else(|| {
            TextCadError::SolutionError("Failed to evaluate y coordinate".to_string())
        })?;

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
        self.point_coords.get(&point_id).copied().ok_or_else(|| {
            TextCadError::SolutionError(format!("Point {:?} coordinates not extracted", point_id))
        })
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

    /// Extract parameter variable value from the Z3 model
    ///
    /// This method evaluates a named parameter variable (e.g., from parametric constraints)
    /// and converts it to a floating-point value.
    ///
    /// # Arguments
    /// * `var_name` - Name of the parameter variable
    /// * `param_var` - Z3 Real variable representing the parameter
    ///
    /// # Returns
    /// Floating-point value of the parameter
    ///
    /// # Example
    /// ```no_run
    /// # use textcad::solution::Solution;
    /// # let model = todo!(); // Get Z3 model from somewhere
    /// # let t_var = todo!(); // Get parameter variable
    /// let mut solution = Solution::new(model);
    /// let t_value = solution.extract_parameter("t_line_point", &t_var).unwrap();
    /// println!("Parameter t = {:.6}", t_value);
    /// ```
    pub fn extract_parameter(&mut self, var_name: &str, param_var: &Real<'ctx>) -> Result<f64> {
        // Check if we've already cached this parameter
        if let Some(&value) = self.parameter_vars.get(var_name) {
            return Ok(value);
        }

        // Evaluate the parameter variable in the model
        let param_value = self.model.eval(param_var, true).ok_or_else(|| {
            TextCadError::SolutionError(format!("Failed to evaluate parameter '{}'", var_name))
        })?;

        // Convert to floating point with enhanced error handling
        let value = rational_to_f64_enhanced(param_value.into(), var_name)?;

        // Cache the result
        self.parameter_vars.insert(var_name.to_string(), value);

        Ok(value)
    }

    /// Get cached parameter value by name
    ///
    /// Returns the parameter value if it has been previously extracted,
    /// otherwise returns an error.
    ///
    /// # Arguments
    /// * `var_name` - Name of the parameter variable
    ///
    /// # Returns
    /// Floating-point value of the parameter
    pub fn get_parameter(&self, var_name: &str) -> Result<f64> {
        self.parameter_vars.get(var_name).copied().ok_or_else(|| {
            TextCadError::SolutionError(format!("Parameter '{}' not extracted", var_name))
        })
    }

    /// Get all cached parameter variables
    ///
    /// Returns a reference to the internal HashMap containing all
    /// extracted parameter values.
    pub fn all_parameters(&self) -> &HashMap<String, f64> {
        &self.parameter_vars
    }

    /// Extract line parameters from the Z3 model
    ///
    /// This method calculates comprehensive line parameters including
    /// endpoints, length, and angle.
    ///
    /// # Arguments
    /// * `line_id` - ID of the line to extract parameters for
    /// * `start_coords` - Coordinates of the start point
    /// * `end_coords` - Coordinates of the end point
    ///
    /// # Returns
    /// LineParameters struct with computed values
    pub fn extract_line_parameters(
        &mut self,
        line_id: LineId,
        start_coords: (f64, f64),
        end_coords: (f64, f64),
    ) -> Result<LineParameters> {
        // Check if we've already cached this line's parameters
        if let Some(&params) = self.line_params.get(&line_id) {
            return Ok(params);
        }

        let (x1, y1) = start_coords;
        let (x2, y2) = end_coords;

        // Calculate line parameters
        let dx = x2 - x1;
        let dy = y2 - y1;
        let length = (dx * dx + dy * dy).sqrt();
        let angle = dy.atan2(dx); // Angle from start to end in radians

        let params = LineParameters {
            start: start_coords,
            end: end_coords,
            length,
            angle,
        };

        // Cache the result
        self.line_params.insert(line_id, params);

        Ok(params)
    }

    /// Get cached line parameters by ID
    ///
    /// Returns the parameters if they have been previously extracted,
    /// otherwise returns an error.
    ///
    /// # Arguments
    /// * `line_id` - ID of the line to get parameters for
    ///
    /// # Returns
    /// LineParameters struct with all computed values
    pub fn get_line_parameters(&self, line_id: LineId) -> Result<LineParameters> {
        self.line_params.get(&line_id).copied().ok_or_else(|| {
            TextCadError::SolutionError(format!("Line {:?} parameters not extracted", line_id))
        })
    }

    /// Extract circle parameters from the Z3 model
    ///
    /// This method calculates comprehensive circle parameters including
    /// center, radius, circumference, and area.
    ///
    /// # Arguments
    /// * `circle_id` - ID of the circle to extract parameters for
    /// * `center_coords` - Coordinates of the center point
    /// * `radius_var` - Z3 Real variable representing the radius
    ///
    /// # Returns
    /// CircleParameters struct with computed values
    pub fn extract_circle_parameters(
        &mut self,
        circle_id: CircleId,
        center_coords: (f64, f64),
        radius_var: &Real<'ctx>,
    ) -> Result<CircleParameters> {
        // Check if we've already cached this circle's parameters
        if let Some(&params) = self.circle_params.get(&circle_id) {
            return Ok(params);
        }

        // Extract radius from Z3 model
        let radius_value = self.model.eval(radius_var, true).ok_or_else(|| {
            TextCadError::SolutionError(format!(
                "Failed to evaluate radius for circle {:?}",
                circle_id
            ))
        })?;

        let radius = rational_to_f64_enhanced(radius_value.into(), "radius")?;

        // Calculate derived parameters
        let circumference = 2.0 * std::f64::consts::PI * radius;
        let area = std::f64::consts::PI * radius * radius;

        let params = CircleParameters {
            center: center_coords,
            radius,
            circumference,
            area,
        };

        // Cache the result
        self.circle_params.insert(circle_id, params);

        Ok(params)
    }

    /// Get cached circle parameters by ID
    ///
    /// Returns the parameters if they have been previously extracted,
    /// otherwise returns an error.
    ///
    /// # Arguments
    /// * `circle_id` - ID of the circle to get parameters for
    ///
    /// # Returns
    /// CircleParameters struct with all computed values
    pub fn get_circle_parameters(&self, circle_id: CircleId) -> Result<CircleParameters> {
        self.circle_params.get(&circle_id).copied().ok_or_else(|| {
            TextCadError::SolutionError(format!("Circle {:?} parameters not extracted", circle_id))
        })
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
    rational_to_f64_enhanced(ast, "coordinate")
}

/// Enhanced rational to f64 conversion with better error context
///
/// This function provides enhanced error reporting and handles edge cases
/// more robustly than the basic conversion.
///
/// # Arguments
/// * `ast` - Z3 Dynamic AST node to convert
/// * `context` - Context string for better error messages
///
/// # Returns
/// Floating-point value with enhanced error handling
fn rational_to_f64_enhanced(ast: z3::ast::Dynamic, context: &str) -> Result<f64> {
    // Try to interpret as a real/rational number
    if let Some(real_ast) = ast.as_real() {
        if let Some((numerator, denominator)) = real_ast.as_real() {
            if denominator == 0 {
                return Err(TextCadError::SolutionError(format!(
                    "Division by zero in {} rational: {}/{}",
                    context, numerator, denominator
                )));
            }

            // Check for potential overflow or precision loss
            let result = numerator as f64 / denominator as f64;

            // Validate the result is a finite number
            if !result.is_finite() {
                return Err(TextCadError::SolutionError(format!(
                    "Non-finite result in {} conversion: {}/{} = {}",
                    context, numerator, denominator, result
                )));
            }

            // Check for extremely small denominators that might cause precision issues
            if denominator.abs() < 1000 && numerator.abs() > 1_000_000_000 {
                eprintln!(
                    "Warning: Potential precision loss in {} conversion: {}/{}",
                    context, numerator, denominator
                );
            }

            Ok(result)
        } else {
            Err(TextCadError::SolutionError(format!(
                "Failed to extract rational value for {}: AST does not contain rational",
                context
            )))
        }
    } else {
        Err(TextCadError::SolutionError(format!(
            "AST is not a real number for {}: got {:?}",
            context, ast
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::PointId;
    use generational_arena::Index;
    use z3::ast::{Ast, Real};
    use z3::{Config, Context, SatResult, Solver};

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

        let (px, py) = solution
            .extract_point_coordinates(point_id, &x, &y)
            .unwrap();
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

    #[test]
    fn test_parameter_extraction() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        // Create a parameter variable and set it to 0.75
        let t = Real::new_const(&ctx, "t");
        let three_fourths = Real::from_real(&ctx, 3, 4);
        solver.assert(&t._eq(&three_fourths));

        assert_eq!(solver.check(), SatResult::Sat);
        let model = solver.get_model().unwrap();
        let mut solution = Solution::new(model);

        // Extract parameter
        let t_value = solution.extract_parameter("t", &t).unwrap();
        assert!((t_value - 0.75).abs() < 1e-6);

        // Test cached access
        let t_value_cached = solution.get_parameter("t").unwrap();
        assert_eq!(t_value, t_value_cached);
    }

    #[test]
    fn test_line_parameters_calculation() {
        use crate::entity::LineId;

        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        // Add a trivial constraint to get a satisfiable model
        let x = Real::new_const(&ctx, "dummy");
        let zero = Real::from_real(&ctx, 0, 1);
        solver.assert(&x._eq(&zero));

        assert_eq!(solver.check(), SatResult::Sat);
        let model = solver.get_model().unwrap();
        let mut solution = Solution::new(model);

        let line_id = LineId(Index::from_raw_parts(0, 0));
        let start = (0.0, 0.0);
        let end = (3.0, 4.0);

        let params = solution
            .extract_line_parameters(line_id, start, end)
            .unwrap();

        assert_eq!(params.start, start);
        assert_eq!(params.end, end);
        assert!((params.length - 5.0).abs() < 1e-6); // 3-4-5 triangle
        assert!((params.angle - (4.0_f64.atan2(3.0))).abs() < 1e-6);

        // Test cached access
        let params_cached = solution.get_line_parameters(line_id).unwrap();
        assert_eq!(params.length, params_cached.length);
    }

    #[test]
    fn test_circle_parameters_calculation() {
        use crate::entity::CircleId;

        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        // Create a radius variable and set it to 2.0
        let radius = Real::new_const(&ctx, "radius");
        let two = Real::from_real(&ctx, 2, 1);
        solver.assert(&radius._eq(&two));

        assert_eq!(solver.check(), SatResult::Sat);
        let model = solver.get_model().unwrap();
        let mut solution = Solution::new(model);

        let circle_id = CircleId(Index::from_raw_parts(0, 0));
        let center = (1.0, 1.0);

        let params = solution
            .extract_circle_parameters(circle_id, center, &radius)
            .unwrap();

        assert_eq!(params.center, center);
        assert!((params.radius - 2.0).abs() < 1e-6);
        assert!((params.circumference - (2.0 * std::f64::consts::PI * 2.0)).abs() < 1e-6);
        assert!((params.area - (std::f64::consts::PI * 4.0)).abs() < 1e-6);

        // Test cached access
        let params_cached = solution.get_circle_parameters(circle_id).unwrap();
        assert_eq!(params.radius, params_cached.radius);
    }

    #[test]
    fn test_enhanced_rational_conversion_errors() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);

        // Test integer AST (should work)
        let two = Real::from_real(&ctx, 2, 1);
        let two_ast = two.simplify();
        let result = rational_to_f64_enhanced(two_ast.into(), "test");
        assert!(result.is_ok());
        assert!((result.unwrap() - 2.0).abs() < 1e-10);

        // Create an AST that's not a real number (should fail)
        let bool_ast = z3::ast::Bool::new_const(&ctx, "test_bool");
        let result = rational_to_f64_enhanced(bool_ast.into(), "test_bool");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not a real number")
        );
    }

    #[test]
    fn test_get_nonexistent_parameter() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        // Add a trivial constraint to get a satisfiable model
        let x = Real::new_const(&ctx, "dummy");
        let zero = Real::from_real(&ctx, 0, 1);
        solver.assert(&x._eq(&zero));

        assert_eq!(solver.check(), SatResult::Sat);
        let model = solver.get_model().unwrap();
        let solution = Solution::new(model);

        let result = solution.get_parameter("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not extracted"));
    }

    #[test]
    fn test_all_parameters_access() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        let t1 = Real::new_const(&ctx, "t1");
        let t2 = Real::new_const(&ctx, "t2");
        let half = Real::from_real(&ctx, 1, 2);
        let quarter = Real::from_real(&ctx, 1, 4);

        solver.assert(&t1._eq(&half));
        solver.assert(&t2._eq(&quarter));

        assert_eq!(solver.check(), SatResult::Sat);
        let model = solver.get_model().unwrap();
        let mut solution = Solution::new(model);

        solution.extract_parameter("t1", &t1).unwrap();
        solution.extract_parameter("t2", &t2).unwrap();

        let all_params = solution.all_parameters();
        assert_eq!(all_params.len(), 2);
        assert!(all_params.contains_key("t1"));
        assert!(all_params.contains_key("t2"));
        assert!((all_params["t1"] - 0.5).abs() < 1e-6);
        assert!((all_params["t2"] - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_line_parameters_with_zero_length() {
        use crate::entity::LineId;

        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let solver = Solver::new(&ctx);

        // Add a trivial constraint to get a satisfiable model
        let x = Real::new_const(&ctx, "dummy");
        let zero = Real::from_real(&ctx, 0, 1);
        solver.assert(&x._eq(&zero));

        assert_eq!(solver.check(), SatResult::Sat);
        let model = solver.get_model().unwrap();
        let mut solution = Solution::new(model);

        let line_id = LineId(Index::from_raw_parts(0, 0));
        let start = (2.0, 3.0);
        let end = (2.0, 3.0); // Same point

        let params = solution
            .extract_line_parameters(line_id, start, end)
            .unwrap();

        assert_eq!(params.start, start);
        assert_eq!(params.end, end);
        assert!((params.length - 0.0).abs() < 1e-10);
        // Angle of zero-length line is 0.0
        assert!((params.angle - 0.0).abs() < 1e-10);
    }

    // Property-based tests using proptest
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_rational_conversion_preserves_values(
            numerator in -10000i32..10000i32,
            denominator in 1i32..10000i32,
        ) {
            let cfg = Config::new();
            let ctx = Context::new(&cfg);

            let rational = Real::from_real(&ctx, numerator, denominator);
            let ast = rational.simplify();

            let result = rational_to_f64_enhanced(ast.into(), "test");
            prop_assert!(result.is_ok());

            let value = result.unwrap();
            let expected = numerator as f64 / denominator as f64;

            prop_assert!((value - expected).abs() < 1e-12);
            prop_assert!(value.is_finite());
        }

        #[test]
        fn prop_line_parameters_geometric_properties(
            x1 in -50.0f64..50.0f64,
            y1 in -50.0f64..50.0f64,
            x2 in -50.0f64..50.0f64,
            y2 in -50.0f64..50.0f64,
        ) {
            let cfg = Config::new();
            let ctx = Context::new(&cfg);
            let solver = Solver::new(&ctx);

            // Add a trivial constraint to get a satisfiable model
            let dummy = Real::new_const(&ctx, "dummy");
            let zero = Real::from_real(&ctx, 0, 1);
            solver.assert(&dummy._eq(&zero));

            prop_assert_eq!(solver.check(), SatResult::Sat);
            let model = solver.get_model().unwrap();
            let mut solution = Solution::new(model);

            use crate::entity::LineId;
            let line_id = LineId(Index::from_raw_parts(0, 0));

            let params = solution.extract_line_parameters(line_id, (x1, y1), (x2, y2))?;

            // Length is always non-negative
            prop_assert!(params.length >= 0.0);

            // Angle is in [-π, π]
            prop_assert!(params.angle >= -std::f64::consts::PI);
            prop_assert!(params.angle <= std::f64::consts::PI);

            // Length calculation is correct
            let dx = x2 - x1;
            let dy = y2 - y1;
            let expected_length = (dx * dx + dy * dy).sqrt();
            prop_assert!((params.length - expected_length).abs() < 1e-12);
        }

        #[test]
        fn prop_parameter_extraction_consistency(
            numerator in -1000i32..1000i32,
            denominator in 1i32..1000i32,
        ) {
            let cfg = Config::new();
            let ctx = Context::new(&cfg);
            let solver = Solver::new(&ctx);

            let param = Real::new_const(&ctx, "test_param");
            let rational_val = Real::from_real(&ctx, numerator, denominator);

            solver.assert(&param._eq(&rational_val));

            if solver.check() == SatResult::Sat {
                let model = solver.get_model().unwrap();
                let mut solution = Solution::new(model);

                let extracted1 = solution.extract_parameter("test_param", &param)?;
                let extracted2 = solution.extract_parameter("test_param", &param)?;

                // Extractions should be idempotent
                prop_assert_eq!(extracted1, extracted2);

                // Should match expected value
                let expected = numerator as f64 / denominator as f64;
                prop_assert!((extracted1 - expected).abs() < 1e-12);

                // Cached access should work
                let cached = solution.get_parameter("test_param")?;
                prop_assert_eq!(extracted1, cached);
            }
        }

        #[test]
        fn prop_circle_parameters_positive_properties(
            cx in -25.0f64..25.0f64,
            cy in -25.0f64..25.0f64,
            radius_num in 1i32..1000i32,
            radius_den in 1i32..100i32,
        ) {
            let cfg = Config::new();
            let ctx = Context::new(&cfg);
            let solver = Solver::new(&ctx);

            let radius_var = Real::new_const(&ctx, "radius");
            let radius_val = Real::from_real(&ctx, radius_num, radius_den);
            solver.assert(&radius_var._eq(&radius_val));

            if solver.check() == SatResult::Sat {
                let model = solver.get_model().unwrap();
                let mut solution = Solution::new(model);

                use crate::entity::CircleId;
                let circle_id = CircleId(Index::from_raw_parts(0, 0));

                let params = solution.extract_circle_parameters(circle_id, (cx, cy), &radius_var)?;

                // All parameters should be positive for positive radius
                prop_assert!(params.radius > 0.0);
                prop_assert!(params.circumference > 0.0);
                prop_assert!(params.area > 0.0);

                // Mathematical relationships should hold
                let expected_circumference = 2.0 * std::f64::consts::PI * params.radius;
                let expected_area = std::f64::consts::PI * params.radius * params.radius;

                prop_assert!((params.circumference - expected_circumference).abs() < 1e-10);
                prop_assert!((params.area - expected_area).abs() < 1e-10);
            }
        }
    }
}
