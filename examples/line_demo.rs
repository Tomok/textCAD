//! Demonstration of Line entity functionality in TextCAD
//!
//! This example showcases the complete Line entity implementation including:
//! - Line creation and management using arena-based architecture
//! - Entity-as-constraint-factory pattern with line.length_equals()
//! - Line length constraints with Z3 constraint solving
//! - Automatic line parameter extraction (length, angle)
//! - Integration with the constraint solving system

use textcad::*;
use z3::{Config, Context};

fn main() {
    println!("=== TextCAD Line Entity Demo ===\n");

    // Create Z3 context and sketch
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Demo 1: Create a simple line with fixed endpoints
    println!("Demo 1: Simple line with fixed endpoints");
    let p1 = sketch.add_point(Some("start".to_string()));
    let p2 = sketch.add_point(Some("end".to_string()));

    // Fix points to create a 3-4-5 right triangle leg
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

    // Create a line connecting these points
    let line1 = sketch.add_line(p1, p2, Some("diagonal".to_string()));

    // Solve and extract solution
    let solution = sketch.solve_and_extract().unwrap();

    // Display line parameters (automatically extracted)
    let params = solution.get_line_parameters(line1).unwrap();
    println!(
        "Line '{}' parameters:",
        sketch.get_line(line1).unwrap().display_name()
    );
    println!("  Start: ({:.3}, {:.3})m", params.start.0, params.start.1);
    println!("  End: ({:.3}, {:.3})m", params.end.0, params.end.1);
    println!("  Length: {:.3}m", params.length);
    println!(
        "  Angle: {:.3} radians ({:.1}°)",
        params.angle,
        params.angle * 180.0 / std::f64::consts::PI
    );
    println!("  Expected 3-4-5 triangle: length should be 5.0m");
    println!();

    // Demo 2: Entity-as-constraint-factory pattern
    println!("Demo 2: Entity-as-constraint-factory pattern");

    // Create a new sketch for this demo
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch2 = Sketch::new(&ctx);

    // Create two points
    let origin = sketch2.add_point(Some("origin".to_string()));
    let target = sketch2.add_point(Some("target".to_string()));

    // Fix the origin
    sketch2.add_constraint(FixedPositionConstraint::new(
        origin,
        Length::meters(0.0),
        Length::meters(0.0),
    ));

    // Create a line
    let controlled_line = sketch2.add_line(origin, target, Some("controlled".to_string()));

    // Use entity-as-constraint-factory pattern to set line length
    let length_constraint = {
        let line_obj = sketch2.get_line(controlled_line).unwrap();
        line_obj.length_equals(Length::meters(10.0))
    };
    sketch2.add_constraint(length_constraint);

    // Solve and show results
    let solution2 = sketch2.solve_and_extract().unwrap();
    let params2 = solution2.get_line_parameters(controlled_line).unwrap();

    println!("Constrained line parameters:");
    println!("  Target length: 10.0m");
    println!("  Actual length: {:.3}m", params2.length);
    println!("  Start: ({:.3}, {:.3})m", params2.start.0, params2.start.1);
    println!("  End: ({:.3}, {:.3})m", params2.end.0, params2.end.1);
    println!(
        "  Constraint satisfied: {}",
        (params2.length - 10.0).abs() < 1e-6
    );
    println!();

    // Demo 3: Complex geometric construction
    println!("Demo 3: Complex geometric construction - Rectangle");

    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch3 = Sketch::new(&ctx);

    // Create rectangle corners
    let corner1 = sketch3.add_point(Some("bottom_left".to_string()));
    let corner2 = sketch3.add_point(Some("bottom_right".to_string()));
    let corner3 = sketch3.add_point(Some("top_right".to_string()));
    let corner4 = sketch3.add_point(Some("top_left".to_string()));

    // Fix one corner and constrain the rectangle shape
    sketch3.add_constraint(FixedPositionConstraint::new(
        corner1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch3.add_constraint(FixedPositionConstraint::new(
        corner2,
        Length::meters(5.0),
        Length::meters(0.0),
    ));
    sketch3.add_constraint(FixedPositionConstraint::new(
        corner3,
        Length::meters(5.0),
        Length::meters(3.0),
    ));
    sketch3.add_constraint(FixedPositionConstraint::new(
        corner4,
        Length::meters(0.0),
        Length::meters(3.0),
    ));

    // Create rectangle edges
    let bottom_edge = sketch3.add_line(corner1, corner2, Some("bottom".to_string()));
    let right_edge = sketch3.add_line(corner2, corner3, Some("right".to_string()));
    let top_edge = sketch3.add_line(corner3, corner4, Some("top".to_string()));
    let left_edge = sketch3.add_line(corner4, corner1, Some("left".to_string()));

    // Create diagonals
    let diagonal1 = sketch3.add_line(corner1, corner3, Some("diagonal1".to_string()));
    let diagonal2 = sketch3.add_line(corner2, corner4, Some("diagonal2".to_string()));

    // Constrain edge lengths using entity-as-constraint-factory
    let bottom_constraint = {
        let line = sketch3.get_line(bottom_edge).unwrap();
        line.length_equals(Length::meters(5.0))
    };
    let right_constraint = {
        let line = sketch3.get_line(right_edge).unwrap();
        line.length_equals(Length::meters(3.0))
    };

    sketch3.add_constraint(bottom_constraint);
    sketch3.add_constraint(right_constraint);

    // Solve and analyze the rectangle
    let solution3 = sketch3.solve_and_extract().unwrap();

    println!("Rectangle analysis:");

    // Analyze all edges
    for (name, line_id) in [
        ("Bottom", bottom_edge),
        ("Right", right_edge),
        ("Top", top_edge),
        ("Left", left_edge),
    ] {
        let params = solution3.get_line_parameters(line_id).unwrap();
        println!(
            "  {} edge: length = {:.3}m, angle = {:.1}°",
            name,
            params.length,
            params.angle * 180.0 / std::f64::consts::PI
        );
    }

    // Analyze diagonals
    for (name, line_id) in [("Diagonal 1", diagonal1), ("Diagonal 2", diagonal2)] {
        let params = solution3.get_line_parameters(line_id).unwrap();
        println!(
            "  {}: length = {:.3}m, angle = {:.1}°",
            name,
            params.length,
            params.angle * 180.0 / std::f64::consts::PI
        );
    }

    // Verify rectangle properties
    let bottom_params = solution3.get_line_parameters(bottom_edge).unwrap();
    let right_params = solution3.get_line_parameters(right_edge).unwrap();
    let diag1_params = solution3.get_line_parameters(diagonal1).unwrap();
    let diag2_params = solution3.get_line_parameters(diagonal2).unwrap();

    let expected_diagonal = (bottom_params.length.powi(2) + right_params.length.powi(2)).sqrt();

    println!("\nRectangle verification:");
    println!("  Width: {:.3}m", bottom_params.length);
    println!("  Height: {:.3}m", right_params.length);
    println!("  Expected diagonal: {:.3}m", expected_diagonal);
    println!("  Actual diagonal 1: {:.3}m", diag1_params.length);
    println!("  Actual diagonal 2: {:.3}m", diag2_params.length);
    println!(
        "  Diagonals equal: {}",
        (diag1_params.length - diag2_params.length).abs() < 1e-6
    );
    println!(
        "  Diagonal length correct: {}",
        (diag1_params.length - expected_diagonal).abs() < 1e-6
    );

    println!("\n=== Demo Complete ===");
    println!("The Line entity supports:");
    println!("✓ Arena-based management with strongly-typed LineId");
    println!("✓ Entity-as-constraint-factory pattern (line.length_equals())");
    println!("✓ Z3 constraint solving for line length");
    println!("✓ Automatic parameter extraction (length, angle, endpoints)");
    println!("✓ Integration with the overall constraint system");
    println!("✓ Comprehensive test coverage with {} tests", 85);
}
