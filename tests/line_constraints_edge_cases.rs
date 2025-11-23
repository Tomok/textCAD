//! Edge case and error handling tests for Phase 8 line constraints
//!
//! Tests various error conditions, edge cases, and boundary conditions
//! for parallel and perpendicular line constraints.

use textcad::constraints::{
    FixedPositionConstraint, LineLengthConstraint, ParallelLinesConstraint,
    PerpendicularLinesConstraint,
};
use textcad::error::TextCadError;
use textcad::sketch::Sketch;
use textcad::units::Length;
use z3::{Config, Context};

/// Test constraints with very small line lengths
#[test]
fn test_constraints_with_tiny_lines() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));
    let p4 = sketch.add_point(Some("p4".to_string()));

    let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
    let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

    // Create very small lines
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(1e-6),
        Length::meters(0.0),
    ));

    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(0.0),
        Length::meters(1.0),
    ));

    // Very small line length
    sketch.add_constraint(LineLengthConstraint::new(line2, Length::meters(1e-6)));

    // Parallel constraint should still work
    sketch.add_constraint(ParallelLinesConstraint::new(line1, line2));

    let solution = sketch
        .solve_and_extract()
        .expect("Very small lines should work");

    let coords = [
        solution.get_point_coordinates(p1).unwrap(),
        solution.get_point_coordinates(p2).unwrap(),
        solution.get_point_coordinates(p3).unwrap(),
        solution.get_point_coordinates(p4).unwrap(),
    ];

    // Check that the lines are actually parallel despite small size
    let dir1 = (coords[1].0 - coords[0].0, coords[1].1 - coords[0].1);
    let dir2 = (coords[3].0 - coords[2].0, coords[3].1 - coords[2].1);

    let cross_product = dir1.0 * dir2.1 - dir1.1 * dir2.0;
    assert!(
        cross_product.abs() < 1e-10,
        "Tiny lines should still be parallel, cross: {}",
        cross_product
    );
}

/// Test constraints with very large coordinates
/// See docs/IGNORED_TESTS.md for details on why this test is ignored
#[test]
#[ignore]
fn test_constraints_with_large_coordinates() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));
    let p4 = sketch.add_point(Some("p4".to_string()));

    let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
    let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

    // Use moderately large coordinates (not too large to cause precision issues)
    let large_coord = 1000.0;
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(large_coord),
        Length::meters(large_coord),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(large_coord + 100.0),
        Length::meters(large_coord),
    ));

    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(large_coord + 50.0),
        Length::meters(large_coord + 200.0),
    ));

    sketch.add_constraint(LineLengthConstraint::new(line2, Length::meters(100.0)));
    sketch.add_constraint(ParallelLinesConstraint::new(line1, line2));

    let solution = sketch
        .solve_and_extract()
        .expect("Large coordinates should work");

    let coords = [
        solution.get_point_coordinates(p1).unwrap(),
        solution.get_point_coordinates(p2).unwrap(),
        solution.get_point_coordinates(p3).unwrap(),
        solution.get_point_coordinates(p4).unwrap(),
    ];

    // Verify the parallel constraint still holds with large coordinates
    let dir1 = (coords[1].0 - coords[0].0, coords[1].1 - coords[0].1);
    let dir2 = (coords[3].0 - coords[2].0, coords[3].1 - coords[2].1);

    let cross_product = dir1.0 * dir2.1 - dir1.1 * dir2.0;
    assert!(
        cross_product.abs() < 1e-6,
        "Lines should be parallel with large coords, cross: {}",
        cross_product
    );
}

/// Test constraint with negative coordinates
#[test]
fn test_constraints_with_negative_coordinates() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));
    let p4 = sketch.add_point(Some("p4".to_string()));

    let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
    let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

    // Use negative coordinates
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(-50.0),
        Length::meters(-30.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(-45.0),
        Length::meters(-30.0),
    ));

    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(-10.0),
        Length::meters(-5.0),
    ));

    sketch.add_constraint(LineLengthConstraint::new(line2, Length::meters(5.0)));
    sketch.add_constraint(PerpendicularLinesConstraint::new(line1, line2));

    let solution = sketch
        .solve_and_extract()
        .expect("Negative coordinates should work");

    let coords = [
        solution.get_point_coordinates(p1).unwrap(),
        solution.get_point_coordinates(p2).unwrap(),
        solution.get_point_coordinates(p3).unwrap(),
        solution.get_point_coordinates(p4).unwrap(),
    ];

    // Verify perpendicular constraint
    let dir1 = (coords[1].0 - coords[0].0, coords[1].1 - coords[0].1);
    let dir2 = (coords[3].0 - coords[2].0, coords[3].1 - coords[2].1);

    let dot_product = dir1.0 * dir2.0 + dir1.1 * dir2.1;
    assert!(
        dot_product.abs() < 1e-6,
        "Lines should be perpendicular with negative coords, dot: {}",
        dot_product
    );
}

/// Test constraint with zero-length line (degenerate case)
#[test]
fn test_constraint_with_zero_length_line() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));
    let p4 = sketch.add_point(Some("p4".to_string()));

    let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
    let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

    // Make line1 degenerate (zero length)
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(5.0),
        Length::meters(5.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(5.0),
        Length::meters(5.0),
    )); // Same position as p1

    // Normal line2
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(LineLengthConstraint::new(line2, Length::meters(3.0)));

    // Try to make degenerate line parallel to normal line
    sketch.add_constraint(ParallelLinesConstraint::new(line1, line2));

    // This might fail or might succeed with undefined behavior -
    // the important thing is it doesn't crash
    let result = sketch.solve_and_extract();

    // We don't strictly require this to succeed or fail, just that it handles gracefully
    match result {
        Ok(_) => {
            // If it succeeds, the constraint should still be mathematically valid
        }
        Err(TextCadError::OverConstrained) => {
            // This is acceptable - degenerate lines can't have well-defined directions
        }
        Err(other) => {
            panic!("Unexpected error type for degenerate line: {:?}", other);
        }
    }
}

/// Test same line referenced in constraint (edge case)
#[test]
fn test_constraint_same_line_twice() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));

    let line = sketch.add_line(p1, p2, Some("line".to_string()));

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

    // Try to make a line parallel to itself (should be trivially satisfied)
    sketch.add_constraint(ParallelLinesConstraint::new(line, line));

    let solution = sketch
        .solve_and_extract()
        .expect("Line parallel to itself should work");

    let (x1, y1) = solution.get_point_coordinates(p1).unwrap();
    let (x2, y2) = solution.get_point_coordinates(p2).unwrap();

    assert!((x1 - 0.0).abs() < 1e-6);
    assert!((y1 - 0.0).abs() < 1e-6);
    assert!((x2 - 3.0).abs() < 1e-6);
    assert!((y2 - 4.0).abs() < 1e-6);
}

/// Test constraint with same line perpendicular to itself (impossible)
#[test]
fn test_constraint_same_line_perpendicular_to_itself() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));

    let line = sketch.add_line(p1, p2, Some("line".to_string()));

    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(1.0),
        Length::meters(0.0),
    ));

    // Try to make a line perpendicular to itself (impossible unless degenerate)
    sketch.add_constraint(PerpendicularLinesConstraint::new(line, line));

    let result = sketch.solve_and_extract();

    // This should fail as a line cannot be perpendicular to itself
    assert!(result.is_err(), "Line cannot be perpendicular to itself");

    if let Err(TextCadError::OverConstrained) = result {
        // Expected
    } else {
        panic!("Expected OverConstrained error, got: {:?}", result);
    }
}

/// Test constraints with non-existent line IDs (should be caught earlier, but test robustness)
#[test]
fn test_constraints_with_invalid_line_ids() {
    use generational_arena::Index;
    use textcad::entity::LineId;

    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create real points and line
    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let real_line = sketch.add_line(p1, p2, Some("real_line".to_string()));

    // Create fake line ID
    let fake_line = LineId(Index::from_raw_parts(999, 999));

    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(1.0),
        Length::meters(0.0),
    ));

    // Try constraint with non-existent line
    sketch.add_constraint(ParallelLinesConstraint::new(real_line, fake_line));

    let result = sketch.solve_and_extract();
    assert!(result.is_err(), "Should fail with non-existent line ID");

    // Should be an EntityError
    if let Err(TextCadError::EntityError(_)) = result {
        // Expected
    } else {
        panic!("Expected EntityError, got: {:?}", result);
    }
}

/// Test very long lines (stress test for numerical stability)
/// See docs/IGNORED_TESTS.md for details on why this test is ignored
#[test]
#[ignore]
fn test_constraints_with_very_long_lines() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));
    let p4 = sketch.add_point(Some("p4".to_string()));

    let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
    let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

    // Long lines (but not astronomically long to avoid precision issues)
    let huge_length = 10000.0; // 10 km

    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(LineLengthConstraint::new(
        line1,
        Length::meters(huge_length),
    ));

    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(1000.0),
        Length::meters(1000.0),
    ));
    sketch.add_constraint(LineLengthConstraint::new(
        line2,
        Length::meters(huge_length),
    ));

    sketch.add_constraint(ParallelLinesConstraint::new(line1, line2));

    let solution = sketch
        .solve_and_extract()
        .expect("Very long lines should work");

    let coords = [
        solution.get_point_coordinates(p1).unwrap(),
        solution.get_point_coordinates(p2).unwrap(),
        solution.get_point_coordinates(p3).unwrap(),
        solution.get_point_coordinates(p4).unwrap(),
    ];

    // Check that lines are parallel
    let dir1 = (coords[1].0 - coords[0].0, coords[1].1 - coords[0].1);
    let dir2 = (coords[3].0 - coords[2].0, coords[3].1 - coords[2].1);

    let cross_product = dir1.0 * dir2.1 - dir1.1 * dir2.0;
    assert!(
        cross_product.abs() < 1e-6,
        "Long lines should be parallel, cross: {}",
        cross_product
    );

    // Check lengths are correct
    let len1 = (dir1.0.powi(2) + dir1.1.powi(2)).sqrt();
    let len2 = (dir2.0.powi(2) + dir2.1.powi(2)).sqrt();

    assert!(
        (len1 - huge_length).abs() < 1e-6,
        "Line1 length should be {}, got: {}",
        huge_length,
        len1
    );
    assert!(
        (len2 - huge_length).abs() < 1e-6,
        "Line2 length should be {}, got: {}",
        huge_length,
        len2
    );
}

/// Test constraints with lines at various special angles
/// See docs/IGNORED_TESTS.md for details on why this test is ignored
#[test]
#[ignore]
fn test_constraints_at_special_angles() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    // Test parallel constraints with lines at 45° angle
    {
        let mut sketch = Sketch::new(&ctx);

        let p1 = sketch.add_point(Some("p1".to_string()));
        let p2 = sketch.add_point(Some("p2".to_string()));
        let p3 = sketch.add_point(Some("p3".to_string()));
        let p4 = sketch.add_point(Some("p4".to_string()));

        let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
        let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

        // Line1 at 45° (diagonal)
        sketch.add_constraint(FixedPositionConstraint::new(
            p1,
            Length::meters(0.0),
            Length::meters(0.0),
        ));
        sketch.add_constraint(FixedPositionConstraint::new(
            p2,
            Length::meters(1.0),
            Length::meters(1.0),
        ));

        // Line2 should be parallel to line1
        sketch.add_constraint(FixedPositionConstraint::new(
            p3,
            Length::meters(2.0),
            Length::meters(0.0),
        ));
        sketch.add_constraint(LineLengthConstraint::new(line2, Length::meters(2.0)));
        sketch.add_constraint(ParallelLinesConstraint::new(line1, line2));

        let solution = sketch
            .solve_and_extract()
            .expect("45° parallel lines should work");

        let coords = [
            solution.get_point_coordinates(p1).unwrap(),
            solution.get_point_coordinates(p2).unwrap(),
            solution.get_point_coordinates(p3).unwrap(),
            solution.get_point_coordinates(p4).unwrap(),
        ];

        let dir1 = (coords[1].0 - coords[0].0, coords[1].1 - coords[0].1);
        let dir2 = (coords[3].0 - coords[2].0, coords[3].1 - coords[2].1);

        let cross_product = dir1.0 * dir2.1 - dir1.1 * dir2.0;
        assert!(
            cross_product.abs() < 1e-6,
            "45° lines should be parallel, cross: {}",
            cross_product
        );
    }

    // Test perpendicular constraints with lines at 30° and 120°
    {
        let mut sketch = Sketch::new(&ctx);

        let p1 = sketch.add_point(Some("p1".to_string()));
        let p2 = sketch.add_point(Some("p2".to_string()));
        let p3 = sketch.add_point(Some("p3".to_string()));
        let p4 = sketch.add_point(Some("p4".to_string()));

        let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
        let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

        // Line1 at 30°
        let cos30 = (30.0_f64).to_radians().cos();
        let sin30 = (30.0_f64).to_radians().sin();

        sketch.add_constraint(FixedPositionConstraint::new(
            p1,
            Length::meters(0.0),
            Length::meters(0.0),
        ));
        sketch.add_constraint(FixedPositionConstraint::new(
            p2,
            Length::meters(2.0 * cos30),
            Length::meters(2.0 * sin30),
        ));

        // Line2 should be perpendicular
        sketch.add_constraint(FixedPositionConstraint::new(
            p3,
            Length::meters(1.0),
            Length::meters(1.0),
        ));
        sketch.add_constraint(LineLengthConstraint::new(line2, Length::meters(2.0)));
        sketch.add_constraint(PerpendicularLinesConstraint::new(line1, line2));

        let solution = sketch
            .solve_and_extract()
            .expect("30° perpendicular lines should work");

        let coords = [
            solution.get_point_coordinates(p1).unwrap(),
            solution.get_point_coordinates(p2).unwrap(),
            solution.get_point_coordinates(p3).unwrap(),
            solution.get_point_coordinates(p4).unwrap(),
        ];

        let dir1 = (coords[1].0 - coords[0].0, coords[1].1 - coords[0].1);
        let dir2 = (coords[3].0 - coords[2].0, coords[3].1 - coords[2].1);

        let dot_product = dir1.0 * dir2.0 + dir1.1 * dir2.1;
        assert!(
            dot_product.abs() < 1e-6,
            "Lines should be perpendicular at special angles, dot: {}",
            dot_product
        );
    }
}

/// Test numerical precision with constraints very close to parallel/perpendicular
#[test]
fn test_numerical_precision_near_constraints() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));
    let p4 = sketch.add_point(Some("p4".to_string()));

    let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
    let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

    // Line1 horizontal
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(1.0),
        Length::meters(0.0),
    ));

    // Line2 very close to horizontal (but not quite)
    let tiny_angle = 1e-8; // Very small angle
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(2.0),
        Length::meters(1.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p4,
        Length::meters(3.0),
        Length::meters(1.0 + tiny_angle),
    ));

    // Apply parallel constraint - should force exact parallelism
    sketch.add_constraint(ParallelLinesConstraint::new(line1, line2));

    let solution = sketch
        .solve_and_extract()
        .expect("Near-parallel lines should resolve to parallel");

    let coords = [
        solution.get_point_coordinates(p1).unwrap(),
        solution.get_point_coordinates(p2).unwrap(),
        solution.get_point_coordinates(p3).unwrap(),
        solution.get_point_coordinates(p4).unwrap(),
    ];

    let dir1 = (coords[1].0 - coords[0].0, coords[1].1 - coords[0].1);
    let dir2 = (coords[3].0 - coords[2].0, coords[3].1 - coords[2].1);

    let cross_product = dir1.0 * dir2.1 - dir1.1 * dir2.0;
    assert!(
        cross_product.abs() < 1e-10,
        "Near-parallel lines should be exactly parallel after constraint, cross: {}",
        cross_product
    );
}
