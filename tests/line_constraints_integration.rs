//! Integration tests for Phase 8 line constraints
//!
//! Tests complete workflows including sketch creation, constraint application,
//! solving, and solution extraction for parallel and perpendicular line constraints.

use textcad::constraints::{
    FixedPositionConstraint, LineLengthConstraint, ParallelLinesConstraint,
    PerpendicularLinesConstraint,
};
use textcad::error::TextCadError;
use textcad::sketch::Sketch;
use textcad::units::Length;
use z3::{Config, Context};

#[test]
fn test_parallel_lines_integration_simple() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create two horizontal lines that should be parallel
    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));
    let p4 = sketch.add_point(Some("p4".to_string()));

    let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
    let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

    // Fix line1 as horizontal at y=0
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

    // Fix starting point of line2
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(1.0),
        Length::meters(2.0),
    ));

    // Set line2 length
    sketch.add_constraint(LineLengthConstraint::new(line2, Length::meters(4.0)));

    // Make lines parallel
    sketch.add_constraint(ParallelLinesConstraint::new(line1, line2));

    // Solve
    let solution = sketch
        .solve_and_extract()
        .expect("Should solve successfully");

    // Verify the lines are parallel (same direction)
    let (x1, y1) = solution.get_point_coordinates(p1).unwrap();
    let (x2, y2) = solution.get_point_coordinates(p2).unwrap();
    let (x3, y3) = solution.get_point_coordinates(p3).unwrap();
    let (x4, y4) = solution.get_point_coordinates(p4).unwrap();

    // Line1 direction: (3, 0)
    let line1_dx = x2 - x1;
    let line1_dy = y2 - y1;

    // Line2 direction: (x4-x3, y4-y3) should be parallel to (3, 0)
    let line2_dx = x4 - x3;
    let line2_dy = y4 - y3;

    // Check parallel condition: cross product should be zero
    let cross_product = line1_dx * line2_dy - line1_dy * line2_dx;
    assert!(
        (cross_product).abs() < 1e-6,
        "Lines should be parallel, cross product: {}",
        cross_product
    );

    // Check that line2 has the correct length
    let line2_length = ((line2_dx).powi(2) + (line2_dy).powi(2)).sqrt();
    assert!(
        (line2_length - 4.0).abs() < 1e-6,
        "Line2 should have length 4.0m, got: {}",
        line2_length
    );
}

#[test]
fn test_perpendicular_lines_integration_simple() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create two lines that should be perpendicular
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
        Length::meters(5.0),
        Length::meters(0.0),
    ));

    // Fix starting point of line2
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(2.0),
        Length::meters(1.0),
    ));

    // Set line2 length
    sketch.add_constraint(LineLengthConstraint::new(line2, Length::meters(3.0)));

    // Make lines perpendicular
    sketch.add_constraint(PerpendicularLinesConstraint::new(line1, line2));

    // Solve
    let solution = sketch
        .solve_and_extract()
        .expect("Should solve successfully");

    // Verify the lines are perpendicular
    let (x1, y1) = solution.get_point_coordinates(p1).unwrap();
    let (x2, y2) = solution.get_point_coordinates(p2).unwrap();
    let (x3, y3) = solution.get_point_coordinates(p3).unwrap();
    let (x4, y4) = solution.get_point_coordinates(p4).unwrap();

    // Line1 direction: (5, 0) - horizontal
    let line1_dx = x2 - x1;
    let line1_dy = y2 - y1;

    // Line2 direction: should be vertical (perpendicular to horizontal)
    let line2_dx = x4 - x3;
    let line2_dy = y4 - y3;

    // Check perpendicular condition: dot product should be zero
    let dot_product = line1_dx * line2_dx + line1_dy * line2_dy;
    assert!(
        (dot_product).abs() < 1e-6,
        "Lines should be perpendicular, dot product: {}",
        dot_product
    );

    // Check that line2 has the correct length
    let line2_length = ((line2_dx).powi(2) + (line2_dy).powi(2)).sqrt();
    assert!(
        (line2_length - 3.0).abs() < 1e-6,
        "Line2 should have length 3.0m, got: {}",
        line2_length
    );

    // Since line1 is horizontal and line2 is perpendicular, line2 should be vertical
    assert!(
        (line2_dx).abs() < 1e-6,
        "Line2 should be vertical, dx: {}",
        line2_dx
    );
}

#[test]
fn test_rectangle_construction_with_all_constraints() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a rectangle using all constraint types
    let bottom_left = sketch.add_point(Some("bottom_left".to_string()));
    let bottom_right = sketch.add_point(Some("bottom_right".to_string()));
    let top_right = sketch.add_point(Some("top_right".to_string()));
    let top_left = sketch.add_point(Some("top_left".to_string()));

    let bottom = sketch.add_line(bottom_left, bottom_right, Some("bottom".to_string()));
    let right = sketch.add_line(bottom_right, top_right, Some("right".to_string()));
    let top = sketch.add_line(top_right, top_left, Some("top".to_string()));
    let left = sketch.add_line(top_left, bottom_left, Some("left".to_string()));

    // Fix the bottom-left corner
    sketch.add_constraint(FixedPositionConstraint::new(
        bottom_left,
        Length::meters(0.0),
        Length::meters(0.0),
    ));

    // Fix the bottom-right corner to define the base
    sketch.add_constraint(FixedPositionConstraint::new(
        bottom_right,
        Length::meters(6.0),
        Length::meters(0.0),
    ));

    // Set dimensions
    sketch.add_constraint(LineLengthConstraint::new(bottom, Length::meters(6.0)));
    sketch.add_constraint(LineLengthConstraint::new(left, Length::meters(4.0)));

    // Parallel constraints
    sketch.add_constraint(ParallelLinesConstraint::new(top, bottom));
    sketch.add_constraint(ParallelLinesConstraint::new(left, right));

    // Perpendicular constraints
    sketch.add_constraint(PerpendicularLinesConstraint::new(bottom, left));
    sketch.add_constraint(PerpendicularLinesConstraint::new(bottom, right));

    // Solve
    let solution = sketch
        .solve_and_extract()
        .expect("Rectangle should solve successfully");

    // Verify all coordinates
    let bl_coords = solution.get_point_coordinates(bottom_left).unwrap();
    let br_coords = solution.get_point_coordinates(bottom_right).unwrap();
    let tr_coords = solution.get_point_coordinates(top_right).unwrap();
    let tl_coords = solution.get_point_coordinates(top_left).unwrap();

    // Check fixed positions
    assert!((bl_coords.0 - 0.0).abs() < 1e-6);
    assert!((bl_coords.1 - 0.0).abs() < 1e-6);
    assert!((br_coords.0 - 6.0).abs() < 1e-6);
    assert!((br_coords.1 - 0.0).abs() < 1e-6);

    // Check rectangle properties
    let width = br_coords.0 - bl_coords.0;
    let height = (tl_coords.1 - bl_coords.1).abs(); // Take absolute value in case direction is opposite

    assert!(
        (width - 6.0).abs() < 1e-6,
        "Width should be 6.0m, got: {}",
        width
    );
    assert!(
        (height - 4.0).abs() < 1e-6,
        "Height should be 4.0m, got: {}",
        height
    );

    // Check right angles at corners
    let bl = (bl_coords.0, bl_coords.1);
    let br = (br_coords.0, br_coords.1);
    let tr = (tr_coords.0, tr_coords.1);
    let tl = (tl_coords.0, tl_coords.1);

    // Bottom edge vector
    let bottom_vec = (br.0 - bl.0, br.1 - bl.1);
    // Left edge vector
    let left_vec = (tl.0 - bl.0, tl.1 - bl.1);

    // Check perpendicular (dot product = 0)
    let dot = bottom_vec.0 * left_vec.0 + bottom_vec.1 * left_vec.1;
    assert!(
        dot.abs() < 1e-6,
        "Bottom and left edges should be perpendicular, dot: {}",
        dot
    );

    // Verify it's actually a rectangle (opposite sides equal)
    let top_width = (tr.0 - tl.0).abs();
    let right_height = (tr.1 - br.1).abs();
    assert!(
        (top_width - width).abs() < 1e-6,
        "Top edge should equal bottom edge"
    );
    assert!(
        (right_height - height).abs() < 1e-6,
        "Right edge should equal left edge"
    );
}

#[test]
fn test_conflicting_constraints_detection() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Try to create constraints that conflict: two lines that are both
    // parallel and perpendicular to each other (impossible)
    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));
    let p4 = sketch.add_point(Some("p4".to_string()));

    let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
    let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

    // Fix some positions
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
    sketch.add_constraint(FixedPositionConstraint::new(
        p4,
        Length::meters(4.0),
        Length::meters(1.0),
    ));

    // Add conflicting constraints
    sketch.add_constraint(ParallelLinesConstraint::new(line1, line2));
    sketch.add_constraint(PerpendicularLinesConstraint::new(line1, line2));

    // This should fail to solve due to conflicting constraints
    let result = sketch.solve_and_extract();
    assert!(
        result.is_err(),
        "Should fail due to conflicting constraints"
    );

    // Should be an OverConstrained error
    if let Err(TextCadError::OverConstrained) = result {
        // Expected
    } else {
        panic!("Expected OverConstrained error, got: {:?}", result);
    }
}

#[test]
fn test_chained_parallel_constraints() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create three lines where A || B and B || C, so A || B || C
    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));
    let p4 = sketch.add_point(Some("p4".to_string()));
    let p5 = sketch.add_point(Some("p5".to_string()));
    let p6 = sketch.add_point(Some("p6".to_string()));

    let line_a = sketch.add_line(p1, p2, Some("line_a".to_string()));
    let line_b = sketch.add_line(p3, p4, Some("line_b".to_string()));
    let line_c = sketch.add_line(p5, p6, Some("line_c".to_string()));

    // Fix line A as horizontal reference
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

    // Fix starting points for other lines
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(1.0),
        Length::meters(2.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p5,
        Length::meters(-1.0),
        Length::meters(-1.0),
    ));

    // Set lengths
    sketch.add_constraint(LineLengthConstraint::new(line_b, Length::meters(3.0)));
    sketch.add_constraint(LineLengthConstraint::new(line_c, Length::meters(2.0)));

    // Chain parallel constraints: A || B, B || C
    sketch.add_constraint(ParallelLinesConstraint::new(line_a, line_b));
    sketch.add_constraint(ParallelLinesConstraint::new(line_b, line_c));

    // Solve
    let solution = sketch
        .solve_and_extract()
        .expect("Chained parallel constraints should solve");

    // Verify all lines are parallel to each other
    let coords = [
        solution.get_point_coordinates(p1).unwrap(),
        solution.get_point_coordinates(p2).unwrap(),
        solution.get_point_coordinates(p3).unwrap(),
        solution.get_point_coordinates(p4).unwrap(),
        solution.get_point_coordinates(p5).unwrap(),
        solution.get_point_coordinates(p6).unwrap(),
    ];

    // Direction vectors
    let dir_a = (coords[1].0 - coords[0].0, coords[1].1 - coords[0].1);
    let dir_b = (coords[3].0 - coords[2].0, coords[3].1 - coords[2].1);
    let dir_c = (coords[5].0 - coords[4].0, coords[5].1 - coords[4].1);

    // Normalize directions for comparison
    let norm_a = (dir_a.0.powi(2) + dir_a.1.powi(2)).sqrt();
    let norm_b = (dir_b.0.powi(2) + dir_b.1.powi(2)).sqrt();
    let norm_c = (dir_c.0.powi(2) + dir_c.1.powi(2)).sqrt();

    let unit_a = (dir_a.0 / norm_a, dir_a.1 / norm_a);
    let unit_b = (dir_b.0 / norm_b, dir_b.1 / norm_b);
    let unit_c = (dir_c.0 / norm_c, dir_c.1 / norm_c);

    // Check A || B (cross product ≈ 0)
    let cross_ab = unit_a.0 * unit_b.1 - unit_a.1 * unit_b.0;
    assert!(
        cross_ab.abs() < 1e-6,
        "Lines A and B should be parallel, cross: {}",
        cross_ab
    );

    // Check B || C (cross product ≈ 0)
    let cross_bc = unit_b.0 * unit_c.1 - unit_b.1 * unit_c.0;
    assert!(
        cross_bc.abs() < 1e-6,
        "Lines B and C should be parallel, cross: {}",
        cross_bc
    );

    // Check A || C (transitivity)
    let cross_ac = unit_a.0 * unit_c.1 - unit_a.1 * unit_c.0;
    assert!(
        cross_ac.abs() < 1e-6,
        "Lines A and C should be parallel (transitivity), cross: {}",
        cross_ac
    );
}

#[test]
fn test_entity_as_constraint_factory_integration() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Test the line.parallel_to() and line.perpendicular_to() methods
    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));
    let p4 = sketch.add_point(Some("p4".to_string()));

    let line1 = sketch.add_line(p1, p2, Some("line1".to_string()));
    let line2 = sketch.add_line(p3, p4, Some("line2".to_string()));

    // Get line entities to use factory methods
    let line1_entity = sketch.get_line(line1).unwrap().clone();
    let line2_entity = sketch.get_line(line2).unwrap().clone();

    // Fix line1 horizontally
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

    // Fix line2 start point and length
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(1.0),
        Length::meters(2.0),
    ));
    sketch.add_constraint(line2_entity.length_equals(Length::meters(2.5)));

    // Use entity-as-constraint-factory pattern
    sketch.add_constraint(line2_entity.perpendicular_to(&line1_entity));

    // Solve
    let solution = sketch
        .solve_and_extract()
        .expect("Entity factory constraints should solve");

    // Verify perpendicular relationship using factory-generated constraint
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
        "Lines should be perpendicular via factory method, dot: {}",
        dot_product
    );

    // Verify length constraint from factory method
    let line2_length = (dir2.0.powi(2) + dir2.1.powi(2)).sqrt();
    assert!(
        (line2_length - 2.5).abs() < 1e-6,
        "Line2 should have length 2.5m via factory, got: {}",
        line2_length
    );
}

#[test]
fn test_mixed_constraint_types_complex() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a more complex scenario: an L-shape with specific constraints
    let corner = sketch.add_point(Some("corner".to_string()));
    let horizontal_end = sketch.add_point(Some("h_end".to_string()));
    let vertical_end = sketch.add_point(Some("v_end".to_string()));

    // Create an auxiliary line for parallel constraint testing
    let aux_start = sketch.add_point(Some("aux_start".to_string()));
    let aux_end = sketch.add_point(Some("aux_end".to_string()));

    let horizontal = sketch.add_line(corner, horizontal_end, Some("horizontal".to_string()));
    let vertical = sketch.add_line(corner, vertical_end, Some("vertical".to_string()));
    let auxiliary = sketch.add_line(aux_start, aux_end, Some("auxiliary".to_string()));

    // Fix the corner at origin
    sketch.add_constraint(FixedPositionConstraint::new(
        corner,
        Length::meters(0.0),
        Length::meters(0.0),
    ));

    // Set specific lengths
    sketch.add_constraint(LineLengthConstraint::new(horizontal, Length::meters(8.0)));
    sketch.add_constraint(LineLengthConstraint::new(vertical, Length::meters(6.0)));
    sketch.add_constraint(LineLengthConstraint::new(auxiliary, Length::meters(4.0)));

    // Make the L-shape perpendicular
    sketch.add_constraint(PerpendicularLinesConstraint::new(horizontal, vertical));

    // Fix the horizontal line direction (along X-axis)
    sketch.add_constraint(FixedPositionConstraint::new(
        horizontal_end,
        Length::meters(8.0),
        Length::meters(0.0),
    ));

    // Position auxiliary line and make it parallel to vertical
    sketch.add_constraint(FixedPositionConstraint::new(
        aux_start,
        Length::meters(10.0),
        Length::meters(3.0),
    ));
    sketch.add_constraint(ParallelLinesConstraint::new(auxiliary, vertical));

    // Solve
    let solution = sketch
        .solve_and_extract()
        .expect("Complex mixed constraints should solve");

    // Verify all relationships
    let corner_pos = solution.get_point_coordinates(corner).unwrap();
    let h_end_pos = solution.get_point_coordinates(horizontal_end).unwrap();
    let v_end_pos = solution.get_point_coordinates(vertical_end).unwrap();
    let aux_start_pos = solution.get_point_coordinates(aux_start).unwrap();
    let aux_end_pos = solution.get_point_coordinates(aux_end).unwrap();

    // Check fixed positions
    assert!((corner_pos.0 - 0.0).abs() < 1e-6);
    assert!((corner_pos.1 - 0.0).abs() < 1e-6);
    assert!((h_end_pos.0 - 8.0).abs() < 1e-6);
    assert!((h_end_pos.1 - 0.0).abs() < 1e-6);

    // Check L-shape is perpendicular
    let h_dir = (h_end_pos.0 - corner_pos.0, h_end_pos.1 - corner_pos.1);
    let v_dir = (v_end_pos.0 - corner_pos.0, v_end_pos.1 - corner_pos.1);
    let dot_lshape = h_dir.0 * v_dir.0 + h_dir.1 * v_dir.1;
    assert!(
        dot_lshape.abs() < 1e-6,
        "L-shape should be perpendicular, dot: {}",
        dot_lshape
    );

    // Check auxiliary line is parallel to vertical
    let aux_dir = (
        aux_end_pos.0 - aux_start_pos.0,
        aux_end_pos.1 - aux_start_pos.1,
    );
    let cross_aux_vert = v_dir.0 * aux_dir.1 - v_dir.1 * aux_dir.0;
    assert!(
        cross_aux_vert.abs() < 1e-6,
        "Auxiliary should be parallel to vertical, cross: {}",
        cross_aux_vert
    );

    // Check all lengths
    let h_length = (h_dir.0.powi(2) + h_dir.1.powi(2)).sqrt();
    let v_length = (v_dir.0.powi(2) + v_dir.1.powi(2)).sqrt();
    let aux_length = (aux_dir.0.powi(2) + aux_dir.1.powi(2)).sqrt();

    assert!(
        (h_length - 8.0).abs() < 1e-6,
        "Horizontal should be 8.0m, got: {}",
        h_length
    );
    assert!(
        (v_length - 6.0).abs() < 1e-6,
        "Vertical should be 6.0m, got: {}",
        v_length
    );
    assert!(
        (aux_length - 4.0).abs() < 1e-6,
        "Auxiliary should be 4.0m, got: {}",
        aux_length
    );
}
