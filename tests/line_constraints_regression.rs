//! Regression tests for Phase 8 line constraints
//!
//! These tests ensure that the new line constraints don't break existing
//! functionality and properly interact with other constraints.

use textcad::constraints::{
    CoincidentPointsConstraint, FixedPositionConstraint, LineLengthConstraint,
    ParallelLinesConstraint, PerpendicularLinesConstraint,
};
use textcad::error::TextCadError;
use textcad::sketch::Sketch;
use textcad::units::Length;
use z3::{Config, Context};

/// Test that existing point constraints still work after adding line constraints
#[test]
fn test_point_constraints_still_work() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create basic point constraints (should work as before)
    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));

    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(3.0),
        Length::meters(4.0),
    ));
    sketch.add_constraint(CoincidentPointsConstraint::new(p1, p2));

    let solution = sketch
        .solve_and_extract()
        .expect("Basic point constraints should still work");

    let (x1, y1) = solution.get_point_coordinates(p1).unwrap();
    let (x2, y2) = solution.get_point_coordinates(p2).unwrap();

    assert!((x1 - 3.0).abs() < 1e-6);
    assert!((y1 - 4.0).abs() < 1e-6);
    assert!((x1 - x2).abs() < 1e-6, "Points should be coincident");
    assert!((y1 - y2).abs() < 1e-6, "Points should be coincident");
}

/// Test that line length constraints work with point constraints
#[test]
fn test_line_length_with_point_constraints() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));

    let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));

    // Combine point and line constraints
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(CoincidentPointsConstraint::new(p2, p3));
    sketch.add_constraint(LineLengthConstraint::new(line1, Length::meters(5.0)));

    let solution = sketch
        .solve_and_extract()
        .expect("Line length with point constraints should work");

    let (x1, y1) = solution.get_point_coordinates(p1).unwrap();
    let (x2, y2) = solution.get_point_coordinates(p2).unwrap();
    let (x3, y3) = solution.get_point_coordinates(p3).unwrap();

    // Check fixed position
    assert!((x1 - 0.0).abs() < 1e-6);
    assert!((y1 - 0.0).abs() < 1e-6);

    // Check coincident points
    assert!((x2 - x3).abs() < 1e-6);
    assert!((y2 - y3).abs() < 1e-6);

    // Check line length
    let length = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    assert!((length - 5.0).abs() < 1e-6);
}

/// Test multiple line constraints on the same line
#[test]
fn test_multiple_constraints_same_line() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a line that is both parallel to another line AND has a fixed length
    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));
    let p4 = sketch.add_point(Some("p4".to_string()));

    let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
    let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

    // Fix line1 as horizontal
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

    // Fix line2 start point
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(2.0),
        Length::meters(3.0),
    ));

    // Apply multiple constraints to line2
    sketch.add_constraint(LineLengthConstraint::new(line2, Length::meters(4.0)));
    sketch.add_constraint(ParallelLinesConstraint::new(line1, line2));

    let solution = sketch
        .solve_and_extract()
        .expect("Multiple constraints on same line should work");

    let coords = [
        solution.get_point_coordinates(p1).unwrap(),
        solution.get_point_coordinates(p2).unwrap(),
        solution.get_point_coordinates(p3).unwrap(),
        solution.get_point_coordinates(p4).unwrap(),
    ];

    // Check line1 is horizontal as fixed
    assert!((coords[0].0 - 0.0).abs() < 1e-6);
    assert!((coords[0].1 - 0.0).abs() < 1e-6);
    assert!((coords[1].0 - 6.0).abs() < 1e-6);
    assert!((coords[1].1 - 0.0).abs() < 1e-6);

    // Check line2 start point
    assert!((coords[2].0 - 2.0).abs() < 1e-6);
    assert!((coords[2].1 - 3.0).abs() < 1e-6);

    // Check line2 is parallel to line1 (horizontal)
    let line1_dir = (coords[1].0 - coords[0].0, coords[1].1 - coords[0].1);
    let line2_dir = (coords[3].0 - coords[2].0, coords[3].1 - coords[2].1);
    let cross_product = line1_dir.0 * line2_dir.1 - line1_dir.1 * line2_dir.0;
    assert!(
        cross_product.abs() < 1e-6,
        "Lines should be parallel, cross: {}",
        cross_product
    );

    // Check line2 has correct length
    let line2_length = (line2_dir.0.powi(2) + line2_dir.1.powi(2)).sqrt();
    assert!(
        (line2_length - 4.0).abs() < 1e-6,
        "Line2 should have length 4.0m, got: {}",
        line2_length
    );

    // Since line2 is horizontal and parallel to line1, it should be exactly horizontal
    assert!(
        line2_dir.1.abs() < 1e-6,
        "Line2 should be horizontal, dy: {}",
        line2_dir.1
    );
}

/// Test constraint order independence - different orders should give same result
#[test]
fn test_constraint_order_independence() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    // First ordering: parallel first, then length
    let mut sketch1 = Sketch::new(&ctx);
    let p1_a = sketch1.add_point(Some("p1".to_string()));
    let p2_a = sketch1.add_point(Some("p2".to_string()));
    let p3_a = sketch1.add_point(Some("p3".to_string()));
    let p4_a = sketch1.add_point(Some("p4".to_string()));

    let line1_a = sketch1.add_line(p1_a, p2_a, Some("line1".to_string()));
    let line2_a = sketch1.add_line(p3_a, p4_a, Some("line2".to_string()));

    // Fixed positions
    sketch1.add_constraint(FixedPositionConstraint::new(
        p1_a,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch1.add_constraint(FixedPositionConstraint::new(
        p2_a,
        Length::meters(5.0),
        Length::meters(0.0),
    ));
    sketch1.add_constraint(FixedPositionConstraint::new(
        p3_a,
        Length::meters(1.0),
        Length::meters(2.0),
    ));

    // Order 1: Parallel first, then length
    sketch1.add_constraint(ParallelLinesConstraint::new(line1_a, line2_a));
    sketch1.add_constraint(LineLengthConstraint::new(line2_a, Length::meters(3.0)));

    let solution1 = sketch1
        .solve_and_extract()
        .expect("First ordering should work");

    // Second ordering: length first, then parallel
    let mut sketch2 = Sketch::new(&ctx);
    let p1_b = sketch2.add_point(Some("p1".to_string()));
    let p2_b = sketch2.add_point(Some("p2".to_string()));
    let p3_b = sketch2.add_point(Some("p3".to_string()));
    let p4_b = sketch2.add_point(Some("p4".to_string()));

    let line1_b = sketch2.add_line(p1_b, p2_b, Some("line1".to_string()));
    let line2_b = sketch2.add_line(p3_b, p4_b, Some("line2".to_string()));

    // Same fixed positions
    sketch2.add_constraint(FixedPositionConstraint::new(
        p1_b,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch2.add_constraint(FixedPositionConstraint::new(
        p2_b,
        Length::meters(5.0),
        Length::meters(0.0),
    ));
    sketch2.add_constraint(FixedPositionConstraint::new(
        p3_b,
        Length::meters(1.0),
        Length::meters(2.0),
    ));

    // Order 2: Length first, then parallel
    sketch2.add_constraint(LineLengthConstraint::new(line2_b, Length::meters(3.0)));
    sketch2.add_constraint(ParallelLinesConstraint::new(line1_b, line2_b));

    let solution2 = sketch2
        .solve_and_extract()
        .expect("Second ordering should work");

    // Results should satisfy the same constraints (but coordinates might differ due to solver variance)
    let coords1 = [
        solution1.get_point_coordinates(p1_a).unwrap(),
        solution1.get_point_coordinates(p2_a).unwrap(),
        solution1.get_point_coordinates(p3_a).unwrap(),
        solution1.get_point_coordinates(p4_a).unwrap(),
    ];

    let coords2 = [
        solution2.get_point_coordinates(p1_b).unwrap(),
        solution2.get_point_coordinates(p2_b).unwrap(),
        solution2.get_point_coordinates(p3_b).unwrap(),
        solution2.get_point_coordinates(p4_b).unwrap(),
    ];

    // Verify both solutions satisfy the parallel constraint
    let dir1_1 = (coords1[1].0 - coords1[0].0, coords1[1].1 - coords1[0].1);
    let dir2_1 = (coords1[3].0 - coords1[2].0, coords1[3].1 - coords1[2].1);
    let cross1 = dir1_1.0 * dir2_1.1 - dir1_1.1 * dir2_1.0;
    assert!(
        cross1.abs() < 1e-6,
        "Solution 1 should satisfy parallel constraint"
    );

    let dir1_2 = (coords2[1].0 - coords2[0].0, coords2[1].1 - coords2[0].1);
    let dir2_2 = (coords2[3].0 - coords2[2].0, coords2[3].1 - coords2[2].1);
    let cross2 = dir1_2.0 * dir2_2.1 - dir1_2.1 * dir2_2.0;
    assert!(
        cross2.abs() < 1e-6,
        "Solution 2 should satisfy parallel constraint"
    );

    // Verify both solutions satisfy the length constraint
    let len1 = (dir2_1.0.powi(2) + dir2_1.1.powi(2)).sqrt();
    let len2 = (dir2_2.0.powi(2) + dir2_2.1.powi(2)).sqrt();
    assert!(
        (len1 - 3.0).abs() < 1e-6,
        "Solution 1 line2 should have length 3.0"
    );
    assert!(
        (len2 - 3.0).abs() < 1e-6,
        "Solution 2 line2 should have length 3.0"
    );
}

/// Test that existing line length constraints still work correctly
#[test]
fn test_existing_line_length_constraints_unchanged() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // This should work exactly as it did before adding parallel/perpendicular constraints
    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let line = sketch.add_line(p1, p2, Some("line".to_string()));

    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(1.0),
        Length::meters(2.0),
    ));
    sketch.add_constraint(LineLengthConstraint::new(line, Length::meters(7.0)));

    let solution = sketch
        .solve_and_extract()
        .expect("Existing line length constraints should work");

    let (x1, y1) = solution.get_point_coordinates(p1).unwrap();
    let (x2, y2) = solution.get_point_coordinates(p2).unwrap();

    assert!((x1 - 1.0).abs() < 1e-6);
    assert!((y1 - 2.0).abs() < 1e-6);

    let length = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    assert!((length - 7.0).abs() < 1e-6);
}

/// Test that invalid constraint combinations are properly detected
#[test]
fn test_invalid_constraint_combinations() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a scenario that should be impossible:
    // Two fixed lines that we try to make both parallel and perpendicular
    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));
    let p4 = sketch.add_point(Some("p4".to_string()));

    let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
    let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

    // Fix both lines in conflicting orientations
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(4.0),
        Length::meters(0.0),
    )); // Horizontal line

    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p4,
        Length::meters(0.0),
        Length::meters(3.0),
    )); // Vertical line

    // Try to make the horizontal and vertical lines parallel (impossible)
    sketch.add_constraint(ParallelLinesConstraint::new(line1, line2));

    let result = sketch.solve_and_extract();
    assert!(
        result.is_err(),
        "Should fail: horizontal and vertical lines cannot be parallel"
    );

    if let Err(TextCadError::OverConstrained) = result {
        // Expected
    } else {
        panic!("Expected OverConstrained error, got: {:?}", result);
    }
}

/// Test complex mixed constraint scenarios that should work
#[test]
fn test_complex_mixed_constraints_working() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a more complex but solvable scenario:
    // A square where we specify some constraints redundantly but consistently
    let bottom_left = sketch.add_point(Some("bottom_left".to_string()));
    let bottom_right = sketch.add_point(Some("bottom_right".to_string()));
    let top_right = sketch.add_point(Some("top_right".to_string()));
    let top_left = sketch.add_point(Some("top_left".to_string()));

    let bottom = sketch.add_line(bottom_left, bottom_right, Some("bottom".to_string()));
    let right = sketch.add_line(bottom_right, top_right, Some("right".to_string()));
    let top = sketch.add_line(top_right, top_left, Some("top".to_string()));
    let left = sketch.add_line(top_left, bottom_left, Some("left".to_string()));

    // Fix one corner
    sketch.add_constraint(FixedPositionConstraint::new(
        bottom_left,
        Length::meters(0.0),
        Length::meters(0.0),
    ));

    // Fix adjacent corner to define orientation and size
    sketch.add_constraint(FixedPositionConstraint::new(
        bottom_right,
        Length::meters(5.0),
        Length::meters(0.0),
    ));

    // Add redundant but consistent constraints
    sketch.add_constraint(LineLengthConstraint::new(bottom, Length::meters(5.0))); // Redundant with fixed positions
    sketch.add_constraint(LineLengthConstraint::new(left, Length::meters(5.0))); // Square constraint
    sketch.add_constraint(LineLengthConstraint::new(right, Length::meters(5.0))); // Square constraint
    sketch.add_constraint(LineLengthConstraint::new(top, Length::meters(5.0))); // Square constraint

    // Parallel constraints
    sketch.add_constraint(ParallelLinesConstraint::new(bottom, top));
    sketch.add_constraint(ParallelLinesConstraint::new(left, right));

    // Perpendicular constraints
    sketch.add_constraint(PerpendicularLinesConstraint::new(bottom, left));
    sketch.add_constraint(PerpendicularLinesConstraint::new(bottom, right));

    let solution = sketch
        .solve_and_extract()
        .expect("Complex but consistent constraints should work");

    // Verify it's actually a square
    let coords = [
        solution.get_point_coordinates(bottom_left).unwrap(),
        solution.get_point_coordinates(bottom_right).unwrap(),
        solution.get_point_coordinates(top_right).unwrap(),
        solution.get_point_coordinates(top_left).unwrap(),
    ];

    // Check that it forms a valid square (don't require specific orientation)
    let sides = [
        ((coords[1].0 - coords[0].0), (coords[1].1 - coords[0].1)), // bottom
        ((coords[2].0 - coords[1].0), (coords[2].1 - coords[1].1)), // right
        ((coords[3].0 - coords[2].0), (coords[3].1 - coords[2].1)), // top
        ((coords[0].0 - coords[3].0), (coords[0].1 - coords[3].1)), // left
    ];

    // Check all sides have length 5
    for (i, side) in sides.iter().enumerate() {
        let length = (side.0.powi(2) + side.1.powi(2)).sqrt();
        assert!(
            (length - 5.0).abs() < 1e-6,
            "Side {} should have length 5.0, got {}",
            i,
            length
        );
    }

    // Check adjacent sides are perpendicular
    for i in 0..4 {
        let current = sides[i];
        let next = sides[(i + 1) % 4];
        let dot = current.0 * next.0 + current.1 * next.1;
        assert!(
            dot.abs() < 1e-6,
            "Adjacent sides {} and {} should be perpendicular",
            i,
            (i + 1) % 4
        );
    }
}

/// Test that the solution extraction API works with line constraints
#[test]
fn test_solution_extraction_with_line_constraints() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));
    let p4 = sketch.add_point(Some("p4".to_string()));

    let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
    let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

    // Create a simple perpendicular configuration
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
        Length::meters(1.0),
    ));
    sketch.add_constraint(LineLengthConstraint::new(line2, Length::meters(2.0)));
    sketch.add_constraint(PerpendicularLinesConstraint::new(line1, line2));

    let mut solution = sketch
        .solve_and_extract()
        .expect("Solution extraction should work with line constraints");

    // Test that we can extract line parameters
    let p1_coords = solution.get_point_coordinates(p1).unwrap();
    let p2_coords = solution.get_point_coordinates(p2).unwrap();
    let p3_coords = solution.get_point_coordinates(p3).unwrap();
    let p4_coords = solution.get_point_coordinates(p4).unwrap();

    // Test line parameter extraction
    let line1_params = solution
        .extract_line_parameters(line1, p1_coords, p2_coords)
        .expect("Should extract line1 parameters");
    let line2_params = solution
        .extract_line_parameters(line2, p3_coords, p4_coords)
        .expect("Should extract line2 parameters");

    // Check line1 parameters (horizontal line from (0,0) to (3,0))
    assert!((line1_params.length - 3.0).abs() < 1e-6);
    assert!((line1_params.angle.to_radians() - 0.0).abs() < 1e-6); // Should be horizontal

    // Check line2 parameters (perpendicular line with length 2.0)
    assert!((line2_params.length - 2.0).abs() < 1e-6);

    // Check that line2 is perpendicular to line1 by checking the dot product of their direction vectors
    let line1_dir = (p2_coords.0 - p1_coords.0, p2_coords.1 - p1_coords.1);
    let line2_dir = (p4_coords.0 - p3_coords.0, p4_coords.1 - p3_coords.1);
    let dot_product = line1_dir.0 * line2_dir.0 + line1_dir.1 * line2_dir.1;
    assert!(dot_product.abs() < 1e-6, "Lines should be perpendicular");
}

/// Test performance with many line constraints (regression for performance)
#[test]
fn test_performance_many_line_constraints() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a grid of lines with parallel constraints (should still solve reasonably fast)
    const GRID_SIZE: usize = 5;
    let mut points = Vec::new();
    let mut h_lines = Vec::new();
    let mut v_lines = Vec::new();

    // Create grid of points
    for i in 0..GRID_SIZE {
        for j in 0..GRID_SIZE {
            let point = sketch.add_point(Some(format!("p_{}_{}", i, j)));
            points.push(point);
        }
    }

    // Create horizontal lines
    for i in 0..GRID_SIZE {
        for j in 0..GRID_SIZE - 1 {
            let line = sketch.add_line(
                points[i * GRID_SIZE + j],
                points[i * GRID_SIZE + j + 1],
                Some(format!("h_{}_{}", i, j)),
            );
            h_lines.push(line);
        }
    }

    // Create vertical lines
    for i in 0..GRID_SIZE - 1 {
        for j in 0..GRID_SIZE {
            let line = sketch.add_line(
                points[i * GRID_SIZE + j],
                points[(i + 1) * GRID_SIZE + j],
                Some(format!("v_{}_{}", i, j)),
            );
            v_lines.push(line);
        }
    }

    // Fix some reference points to define the grid
    sketch.add_constraint(FixedPositionConstraint::new(
        points[0],
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        points[1],
        Length::meters(1.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        points[GRID_SIZE],
        Length::meters(0.0),
        Length::meters(1.0),
    ));

    // Add parallel constraints for all horizontal lines
    for i in 1..h_lines.len() {
        sketch.add_constraint(ParallelLinesConstraint::new(h_lines[0], h_lines[i]));
    }

    // Add parallel constraints for all vertical lines
    for i in 1..v_lines.len() {
        sketch.add_constraint(ParallelLinesConstraint::new(v_lines[0], v_lines[i]));
    }

    // Add perpendicular constraints between horizontal and vertical
    sketch.add_constraint(PerpendicularLinesConstraint::new(h_lines[0], v_lines[0]));

    // This should solve without timing out
    let start = std::time::Instant::now();
    let _solution = sketch
        .solve_and_extract()
        .expect("Many line constraints should solve");
    let elapsed = start.elapsed();

    // Should solve in reasonable time (less than 5 seconds for this small grid)
    assert!(
        elapsed.as_secs() < 5,
        "Grid constraint solving took too long: {:?}",
        elapsed
    );
}
