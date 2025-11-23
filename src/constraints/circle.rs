//! Circle-related constraints for geometric modeling
//!
//! Implements constraints that apply to Circle entities, including radius constraints.

use crate::constraint::{Constraint, SketchQuery};
use crate::entity::CircleId;
use crate::error::{Result, TextCadError};
use crate::units::Length;
use z3::ast::{Ast, Real};

/// Constraint that sets the radius of a circle to a specific value
#[derive(Debug, Clone)]
pub struct CircleRadiusConstraint {
    /// Circle to constrain
    pub circle: CircleId,
    /// Target radius for the circle
    pub radius: Length,
}

impl CircleRadiusConstraint {
    /// Create a new circle radius constraint
    pub fn new(circle: CircleId, radius: Length) -> Self {
        Self { circle, radius }
    }
}

impl Constraint for CircleRadiusConstraint {
    fn apply(
        &self,
        context: &z3::Context,
        solver: &z3::Solver,
        sketch: &dyn SketchQuery,
    ) -> Result<()> {
        // Get the circle's radius variable
        let radius_var = sketch.circle_center_and_radius(self.circle).map_err(|_| {
            TextCadError::EntityError(format!("Circle {:?} not found", self.circle))
        })?;

        // Create Z3 constant for target radius (in meters)
        let target_radius_meters = self.radius.to_meters();

        // Convert to rational representation for precision
        // Use 10,000 as denominator for good precision (i32 limits)
        let numerator = (target_radius_meters * 10_000.0).round() as i32;
        let denominator = 10_000i32;

        let target = Real::from_real(context, numerator, denominator);

        // Assert radius equals target
        solver.assert(&radius_var.1._eq(&target));

        Ok(())
    }

    fn description(&self) -> String {
        format!(
            "Circle {:?} has radius {} meters",
            self.circle,
            self.radius.to_meters()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sketch::Sketch;
    use crate::units::Length;
    use z3::{Config, Context};

    #[test]
    fn test_circle_radius_constraint_creation() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        let center = sketch.add_point(None);
        let circle = sketch.add_circle(center, None);

        let constraint = CircleRadiusConstraint::new(circle, Length::meters(2.5));

        assert_eq!(constraint.radius.to_meters(), 2.5);
        assert!(constraint.description().contains("2.5"));
    }

    #[test]
    fn test_circle_radius_constraint_with_millimeters() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        let center = sketch.add_point(None);
        let circle = sketch.add_circle(center, None);

        // 500mm = 0.5m
        let constraint = CircleRadiusConstraint::new(circle, Length::millimeters(500.0));

        assert_eq!(constraint.radius.to_meters(), 0.5);
        assert_eq!(constraint.radius.to_millimeters(), 500.0);
    }
}
