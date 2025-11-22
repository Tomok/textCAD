//! Phase 5 Demo: Basic Constraint System
//!
//! This example demonstrates the basic constraint system functionality
//! implemented in Phase 5, including point positioning and coincidence constraints.

use textcad::{
    constraints::{CoincidentPointsConstraint, FixedPositionConstraint},
    sketch::Sketch,
    units::Length,
};
use z3::{Config, Context};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("TextCAD Phase 5 Demo: Basic Constraint System\n");

    // Initialize Z3 context
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Demo 1: Single point with fixed position
    println!("=== Demo 1: Fixed Position Constraint ===");
    let p1 = sketch.add_point(Some("P1".to_string()));
    
    // Fix P1 at coordinates (3, 4) meters
    let constraint1 = FixedPositionConstraint::new(
        p1,
        Length::meters(3.0),
        Length::meters(4.0),
    );
    sketch.add_constraint(constraint1);

    // Solve and extract solution
    let solution = sketch.solve_and_extract()?;
    let (x1, y1) = solution.get_point_coordinates(p1)?;
    println!("P1 position: ({:.3}, {:.3}) meters", x1, y1);
    println!("Expected: (3.000, 4.000) meters");
    println!("Match: {}\n", (x1 - 3.0).abs() < 1e-6 && (y1 - 4.0).abs() < 1e-6);

    // Demo 2: Coincident points
    println!("=== Demo 2: Coincident Points Constraint ===");
    
    // Create a new sketch for the second demo
    let mut sketch2 = Sketch::new(&ctx);
    let p2 = sketch2.add_point(Some("P2".to_string()));
    let p3 = sketch2.add_point(Some("P3".to_string()));

    // Fix P2 at origin
    sketch2.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(0.0),
        Length::meters(0.0),
    ));

    // Make P3 coincident with P2
    sketch2.add_constraint(CoincidentPointsConstraint::new(p2, p3));

    let solution2 = sketch2.solve_and_extract()?;
    let (x2, y2) = solution2.get_point_coordinates(p2)?;
    let (x3, y3) = solution2.get_point_coordinates(p3)?;
    
    println!("P2 position: ({:.3}, {:.3}) meters", x2, y2);
    println!("P3 position: ({:.3}, {:.3}) meters", x3, y3);
    println!("Points are coincident: {}\n", 
        (x2 - x3).abs() < 1e-6 && (y2 - y3).abs() < 1e-6);

    // Demo 3: Multiple constraints with different units
    println!("=== Demo 3: Multiple Constraints with Unit Conversion ===");
    
    let mut sketch3 = Sketch::new(&ctx);
    let p4 = sketch3.add_point(Some("P4".to_string()));
    let p5 = sketch3.add_point(Some("P5".to_string()));

    // Fix P4 using millimeters and centimeters
    sketch3.add_constraint(FixedPositionConstraint::new(
        p4,
        Length::millimeters(1000.0), // 1 meter in mm
        Length::centimeters(150.0),  // 1.5 meters in cm
    ));

    // Make P5 coincident with P4
    sketch3.add_constraint(CoincidentPointsConstraint::new(p4, p5));

    let solution3 = sketch3.solve_and_extract()?;
    let (x4, y4) = solution3.get_point_coordinates(p4)?;
    let (x5, y5) = solution3.get_point_coordinates(p5)?;

    println!("P4 position (from mm/cm input): ({:.3}, {:.3}) meters", x4, y4);
    println!("P5 position (coincident): ({:.3}, {:.3}) meters", x5, y5);
    println!("Expected: (1.000, 1.500) meters");
    println!("Unit conversion works: {}\n", 
        (x4 - 1.0).abs() < 1e-6 && (y4 - 1.5).abs() < 1e-6);

    // Demo 4: Overconstrained system (error handling)
    println!("=== Demo 4: Overconstrained System ===");
    
    let mut sketch4 = Sketch::new(&ctx);
    let p6 = sketch4.add_point(Some("P6".to_string()));

    // Try to fix the same point at two different positions
    sketch4.add_constraint(FixedPositionConstraint::new(
        p6,
        Length::meters(1.0),
        Length::meters(1.0),
    ));
    sketch4.add_constraint(FixedPositionConstraint::new(
        p6,
        Length::meters(2.0),
        Length::meters(2.0),
    ));

    match sketch4.solve_and_extract() {
        Ok(_) => println!("Unexpected: System should be overconstrained!"),
        Err(error) => {
            println!("Correctly detected overconstrained system:");
            println!("Error: {}", error);
        }
    }

    println!("\n=== Phase 5 Demo Complete ===");
    println!("✅ Fixed position constraints");
    println!("✅ Coincident point constraints");
    println!("✅ Unit conversion support");
    println!("✅ Error handling for overconstrained systems");
    println!("✅ Solution extraction and coordinate access");

    Ok(())
}