//! Property-based tests for constraints using proptest
//!
//! These tests verify that constraints hold for randomly generated inputs,
//! ensuring robustness and correctness across a wide range of scenarios.

#[cfg(test)]
mod tests {
    use super::super::{CoincidentPointsConstraint, FixedPositionConstraint};
    use crate::sketch::Sketch;
    use crate::units::Length;
    use proptest::prelude::*;
    use z3::{Config, Context};

    // Property test: Fixed position constraint always produces the correct coordinates
    proptest! {
        #[test]
        fn prop_fixed_position_constraint(
            x_meters in -100.0f64..100.0f64,
            y_meters in -100.0f64..100.0f64
        ) {
            let cfg = Config::new();
            let ctx = Context::new(&cfg);
            let mut sketch = Sketch::new(&ctx);
            
            let p1 = sketch.add_point(Some("p1".to_string()));
            let constraint = FixedPositionConstraint::new(
                p1,
                Length::meters(x_meters),
                Length::meters(y_meters),
            );
            sketch.add_constraint(constraint);
            
            let solution = sketch.solve_and_extract()?;
            let (ex, ey) = solution.get_point_coordinates(p1)?;
            
            prop_assert!((ex - x_meters).abs() < 1e-6, 
                "Expected x: {}, got: {}", x_meters, ex);
            prop_assert!((ey - y_meters).abs() < 1e-6,
                "Expected y: {}, got: {}", y_meters, ey);
        }
    }

    // Property test: Coincident points always have the same coordinates
    proptest! {
        #[test] 
        fn prop_coincident_points_same_coordinates(
            x_meters in -100.0f64..100.0f64,
            y_meters in -100.0f64..100.0f64
        ) {
            let cfg = Config::new();
            let ctx = Context::new(&cfg);
            let mut sketch = Sketch::new(&ctx);
            
            let p1 = sketch.add_point(Some("p1".to_string()));
            let p2 = sketch.add_point(Some("p2".to_string()));
            
            // Fix p1 at random position
            sketch.add_constraint(FixedPositionConstraint::new(
                p1,
                Length::meters(x_meters),
                Length::meters(y_meters),
            ));
            
            // Make p2 coincident with p1
            sketch.add_constraint(CoincidentPointsConstraint::new(p1, p2));
            
            let solution = sketch.solve_and_extract()?;
            let (x1, y1) = solution.get_point_coordinates(p1)?;
            let (x2, y2) = solution.get_point_coordinates(p2)?;
            
            prop_assert!((x1 - x2).abs() < 1e-6, 
                "Points should have same x: {} vs {}", x1, x2);
            prop_assert!((y1 - y2).abs() < 1e-6,
                "Points should have same y: {} vs {}", y1, y2);
        }
    }

    // Property test: Unit conversions work correctly
    proptest! {
        #[test]
        fn prop_unit_conversions_work(
            meters in 0.001f64..100.0f64
        ) {
            let cfg = Config::new();
            let ctx = Context::new(&cfg);
            let mut sketch = Sketch::new(&ctx);
            
            let p1 = sketch.add_point(Some("p1".to_string()));
            
            // Create constraint using millimeters (should be converted to meters)
            let constraint = FixedPositionConstraint::new(
                p1,
                Length::millimeters(meters * 1000.0), // Convert to mm
                Length::centimeters(meters * 100.0),  // Convert to cm
            );
            sketch.add_constraint(constraint);
            
            let solution = sketch.solve_and_extract()?;
            let (x, y) = solution.get_point_coordinates(p1)?;
            
            prop_assert!((x - meters).abs() < 1e-6,
                "X conversion failed: expected {}, got {}", meters, x);
            prop_assert!((y - meters).abs() < 1e-6,
                "Y conversion failed: expected {}, got {}", meters, y);
        }
    }

    // Property test: Multiple constraints are consistent
    proptest! {
        #[test]
        fn prop_constraint_consistency(
            x1 in -50.0f64..50.0f64,
            y1 in -50.0f64..50.0f64
        ) {
            let cfg = Config::new();
            let ctx = Context::new(&cfg);
            let mut sketch = Sketch::new(&ctx);
            
            let p1 = sketch.add_point(Some("p1".to_string()));
            let p2 = sketch.add_point(Some("p2".to_string()));
            let p3 = sketch.add_point(Some("p3".to_string()));
            
            // Create a chain: p1 fixed -> p2 coincident with p1 -> p3 coincident with p2
            sketch.add_constraint(FixedPositionConstraint::new(
                p1,
                Length::meters(x1),
                Length::meters(y1),
            ));
            sketch.add_constraint(CoincidentPointsConstraint::new(p1, p2));
            sketch.add_constraint(CoincidentPointsConstraint::new(p2, p3));
            
            let solution = sketch.solve_and_extract()?;
            let (px1, py1) = solution.get_point_coordinates(p1)?;
            let (px2, py2) = solution.get_point_coordinates(p2)?;
            let (px3, py3) = solution.get_point_coordinates(p3)?;
            
            // All points should be at the same location as p1
            prop_assert!((px1 - x1).abs() < 1e-6, "P1 position incorrect");
            prop_assert!((py1 - y1).abs() < 1e-6, "P1 position incorrect");
            prop_assert!((px1 - px2).abs() < 1e-6, "P1 and P2 should be coincident");
            prop_assert!((py1 - py2).abs() < 1e-6, "P1 and P2 should be coincident");
            prop_assert!((px2 - px3).abs() < 1e-6, "P2 and P3 should be coincident");
            prop_assert!((py2 - py3).abs() < 1e-6, "P2 and P3 should be coincident");
        }
    }

    // Property test: Constraint order doesn't matter
    proptest! {
        #[test]
        fn prop_constraint_order_independence(
            x in -10.0f64..10.0f64,
            y in -10.0f64..10.0f64,
            apply_coincident_first in any::<bool>()
        ) {
            let cfg = Config::new();
            let ctx = Context::new(&cfg);
            let mut sketch = Sketch::new(&ctx);
            
            let p1 = sketch.add_point(Some("p1".to_string()));
            let p2 = sketch.add_point(Some("p2".to_string()));
            
            let fix_constraint = FixedPositionConstraint::new(
                p1,
                Length::meters(x),
                Length::meters(y),
            );
            let coincident_constraint = CoincidentPointsConstraint::new(p1, p2);
            
            if apply_coincident_first {
                sketch.add_constraint(coincident_constraint);
                sketch.add_constraint(fix_constraint);
            } else {
                sketch.add_constraint(fix_constraint);
                sketch.add_constraint(coincident_constraint);
            }
            
            let solution = sketch.solve_and_extract()?;
            let (x1, y1) = solution.get_point_coordinates(p1)?;
            let (x2, y2) = solution.get_point_coordinates(p2)?;
            
            prop_assert!((x1 - x).abs() < 1e-6, "P1 position incorrect");
            prop_assert!((y1 - y).abs() < 1e-6, "P1 position incorrect");
            prop_assert!((x1 - x2).abs() < 1e-6, "Points should be coincident");
            prop_assert!((y1 - y2).abs() < 1e-6, "Points should be coincident");
        }
    }

    // Property test: Solution extraction is idempotent
    proptest! {
        #[test]
        fn prop_solution_extraction_idempotent(
            x in -10.0f64..10.0f64,
            y in -10.0f64..10.0f64
        ) {
            let cfg = Config::new();
            let ctx = Context::new(&cfg);
            let mut sketch = Sketch::new(&ctx);
            
            let p1 = sketch.add_point(Some("p1".to_string()));
            sketch.add_constraint(FixedPositionConstraint::new(
                p1,
                Length::meters(x),
                Length::meters(y),
            ));
            
            let solution = sketch.solve_and_extract()?;
            
            // Extract coordinates multiple times
            let (x1, y1) = solution.get_point_coordinates(p1)?;
            let (x2, y2) = solution.get_point_coordinates(p1)?;
            let (x3, y3) = solution.get_point_coordinates(p1)?;
            
            prop_assert!((x1 - x2).abs() < 1e-15, "Repeated extraction should be identical");
            prop_assert!((y1 - y2).abs() < 1e-15, "Repeated extraction should be identical");
            prop_assert!((x2 - x3).abs() < 1e-15, "Repeated extraction should be identical");
            prop_assert!((y2 - y3).abs() < 1e-15, "Repeated extraction should be identical");
        }
    }
}