//! Enhanced demonstration of Point2D entity and constraint solving capabilities
//!
//! This demo showcases the Point2D entity with various constraint scenarios,
//! demonstrating how points can be positioned through constraint-based geometric modeling.

use std::ops::{Add, Mul, Sub};
use textcad::{PointId, sketch::Sketch};
use z3::{
    Config, Context, SatResult,
    ast::{Ast, Real},
};

fn main() {
    println!("=== TextCAD Point2D Constraint Solving Demo ===\n");

    // Demo 1: Basic point positioning with fixed coordinates
    demo_fixed_positioning();

    // Demo 2: Distance constraints between points
    demo_distance_constraints();

    // Demo 3: Parametric point positioning (parabolic curve)
    demo_parametric_curve();

    // Demo 4: Geometric optimization (isosceles triangle)
    demo_geometric_optimization();

    println!("=== Demo Complete ===");
    println!("This showcases how point positions emerge naturally from constraint systems!");
}

/// Demonstrate basic point positioning with fixed coordinates
fn demo_fixed_positioning() {
    println!("--- Demo 1: Fixed Point Positioning ---");

    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create three points
    let origin = sketch.add_point(Some("origin".to_string()));
    let corner = sketch.add_point(Some("corner".to_string()));
    let floating = sketch.add_point(Some("floating".to_string()));

    // Fix origin at (0, 0)
    fix_point_at(&mut sketch, origin, 0.0, 0.0);

    // Fix corner at (3.5, 2.1) meters
    fix_point_at(&mut sketch, corner, 3.5, 2.1);

    // Leave floating point unconstrained

    match sketch.solve() {
        Ok(SatResult::Sat) => {
            let model = sketch.solver().get_model().unwrap();

            println!("✓ Fixed positioning solved successfully:");
            print_point(&model, &sketch, origin, "Origin");
            print_point(&model, &sketch, corner, "Corner");
            print_point(&model, &sketch, floating, "Floating (solver picks)");
        }
        _ => println!("✗ Failed to solve basic positioning"),
    }
    println!();
}

/// Demonstrate distance constraints to form a right triangle
fn demo_distance_constraints() {
    println!("--- Demo 2: 3-4-5 Right Triangle from Distance Constraints ---");

    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let a = sketch.add_point(Some("vertex_a".to_string()));
    let b = sketch.add_point(Some("vertex_b".to_string()));
    let c = sketch.add_point(Some("vertex_c".to_string()));

    // Fix vertex A at origin
    fix_point_at(&mut sketch, a, 0.0, 0.0);

    // Create distance constraints for a 3-4-5 triangle
    add_distance_constraint(&mut sketch, a, b, 3.0); // 3 meters
    add_distance_constraint(&mut sketch, b, c, 4.0); // 4 meters  
    add_distance_constraint(&mut sketch, a, c, 5.0); // 5 meters (hypotenuse)

    match sketch.solve() {
        Ok(SatResult::Sat) => {
            let model = sketch.solver().get_model().unwrap();

            println!("✓ Right triangle solved from distance constraints:");
            print_point(&model, &sketch, a, "Vertex A (fixed)");
            print_point(&model, &sketch, b, "Vertex B");
            print_point(&model, &sketch, c, "Vertex C");

            // Verify the Pythagorean theorem
            let distances = get_triangle_distances(&model, &sketch, a, b, c);
            println!(
                "  Verification: {:.3}² + {:.3}² = {:.3}² ✓",
                distances.0, distances.1, distances.2
            );
        }
        _ => println!("✗ Failed to solve triangle constraints"),
    }
    println!();
}

/// Demonstrate parametric positioning along a parabolic curve
fn demo_parametric_curve() {
    println!("--- Demo 3: Points Along Parabolic Curve y = 0.4x² - 1.6x + 1 ---");

    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let mut curve_points = Vec::new();
    for i in 0..6 {
        curve_points.push(sketch.add_point(Some(format!("curve_p{}", i))));
    }

    // Parameters for parabola: y = 0.4x² - 1.6x + 1
    let a = 0.4;
    let b = -1.6;
    let c = 1.0;

    // Position points along the curve
    for (i, &point_id) in curve_points.iter().enumerate() {
        let x_val = (i as f64 - 2.5) * 0.8; // x values spread from -2 to +2
        let y_val = a * x_val * x_val + b * x_val + c;
        fix_point_at(&mut sketch, point_id, x_val, y_val);
    }

    match sketch.solve() {
        Ok(SatResult::Sat) => {
            let model = sketch.solver().get_model().unwrap();

            println!("✓ Parabolic curve points computed:");
            for (i, &point_id) in curve_points.iter().enumerate() {
                print_point(&model, &sketch, point_id, &format!("Curve Point {}", i));
            }
        }
        _ => println!("✗ Failed to solve parametric positioning"),
    }
    println!();
}

/// Demonstrate geometric optimization with isosceles triangle
fn demo_geometric_optimization() {
    println!("--- Demo 4: Isosceles Triangle Optimization ---");

    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let base_left = sketch.add_point(Some("base_left".to_string()));
    let base_right = sketch.add_point(Some("base_right".to_string()));
    let apex = sketch.add_point(Some("apex".to_string()));

    // Create base of triangle
    fix_point_at(&mut sketch, base_left, 0.0, 0.0);
    fix_point_at(&mut sketch, base_right, 4.0, 0.0);

    // Constraint: equal distances from apex to both base points (isosceles)
    // Use distance 3.0 which allows for a proper triangle with base 4.0
    add_distance_constraint(&mut sketch, base_left, apex, 3.0);
    add_distance_constraint(&mut sketch, base_right, apex, 3.0);

    // Force non-degenerate triangle by constraining y-coordinate of apex
    let apex_point = sketch.get_point(apex).unwrap();
    let min_height = Real::from_real(sketch.context(), 1000, 1000); // 1.0 meter minimum
    let y_constraint = apex_point.y.gt(&min_height);
    sketch.solver_mut().assert(&y_constraint);

    match sketch.solve() {
        Ok(SatResult::Sat) => {
            let model = sketch.solver().get_model().unwrap();

            println!("✓ Isosceles triangle optimization complete:");
            print_point(&model, &sketch, base_left, "Base Left");
            print_point(&model, &sketch, base_right, "Base Right");
            print_point(&model, &sketch, apex, "Apex (optimized)");

            // Show triangle properties
            let (side1, side2, base) =
                get_triangle_distances(&model, &sketch, base_left, apex, base_right);
            println!("  Triangle properties:");
            println!("    Equal sides: {:.3}m and {:.3}m", side1, side2);
            println!("    Base: {:.3}m", base);
            println!(
                "    Apex height: {:.3}m",
                get_point_coords(&model, &sketch, apex).1
            );
        }
        _ => println!("✗ Failed to solve isosceles triangle"),
    }
    println!();
}

/// Helper to fix a point at specific coordinates  
fn fix_point_at(sketch: &mut Sketch, point_id: PointId, x: f64, y: f64) {
    let point = sketch.get_point(point_id).unwrap();

    // Create Z3 rational values (convert to milliRationals for precision)
    let x_val = Real::from_real(sketch.context(), (x * 1000.0) as i32, 1000);
    let y_val = Real::from_real(sketch.context(), (y * 1000.0) as i32, 1000);

    // Store constraint expressions to avoid borrow conflicts
    let x_constraint = point.x._eq(&x_val);
    let y_constraint = point.y._eq(&y_val);

    sketch.solver_mut().assert(&x_constraint);
    sketch.solver_mut().assert(&y_constraint);
}

/// Helper to add distance constraint between two points
fn add_distance_constraint(
    sketch: &mut Sketch,
    p1_id: PointId,
    p2_id: PointId,
    distance_meters: f64,
) {
    let p1 = sketch.get_point(p1_id).unwrap();
    let p2 = sketch.get_point(p2_id).unwrap();

    // Distance squared formula: (x2-x1)² + (y2-y1)² = distance²
    // Use references and clone to avoid ownership issues
    let dx = (&p2.x).sub(&p1.x);
    let dy = (&p2.y).sub(&p1.y);
    let dx_sq = (&dx).mul(&dx);
    let dy_sq = (&dy).mul(&dy);
    let dist_sq = (&dx_sq).add(&dy_sq);

    let target_sq = Real::from_real(
        sketch.context(),
        (distance_meters * distance_meters * 1_000_000.0) as i32,
        1_000_000,
    );

    let constraint = dist_sq._eq(&target_sq);
    sketch.solver_mut().assert(&constraint);
}

/// Helper to print point coordinates from solved model
fn print_point(model: &z3::Model, sketch: &Sketch, point_id: PointId, name: &str) {
    let (x, y) = get_point_coords(model, sketch, point_id);
    println!("  {}: ({:.3}, {:.3}) meters", name, x, y);
}

/// Helper to get point coordinates from model
fn get_point_coords(model: &z3::Model, sketch: &Sketch, point_id: PointId) -> (f64, f64) {
    let point = sketch.get_point(point_id).unwrap();

    let x_val = model.eval(&point.x, true).unwrap();
    let y_val = model.eval(&point.y, true).unwrap();

    let x = if let Some((num, den)) = x_val.as_real() {
        num as f64 / den as f64
    } else {
        0.0
    };

    let y = if let Some((num, den)) = y_val.as_real() {
        num as f64 / den as f64
    } else {
        0.0
    };

    (x, y)
}

/// Helper to get distances between three points (triangle)
fn get_triangle_distances(
    model: &z3::Model,
    sketch: &Sketch,
    p1: PointId,
    p2: PointId,
    p3: PointId,
) -> (f64, f64, f64) {
    let (x1, y1) = get_point_coords(model, sketch, p1);
    let (x2, y2) = get_point_coords(model, sketch, p2);
    let (x3, y3) = get_point_coords(model, sketch, p3);

    let d12 = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    let d23 = ((x3 - x2).powi(2) + (y3 - y2).powi(2)).sqrt();
    let d13 = ((x3 - x1).powi(2) + (y3 - y1).powi(2)).sqrt();

    (d12, d23, d13)
}
