//! Regression tests for Phase 10 parametric constraints
//!
//! Tests to ensure PointOnLineConstraint works correctly with existing constraint types
//! and doesn't break existing functionality or create unexpected interactions.

use textcad::constraints::{
    FixedPositionConstraint, LineLengthConstraint, ParallelLinesConstraint,
    PerpendicularLinesConstraint, PointOnLineConstraint,
};
use textcad::error::TextCadError;
use textcad::sketch::Sketch;
use textcad::units::Length;
use z3::{Config, Context};

/// Test PointOnLineConstraint working with existing ParallelLinesConstraint
#[test]
fn test_point_on_line_with_parallel_lines() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create two parallel lines
    let p1 = sketch.add_point(Some("line1_start".to_string()));
    let p2 = sketch.add_point(Some("line1_end".to_string()));
    let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));

    let p3 = sketch.add_point(Some("line2_start".to_string()));
    let p4 = sketch.add_point(Some("line2_end".to_string()));
    let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

    // Points on each line
    let p5 = sketch.add_point(Some("on_line1".to_string()));
    let p6 = sketch.add_point(Some("on_line2".to_string()));

    // Fix first line horizontally
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(5.0),
        Length::meters(0.0),
    ));

    // Fix start of second line, let end be determined by parallel constraint
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(1.0),
        Length::meters(3.0),
    ));
    sketch.add_constraint(LineLengthConstraint::new(line2, Length::meters(5.0)));

    // Make lines parallel
    sketch.add_constraint(ParallelLinesConstraint::new(line1, line2));

    // Add points on lines
    sketch.add_constraint(PointOnLineConstraint::new(line1, p5));
    sketch.add_constraint(PointOnLineConstraint::new(line2, p6));

    let solution = sketch
        .solve_and_extract()
        .expect("Point on parallel lines should work");

    // Verify lines are parallel
    let coords1 = [
        solution.get_point_coordinates(p1).unwrap(),
        solution.get_point_coordinates(p2).unwrap(),
    ];
    let coords2 = [
        solution.get_point_coordinates(p3).unwrap(),
        solution.get_point_coordinates(p4).unwrap(),
    ];

    let dir1 = (coords1[1].0 - coords1[0].0, coords1[1].1 - coords1[0].1);
    let dir2 = (coords2[1].0 - coords2[0].0, coords2[1].1 - coords2[0].1);

    let cross_product = dir1.0 * dir2.1 - dir1.1 * dir2.0;
    assert!(
        cross_product.abs() < 1e-6,
        "Lines should be parallel, cross product: {}",
        cross_product
    );

    // Verify points are on their respective lines
    let (x5, y5) = solution.get_point_coordinates(p5).unwrap();
    let (x6, y6) = solution.get_point_coordinates(p6).unwrap();

    // p5 should be on horizontal line y=0
    assert!((y5 - 0.0).abs() < 1e-6, "Point should be on line1 (y=0)");
    assert!(
        x5 >= -1e-6 && x5 <= 5.0 + 1e-6,
        "Point should be on line1 segment"
    );

    // p6 should be on parallel line y=3
    assert!((y6 - 3.0).abs() < 1e-6, "Point should be on line2 (y=3)");
    assert!(
        x6 >= 1.0 - 1e-6 && x6 <= 6.0 + 1e-6,
        "Point should be on line2 segment"
    );
}

/// Test PointOnLineConstraint working with PerpendicularLinesConstraint
#[test]
fn test_point_on_line_with_perpendicular_lines() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create two perpendicular lines
    let p1 = sketch.add_point(Some("horizontal_start".to_string()));
    let p2 = sketch.add_point(Some("horizontal_end".to_string()));
    let h_line = sketch.add_line(p1, p2, Some("horizontal".to_string()));

    let p3 = sketch.add_point(Some("vertical_start".to_string()));
    let p4 = sketch.add_point(Some("vertical_end".to_string()));
    let v_line = sketch.add_line(p3, p4, Some("vertical".to_string()));

    // Points on each line
    let p5 = sketch.add_point(Some("on_horizontal".to_string()));
    let p6 = sketch.add_point(Some("on_vertical".to_string()));

    // Fix horizontal line
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(4.0),
        Length::meters(0.0),
    ));

    // Fix start of vertical line, let end be determined by perpendicular constraint
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(2.0),
        Length::meters(-1.0),
    ));
    sketch.add_constraint(LineLengthConstraint::new(v_line, Length::meters(3.0)));

    // Make lines perpendicular
    sketch.add_constraint(PerpendicularLinesConstraint::new(h_line, v_line));

    // Add points on lines
    sketch.add_constraint(PointOnLineConstraint::new(h_line, p5));
    sketch.add_constraint(PointOnLineConstraint::new(v_line, p6));

    let solution = sketch
        .solve_and_extract()
        .expect("Point on perpendicular lines should work");

    // Verify lines are perpendicular
    let coords_h = [
        solution.get_point_coordinates(p1).unwrap(),
        solution.get_point_coordinates(p2).unwrap(),
    ];
    let coords_v = [
        solution.get_point_coordinates(p3).unwrap(),
        solution.get_point_coordinates(p4).unwrap(),
    ];

    let dir_h = (coords_h[1].0 - coords_h[0].0, coords_h[1].1 - coords_h[0].1);
    let dir_v = (coords_v[1].0 - coords_v[0].0, coords_v[1].1 - coords_v[0].1);

    let dot_product = dir_h.0 * dir_v.0 + dir_h.1 * dir_v.1;
    assert!(
        dot_product.abs() < 1e-6,
        "Lines should be perpendicular, dot product: {}",
        dot_product
    );

    // Verify points are on their respective lines
    let (x5, y5) = solution.get_point_coordinates(p5).unwrap();
    let (x6, y6) = solution.get_point_coordinates(p6).unwrap();

    // p5 should be on horizontal line y=0
    assert!(
        (y5 - 0.0).abs() < 1e-6,
        "Point should be on horizontal line"
    );

    // p6 should be on vertical line x=2
    assert!((x6 - 2.0).abs() < 1e-6, "Point should be on vertical line");
}

/// Test constraint order independence with existing constraints
#[test]
fn test_constraint_order_with_existing_constraints() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    // Test adding point-on-line before other constraints
    let solution1 = {
        let mut sketch = Sketch::new(&ctx);
        let p1 = sketch.add_point(Some("start".to_string()));
        let p2 = sketch.add_point(Some("end".to_string()));
        let p3 = sketch.add_point(Some("other_start".to_string()));
        let p4 = sketch.add_point(Some("other_end".to_string()));
        let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
        let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));
        let p5 = sketch.add_point(Some("on_line".to_string()));

        // Order 1: Point-on-line first
        sketch.add_constraint(PointOnLineConstraint::new(line1, p5));
        sketch.add_constraint(ParallelLinesConstraint::new(line1, line2));
        sketch.add_constraint(FixedPositionConstraint::new(
            p1,
            Length::meters(0.0),
            Length::meters(0.0),
        ));
        sketch.add_constraint(FixedPositionConstraint::new(
            p2,
            Length::meters(3.0),
            Length::meters(0.0),
        ));
        sketch.add_constraint(FixedPositionConstraint::new(
            p3,
            Length::meters(1.0),
            Length::meters(2.0),
        ));
        sketch.add_constraint(LineLengthConstraint::new(line2, Length::meters(3.0)));

        sketch.solve_and_extract()
    };

    // Test adding point-on-line after other constraints
    let solution2 = {
        let mut sketch = Sketch::new(&ctx);
        let p1 = sketch.add_point(Some("start".to_string()));
        let p2 = sketch.add_point(Some("end".to_string()));
        let p3 = sketch.add_point(Some("other_start".to_string()));
        let p4 = sketch.add_point(Some("other_end".to_string()));
        let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
        let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));
        let p5 = sketch.add_point(Some("on_line".to_string()));

        // Order 2: Point-on-line last
        sketch.add_constraint(FixedPositionConstraint::new(
            p1,
            Length::meters(0.0),
            Length::meters(0.0),
        ));
        sketch.add_constraint(FixedPositionConstraint::new(
            p2,
            Length::meters(3.0),
            Length::meters(0.0),
        ));
        sketch.add_constraint(FixedPositionConstraint::new(
            p3,
            Length::meters(1.0),
            Length::meters(2.0),
        ));
        sketch.add_constraint(LineLengthConstraint::new(line2, Length::meters(3.0)));
        sketch.add_constraint(ParallelLinesConstraint::new(line1, line2));
        sketch.add_constraint(PointOnLineConstraint::new(line1, p5));

        sketch.solve_and_extract()
    };

    // Both should succeed
    assert!(solution1.is_ok(), "Order 1 should work");
    assert!(solution2.is_ok(), "Order 2 should work");
}

/// Test that existing constraint behavior is preserved
#[test]
fn test_existing_constraints_still_work() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create setup that worked before parametric constraints
    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));
    let p4 = sketch.add_point(Some("p4".to_string()));

    let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
    let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

    // Classical constraints that should still work
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(3.0),
        Length::meters(4.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(1.0),
        Length::meters(1.0),
    ));

    sketch.add_constraint(LineLengthConstraint::new(line1, Length::meters(5.0)));
    sketch.add_constraint(LineLengthConstraint::new(line2, Length::meters(2.0)));
    sketch.add_constraint(PerpendicularLinesConstraint::new(line1, line2));

    let solution = sketch
        .solve_and_extract()
        .expect("Existing constraints should still work");

    // Verify expected behavior
    let (x1, y1) = solution.get_point_coordinates(p1).unwrap();
    let (x2, y2) = solution.get_point_coordinates(p2).unwrap();
    let (x3, y3) = solution.get_point_coordinates(p3).unwrap();
    let (x4, y4) = solution.get_point_coordinates(p4).unwrap();

    // Check line1 length
    let line1_length = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    assert!(
        (line1_length - 5.0).abs() < 1e-6,
        "Line1 should have length 5.0"
    );

    // Check line2 length
    let line2_length = ((x4 - x3).powi(2) + (y4 - y3).powi(2)).sqrt();
    assert!(
        (line2_length - 2.0).abs() < 1e-6,
        "Line2 should have length 2.0"
    );

    // Check perpendicularity
    let dir1 = (x2 - x1, y2 - y1);
    let dir2 = (x4 - x3, y4 - y3);
    let dot_product = dir1.0 * dir2.0 + dir1.1 * dir2.1;
    assert!(dot_product.abs() < 1e-6, "Lines should be perpendicular");
}

/// Test mixed constraints with point relationships
#[test]
fn test_mixed_constraints_complex_relationship() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a triangle with a point on one side
    let a = sketch.add_point(Some("vertex_a".to_string()));
    let b = sketch.add_point(Some("vertex_b".to_string()));
    let c = sketch.add_point(Some("vertex_c".to_string()));

    let side_ab = sketch.add_line(a, b, Some("side_ab".to_string()));
    let side_bc = sketch.add_line(b, c, Some("side_bc".to_string()));
    let side_ca = sketch.add_line(c, a, Some("side_ca".to_string()));

    // Point on side AB
    let p = sketch.add_point(Some("midpoint".to_string()));

    // Fix two triangle vertices to create a right triangle
    sketch.add_constraint(FixedPositionConstraint::new(
        a,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        b,
        Length::meters(4.0),
        Length::meters(0.0),
    ));

    // Position C to make a right triangle (CA ⊥ AB)
    // Since AB is horizontal (0,0) to (4,0), CA must be vertical
    sketch.add_constraint(FixedPositionConstraint::new(
        c,
        Length::meters(0.0), // Same x as A for vertical line
        Length::meters(3.0), // Above A
    ));

    // Verify it's a right triangle with perpendicular sides
    sketch.add_constraint(PerpendicularLinesConstraint::new(side_ca, side_ab));

    // Point on side AB
    sketch.add_constraint(PointOnLineConstraint::new(side_ab, p));

    // Force point to be at midpoint of AB
    sketch.add_constraint(FixedPositionConstraint::new(
        p,
        Length::meters(2.0), // midpoint of AB (0,0) to (4,0)
        Length::meters(0.0),
    ));

    let solution = sketch
        .solve_and_extract()
        .expect("Complex mixed constraints should work");

    // Verify triangle properties
    let coords_a = solution.get_point_coordinates(a).unwrap();
    let coords_b = solution.get_point_coordinates(b).unwrap();
    let coords_c = solution.get_point_coordinates(c).unwrap();
    let coords_p = solution.get_point_coordinates(p).unwrap();

    // Verify point is at midpoint of AB
    let expected_midpoint = (
        (coords_a.0 + coords_b.0) / 2.0,
        (coords_a.1 + coords_b.1) / 2.0,
    );
    assert!(
        (coords_p.0 - expected_midpoint.0).abs() < 1e-6,
        "Point should be at midpoint"
    );
    assert!(
        (coords_p.1 - expected_midpoint.1).abs() < 1e-6,
        "Point should be at midpoint"
    );

    // Verify perpendicularity was maintained
    let dir_ab = (coords_b.0 - coords_a.0, coords_b.1 - coords_a.1);
    let dir_ca = (coords_a.0 - coords_c.0, coords_a.1 - coords_c.1);
    let dot_product = dir_ab.0 * dir_ca.0 + dir_ab.1 * dir_ca.1;
    assert!(
        dot_product.abs() < 1e-6,
        "Lines should remain perpendicular"
    );
}

/// Test constraint removal and modification scenarios
#[test]
fn test_constraint_modification_scenarios() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    // Test that we can solve similar problems with slight modifications
    // This simulates the scenario where constraints are modified during design

    // Base case: Point on horizontal line
    {
        let mut sketch = Sketch::new(&ctx);
        let p1 = sketch.add_point(Some("start".to_string()));
        let p2 = sketch.add_point(Some("end".to_string()));
        let line = sketch.add_line(p1, p2, Some("line".to_string()));
        let p3 = sketch.add_point(Some("on_line".to_string()));

        sketch.add_constraint(FixedPositionConstraint::new(
            p1,
            Length::meters(0.0),
            Length::meters(0.0),
        ));
        sketch.add_constraint(FixedPositionConstraint::new(
            p2,
            Length::meters(5.0),
            Length::meters(0.0),
        ));
        sketch.add_constraint(PointOnLineConstraint::new(line, p3));

        let solution = sketch.solve_and_extract().expect("Base case should work");
        let (_, y3) = solution.get_point_coordinates(p3).unwrap();
        assert!((y3 - 0.0).abs() < 1e-6);
    }

    // Modified case: Same structure but vertical line
    {
        let mut sketch = Sketch::new(&ctx);
        let p1 = sketch.add_point(Some("start".to_string()));
        let p2 = sketch.add_point(Some("end".to_string()));
        let line = sketch.add_line(p1, p2, Some("line".to_string()));
        let p3 = sketch.add_point(Some("on_line".to_string()));

        sketch.add_constraint(FixedPositionConstraint::new(
            p1,
            Length::meters(2.0),
            Length::meters(0.0),
        ));
        sketch.add_constraint(FixedPositionConstraint::new(
            p2,
            Length::meters(2.0), // Same x, different y = vertical
            Length::meters(5.0),
        ));
        sketch.add_constraint(PointOnLineConstraint::new(line, p3));

        let solution = sketch
            .solve_and_extract()
            .expect("Modified case should work");
        let (x3, _) = solution.get_point_coordinates(p3).unwrap();
        assert!((x3 - 2.0).abs() < 1e-6);
    }
}

/// Test that parametric constraints don't interfere with unsatisfiable constraint detection
#[test]
fn test_unsatisfiable_constraint_detection_still_works() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));
    let line = sketch.add_line(p1, p2, Some("line".to_string()));

    // Create impossible constraint combination
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(3.0),
        Length::meters(0.0),
    ));

    // Try to put point on line but also fix it far away from line
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(1.5),  // On the line
        Length::meters(10.0), // Far from the line
    ));
    sketch.add_constraint(PointOnLineConstraint::new(line, p3));

    // This should be over-constrained
    let result = sketch.solve_and_extract();
    assert!(
        result.is_err(),
        "Impossible constraints should still be detected"
    );
    assert!(matches!(result, Err(TextCadError::OverConstrained)));
}
