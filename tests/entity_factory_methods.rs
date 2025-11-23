//! Tests for entity-as-constraint-factory methods
//!
//! Tests the Line entity methods: parallel_to(), perpendicular_to(), and length_equals()
//! that provide a convenient API for creating constraint objects.

use generational_arena::Index;
use textcad::constraint::Constraint;
use textcad::constraints::{
    FixedPositionConstraint, LineLengthConstraint, ParallelLinesConstraint,
    PerpendicularLinesConstraint,
};
use textcad::entities::Line;
use textcad::entities::PointId;
use textcad::entity::LineId;
use textcad::sketch::Sketch;
use textcad::units::Length;
use z3::{Config, Context};

/// Test that Line::length_equals() creates the correct constraint
#[test]
fn test_line_length_equals_factory_method() {
    // Create a line entity
    let line_id = LineId(Index::from_raw_parts(0, 0));
    let start_id = PointId(Index::from_raw_parts(0, 0));
    let end_id = PointId(Index::from_raw_parts(1, 0));
    let line = Line::new(line_id, start_id, end_id, Some("test_line".to_string()));

    // Use factory method
    let target_length = Length::meters(5.5);
    let constraint = line.length_equals(target_length);

    // Verify the constraint was created correctly
    assert_eq!(constraint.line, line_id);
    assert_eq!(constraint.length, target_length);
    assert!(constraint.description().contains("5.500m"));
    assert!(constraint.description().contains(&format!("{:?}", line_id)));
}

/// Test that Line::parallel_to() creates the correct constraint
#[test]
fn test_line_parallel_to_factory_method() {
    let line1_id = LineId(Index::from_raw_parts(0, 0));
    let line2_id = LineId(Index::from_raw_parts(1, 0));
    let start_id = PointId(Index::from_raw_parts(0, 0));
    let end_id = PointId(Index::from_raw_parts(1, 0));

    let line1 = Line::new(line1_id, start_id, end_id, Some("line1".to_string()));
    let line2 = Line::new(line2_id, start_id, end_id, Some("line2".to_string()));

    // Use factory method
    let constraint = line1.parallel_to(&line2);

    // Verify the constraint was created correctly
    assert_eq!(constraint.line1, line1_id);
    assert_eq!(constraint.line2, line2_id);
    assert!(constraint.description().contains("parallel"));
    assert!(
        constraint
            .description()
            .contains(&format!("{:?}", line1_id))
    );
    assert!(
        constraint
            .description()
            .contains(&format!("{:?}", line2_id))
    );
}

/// Test that Line::perpendicular_to() creates the correct constraint
#[test]
fn test_line_perpendicular_to_factory_method() {
    let line1_id = LineId(Index::from_raw_parts(0, 0));
    let line2_id = LineId(Index::from_raw_parts(1, 0));
    let start_id = PointId(Index::from_raw_parts(0, 0));
    let end_id = PointId(Index::from_raw_parts(1, 0));

    let line1 = Line::new(line1_id, start_id, end_id, Some("line1".to_string()));
    let line2 = Line::new(line2_id, start_id, end_id, Some("line2".to_string()));

    // Use factory method
    let constraint = line1.perpendicular_to(&line2);

    // Verify the constraint was created correctly
    assert_eq!(constraint.line1, line1_id);
    assert_eq!(constraint.line2, line2_id);
    assert!(constraint.description().contains("perpendicular"));
    assert!(
        constraint
            .description()
            .contains(&format!("{:?}", line1_id))
    );
    assert!(
        constraint
            .description()
            .contains(&format!("{:?}", line2_id))
    );
}

/// Test factory methods with different unit types
#[test]
fn test_line_length_equals_with_different_units() {
    let line_id = LineId(Index::from_raw_parts(0, 0));
    let start_id = PointId(Index::from_raw_parts(0, 0));
    let end_id = PointId(Index::from_raw_parts(1, 0));
    let line = Line::new(line_id, start_id, end_id, None);

    // Test with different units
    let constraint_meters = line.length_equals(Length::meters(2.0));
    let constraint_mm = line.length_equals(Length::millimeters(2000.0));
    let constraint_cm = line.length_equals(Length::centimeters(200.0));

    // All should represent the same length in meters
    assert_eq!(constraint_meters.length.to_meters(), 2.0);
    assert_eq!(constraint_mm.length.to_meters(), 2.0);
    assert_eq!(constraint_cm.length.to_meters(), 2.0);

    // But should show different values in their descriptions
    assert!(constraint_meters.description().contains("2.000m"));
    assert!(constraint_mm.description().contains("2.000m")); // Converted to meters for display
    assert!(constraint_cm.description().contains("2.000m")); // Converted to meters for display
}

/// Test chaining factory method calls
#[test]
fn test_chaining_factory_methods() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create points and lines
    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));
    let p4 = sketch.add_point(Some("p4".to_string()));

    let line1_id = sketch.add_line(p1, p2, Some("line1".to_string()));
    let line2_id = sketch.add_line(p3, p4, Some("line2".to_string()));

    // Get line entities to use factory methods
    let line1 = sketch.get_line(line1_id).unwrap().clone();
    let line2 = sketch.get_line(line2_id).unwrap().clone();

    // Fix some positions
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
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(1.0),
        Length::meters(2.0),
    ));

    // Use multiple factory methods on the same line
    sketch.add_constraint(line2.length_equals(Length::meters(3.0)));
    sketch.add_constraint(line2.parallel_to(&line1));

    let solution = sketch
        .solve_and_extract()
        .expect("Chained factory methods should work");

    // Verify both constraints are satisfied
    let coords = [
        solution.get_point_coordinates(p1).unwrap(),
        solution.get_point_coordinates(p2).unwrap(),
        solution.get_point_coordinates(p3).unwrap(),
        solution.get_point_coordinates(p4).unwrap(),
    ];

    // Check parallel constraint
    let dir1 = (coords[1].0 - coords[0].0, coords[1].1 - coords[0].1);
    let dir2 = (coords[3].0 - coords[2].0, coords[3].1 - coords[2].1);
    let cross_product = dir1.0 * dir2.1 - dir1.1 * dir2.0;
    assert!(
        cross_product.abs() < 1e-6,
        "Lines should be parallel from factory method"
    );

    // Check length constraint
    let line2_length = (dir2.0.powi(2) + dir2.1.powi(2)).sqrt();
    assert!(
        (line2_length - 3.0).abs() < 1e-6,
        "Line2 should have length 3.0m from factory method"
    );
}

/// Test factory methods preserve line entity properties
#[test]
fn test_factory_methods_preserve_line_properties() {
    let line1_id = LineId(Index::from_raw_parts(0, 0));
    let line2_id = LineId(Index::from_raw_parts(1, 0));
    let start1_id = PointId(Index::from_raw_parts(0, 0));
    let end1_id = PointId(Index::from_raw_parts(1, 0));
    let start2_id = PointId(Index::from_raw_parts(2, 0));
    let end2_id = PointId(Index::from_raw_parts(3, 0));

    let line1 = Line::new(line1_id, start1_id, end1_id, Some("first_line".to_string()));
    let line2 = Line::new(
        line2_id,
        start2_id,
        end2_id,
        Some("second_line".to_string()),
    );

    // Create constraints using factory methods
    let length_constraint = line1.length_equals(Length::meters(7.0));
    let parallel_constraint = line1.parallel_to(&line2);
    let perpendicular_constraint = line1.perpendicular_to(&line2);

    // Verify constraints reference the correct lines
    assert_eq!(length_constraint.line, line1_id);

    assert_eq!(parallel_constraint.line1, line1_id);
    assert_eq!(parallel_constraint.line2, line2_id);

    assert_eq!(perpendicular_constraint.line1, line1_id);
    assert_eq!(perpendicular_constraint.line2, line2_id);

    // Verify line entities are unchanged
    assert_eq!(line1.id, line1_id);
    assert_eq!(line1.start, start1_id);
    assert_eq!(line1.end, end1_id);
    assert_eq!(line1.name, Some("first_line".to_string()));

    assert_eq!(line2.id, line2_id);
    assert_eq!(line2.start, start2_id);
    assert_eq!(line2.end, end2_id);
    assert_eq!(line2.name, Some("second_line".to_string()));
}

/// Test factory methods work with cloned line entities
#[test]
fn test_factory_methods_with_cloned_lines() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));
    let p4 = sketch.add_point(Some("p4".to_string()));

    let line1_id = sketch.add_line(p1, p2, Some("line1".to_string()));
    let line2_id = sketch.add_line(p3, p4, Some("line2".to_string()));

    // Clone line entities from sketch
    let line1_clone = sketch.get_line(line1_id).unwrap().clone();
    let line2_clone = sketch.get_line(line2_id).unwrap().clone();

    // Verify cloning worked
    assert_eq!(line1_clone.id, line1_id);
    assert_eq!(line2_clone.id, line2_id);

    // Use factory methods with cloned entities
    let constraint = line1_clone.perpendicular_to(&line2_clone);

    // Verify constraint references original IDs
    assert_eq!(constraint.line1, line1_id);
    assert_eq!(constraint.line2, line2_id);

    // Test that constraint works in sketch
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
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(2.0),
        Length::meters(1.0),
    ));
    sketch.add_constraint(line2_clone.length_equals(Length::meters(4.0)));
    sketch.add_constraint(constraint);

    let solution = sketch
        .solve_and_extract()
        .expect("Factory methods with cloned entities should work");

    // Verify perpendicular constraint
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
        "Cloned lines should be perpendicular via factory"
    );
}

/// Test factory method API consistency with direct constraint creation
#[test]
fn test_factory_vs_direct_constraint_equivalence() {
    let line1_id = LineId(Index::from_raw_parts(0, 0));
    let line2_id = LineId(Index::from_raw_parts(1, 0));
    let start_id = PointId(Index::from_raw_parts(0, 0));
    let end_id = PointId(Index::from_raw_parts(1, 0));

    let line1 = Line::new(line1_id, start_id, end_id, Some("line1".to_string()));
    let line2 = Line::new(line2_id, start_id, end_id, Some("line2".to_string()));

    // Create constraints using factory methods
    let length_via_factory = line1.length_equals(Length::meters(5.0));
    let parallel_via_factory = line1.parallel_to(&line2);
    let perpendicular_via_factory = line1.perpendicular_to(&line2);

    // Create equivalent constraints directly
    let length_direct = LineLengthConstraint::new(line1_id, Length::meters(5.0));
    let parallel_direct = ParallelLinesConstraint::new(line1_id, line2_id);
    let perpendicular_direct = PerpendicularLinesConstraint::new(line1_id, line2_id);

    // Verify they're equivalent
    assert_eq!(length_via_factory.line, length_direct.line);
    assert_eq!(length_via_factory.length, length_direct.length);
    assert_eq!(
        length_via_factory.description(),
        length_direct.description()
    );

    assert_eq!(parallel_via_factory.line1, parallel_direct.line1);
    assert_eq!(parallel_via_factory.line2, parallel_direct.line2);
    assert_eq!(
        parallel_via_factory.description(),
        parallel_direct.description()
    );

    assert_eq!(perpendicular_via_factory.line1, perpendicular_direct.line1);
    assert_eq!(perpendicular_via_factory.line2, perpendicular_direct.line2);
    assert_eq!(
        perpendicular_via_factory.description(),
        perpendicular_direct.description()
    );
}

/// Test factory methods in complex constraint scenarios
#[test]
fn test_factory_methods_in_complex_scenarios() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a complex scenario: construction of a house-like shape
    // using only factory methods for line constraints

    // Base of house
    let base_left = sketch.add_point(Some("base_left".to_string()));
    let base_right = sketch.add_point(Some("base_right".to_string()));

    // Walls
    let wall_left_top = sketch.add_point(Some("wall_left_top".to_string()));
    let wall_right_top = sketch.add_point(Some("wall_right_top".to_string()));

    // Roof peak
    let roof_peak = sketch.add_point(Some("roof_peak".to_string()));

    // Lines forming the house
    let base = sketch.add_line(base_left, base_right, Some("base".to_string()));
    let left_wall = sketch.add_line(base_left, wall_left_top, Some("left_wall".to_string()));
    let right_wall = sketch.add_line(base_right, wall_right_top, Some("right_wall".to_string()));
    let roof_top = sketch.add_line(wall_left_top, wall_right_top, Some("roof_top".to_string()));
    let roof_left = sketch.add_line(wall_left_top, roof_peak, Some("roof_left".to_string()));
    let roof_right = sketch.add_line(wall_right_top, roof_peak, Some("roof_right".to_string()));

    // Get line entities for factory methods
    let base_line = sketch.get_line(base).unwrap().clone();
    let left_wall_line = sketch.get_line(left_wall).unwrap().clone();
    let right_wall_line = sketch.get_line(right_wall).unwrap().clone();
    let _roof_top_line = sketch.get_line(roof_top).unwrap().clone();
    let _roof_left_line = sketch.get_line(roof_left).unwrap().clone();
    let _roof_right_line = sketch.get_line(roof_right).unwrap().clone();

    // Fix base position
    sketch.add_constraint(FixedPositionConstraint::new(
        base_left,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        base_right,
        Length::meters(6.0),
        Length::meters(0.0),
    ));

    // Use factory methods to define the house geometry (simplified to avoid over-constraining)
    sketch.add_constraint(left_wall_line.length_equals(Length::meters(4.0)));
    sketch.add_constraint(right_wall_line.length_equals(Length::meters(4.0)));

    // Parallel and perpendicular constraints using factory methods
    sketch.add_constraint(left_wall_line.parallel_to(&right_wall_line)); // Walls parallel
    sketch.add_constraint(left_wall_line.perpendicular_to(&base_line)); // Walls perpendicular to base

    let solution = sketch
        .solve_and_extract()
        .expect("Complex house construction with factory methods should work");

    // Verify house geometry - simplified verification
    let coords = [
        (
            "base_left",
            solution.get_point_coordinates(base_left).unwrap(),
        ),
        (
            "base_right",
            solution.get_point_coordinates(base_right).unwrap(),
        ),
        (
            "wall_left_top",
            solution.get_point_coordinates(wall_left_top).unwrap(),
        ),
        (
            "wall_right_top",
            solution.get_point_coordinates(wall_right_top).unwrap(),
        ),
    ];

    // Base should be horizontal and 6m wide
    let base_width = coords[1].1.0 - coords[0].1.0;
    assert!((base_width - 6.0).abs() < 1e-6, "Base should be 6m wide");
    assert!(
        (coords[0].1.1 - coords[1].1.1).abs() < 1e-6,
        "Base should be horizontal"
    );

    // Walls should be parallel and perpendicular to base
    let left_wall_vec = (coords[2].1.0 - coords[0].1.0, coords[2].1.1 - coords[0].1.1);
    let right_wall_vec = (coords[3].1.0 - coords[1].1.0, coords[3].1.1 - coords[1].1.1);
    let base_vec = (coords[1].1.0 - coords[0].1.0, coords[1].1.1 - coords[0].1.1);

    // Check walls are parallel (cross product = 0)
    let cross_walls = left_wall_vec.0 * right_wall_vec.1 - left_wall_vec.1 * right_wall_vec.0;
    assert!(cross_walls.abs() < 1e-6, "Walls should be parallel");

    // Check walls are perpendicular to base (dot product = 0)
    let dot_left_base = left_wall_vec.0 * base_vec.0 + left_wall_vec.1 * base_vec.1;
    assert!(
        dot_left_base.abs() < 1e-6,
        "Left wall should be perpendicular to base"
    );

    // Check wall lengths
    let left_wall_length = (left_wall_vec.0.powi(2) + left_wall_vec.1.powi(2)).sqrt();
    let right_wall_length = (right_wall_vec.0.powi(2) + right_wall_vec.1.powi(2)).sqrt();
    assert!(
        (left_wall_length - 4.0).abs() < 1e-6,
        "Left wall should be 4m high"
    );
    assert!(
        (right_wall_length - 4.0).abs() < 1e-6,
        "Right wall should be 4m high"
    );
}
