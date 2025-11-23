//! Demonstration of SVG export functionality in TextCAD
//!
//! This example showcases the SVG export system including:
//! - Export trait and SVGExporter implementation
//! - Coordinate transformation (meters to SVG units)
//! - Bounding box calculation
//! - Line and circle rendering
//! - Complete SVG document generation

use std::fs;
use textcad::*;
use z3::{Config, Context};

fn main() {
    println!("=== TextCAD SVG Export Demo ===\n");

    // Demo 1: Simple line export
    println!("Demo 1: Exporting a simple line to SVG");
    demo_simple_line();
    println!();

    // Demo 2: Multiple lines forming a triangle
    println!("Demo 2: Exporting a triangle to SVG");
    demo_triangle();
    println!();

    // Demo 3: Circle export
    println!("Demo 3: Exporting a circle to SVG");
    demo_circle();
    println!();

    // Demo 4: Complex sketch with lines and circles
    println!("Demo 4: Exporting a complex sketch with both lines and circles");
    demo_complex_sketch();
    println!();

    println!("=== All SVG files generated successfully! ===");
    println!("Check the following files:");
    println!("  - simple_line.svg");
    println!("  - triangle.svg");
    println!("  - circle.svg");
    println!("  - complex_sketch.svg");
}

/// Demo 1: Export a simple line
fn demo_simple_line() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create two points
    let p1 = sketch.add_point(Some("start".to_string()));
    let p2 = sketch.add_point(Some("end".to_string()));

    // Fix the points
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(0.1), // 10cm
        Length::meters(0.1),
    ));

    // Create a line
    let _line = sketch.add_line(p1, p2, Some("simple_line".to_string()));

    // Solve and extract solution
    let solution = sketch.solve_and_extract().unwrap();

    // Export to SVG
    let exporter = SVGExporter::new();
    let svg = exporter.export(&sketch, &solution).unwrap();

    // Write to file
    fs::write("simple_line.svg", &svg).expect("Failed to write SVG file");
    println!("  Exported to simple_line.svg");
    println!("  Line from (0, 0) to (0.1, 0.1) meters");
}

/// Demo 2: Export a triangle
fn demo_triangle() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create three points for a triangle
    let p1 = sketch.add_point(Some("vertex_a".to_string()));
    let p2 = sketch.add_point(Some("vertex_b".to_string()));
    let p3 = sketch.add_point(Some("vertex_c".to_string()));

    // Fix the points to form a 3-4-5 right triangle
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(0.03), // 3cm
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(0.0),
        Length::meters(0.04), // 4cm
    ));

    // Create three lines to form the triangle
    let _line1 = sketch.add_line(p1, p2, Some("base".to_string()));
    let _line2 = sketch.add_line(p2, p3, Some("hypotenuse".to_string()));
    let _line3 = sketch.add_line(p3, p1, Some("height".to_string()));

    // Solve and extract solution
    let solution = sketch.solve_and_extract().unwrap();

    // Export to SVG
    let exporter = SVGExporter::new();
    let svg = exporter.export(&sketch, &solution).unwrap();

    // Write to file
    fs::write("triangle.svg", &svg).expect("Failed to write SVG file");
    println!("  Exported to triangle.svg");
    println!("  3-4-5 right triangle");
}

/// Demo 3: Export a circle
fn demo_circle() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a center point
    let center = sketch.add_point(Some("center".to_string()));

    // Fix the center point
    sketch.add_constraint(FixedPositionConstraint::new(
        center,
        Length::meters(0.05), // 5cm
        Length::meters(0.05),
    ));

    // Create a circle
    let _circle = sketch.add_circle(center, Some("test_circle".to_string()));

    // Note: Circle radius constraints will be added in a future phase
    // For now, the circle will have a symbolic radius that gets solved
    // The SVG export will extract and display the radius from the Z3 model

    // Solve and extract solution
    let solution = sketch.solve_and_extract().unwrap();

    // Export to SVG
    let exporter = SVGExporter::new();
    let svg = exporter.export(&sketch, &solution).unwrap();

    // Write to file
    fs::write("circle.svg", &svg).expect("Failed to write SVG file");
    println!("  Exported to circle.svg");
    println!("  Circle at (5cm, 5cm) with symbolic radius");
}

/// Demo 4: Export a complex sketch with both lines and circles
fn demo_complex_sketch() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a square with a circle in the center
    let corner1 = sketch.add_point(Some("bottom_left".to_string()));
    let corner2 = sketch.add_point(Some("bottom_right".to_string()));
    let corner3 = sketch.add_point(Some("top_right".to_string()));
    let corner4 = sketch.add_point(Some("top_left".to_string()));
    let center = sketch.add_point(Some("center".to_string()));

    // Define the square (10cm x 10cm)
    sketch.add_constraint(FixedPositionConstraint::new(
        corner1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        corner2,
        Length::meters(0.1),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        corner3,
        Length::meters(0.1),
        Length::meters(0.1),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        corner4,
        Length::meters(0.0),
        Length::meters(0.1),
    ));

    // Fix the center point
    sketch.add_constraint(FixedPositionConstraint::new(
        center,
        Length::meters(0.05),
        Length::meters(0.05),
    ));

    // Create the square lines
    let _bottom = sketch.add_line(corner1, corner2, Some("bottom".to_string()));
    let _right = sketch.add_line(corner2, corner3, Some("right".to_string()));
    let _top = sketch.add_line(corner3, corner4, Some("top".to_string()));
    let _left = sketch.add_line(corner4, corner1, Some("left".to_string()));

    // Create a circle in the center
    let _circle = sketch.add_circle(center, Some("center_circle".to_string()));

    // Note: Circle radius constraints will be added in a future phase
    // For now, the circle will have a symbolic radius

    // Solve and extract solution
    let solution = sketch.solve_and_extract().unwrap();

    // Export to SVG
    let exporter = SVGExporter::new();
    let svg = exporter.export(&sketch, &solution).unwrap();

    // Write to file
    fs::write("complex_sketch.svg", &svg).expect("Failed to write SVG file");
    println!("  Exported to complex_sketch.svg");
    println!("  10cm x 10cm square with circle in center");
}
