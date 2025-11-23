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

    // Property test: Parallel lines constraint actually produces parallel lines
    // See docs/IGNORED_TESTS.md for details on why this test is ignored
    proptest! {
        #[test]
        #[ignore]
        fn prop_parallel_lines_are_actually_parallel(
            x1 in 1.0f64..5.0f64,
            y1 in 1.0f64..5.0f64,
            x2 in 6.0f64..10.0f64,
            y2 in 1.0f64..5.0f64,
            x3 in 1.0f64..5.0f64,
            y3 in 6.0f64..10.0f64,
            line2_length in 3.0f64..8.0f64
        ) {
            use crate::constraints::line::ParallelLinesConstraint;

            // This setup ensures non-degenerate lines that are well-separated

            let cfg = Config::new();
            let ctx = Context::new(&cfg);
            let mut sketch = Sketch::new(&ctx);

            // Create two lines
            let p1 = sketch.add_point(Some("p1".to_string()));
            let p2 = sketch.add_point(Some("p2".to_string()));
            let p3 = sketch.add_point(Some("p3".to_string()));
            let p4 = sketch.add_point(Some("p4".to_string()));

            let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
            let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

            // Fix line1 endpoints
            sketch.add_constraint(FixedPositionConstraint::new(p1, Length::meters(x1), Length::meters(y1)));
            sketch.add_constraint(FixedPositionConstraint::new(p2, Length::meters(x2), Length::meters(y2)));

            // Fix line2 start point and length
            sketch.add_constraint(FixedPositionConstraint::new(p3, Length::meters(x3), Length::meters(y3)));
            sketch.add_constraint(crate::constraints::LineLengthConstraint::new(line2, Length::meters(line2_length)));

            // Apply parallel constraint
            sketch.add_constraint(ParallelLinesConstraint::new(line1, line2));

            let solution = sketch.solve_and_extract()?;

            // Get all coordinates
            let (px1, py1) = solution.get_point_coordinates(p1)?;
            let (px2, py2) = solution.get_point_coordinates(p2)?;
            let (px3, py3) = solution.get_point_coordinates(p3)?;
            let (px4, py4) = solution.get_point_coordinates(p4)?;

            // Calculate direction vectors
            let line1_dir = (px2 - px1, py2 - py1);
            let line2_dir = (px4 - px3, py4 - py3);

            // Check parallel condition: cross product should be zero
            let cross_product = line1_dir.0 * line2_dir.1 - line1_dir.1 * line2_dir.0;
            prop_assert!(cross_product.abs() < 1e-6,
                "Lines should be parallel, cross product: {}", cross_product);

            // Verify line2 has the correct length
            let computed_length = (line2_dir.0.powi(2) + line2_dir.1.powi(2)).sqrt();
            prop_assert!((computed_length - line2_length).abs() < 1e-6,
                "Line2 length constraint violated: expected {}, got {}", line2_length, computed_length);
        }
    }

    // Property test: Perpendicular lines constraint produces perpendicular lines
    // See docs/IGNORED_TESTS.md for details on why this test is ignored
    proptest! {
        #[test]
        #[ignore]
        fn prop_perpendicular_lines_are_actually_perpendicular(
            x1 in 1.0f64..5.0f64,
            y1 in 1.0f64..5.0f64,
            x2 in 6.0f64..10.0f64,
            y2 in 1.0f64..5.0f64,
            x3 in 1.0f64..5.0f64,
            y3 in 6.0f64..10.0f64,
            line2_length in 3.0f64..8.0f64
        ) {
            use crate::constraints::line::PerpendicularLinesConstraint;

            // This setup ensures non-degenerate lines that are well-separated

            let cfg = Config::new();
            let ctx = Context::new(&cfg);
            let mut sketch = Sketch::new(&ctx);

            // Create two lines
            let p1 = sketch.add_point(Some("p1".to_string()));
            let p2 = sketch.add_point(Some("p2".to_string()));
            let p3 = sketch.add_point(Some("p3".to_string()));
            let p4 = sketch.add_point(Some("p4".to_string()));

            let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
            let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

            // Fix line1 endpoints
            sketch.add_constraint(FixedPositionConstraint::new(p1, Length::meters(x1), Length::meters(y1)));
            sketch.add_constraint(FixedPositionConstraint::new(p2, Length::meters(x2), Length::meters(y2)));

            // Fix line2 start point and length
            sketch.add_constraint(FixedPositionConstraint::new(p3, Length::meters(x3), Length::meters(y3)));
            sketch.add_constraint(crate::constraints::LineLengthConstraint::new(line2, Length::meters(line2_length)));

            // Apply perpendicular constraint
            sketch.add_constraint(PerpendicularLinesConstraint::new(line1, line2));

            let solution = sketch.solve_and_extract()?;

            // Get all coordinates
            let (px1, py1) = solution.get_point_coordinates(p1)?;
            let (px2, py2) = solution.get_point_coordinates(p2)?;
            let (px3, py3) = solution.get_point_coordinates(p3)?;
            let (px4, py4) = solution.get_point_coordinates(p4)?;

            // Calculate direction vectors
            let line1_dir = (px2 - px1, py2 - py1);
            let line2_dir = (px4 - px3, py4 - py3);

            // Check perpendicular condition: dot product should be zero
            let dot_product = line1_dir.0 * line2_dir.0 + line1_dir.1 * line2_dir.1;
            prop_assert!(dot_product.abs() < 1e-6,
                "Lines should be perpendicular, dot product: {}", dot_product);

            // Verify line2 has the correct length
            let computed_length = (line2_dir.0.powi(2) + line2_dir.1.powi(2)).sqrt();
            prop_assert!((computed_length - line2_length).abs() < 1e-6,
                "Line2 length constraint violated: expected {}, got {}", line2_length, computed_length);
        }
    }

    // Property test: Line length constraint always produces correct length
    // See docs/IGNORED_TESTS.md for details on why this test is ignored
    proptest! {
        #[test]
        #[ignore]
        fn prop_line_length_constraint_correctness(
            x1 in 1.0f64..5.0f64,
            y1 in 1.0f64..5.0f64,
            target_length in 3.0f64..8.0f64
        ) {
            use crate::constraints::LineLengthConstraint;

            let cfg = Config::new();
            let ctx = Context::new(&cfg);
            let mut sketch = Sketch::new(&ctx);

            let p1 = sketch.add_point(Some("p1".to_string()));
            let p2 = sketch.add_point(Some("p2".to_string()));
            let line = sketch.add_line(p1, p2, Some("line".to_string()));

            // Fix one endpoint
            sketch.add_constraint(FixedPositionConstraint::new(p1, Length::meters(x1), Length::meters(y1)));

            // Apply length constraint
            sketch.add_constraint(LineLengthConstraint::new(line, Length::meters(target_length)));

            let solution = sketch.solve_and_extract()?;

            let (px1, py1) = solution.get_point_coordinates(p1)?;
            let (px2, py2) = solution.get_point_coordinates(p2)?;

            let computed_length = ((px2 - px1).powi(2) + (py2 - py1).powi(2)).sqrt();
            prop_assert!((computed_length - target_length).abs() < 1e-6,
                "Line length constraint violated: expected {}, got {}", target_length, computed_length);
        }
    }

    // Property test: Chained parallel constraints maintain transitivity
    // See docs/IGNORED_TESTS.md for details on why this test is ignored
    proptest! {
        #[test]
        #[ignore]
        fn prop_parallel_transitivity(
            base_x1 in 1.0f64..3.0f64,
            base_y1 in 1.0f64..3.0f64,
            base_x2 in 4.0f64..6.0f64,
            base_y2 in 1.0f64..3.0f64,
            pos2_x in 1.0f64..3.0f64,
            pos2_y in 4.0f64..6.0f64,
            pos3_x in 4.0f64..6.0f64,
            pos3_y in 7.0f64..9.0f64,
            length2 in 2.0f64..4.0f64,
            length3 in 2.0f64..4.0f64
        ) {
            use crate::constraints::line::ParallelLinesConstraint;

            // All points are now well-separated by design

            let cfg = Config::new();
            let ctx = Context::new(&cfg);
            let mut sketch = Sketch::new(&ctx);

            // Create three lines: A || B and B || C, so A || C by transitivity
            let p1 = sketch.add_point(Some("p1".to_string()));
            let p2 = sketch.add_point(Some("p2".to_string()));
            let p3 = sketch.add_point(Some("p3".to_string()));
            let p4 = sketch.add_point(Some("p4".to_string()));
            let p5 = sketch.add_point(Some("p5".to_string()));
            let p6 = sketch.add_point(Some("p6".to_string()));

            let line_a = sketch.add_line(p1, p2, Some("line_a".to_string()));
            let line_b = sketch.add_line(p3, p4, Some("line_b".to_string()));
            let line_c = sketch.add_line(p5, p6, Some("line_c".to_string()));

            // Fix base line A
            sketch.add_constraint(FixedPositionConstraint::new(p1, Length::meters(base_x1), Length::meters(base_y1)));
            sketch.add_constraint(FixedPositionConstraint::new(p2, Length::meters(base_x2), Length::meters(base_y2)));

            // Position and constrain other lines
            sketch.add_constraint(FixedPositionConstraint::new(p3, Length::meters(pos2_x), Length::meters(pos2_y)));
            sketch.add_constraint(FixedPositionConstraint::new(p5, Length::meters(pos3_x), Length::meters(pos3_y)));

            sketch.add_constraint(crate::constraints::LineLengthConstraint::new(line_b, Length::meters(length2)));
            sketch.add_constraint(crate::constraints::LineLengthConstraint::new(line_c, Length::meters(length3)));

            // Apply parallel constraints: A || B and B || C
            sketch.add_constraint(ParallelLinesConstraint::new(line_a, line_b));
            sketch.add_constraint(ParallelLinesConstraint::new(line_b, line_c));

            let solution = sketch.solve_and_extract()?;

            // Get all coordinates
            let coords_a = [solution.get_point_coordinates(p1)?, solution.get_point_coordinates(p2)?];
            let coords_b = [solution.get_point_coordinates(p3)?, solution.get_point_coordinates(p4)?];
            let coords_c = [solution.get_point_coordinates(p5)?, solution.get_point_coordinates(p6)?];

            // Calculate direction vectors
            let dir_a = (coords_a[1].0 - coords_a[0].0, coords_a[1].1 - coords_a[0].1);
            let dir_b = (coords_b[1].0 - coords_b[0].0, coords_b[1].1 - coords_b[0].1);
            let dir_c = (coords_c[1].0 - coords_c[0].0, coords_c[1].1 - coords_c[0].1);

            // Test A || B
            let cross_ab = dir_a.0 * dir_b.1 - dir_a.1 * dir_b.0;
            prop_assert!(cross_ab.abs() < 1e-6, "A || B constraint violated, cross: {}", cross_ab);

            // Test B || C
            let cross_bc = dir_b.0 * dir_c.1 - dir_b.1 * dir_c.0;
            prop_assert!(cross_bc.abs() < 1e-6, "B || C constraint violated, cross: {}", cross_bc);

            // Test transitivity: A || C
            let cross_ac = dir_a.0 * dir_c.1 - dir_a.1 * dir_c.0;
            prop_assert!(cross_ac.abs() < 1e-6, "Parallel transitivity violated A || C, cross: {}", cross_ac);
        }
    }

    // Property test: Entity-as-constraint-factory methods work correctly
    // See docs/IGNORED_TESTS.md for details on why this test is ignored
    proptest! {
        #[test]
        #[ignore]
        fn prop_entity_factory_methods_work(
            x1 in 1.0f64..3.0f64,
            y1 in 1.0f64..3.0f64,
            x2 in 4.0f64..6.0f64,
            y2 in 1.0f64..3.0f64,
            x3 in 1.0f64..3.0f64,
            y3 in 4.0f64..6.0f64,
            line2_length in 2.0f64..5.0f64,
            use_parallel in any::<bool>()
        ) {
            // All points are now well-separated by design

            let cfg = Config::new();
            let ctx = Context::new(&cfg);
            let mut sketch = Sketch::new(&ctx);

            let p1 = sketch.add_point(Some("p1".to_string()));
            let p2 = sketch.add_point(Some("p2".to_string()));
            let p3 = sketch.add_point(Some("p3".to_string()));
            let p4 = sketch.add_point(Some("p4".to_string()));

            let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
            let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

            // Fix line1
            sketch.add_constraint(FixedPositionConstraint::new(p1, Length::meters(x1), Length::meters(y1)));
            sketch.add_constraint(FixedPositionConstraint::new(p2, Length::meters(x2), Length::meters(y2)));

            // Fix line2 start and length
            sketch.add_constraint(FixedPositionConstraint::new(p3, Length::meters(x3), Length::meters(y3)));

            // Use entity-as-constraint-factory methods
            let line1_entity = sketch.get_line(line1).unwrap().clone();
            let line2_entity = sketch.get_line(line2).unwrap().clone();

            sketch.add_constraint(line2_entity.length_equals(Length::meters(line2_length)));

            if use_parallel {
                sketch.add_constraint(line2_entity.parallel_to(&line1_entity));
            } else {
                sketch.add_constraint(line2_entity.perpendicular_to(&line1_entity));
            }

            let solution = sketch.solve_and_extract()?;

            // Verify the constraint was applied correctly
            let coords1 = [solution.get_point_coordinates(p1)?, solution.get_point_coordinates(p2)?];
            let coords2 = [solution.get_point_coordinates(p3)?, solution.get_point_coordinates(p4)?];

            let dir1 = (coords1[1].0 - coords1[0].0, coords1[1].1 - coords1[0].1);
            let dir2 = (coords2[1].0 - coords2[0].0, coords2[1].1 - coords2[0].1);

            if use_parallel {
                let cross_product = dir1.0 * dir2.1 - dir1.1 * dir2.0;
                prop_assert!(cross_product.abs() < 1e-6, "Factory parallel constraint failed, cross: {}", cross_product);
            } else {
                let dot_product = dir1.0 * dir2.0 + dir1.1 * dir2.1;
                prop_assert!(dot_product.abs() < 1e-6, "Factory perpendicular constraint failed, dot: {}", dot_product);
            }

            // Verify length constraint from factory
            let actual_length = (dir2.0.powi(2) + dir2.1.powi(2)).sqrt();
            prop_assert!((actual_length - line2_length).abs() < 1e-6,
                "Factory length constraint failed: expected {}, got {}", line2_length, actual_length);
        }
    }

    // Property tests for Circle entity (Z3-based implementation)
    use crate::entities::Circle;
    use crate::entity::CircleId;

    // Property test: Circle creation with different Z3 contexts and names
    proptest! {
        #[test]
        fn prop_circle_z3_variable_naming(
            circle_index in 0usize..50usize,
            center_index in 0usize..50usize,
            has_name in any::<bool>(),
            name_length in 1usize..10usize
        ) {
            use generational_arena::Index;
            use z3::{Config, Context};

            let cfg = Config::new();
            let ctx = Context::new(&cfg);

            let circle_id = CircleId::from(Index::from_raw_parts(circle_index, 0));
            let center_id = crate::entities::PointId::from(Index::from_raw_parts(center_index, 0));

            let name = if has_name {
                Some("x".repeat(name_length))
            } else {
                None
            };

            let circle = Circle::new(circle_id, center_id, &ctx, name.clone());

            // Circle properties should be properly set
            prop_assert_eq!(circle.center_point(), center_id);
            prop_assert_eq!(circle.center, center_id);
            prop_assert_eq!(circle.id, circle_id);

            // Check Z3 variable naming
            let radius_var_str = circle.radius.to_string();
            if let Some(expected_name) = name.as_ref() {
                prop_assert_eq!(&circle.display_name(), expected_name);
                let expected_radius_name = format!("{}_radius", expected_name);
                prop_assert!(radius_var_str.contains(&expected_radius_name));
            } else {
                prop_assert!(circle.display_name().starts_with("Circle"));
                prop_assert!(radius_var_str.contains("c_radius")); // Default naming
            }

            // Display name should be consistent across multiple calls
            let first_call = circle.display_name();
            let second_call = circle.display_name();
            prop_assert_eq!(first_call, second_call);
        }
    }
}
