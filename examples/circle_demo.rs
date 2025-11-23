//! Circle Entity Demo
//!
//! Demonstrates the basic Circle entity functionality in TextCAD.
//! This example shows:
//! - Creating circles with Z3-integrated radius variables
//! - Arena-based entity management
//! - Entity-ID relationships (circle references center point)
//!
//! Note: This demo focuses on entity creation and management.
//! Circle constraints and solving will be demonstrated in future phases.

use textcad::Sketch;
use z3::{Config, Context};

fn main() {
    println!("=== TextCAD Circle Entity Demo ===\n");

    // Create Z3 context and sketch
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    println!("1. Creating points for circle centers...");
    let center1 = sketch.add_point(Some("center1".to_string()));
    let center2 = sketch.add_point(Some("center2".to_string()));
    println!("   Created center points: {:?} and {:?}", center1, center2);

    println!("\n2. Creating circles with different centers...");
    let circle1 = sketch.add_circle(center1, Some("circle1".to_string()));
    let circle2 = sketch.add_circle(center2, Some("circle2".to_string()));
    let circle3 = sketch.add_circle(center1, None); // Same center, unnamed

    println!(
        "   Created circles: {:?}, {:?}, {:?}",
        circle1, circle2, circle3
    );

    println!("\n3. Circle entity details:");

    // Examine first circle
    let c1 = sketch.get_circle(circle1).unwrap();
    println!("   Circle 1:");
    println!("     ID: {:?}", c1.id);
    println!("     Center: {:?}", c1.center);
    println!("     Name: {:?}", c1.name);
    println!("     Display name: {}", c1.display_name());
    println!("     Z3 radius variable: {}", c1.radius);

    // Examine second circle
    let c2 = sketch.get_circle(circle2).unwrap();
    println!("   Circle 2:");
    println!("     ID: {:?}", c2.id);
    println!("     Center: {:?}", c2.center);
    println!("     Name: {:?}", c2.name);
    println!("     Display name: {}", c2.display_name());
    println!("     Z3 radius variable: {}", c2.radius);

    // Examine unnamed circle
    let c3 = sketch.get_circle(circle3).unwrap();
    println!("   Circle 3 (unnamed):");
    println!("     ID: {:?}", c3.id);
    println!("     Center: {:?}", c3.center);
    println!("     Name: {:?}", c3.name);
    println!("     Display name: {}", c3.display_name());
    println!("     Z3 radius variable: {}", c3.radius);

    println!("\n4. Demonstrating entity relationships:");
    println!("   Circle 1 center point: {:?}", c1.center_point());
    println!("   Circle 2 center point: {:?}", c2.center_point());
    println!("   Circle 3 center point: {:?}", c3.center_point());

    // Verify that circles 1 and 3 share the same center
    assert_eq!(c1.center_point(), c3.center_point());
    println!("   ✓ Circles 1 and 3 correctly share the same center");

    // Verify that circle 2 has a different center
    assert_ne!(c2.center_point(), c1.center_point());
    println!("   ✓ Circle 2 has a different center as expected");

    println!("\n5. Z3 symbolic variables:");
    println!("   Each circle has its own symbolic radius variable for constraint solving:");
    println!("     Circle 1 radius: {}", c1.radius);
    println!("     Circle 2 radius: {}", c2.radius);
    println!("     Circle 3 radius: {}", c3.radius);

    // Verify that radius variables are distinct
    assert_ne!(c1.radius.to_string(), c2.radius.to_string());
    assert_ne!(c1.radius.to_string(), c3.radius.to_string());
    assert_ne!(c2.radius.to_string(), c3.radius.to_string());
    println!("   ✓ All radius variables are distinct for constraint solving");

    println!("\n=== Demo Complete ===");
    println!("\nThe Circle entity is now ready for:");
    println!("  - Circle radius constraints (Phase 10)");
    println!("  - Point-on-circle constraints (Phase 10)");
    println!("  - Circle-circle relationships (future phases)");
    println!("  - SVG export with circles (Phase 12)");
    println!("\nNext: Implement CircleRadiusConstraint and PointOnCircleConstraint");
}
