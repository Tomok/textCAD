//! Phase 8 Demo: Line Constraints (length, parallel, perpendicular)
//!
//! Demonstrates the implementation of Phase 8 constraints:
//! - LineLengthConstraint: Forces a line to have a specific length
//! - ParallelLinesConstraint: Forces two lines to be parallel
//! - PerpendicularLinesConstraint: Forces two lines to be perpendicular
//!
//! This example constructs a rectangular frame with additional constraints
//! to verify the geometric relationships work correctly with Z3 solving.

use textcad::{FixedPositionConstraint, Length, Sketch, TextCadError};
use z3::{Config, Context};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TextCAD Phase 8 Demo: Line Constraints ===\n");

    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Example 1: Rectangle with parallel and perpendicular constraints
    println!("1. Constructing a rectangle using parallel and perpendicular constraints:");

    // Create four corner points for the rectangle
    let bottom_left = sketch.add_point(Some("bottom_left".to_string()));
    let bottom_right = sketch.add_point(Some("bottom_right".to_string()));
    let top_right = sketch.add_point(Some("top_right".to_string()));
    let top_left = sketch.add_point(Some("top_left".to_string()));

    // Create four lines forming the rectangle
    let bottom = sketch.add_line(bottom_left, bottom_right, Some("bottom".to_string()));
    let right = sketch.add_line(bottom_right, top_right, Some("right".to_string()));
    let top = sketch.add_line(top_right, top_left, Some("top".to_string()));
    let left = sketch.add_line(top_left, bottom_left, Some("left".to_string()));

    // Fix the bottom-left corner at the origin
    sketch.add_constraint(FixedPositionConstraint::new(
        bottom_left,
        Length::meters(0.0),
        Length::meters(0.0),
    ));

    // Fix the bottom-right corner to create a horizontal base
    sketch.add_constraint(FixedPositionConstraint::new(
        bottom_right,
        Length::meters(4.0),
        Length::meters(0.0),
    ));

    // Set specific dimensions for the rectangle
    sketch.add_constraint(
        sketch
            .get_line(bottom)
            .unwrap()
            .length_equals(Length::meters(4.0)),
    );
    sketch.add_constraint(
        sketch
            .get_line(left)
            .unwrap()
            .length_equals(Length::meters(3.0)),
    );

    // Apply geometric constraints - need to collect constraint objects first to avoid borrowing conflicts
    let bottom_line = sketch.get_line(bottom).unwrap().clone();
    let left_line = sketch.get_line(left).unwrap().clone();
    let right_line = sketch.get_line(right).unwrap().clone();
    let top_line = sketch.get_line(top).unwrap().clone();

    // Parallel constraints: top || bottom, left || right
    sketch.add_constraint(top_line.parallel_to(&bottom_line));
    sketch.add_constraint(left_line.parallel_to(&right_line));

    // Perpendicular constraints: bottom ⊥ left, bottom ⊥ right
    sketch.add_constraint(bottom_line.perpendicular_to(&left_line));
    sketch.add_constraint(bottom_line.perpendicular_to(&right_line));

    // Solve the constraint system
    println!("   Applying parallel and perpendicular constraints...");
    let mut solution = match sketch.solve_and_extract() {
        Ok(sol) => sol,
        Err(TextCadError::OverConstrained) => {
            println!("   ❌ Constraint system is unsatisfiable!");
            return Err("Rectangle constraints failed".into());
        }
        Err(e) => return Err(Box::new(e)),
    };

    // Extract and display the results
    println!("   ✅ Rectangle solved successfully!");

    let bottom_left_coords = solution.get_point_coordinates(bottom_left)?;
    let bottom_right_coords = solution.get_point_coordinates(bottom_right)?;
    let top_right_coords = solution.get_point_coordinates(top_right)?;
    let top_left_coords = solution.get_point_coordinates(top_left)?;

    println!("   Rectangle coordinates:");
    println!(
        "     Bottom-left:  ({:.3}, {:.3})",
        bottom_left_coords.0, bottom_left_coords.1
    );
    println!(
        "     Bottom-right: ({:.3}, {:.3})",
        bottom_right_coords.0, bottom_right_coords.1
    );
    println!(
        "     Top-right:    ({:.3}, {:.3})",
        top_right_coords.0, top_right_coords.1
    );
    println!(
        "     Top-left:     ({:.3}, {:.3})",
        top_left_coords.0, top_left_coords.1
    );

    // Verify the geometric properties
    let bottom_params =
        solution.extract_line_parameters(bottom, bottom_left_coords, bottom_right_coords)?;
    let left_params =
        solution.extract_line_parameters(left, top_left_coords, bottom_left_coords)?;

    println!("   Measured dimensions:");
    println!("     Width (bottom):  {:.3}m", bottom_params.length);
    println!("     Height (left):   {:.3}m", left_params.length);

    // Example 2: Perpendicular bisector construction
    println!("\n2. Constructing perpendicular bisectors:");

    // Create a new sketch for the bisector example
    let mut bisector_sketch = Sketch::new(&ctx);

    // Create three points forming a triangle
    let a = bisector_sketch.add_point(Some("A".to_string()));
    let b = bisector_sketch.add_point(Some("B".to_string()));
    let c = bisector_sketch.add_point(Some("C".to_string()));

    // Create the sides of the triangle
    let ab = bisector_sketch.add_line(a, b, Some("AB".to_string()));
    let _bc = bisector_sketch.add_line(b, c, Some("BC".to_string()));

    // Create midpoint and perpendicular bisector
    let midpoint_ab = bisector_sketch.add_point(Some("midpoint_AB".to_string()));
    let bisector_end = bisector_sketch.add_point(Some("bisector_end".to_string()));
    let perpendicular_bisector =
        bisector_sketch.add_line(midpoint_ab, bisector_end, Some("perp_bisector".to_string()));

    // Fix triangle vertices
    bisector_sketch.add_constraint(FixedPositionConstraint::new(
        a,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    bisector_sketch.add_constraint(FixedPositionConstraint::new(
        b,
        Length::meters(6.0),
        Length::meters(0.0),
    ));
    bisector_sketch.add_constraint(FixedPositionConstraint::new(
        c,
        Length::meters(3.0),
        Length::meters(4.0),
    ));

    // Constrain midpoint to be exactly in the middle of AB
    bisector_sketch.add_constraint(FixedPositionConstraint::new(
        midpoint_ab,
        Length::meters(3.0),
        Length::meters(0.0),
    ));

    // Make the bisector perpendicular to AB and set its length
    let ab_line = bisector_sketch.get_line(ab).unwrap().clone();
    let bisector_line = bisector_sketch
        .get_line(perpendicular_bisector)
        .unwrap()
        .clone();

    bisector_sketch.add_constraint(bisector_line.perpendicular_to(&ab_line));
    bisector_sketch.add_constraint(bisector_line.length_equals(Length::meters(2.0)));

    // Solve the bisector constraint system
    println!("   Applying perpendicular bisector constraints...");
    let mut bisector_solution = match bisector_sketch.solve_and_extract() {
        Ok(sol) => sol,
        Err(TextCadError::OverConstrained) => {
            println!("   ❌ Bisector constraint system is unsatisfiable!");
            return Err("Bisector constraints failed".into());
        }
        Err(e) => return Err(Box::new(e)),
    };

    println!("   ✅ Perpendicular bisector solved successfully!");

    let a_coords = bisector_solution.get_point_coordinates(a)?;
    let b_coords = bisector_solution.get_point_coordinates(b)?;
    let midpoint_coords = bisector_solution.get_point_coordinates(midpoint_ab)?;
    let bisector_end_coords = bisector_solution.get_point_coordinates(bisector_end)?;

    println!("   Triangle and bisector coordinates:");
    println!(
        "     A:                ({:.3}, {:.3})",
        a_coords.0, a_coords.1
    );
    println!(
        "     B:                ({:.3}, {:.3})",
        b_coords.0, b_coords.1
    );
    println!(
        "     Midpoint of AB:   ({:.3}, {:.3})",
        midpoint_coords.0, midpoint_coords.1
    );
    println!(
        "     Bisector end:     ({:.3}, {:.3})",
        bisector_end_coords.0, bisector_end_coords.1
    );

    let bisector_params = bisector_solution.extract_line_parameters(
        perpendicular_bisector,
        midpoint_coords,
        bisector_end_coords,
    )?;
    println!("   Bisector length:    {:.3}m", bisector_params.length);
    println!(
        "   Bisector angle:     {:.1}°",
        bisector_params.angle.to_degrees()
    );

    println!("\n=== Phase 8 Demo Complete ===");
    println!("✅ Successfully demonstrated:");
    println!("  • LineLengthConstraint: Setting specific line lengths");
    println!("  • ParallelLinesConstraint: Creating parallel relationships using cross product");
    println!(
        "  • PerpendicularLinesConstraint: Creating perpendicular relationships using dot product"
    );
    println!(
        "  • Entity-as-constraint-factory pattern: line.parallel_to() and line.perpendicular_to()"
    );
    println!("  • Integration with existing Z3 solving and solution extraction systems");

    Ok(())
}
