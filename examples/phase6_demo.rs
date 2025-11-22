//! Phase 6 Demo: Enhanced Solution Extraction
//!
//! This example demonstrates the enhanced solution extraction system
//! implemented in Phase 6, including parameter extraction, line/circle
//! parameter calculation, and robust rational conversion.

use generational_arena::Index;
use textcad::{
    constraints::FixedPositionConstraint,
    entity::{CircleId, LineId},
    sketch::Sketch,
    units::Length,
};
use z3::{Config, Context, ast::Ast, ast::Real};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("TextCAD Phase 6 Demo: Enhanced Solution Extraction\n");

    // Initialize Z3 context
    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    // Demo 1: Enhanced Point Coordinate Extraction
    println!("=== Demo 1: Enhanced Point Coordinate Extraction ===");
    let mut sketch1 = Sketch::new(&ctx);
    let p1 = sketch1.add_point(Some("Origin".to_string()));
    let p2 = sketch1.add_point(Some("Destination".to_string()));

    // Fix points at specific positions
    sketch1.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch1.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::centimeters(300.0),  // 3 meters
        Length::millimeters(4000.0), // 4 meters
    ));

    let solution1 = sketch1.solve_and_extract()?;
    let (x1, y1) = solution1.get_point_coordinates(p1)?;
    let (x2, y2) = solution1.get_point_coordinates(p2)?;

    println!("Origin coordinates: ({:.3}, {:.3}) meters", x1, y1);
    println!("Destination coordinates: ({:.3}, {:.3}) meters", x2, y2);

    // Show all extracted coordinates at once
    let all_coords = solution1.all_point_coordinates();
    println!("Total points extracted: {}", all_coords.len());
    println!();

    // Demo 2: Parameter Variable Extraction
    println!("=== Demo 2: Parameter Variable Extraction ===");
    let mut sketch2 = Sketch::new(&ctx);

    // Create some parameter variables manually (simulating parametric constraints)
    let t_param = Real::new_const(sketch2.context(), "t_parameter");
    let scale_param = Real::new_const(sketch2.context(), "scale_factor");

    // Set parameter values: t = 0.75, scale = 2.5
    let three_fourths = Real::from_real(sketch2.context(), 3, 4);
    let two_and_half = Real::from_real(sketch2.context(), 5, 2);

    sketch2.solver_mut().assert(&t_param._eq(&three_fourths));
    sketch2.solver_mut().assert(&scale_param._eq(&two_and_half));

    // Add a point to make the solution meaningful
    let p3 = sketch2.add_point(Some("param_point".to_string()));
    sketch2.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(1.0),
        Length::meters(1.0),
    ));

    let mut solution2 = sketch2.solve_and_extract()?;

    // Extract parameter values
    let t_value = solution2.extract_parameter("t_parameter", &t_param)?;
    let scale_value = solution2.extract_parameter("scale_factor", &scale_param)?;

    println!("Parameter t: {:.6}", t_value);
    println!("Scale factor: {:.6}", scale_value);
    println!("Expected: t = 0.75, scale = 2.5");
    println!(
        "Match: {}",
        (t_value - 0.75).abs() < 1e-6 && (scale_value - 2.5).abs() < 1e-6
    );

    let all_params = solution2.all_parameters();
    println!("Total parameters extracted: {}", all_params.len());
    println!();

    // Demo 3: Line Parameter Calculation
    println!("=== Demo 3: Line Parameter Calculation ===");
    let mut sketch3 = Sketch::new(&ctx);

    // Create two points for a line
    let line_start = sketch3.add_point(Some("LineStart".to_string()));
    let line_end = sketch3.add_point(Some("LineEnd".to_string()));

    // Position them to form a 3-4-5 right triangle
    sketch3.add_constraint(FixedPositionConstraint::new(
        line_start,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch3.add_constraint(FixedPositionConstraint::new(
        line_end,
        Length::meters(3.0),
        Length::meters(4.0),
    ));

    let mut solution3 = sketch3.solve_and_extract()?;

    // Extract line parameters (simulate having a line entity)
    let line_id = LineId(Index::from_raw_parts(0, 0));
    let start_coords = solution3.get_point_coordinates(line_start)?;
    let end_coords = solution3.get_point_coordinates(line_end)?;

    let line_params = solution3.extract_line_parameters(line_id, start_coords, end_coords)?;

    println!(
        "Line start: ({:.3}, {:.3})",
        line_params.start.0, line_params.start.1
    );
    println!(
        "Line end: ({:.3}, {:.3})",
        line_params.end.0, line_params.end.1
    );
    println!("Line length: {:.3} meters", line_params.length);
    println!(
        "Line angle: {:.3} radians ({:.1} degrees)",
        line_params.angle,
        line_params.angle * 180.0 / std::f64::consts::PI
    );
    println!("Expected length: 5.0 meters (3-4-5 triangle)");
    println!("Length match: {}", (line_params.length - 5.0).abs() < 1e-6);
    println!();

    // Demo 4: Circle Parameter Calculation
    println!("=== Demo 4: Circle Parameter Calculation ===");
    let mut sketch4 = Sketch::new(&ctx);

    // Create a circle with center and radius as Z3 variables
    let circle_center = sketch4.add_point(Some("CircleCenter".to_string()));
    let radius_var = Real::new_const(sketch4.context(), "circle_radius");

    // Set center at (2, 3) and radius to 1.5 meters
    sketch4.add_constraint(FixedPositionConstraint::new(
        circle_center,
        Length::meters(2.0),
        Length::meters(3.0),
    ));

    let radius_val = Real::from_real(sketch4.context(), 3, 2); // 1.5 as 3/2
    sketch4.solver_mut().assert(&radius_var._eq(&radius_val));

    let mut solution4 = sketch4.solve_and_extract()?;

    // Extract circle parameters
    let circle_id = CircleId(Index::from_raw_parts(0, 0));
    let center_coords = solution4.get_point_coordinates(circle_center)?;

    let circle_params =
        solution4.extract_circle_parameters(circle_id, center_coords, &radius_var)?;

    println!(
        "Circle center: ({:.3}, {:.3})",
        circle_params.center.0, circle_params.center.1
    );
    println!("Circle radius: {:.3} meters", circle_params.radius);
    println!(
        "Circle circumference: {:.3} meters",
        circle_params.circumference
    );
    println!("Circle area: {:.6} square meters", circle_params.area);

    let expected_circumference = 2.0 * std::f64::consts::PI * 1.5;
    let expected_area = std::f64::consts::PI * 1.5 * 1.5;

    println!(
        "Expected circumference: {:.3} meters",
        expected_circumference
    );
    println!("Expected area: {:.6} square meters", expected_area);
    println!(
        "Circumference match: {}",
        (circle_params.circumference - expected_circumference).abs() < 1e-6
    );
    println!(
        "Area match: {}",
        (circle_params.area - expected_area).abs() < 1e-6
    );
    println!();

    // Demo 5: Enhanced Error Handling and Precision
    println!("=== Demo 5: Enhanced Error Handling ===");
    let mut sketch5 = Sketch::new(&ctx);

    // Test high precision rational conversion
    let high_precision_var = Real::new_const(sketch5.context(), "high_precision");
    let precise_val = Real::from_real(sketch5.context(), 355, 113); // Approximation of π

    sketch5
        .solver_mut()
        .assert(&high_precision_var._eq(&precise_val));

    // Add a dummy point
    let p5 = sketch5.add_point(Some("dummy".to_string()));
    sketch5.add_constraint(FixedPositionConstraint::new(
        p5,
        Length::meters(0.0),
        Length::meters(0.0),
    ));

    let mut solution5 = sketch5.solve_and_extract()?;
    let precise_value = solution5.extract_parameter("high_precision", &high_precision_var)?;

    println!("High precision value: {:.10}", precise_value);
    println!("Expected (355/113): {:.10}", 355.0 / 113.0);
    println!("Actual π: {:.10}", std::f64::consts::PI);
    println!(
        "Error from π: {:.2e}",
        (precise_value - std::f64::consts::PI).abs()
    );
    println!();

    // Demo 6: Solution Caching and Performance
    println!("=== Demo 6: Solution Caching Performance ===");
    let mut sketch6 = Sketch::new(&ctx);

    // Create multiple points
    let points: Vec<_> = (0..5)
        .map(|i| sketch6.add_point(Some(format!("point_{}", i))))
        .collect();

    // Fix them at different positions
    for (i, &point_id) in points.iter().enumerate() {
        sketch6.add_constraint(FixedPositionConstraint::new(
            point_id,
            Length::meters(i as f64),
            Length::meters((i * i) as f64),
        ));
    }

    let solution6 = sketch6.solve_and_extract()?;

    // Access coordinates multiple times to test caching
    println!("Point coordinates (demonstrating caching):");
    for (i, &point_id) in points.iter().enumerate() {
        let (x, y) = solution6.get_point_coordinates(point_id)?;
        println!("  Point {}: ({:.1}, {:.1})", i, x, y);
    }

    let all_coords = solution6.all_point_coordinates();
    println!("Total cached coordinates: {}", all_coords.len());
    println!();

    println!("=== Phase 6 Demo Complete ===");
    println!("✅ Enhanced coordinate extraction with caching");
    println!("✅ Parameter variable extraction and management");
    println!("✅ Line parameter calculation (length, angle)");
    println!("✅ Circle parameter calculation (radius, area, circumference)");
    println!("✅ Robust rational-to-float conversion with error handling");
    println!("✅ High-precision value extraction");
    println!("✅ Performance optimized with intelligent caching");
    println!("✅ Extensible architecture ready for future entity types");

    Ok(())
}
