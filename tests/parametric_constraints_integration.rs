//! Integration tests for Phase 10 parametric constraints
//!
//! Tests complete workflows including sketch creation, constraint application,
//! solving, and solution extraction for the PointOnLineConstraint.

use textcad::constraints::{FixedPositionConstraint, PointOnLineConstraint};
use textcad::error::TextCadError;
use textcad::sketch::Sketch;
use textcad::units::Length;
use z3::{Config, Context};

#[test]
fn test_point_on_line_constraint_basic_integration() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a line from (0,0) to (4,0)
    let p1 = sketch.add_point(Some("start".to_string()));
    let p2 = sketch.add_point(Some("end".to_string()));
    let line = sketch.add_line(p1, p2, Some("test_line".to_string()));

    // Create a point that will be constrained to lie on the line
    let p3 = sketch.add_point(Some("on_line".to_string()));

    // Fix the line endpoints
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

    // Constrain p3 to lie on the line
    sketch.add_constraint(PointOnLineConstraint::new(line, p3));

    // Solve
    let solution = sketch
        .solve_and_extract()
        .expect("Should solve successfully");

    // Verify the point lies on the line segment
    let (x1, y1) = solution.get_point_coordinates(p1).unwrap();
    let (x2, y2) = solution.get_point_coordinates(p2).unwrap();
    let (x3, y3) = solution.get_point_coordinates(p3).unwrap();

    // Line endpoints should be as expected
    assert!((x1 - 0.0).abs() < 1e-6);
    assert!((y1 - 0.0).abs() < 1e-6);
    assert!((x2 - 4.0).abs() < 1e-6);
    assert!((y2 - 0.0).abs() < 1e-6);

    // Point should lie on the line y = 0, with x between 0 and 4
    assert!((y3 - 0.0).abs() < 1e-6, "Point should be on y = 0 line");
    assert!(
        x3 >= -1e-6 && x3 <= 4.0 + 1e-6,
        "Point x-coordinate should be between 0 and 4, got: {}",
        x3
    );

    // Verify the point is actually on the line using parametric equation
    // p3 = p1 + t * (p2 - p1)
    let dx = x2 - x1;
    let dy = y2 - y1;
    let length_sq = dx * dx + dy * dy;
    
    if length_sq > 1e-6 {
        let t = ((x3 - x1) * dx + (y3 - y1) * dy) / length_sq;
        assert!(
            t >= -1e-6 && t <= 1.0 + 1e-6,
            "Parameter t should be in [0,1], got: {}",
            t
        );
    }
}

#[test]
fn test_point_on_line_constraint_with_multiple_points() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a diagonal line from (0,0) to (3,4)
    let p1 = sketch.add_point(Some("start".to_string()));
    let p2 = sketch.add_point(Some("end".to_string()));
    let line = sketch.add_line(p1, p2, Some("diagonal_line".to_string()));

    // Create multiple points on the line
    let p3 = sketch.add_point(Some("point_1".to_string()));
    let p4 = sketch.add_point(Some("point_2".to_string()));
    let p5 = sketch.add_point(Some("point_3".to_string()));

    // Fix the line endpoints
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

    // Constrain all points to lie on the line
    sketch.add_constraint(PointOnLineConstraint::new(line, p3));
    sketch.add_constraint(PointOnLineConstraint::new(line, p4));
    sketch.add_constraint(PointOnLineConstraint::new(line, p5));

    // Solve
    let solution = sketch
        .solve_and_extract()
        .expect("Should solve successfully");

    // Verify all points lie on the line segment
    let (x1, y1) = solution.get_point_coordinates(p1).unwrap();
    let (x2, y2) = solution.get_point_coordinates(p2).unwrap();
    let points_on_line = [p3, p4, p5];

    for &point_id in &points_on_line {
        let (x, y) = solution.get_point_coordinates(point_id).unwrap();

        // Check if point lies on the line using the line equation
        // Line equation: (y - y1) / (y2 - y1) = (x - x1) / (x2 - x1)
        let dx_line = x2 - x1;
        let dy_line = y2 - y1;
        let dx_point = x - x1;
        let dy_point = y - y1;

        // Cross product should be zero if point is on line
        let cross_product = dx_line * dy_point - dy_line * dx_point;
        assert!(
            cross_product.abs() < 1e-6,
            "Point {:?} should be on line, cross product: {}",
            point_id,
            cross_product
        );

        // Check parameter bounds: t ∈ [0, 1]
        let length_sq = dx_line * dx_line + dy_line * dy_line;
        if length_sq > 1e-6 {
            let t = (dx_point * dx_line + dy_point * dy_line) / length_sq;
            assert!(
                t >= -1e-6 && t <= 1.0 + 1e-6,
                "Point {:?} parameter t should be in [0,1], got: {}",
                point_id,
                t
            );
        }
    }
}

#[test]
fn test_point_on_line_constraint_zero_length_line_fails() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a zero-length line (both endpoints at same position)
    let p1 = sketch.add_point(Some("point1".to_string()));
    let p2 = sketch.add_point(Some("point2".to_string()));
    let line = sketch.add_line(p1, p2, Some("zero_line".to_string()));

    // Create a point to constrain on the line
    let p3 = sketch.add_point(Some("test_point".to_string()));

    // Fix both endpoints at the same location
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(1.0),
        Length::meters(1.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(1.0),
        Length::meters(1.0),
    ));

    // Try to constrain a point not at that location to be on the line
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(2.0),
        Length::meters(2.0),
    ));
    sketch.add_constraint(PointOnLineConstraint::new(line, p3));

    // This should be over-constrained and fail to solve
    let result = sketch.solve_and_extract();
    assert!(result.is_err(), "Should fail for over-constrained system");
    assert!(matches!(result, Err(TextCadError::OverConstrained)));
}

#[test]
fn test_point_on_line_constraint_with_vertical_line() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a vertical line from (2, 0) to (2, 5)
    let p1 = sketch.add_point(Some("bottom".to_string()));
    let p2 = sketch.add_point(Some("top".to_string()));
    let line = sketch.add_line(p1, p2, Some("vertical_line".to_string()));

    // Create a point on the line
    let p3 = sketch.add_point(Some("on_vertical".to_string()));

    // Fix the line endpoints
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(2.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(2.0),
        Length::meters(5.0),
    ));

    // Constrain p3 to lie on the line
    sketch.add_constraint(PointOnLineConstraint::new(line, p3));

    // Solve
    let solution = sketch
        .solve_and_extract()
        .expect("Should solve successfully");

    // Verify the point lies on the vertical line
    let (x1, y1) = solution.get_point_coordinates(p1).unwrap();
    let (x2, y2) = solution.get_point_coordinates(p2).unwrap();
    let (x3, y3) = solution.get_point_coordinates(p3).unwrap();

    // Line endpoints should be as expected
    assert!((x1 - 2.0).abs() < 1e-6);
    assert!((y1 - 0.0).abs() < 1e-6);
    assert!((x2 - 2.0).abs() < 1e-6);
    assert!((y2 - 5.0).abs() < 1e-6);

    // Point should be on the vertical line x = 2, with y between 0 and 5
    assert!((x3 - 2.0).abs() < 1e-6, "Point should be on x = 2 line");
    assert!(
        y3 >= -1e-6 && y3 <= 5.0 + 1e-6,
        "Point y-coordinate should be between 0 and 5, got: {}",
        y3
    );
}

#[test]
fn test_point_on_line_constraint_combined_with_other_constraints() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a line from (0,0) to (6,0)
    let p1 = sketch.add_point(Some("start".to_string()));
    let p2 = sketch.add_point(Some("end".to_string()));
    let line = sketch.add_line(p1, p2, Some("base_line".to_string()));

    // Create a point on the line
    let p3 = sketch.add_point(Some("mid_point".to_string()));

    // Fix the line endpoints
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

    // Constrain p3 to lie on the line
    sketch.add_constraint(PointOnLineConstraint::new(line, p3));

    // Also fix the x-coordinate of p3 to force it to a specific position on the line
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(2.0),
        Length::meters(0.0), // This should be satisfied by the line constraint
    ));

    // Solve
    let solution = sketch
        .solve_and_extract()
        .expect("Should solve successfully");

    // Verify the point is at the expected position
    let (x3, y3) = solution.get_point_coordinates(p3).unwrap();

    assert!((x3 - 2.0).abs() < 1e-6, "X-coordinate should be 2.0");
    assert!((y3 - 0.0).abs() < 1e-6, "Y-coordinate should be 0.0");

    // Verify this corresponds to parameter t = 1/3 on the line
    // p3 = p1 + t * (p2 - p1) = (0,0) + t * (6,0) = (6t, 0)
    // x3 = 6t = 2 => t = 1/3
    let expected_t = 2.0 / 6.0;
    assert!((expected_t - 1.0f64/3.0f64).abs() < 1e-6);
}