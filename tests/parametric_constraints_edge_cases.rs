//! Edge case and boundary condition tests for Phase 10 parametric constraints
//!
//! Tests various edge cases, boundary conditions, and stress scenarios
//! for the PointOnLineConstraint implementation.

use textcad::constraints::{FixedPositionConstraint, LineLengthConstraint, PointOnLineConstraint};
use textcad::sketch::Sketch;
use textcad::units::Length;
use z3::{Config, Context};

/// Test PointOnLineConstraint with very short lines (near-zero length)
#[test]
fn test_point_on_very_short_line() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a very short line (1 micrometer)
    let p1 = sketch.add_point(Some("start".to_string()));
    let p2 = sketch.add_point(Some("end".to_string()));
    let line = sketch.add_line(p1, p2, Some("short_line".to_string()));

    // Point to constrain on the line
    let p3 = sketch.add_point(Some("on_line".to_string()));

    // Fix line endpoints with tiny distance
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(1e-6), // 1 micrometer
        Length::meters(0.0),
    ));

    // Constrain point to line
    sketch.add_constraint(PointOnLineConstraint::new(line, p3));

    let solution = sketch
        .solve_and_extract()
        .expect("Very short lines should work");

    let (x1, y1) = solution.get_point_coordinates(p1).unwrap();
    let (x2, y2) = solution.get_point_coordinates(p2).unwrap();
    let (x3, y3) = solution.get_point_coordinates(p3).unwrap();

    // Verify point is on the short line
    let dx = x2 - x1;
    let dy = y2 - y1;
    let length_sq = dx * dx + dy * dy;

    if length_sq > 1e-12 {
        let t = ((x3 - x1) * dx + (y3 - y1) * dy) / length_sq;
        assert!(
            t >= -1e-6 && t <= 1.0 + 1e-6,
            "Parameter t should be in [0,1] for very short line, got: {}",
            t
        );
    }

    // Point should be very close to the line
    let cross_product = (x3 - x1) * (y2 - y1) - (y3 - y1) * (x2 - x1);
    assert!(
        cross_product.abs() < 1e-10,
        "Point should be on very short line, cross product: {}",
        cross_product
    );
}

/// Test PointOnLineConstraint with coincident line endpoints (degenerate line)
#[test]
fn test_point_on_degenerate_line() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a degenerate line (both endpoints at same position)
    let p1 = sketch.add_point(Some("point1".to_string()));
    let p2 = sketch.add_point(Some("point2".to_string()));
    let line = sketch.add_line(p1, p2, Some("degenerate_line".to_string()));

    // Point to constrain on the line
    let p3 = sketch.add_point(Some("test_point".to_string()));

    // Fix both endpoints at the same location
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(5.0),
        Length::meters(3.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(5.0),
        Length::meters(3.0), // Same as p1
    ));

    // Constrain point to degenerate line
    sketch.add_constraint(PointOnLineConstraint::new(line, p3));

    let solution = sketch
        .solve_and_extract()
        .expect("Degenerate line should constrain point to that location");

    let (x1, y1) = solution.get_point_coordinates(p1).unwrap();
    let (x3, y3) = solution.get_point_coordinates(p3).unwrap();

    // Point should be at the same location as the degenerate line
    assert!(
        (x3 - x1).abs() < 1e-6,
        "Point should be at degenerate line location"
    );
    assert!(
        (y3 - y1).abs() < 1e-6,
        "Point should be at degenerate line location"
    );
}

/// Test PointOnLineConstraint with points already at line endpoints
#[test]
fn test_point_already_at_line_endpoint() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a line
    let p1 = sketch.add_point(Some("start".to_string()));
    let p2 = sketch.add_point(Some("end".to_string()));
    let line = sketch.add_line(p1, p2, Some("test_line".to_string()));

    // Point initially at line start
    let p3 = sketch.add_point(Some("at_start".to_string()));

    // Fix line endpoints
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(4.0),
        Length::meters(3.0),
    ));

    // Initially position point at line start
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(0.0),
        Length::meters(0.0),
    ));

    // Add point-on-line constraint (should be already satisfied)
    sketch.add_constraint(PointOnLineConstraint::new(line, p3));

    let solution = sketch
        .solve_and_extract()
        .expect("Point already at endpoint should work");

    let (x1, y1) = solution.get_point_coordinates(p1).unwrap();
    let (x3, y3) = solution.get_point_coordinates(p3).unwrap();

    // Point should remain at the start
    assert!((x3 - x1).abs() < 1e-6);
    assert!((y3 - y1).abs() < 1e-6);
}

/// Test multiple overlapping constraints on same point
#[test]
fn test_multiple_points_same_line_different_constraints() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a line
    let p1 = sketch.add_point(Some("start".to_string()));
    let p2 = sketch.add_point(Some("end".to_string()));
    let line = sketch.add_line(p1, p2, Some("base_line".to_string()));

    // Create multiple points on the same line
    let p3 = sketch.add_point(Some("point_a".to_string()));
    let p4 = sketch.add_point(Some("point_b".to_string()));
    let p5 = sketch.add_point(Some("point_c".to_string()));

    // Fix line endpoints
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(10.0),
        Length::meters(0.0),
    ));

    // All points on same line
    sketch.add_constraint(PointOnLineConstraint::new(line, p3));
    sketch.add_constraint(PointOnLineConstraint::new(line, p4));
    sketch.add_constraint(PointOnLineConstraint::new(line, p5));

    // Add additional constraints to force specific positions
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(2.0), // t = 0.2
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p4,
        Length::meters(5.0), // t = 0.5 (midpoint)
        Length::meters(0.0),
    ));

    let solution = sketch
        .solve_and_extract()
        .expect("Multiple points with mixed constraints should work");

    // Verify all points are on line at expected positions
    let coords = vec![
        solution.get_point_coordinates(p1).unwrap(),
        solution.get_point_coordinates(p2).unwrap(),
        solution.get_point_coordinates(p3).unwrap(),
        solution.get_point_coordinates(p4).unwrap(),
        solution.get_point_coordinates(p5).unwrap(),
    ];

    // Check p3 is at x=2
    assert!((coords[2].0 - 2.0).abs() < 1e-6);
    assert!((coords[2].1 - 0.0).abs() < 1e-6);

    // Check p4 is at x=5 (midpoint)
    assert!((coords[3].0 - 5.0).abs() < 1e-6);
    assert!((coords[3].1 - 0.0).abs() < 1e-6);

    // Check p5 is somewhere on the line
    let (x5, y5) = coords[4];
    assert!((y5 - 0.0).abs() < 1e-6, "p5 should be on y=0 line");
    assert!(
        x5 >= -1e-6 && x5 <= 10.0 + 1e-6,
        "p5 should be on line segment, x = {}",
        x5
    );
}

/// Test constraint behavior with line at various orientations
#[test]
fn test_point_on_line_different_orientations() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    // Test horizontal line
    {
        let mut sketch = Sketch::new(&ctx);
        let p1 = sketch.add_point(Some("h_start".to_string()));
        let p2 = sketch.add_point(Some("h_end".to_string()));
        let line = sketch.add_line(p1, p2, Some("horizontal".to_string()));
        let p3 = sketch.add_point(Some("on_h".to_string()));

        sketch.add_constraint(FixedPositionConstraint::new(
            p1,
            Length::meters(0.0),
            Length::meters(2.0),
        ));
        sketch.add_constraint(FixedPositionConstraint::new(
            p2,
            Length::meters(5.0),
            Length::meters(2.0),
        ));
        sketch.add_constraint(PointOnLineConstraint::new(line, p3));

        let solution = sketch
            .solve_and_extract()
            .expect("Horizontal line should work");
        let (_, y3) = solution.get_point_coordinates(p3).unwrap();
        assert!(
            (y3 - 2.0).abs() < 1e-6,
            "Point should be on horizontal line y=2"
        );
    }

    // Test vertical line
    {
        let mut sketch = Sketch::new(&ctx);
        let p1 = sketch.add_point(Some("v_start".to_string()));
        let p2 = sketch.add_point(Some("v_end".to_string()));
        let line = sketch.add_line(p1, p2, Some("vertical".to_string()));
        let p3 = sketch.add_point(Some("on_v".to_string()));

        sketch.add_constraint(FixedPositionConstraint::new(
            p1,
            Length::meters(3.0),
            Length::meters(0.0),
        ));
        sketch.add_constraint(FixedPositionConstraint::new(
            p2,
            Length::meters(3.0),
            Length::meters(7.0),
        ));
        sketch.add_constraint(PointOnLineConstraint::new(line, p3));

        let solution = sketch
            .solve_and_extract()
            .expect("Vertical line should work");
        let (x3, _) = solution.get_point_coordinates(p3).unwrap();
        assert!(
            (x3 - 3.0).abs() < 1e-6,
            "Point should be on vertical line x=3"
        );
    }

    // Test diagonal line (45°)
    {
        let mut sketch = Sketch::new(&ctx);
        let p1 = sketch.add_point(Some("d_start".to_string()));
        let p2 = sketch.add_point(Some("d_end".to_string()));
        let line = sketch.add_line(p1, p2, Some("diagonal".to_string()));
        let p3 = sketch.add_point(Some("on_d".to_string()));

        sketch.add_constraint(FixedPositionConstraint::new(
            p1,
            Length::meters(0.0),
            Length::meters(0.0),
        ));
        sketch.add_constraint(FixedPositionConstraint::new(
            p2,
            Length::meters(4.0),
            Length::meters(4.0),
        ));
        sketch.add_constraint(PointOnLineConstraint::new(line, p3));

        let solution = sketch
            .solve_and_extract()
            .expect("Diagonal line should work");
        let (x3, y3) = solution.get_point_coordinates(p3).unwrap();

        // Point should be on line y = x
        assert!(
            (y3 - x3).abs() < 1e-6,
            "Point should be on diagonal y=x, got ({}, {})",
            x3,
            y3
        );
    }
}

/// Test point constrained to line with very small parameter differences
#[test]
fn test_point_on_line_parameter_precision() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create line
    let p1 = sketch.add_point(Some("start".to_string()));
    let p2 = sketch.add_point(Some("end".to_string()));
    let line = sketch.add_line(p1, p2, Some("precision_line".to_string()));

    // Create two points very close on the line
    let p3 = sketch.add_point(Some("point_a".to_string()));
    let p4 = sketch.add_point(Some("point_b".to_string()));

    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(100.0),
        Length::meters(0.0),
    ));

    // Constrain both points to line
    sketch.add_constraint(PointOnLineConstraint::new(line, p3));
    sketch.add_constraint(PointOnLineConstraint::new(line, p4));

    // Force them to very close positions
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(50.0), // t = 0.5
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p4,
        Length::meters(50.0 + 1e-6), // t ≈ 0.5 + 1e-8
        Length::meters(0.0),
    ));

    let solution = sketch
        .solve_and_extract()
        .expect("High precision constraints should work");

    let (x3, y3) = solution.get_point_coordinates(p3).unwrap();
    let (x4, y4) = solution.get_point_coordinates(p4).unwrap();

    // Both should be very close but distinguishable
    assert!((x3 - 50.0).abs() < 1e-6);
    assert!((x4 - 50.0 - 1e-6).abs() < 1e-9);
    assert!((y3 - 0.0).abs() < 1e-9);
    assert!((y4 - 0.0).abs() < 1e-9);
}

/// Test constraint with line having irrational slope
#[test]
fn test_point_on_irrational_slope_line() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(Some("start".to_string()));
    let p2 = sketch.add_point(Some("end".to_string()));
    let line = sketch.add_line(p1, p2, Some("irrational_line".to_string()));
    let p3 = sketch.add_point(Some("on_line".to_string()));

    // Create line with slope = sqrt(2) (irrational)
    let sqrt2 = 2.0_f64.sqrt();
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(1.0),
        Length::meters(sqrt2),
    ));

    sketch.add_constraint(PointOnLineConstraint::new(line, p3));

    let solution = sketch
        .solve_and_extract()
        .expect("Irrational slope line should work");

    let (x1, y1) = solution.get_point_coordinates(p1).unwrap();
    let (x2, y2) = solution.get_point_coordinates(p2).unwrap();
    let (x3, y3) = solution.get_point_coordinates(p3).unwrap();

    // Verify point is on the irrational slope line
    let line_dx = x2 - x1;
    let line_dy = y2 - y1;
    let point_dx = x3 - x1;
    let point_dy = y3 - y1;

    let cross_product = line_dx * point_dy - line_dy * point_dx;
    assert!(
        cross_product.abs() < 1e-10,
        "Point should be on irrational slope line, cross product: {}",
        cross_product
    );
}

/// Test multiple constraints creating potential conflicts
#[test]
fn test_point_on_line_with_length_constraint() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create line with both point-on-line and length constraints
    let p1 = sketch.add_point(Some("start".to_string()));
    let p2 = sketch.add_point(Some("end".to_string()));
    let line = sketch.add_line(p1, p2, Some("constrained_line".to_string()));
    let p3 = sketch.add_point(Some("midpoint".to_string()));

    // Fix one endpoint
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));

    // Add line length constraint
    sketch.add_constraint(LineLengthConstraint::new(line, Length::meters(8.0)));

    // Constrain point to be on line
    sketch.add_constraint(PointOnLineConstraint::new(line, p3));

    // Force point to specific location (should be compatible)
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(4.0), // Should be t=0.5 if line is horizontal
        Length::meters(0.0),
    ));

    let solution = sketch
        .solve_and_extract()
        .expect("Compatible constraints should work");

    let (x1, y1) = solution.get_point_coordinates(p1).unwrap();
    let (x2, y2) = solution.get_point_coordinates(p2).unwrap();
    let (x3, y3) = solution.get_point_coordinates(p3).unwrap();

    // Check line has correct length
    let line_length = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    assert!(
        (line_length - 8.0).abs() < 1e-6,
        "Line should have length 8.0, got: {}",
        line_length
    );

    // Check point position
    assert!((x3 - 4.0).abs() < 1e-6);
    assert!((y3 - 0.0).abs() < 1e-6);
}

/// Test constraint order independence
#[test]
fn test_constraint_order_independence() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    // Test with point-on-line added first
    let result1 = {
        let mut sketch = Sketch::new(&ctx);
        let p1 = sketch.add_point(Some("start".to_string()));
        let p2 = sketch.add_point(Some("end".to_string()));
        let line = sketch.add_line(p1, p2, Some("test_line".to_string()));
        let p3 = sketch.add_point(Some("on_line".to_string()));

        // Order 1: Point-on-line first, then position constraints
        sketch.add_constraint(PointOnLineConstraint::new(line, p3));
        sketch.add_constraint(FixedPositionConstraint::new(
            p1,
            Length::meters(0.0),
            Length::meters(0.0),
        ));
        sketch.add_constraint(FixedPositionConstraint::new(
            p2,
            Length::meters(6.0),
            Length::meters(0.0),
        ));

        sketch.solve_and_extract()
    };

    // Test with position constraints added first
    let result2 = {
        let mut sketch = Sketch::new(&ctx);
        let p1 = sketch.add_point(Some("start".to_string()));
        let p2 = sketch.add_point(Some("end".to_string()));
        let line = sketch.add_line(p1, p2, Some("test_line".to_string()));
        let p3 = sketch.add_point(Some("on_line".to_string()));

        // Order 2: Position constraints first, then point-on-line
        sketch.add_constraint(FixedPositionConstraint::new(
            p1,
            Length::meters(0.0),
            Length::meters(0.0),
        ));
        sketch.add_constraint(FixedPositionConstraint::new(
            p2,
            Length::meters(6.0),
            Length::meters(0.0),
        ));
        sketch.add_constraint(PointOnLineConstraint::new(line, p3));

        sketch.solve_and_extract()
    };

    // Both should succeed - this tests that constraint order doesn't break solving
    assert!(result1.is_ok(), "Order 1 should work");
    assert!(result2.is_ok(), "Order 2 should work");
}
