//! Phase 10 Demo: Parametric Constraints with PointOnLineConstraint
//!
//! This example demonstrates the new parametric constraint functionality,
//! specifically the PointOnLineConstraint which places points on line segments
//! using internal parameter variables with t ∈ [0,1].
//!
//! Run with: cargo run --example phase10_demo

use textcad::constraints::{FixedPositionConstraint, LineLengthConstraint, PointOnLineConstraint};
use textcad::sketch::Sketch;
use textcad::units::Length;
use z3::{Config, Context};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🟦 Phase 10 Demo: Parametric Constraints");
    println!("==========================================");

    // Example 1: Basic Point on Line Segment
    println!("\n📍 Example 1: Basic Point on Line Segment");
    println!("------------------------------------------");

    basic_point_on_line_demo()?;

    // Example 2: Multiple Points on Same Line
    println!("\n📍 Example 2: Multiple Points on Same Line");
    println!("-------------------------------------------");

    multiple_points_on_line_demo()?;

    // Example 3: Geometric Construction - Triangle with Point on Base
    println!("\n📍 Example 3: Triangle Construction");
    println!("------------------------------------");

    triangle_construction_demo()?;

    // Example 4: Parametric Line Subdivision
    println!("\n📍 Example 4: Line Subdivision");
    println!("-------------------------------");

    line_subdivision_demo()?;

    println!("\n✅ Phase 10 Demo Complete!");
    println!("All parametric constraints solved successfully.");

    Ok(())
}

/// Demonstrates basic point-on-line constraint functionality
fn basic_point_on_line_demo() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a horizontal line from (0,0) to (5,0)
    let p1 = sketch.add_point(Some("start".to_string()));
    let p2 = sketch.add_point(Some("end".to_string()));
    let line = sketch.add_line(p1, p2, Some("base_line".to_string()));

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
        Length::meters(5.0),
        Length::meters(0.0),
    ));

    // Use entity-as-constraint-factory method to create the constraint
    let line_entity = sketch.get_line(line).unwrap();
    sketch.add_constraint(line_entity.point_on_line(p3));

    // Solve and extract solution
    let solution = sketch.solve_and_extract()?;

    // Display results
    let (x1, y1) = solution.get_point_coordinates(p1)?;
    let (x2, y2) = solution.get_point_coordinates(p2)?;
    let (x3, y3) = solution.get_point_coordinates(p3)?;

    println!("Line endpoints:");
    println!("  Start: ({:.3}, {:.3})", x1, y1);
    println!("  End:   ({:.3}, {:.3})", x2, y2);
    println!("Point on line:");
    println!("  Position: ({:.3}, {:.3})", x3, y3);

    // Calculate and display parameter t
    let dx = x2 - x1;
    let t = if dx.abs() > 1e-6 { (x3 - x1) / dx } else { 0.0 };
    println!("  Parameter t: {:.3} (should be in [0,1])", t);

    // Verify the constraint is satisfied
    assert!(
        t >= -1e-6 && t <= 1.0 + 1e-6,
        "Parameter t should be in [0,1]"
    );
    assert!((y3 - 0.0).abs() < 1e-6, "Point should be on y=0 line");

    Ok(())
}

/// Demonstrates multiple points constrained to the same line
fn multiple_points_on_line_demo() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a diagonal line from (0,0) to (4,3)
    let start = sketch.add_point(Some("line_start".to_string()));
    let end = sketch.add_point(Some("line_end".to_string()));
    let line = sketch.add_line(start, end, Some("diagonal_line".to_string()));

    // Create multiple points on the line
    let point_a = sketch.add_point(Some("point_a".to_string()));
    let point_b = sketch.add_point(Some("point_b".to_string()));
    let point_c = sketch.add_point(Some("point_c".to_string()));

    // Fix the line endpoints
    sketch.add_constraint(FixedPositionConstraint::new(
        start,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        end,
        Length::meters(4.0),
        Length::meters(3.0),
    ));

    // Constrain all points to lie on the line
    sketch.add_constraint(PointOnLineConstraint::new(line, point_a));
    sketch.add_constraint(PointOnLineConstraint::new(line, point_b));
    sketch.add_constraint(PointOnLineConstraint::new(line, point_c));

    // Solve and extract solution
    let solution = sketch.solve_and_extract()?;

    // Display results
    println!("Diagonal line from (0,0) to (4,3):");

    let line_endpoints = [start, end];
    let points_on_line = [point_a, point_b, point_c];
    let point_names = ["A", "B", "C"];

    for (i, &point_id) in line_endpoints.iter().enumerate() {
        let (x, y) = solution.get_point_coordinates(point_id)?;
        let name = if i == 0 { "Start" } else { "End" };
        println!("  {}: ({:.3}, {:.3})", name, x, y);
    }

    println!("Points on line:");
    for (i, &point_id) in points_on_line.iter().enumerate() {
        let (x, y) = solution.get_point_coordinates(point_id)?;

        // Calculate parameter t for this point
        let (start_x, start_y) = solution.get_point_coordinates(start)?;
        let (end_x, end_y) = solution.get_point_coordinates(end)?;
        let dx = end_x - start_x;
        let dy = end_y - start_y;
        let length_sq = dx * dx + dy * dy;
        let t = if length_sq > 1e-6 {
            ((x - start_x) * dx + (y - start_y) * dy) / length_sq
        } else {
            0.0
        };

        println!("  {}: ({:.3}, {:.3}) [t = {:.3}]", point_names[i], x, y, t);

        // Verify constraint satisfaction
        assert!(
            t >= -1e-6 && t <= 1.0 + 1e-6,
            "Parameter t should be in [0,1]"
        );
    }

    Ok(())
}

/// Demonstrates a geometric construction using point-on-line constraints
fn triangle_construction_demo() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a triangle with vertices A, B, C
    let a = sketch.add_point(Some("A".to_string()));
    let b = sketch.add_point(Some("B".to_string()));
    let c = sketch.add_point(Some("C".to_string()));

    // Create the triangle sides
    let ab = sketch.add_line(a, b, Some("side_AB".to_string()));
    let bc = sketch.add_line(b, c, Some("side_BC".to_string()));
    let ca = sketch.add_line(c, a, Some("side_CA".to_string()));

    // Create a point on the base AB
    let d = sketch.add_point(Some("D_on_AB".to_string()));

    // Fix the triangle vertices
    sketch.add_constraint(FixedPositionConstraint::new(
        a,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        b,
        Length::meters(6.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        c,
        Length::meters(3.0),
        Length::meters(4.0),
    ));

    // Constrain point D to lie on side AB
    sketch.add_constraint(PointOnLineConstraint::new(ab, d));

    // Set lengths for the triangle sides
    sketch.add_constraint(LineLengthConstraint::new(ab, Length::meters(6.0)));
    sketch.add_constraint(LineLengthConstraint::new(bc, Length::meters(5.0)));
    sketch.add_constraint(LineLengthConstraint::new(ca, Length::meters(5.0)));

    // Solve and extract solution
    let solution = sketch.solve_and_extract()?;

    // Display triangle information
    println!("Triangle ABC with point D on side AB:");

    let vertices = [(a, "A"), (b, "B"), (c, "C")];
    for (point_id, name) in vertices {
        let (x, y) = solution.get_point_coordinates(point_id)?;
        println!("  {}: ({:.3}, {:.3})", name, x, y);
    }

    let (dx, dy) = solution.get_point_coordinates(d)?;
    println!("  D (on AB): ({:.3}, {:.3})", dx, dy);

    // Calculate position of D along AB
    let (ax, ay) = solution.get_point_coordinates(a)?;
    let (bx, by) = solution.get_point_coordinates(b)?;
    let ab_dx = bx - ax;
    let ab_dy = by - ay;
    let ab_length_sq = ab_dx * ab_dx + ab_dy * ab_dy;
    let t = if ab_length_sq > 1e-6 {
        ((dx - ax) * ab_dx + (dy - ay) * ab_dy) / ab_length_sq
    } else {
        0.0
    };

    println!("  D divides AB in ratio t = {:.3} : {:.3}", t, 1.0 - t);

    // Verify triangle side lengths
    let side_lengths = [("AB", ab, 6.0), ("BC", bc, 5.0), ("CA", ca, 5.0)];

    println!("Triangle side lengths:");
    for (name, line_id, expected) in side_lengths {
        let line = sketch.get_line(line_id).unwrap();
        let (start_x, start_y) = solution.get_point_coordinates(line.start)?;
        let (end_x, end_y) = solution.get_point_coordinates(line.end)?;
        let length = ((end_x - start_x).powi(2) + (end_y - start_y).powi(2)).sqrt();
        println!("  {}: {:.3}m (expected: {:.1}m)", name, length, expected);
        assert!(
            (length - expected).abs() < 1e-3,
            "Side length should match expected value"
        );
    }

    Ok(())
}

/// Demonstrates subdividing a line into equal parts using multiple point-on-line constraints
fn line_subdivision_demo() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a line segment to subdivide
    let start = sketch.add_point(Some("segment_start".to_string()));
    let end = sketch.add_point(Some("segment_end".to_string()));
    let segment = sketch.add_line(start, end, Some("segment".to_string()));

    // Create subdivision points
    let div1 = sketch.add_point(Some("division_1".to_string()));
    let div2 = sketch.add_point(Some("division_2".to_string()));
    let div3 = sketch.add_point(Some("division_3".to_string()));

    // Fix the segment endpoints
    sketch.add_constraint(FixedPositionConstraint::new(
        start,
        Length::meters(1.0),
        Length::meters(2.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        end,
        Length::meters(7.0),
        Length::meters(8.0),
    ));

    // Constrain all division points to lie on the segment
    sketch.add_constraint(PointOnLineConstraint::new(segment, div1));
    sketch.add_constraint(PointOnLineConstraint::new(segment, div2));
    sketch.add_constraint(PointOnLineConstraint::new(segment, div3));

    // To create equal subdivisions, we would need additional constraints
    // For this demo, we'll just show that the points lie on the line
    // In a more advanced implementation, we could add distance constraints

    // Solve and extract solution
    let solution = sketch.solve_and_extract()?;

    // Display subdivision results
    println!("Line segment subdivision:");

    let (start_x, start_y) = solution.get_point_coordinates(start)?;
    let (end_x, end_y) = solution.get_point_coordinates(end)?;

    println!(
        "Segment from ({:.3}, {:.3}) to ({:.3}, {:.3})",
        start_x, start_y, end_x, end_y
    );

    let segment_dx = end_x - start_x;
    let segment_dy = end_y - start_y;
    let segment_length = (segment_dx.powi(2) + segment_dy.powi(2)).sqrt();
    println!("Segment length: {:.3}", segment_length);

    let division_points = [div1, div2, div3];
    let division_names = ["Division 1", "Division 2", "Division 3"];

    println!("Division points on segment:");
    for (i, &point_id) in division_points.iter().enumerate() {
        let (x, y) = solution.get_point_coordinates(point_id)?;

        // Calculate parameter t and distance from start
        let point_dx = x - start_x;
        let point_dy = y - start_y;
        let distance_from_start = (point_dx.powi(2) + point_dy.powi(2)).sqrt();

        let length_sq = segment_dx.powi(2) + segment_dy.powi(2);
        let t = if length_sq > 1e-6 {
            (point_dx * segment_dx + point_dy * segment_dy) / length_sq
        } else {
            0.0
        };

        println!(
            "  {}: ({:.3}, {:.3}) [t = {:.3}, distance = {:.3}]",
            division_names[i], x, y, t, distance_from_start
        );

        // Verify constraint satisfaction
        assert!(
            t >= -1e-6 && t <= 1.0 + 1e-6,
            "Parameter t should be in [0,1]"
        );
        assert!(
            distance_from_start <= segment_length + 1e-6,
            "Distance should not exceed segment length"
        );
    }

    Ok(())
}
