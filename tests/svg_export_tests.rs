//! Integration tests for Phase 12 SVG Export
//!
//! Tests complete workflows including sketch creation, constraint application,
//! solving, solution extraction, and SVG export for various geometric configurations.

use proptest::prelude::*;
use textcad::constraints::{CircleRadiusConstraint, FixedPositionConstraint, LineLengthConstraint};
use textcad::export::{Exporter, SVGExporter};
use textcad::sketch::Sketch;
use textcad::units::Length;
use z3::{Config, Context};

// ============================================================================
// Basic Integration Tests
// ============================================================================

#[test]
fn test_svg_export_empty_sketch() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Add a point so the sketch can be solved (empty sketch has nothing to solve)
    let p1 = sketch.add_point(None);
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));

    let solution = sketch
        .solve_and_extract()
        .expect("Should solve empty sketch");

    let exporter = SVGExporter::new();
    let svg = exporter
        .export(&sketch, &solution)
        .expect("Should export empty sketch");

    // Should produce valid SVG with basic structure
    assert!(svg.contains("<svg"));
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(svg.contains("viewBox="));
    assert!(svg.contains("</svg>"));

    // Should have no line or circle elements
    assert!(!svg.contains("<line"));
    assert!(!svg.contains("<circle"));
}

#[test]
fn test_svg_export_single_line() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(None);
    let p2 = sketch.add_point(None);

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

    let _line = sketch.add_line(p1, p2, None);

    let solution = sketch
        .solve_and_extract()
        .expect("Should solve successfully");

    let exporter = SVGExporter::new();
    let svg = exporter
        .export(&sketch, &solution)
        .expect("Should export successfully");

    // Verify SVG structure
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    assert!(svg.contains("<line"));

    // Verify coordinates are transformed correctly
    // p1 at (0, 0) -> SVG (0, 0)
    // p2 at (0.1, 0.1) -> SVG (100, -100) because Y is flipped
    assert!(svg.contains("x1=\"0.00\""));
    assert!(svg.contains("y1=\"0.00\""));
    assert!(svg.contains("x2=\"100.00\""));
    assert!(svg.contains("y2=\"-100.00\""));

    // Verify stroke attributes
    assert!(svg.contains("stroke=\"black\""));
    assert!(svg.contains("stroke-width=\"2\""));
}

#[test]
fn test_svg_export_single_line_from_implementation_plan() {
    // This is the exact test from IMPLEMENTATION_PLAN.md
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(None);
    let p2 = sketch.add_point(None);

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

    let _line = sketch.add_line(p1, p2, None);

    let solution = sketch
        .solve_and_extract()
        .expect("Should solve successfully");

    let exporter = SVGExporter::new();
    let svg = exporter
        .export(&sketch, &solution)
        .expect("Should export successfully");

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<line"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn test_svg_export_multiple_lines() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a triangle
    let p1 = sketch.add_point(Some("p1".to_string()));
    let p2 = sketch.add_point(Some("p2".to_string()));
    let p3 = sketch.add_point(Some("p3".to_string()));

    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(1.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(0.5),
        Length::meters(0.866), // Approximate equilateral triangle
    ));

    sketch.add_line(p1, p2, Some("line1".to_string()));
    sketch.add_line(p2, p3, Some("line2".to_string()));
    sketch.add_line(p3, p1, Some("line3".to_string()));

    let solution = sketch
        .solve_and_extract()
        .expect("Should solve successfully");

    let exporter = SVGExporter::new();
    let svg = exporter
        .export(&sketch, &solution)
        .expect("Should export successfully");

    // Should contain three line elements
    let line_count = svg.matches("<line").count();
    assert_eq!(line_count, 3, "Should have exactly 3 lines");

    // Verify all lines are closed properly
    assert_eq!(svg.matches("/>").count() >= 3, true);
}

#[test]
fn test_svg_export_single_circle() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let center = sketch.add_point(Some("center".to_string()));

    sketch.add_constraint(FixedPositionConstraint::new(
        center,
        Length::meters(0.5),
        Length::meters(0.5),
    ));

    let circle = sketch.add_circle(center, Some("circle1".to_string()));

    sketch.add_constraint(CircleRadiusConstraint::new(circle, Length::meters(0.2)));

    let solution = sketch
        .solve_and_extract()
        .expect("Should solve successfully");

    let exporter = SVGExporter::new();
    let svg = exporter
        .export(&sketch, &solution)
        .expect("Should export successfully");

    // Verify SVG structure
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    assert!(svg.contains("<circle"));

    // Verify circle attributes
    // Center at (0.5, 0.5) -> SVG (500, -500)
    assert!(svg.contains("cx=\"500.00\""));
    assert!(svg.contains("cy=\"-500.00\""));

    // Radius 0.2m -> 200 SVG units
    assert!(svg.contains("r=\"200.00\""));

    // Verify stroke attributes
    assert!(svg.contains("stroke=\"black\""));
    assert!(svg.contains("fill=\"none\""));
}

#[test]
fn test_svg_export_complex_geometry() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create a square with a circle in the middle
    let p1 = sketch.add_point(Some("corner1".to_string()));
    let p2 = sketch.add_point(Some("corner2".to_string()));
    let p3 = sketch.add_point(Some("corner3".to_string()));
    let p4 = sketch.add_point(Some("corner4".to_string()));
    let center = sketch.add_point(Some("center".to_string()));

    // Square corners
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(1.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p3,
        Length::meters(1.0),
        Length::meters(1.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p4,
        Length::meters(0.0),
        Length::meters(1.0),
    ));

    // Center of square
    sketch.add_constraint(FixedPositionConstraint::new(
        center,
        Length::meters(0.5),
        Length::meters(0.5),
    ));

    // Square sides
    sketch.add_line(p1, p2, Some("bottom".to_string()));
    sketch.add_line(p2, p3, Some("right".to_string()));
    sketch.add_line(p3, p4, Some("top".to_string()));
    sketch.add_line(p4, p1, Some("left".to_string()));

    // Circle
    let circle = sketch.add_circle(center, Some("inner_circle".to_string()));
    sketch.add_constraint(CircleRadiusConstraint::new(circle, Length::meters(0.3)));

    let solution = sketch
        .solve_and_extract()
        .expect("Should solve successfully");

    let exporter = SVGExporter::new();
    let svg = exporter
        .export(&sketch, &solution)
        .expect("Should export successfully");

    // Verify we have both lines and circles
    let line_count = svg.matches("<line").count();
    let circle_count = svg.matches("<circle").count();

    assert_eq!(line_count, 4, "Should have 4 lines");
    assert_eq!(circle_count, 1, "Should have 1 circle");
}

// ============================================================================
// SVG Structure and Namespace Tests
// ============================================================================

#[test]
fn test_svg_namespace_correct() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(None);
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));

    let solution = sketch.solve_and_extract().expect("Should solve");

    let exporter = SVGExporter::new();
    let svg = exporter.export(&sketch, &solution).expect("Should export");

    // Verify correct SVG namespace
    assert!(
        svg.contains("xmlns=\"http://www.w3.org/2000/svg\""),
        "SVG should have correct namespace"
    );
}

#[test]
fn test_svg_viewbox_calculation() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create points at known positions
    let p1 = sketch.add_point(None);
    let p2 = sketch.add_point(None);

    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(1.0),
        Length::meters(2.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(3.0),
        Length::meters(4.0),
    ));

    sketch.add_line(p1, p2, None);

    let solution = sketch.solve_and_extract().expect("Should solve");

    let exporter = SVGExporter::new();
    let svg = exporter.export(&sketch, &solution).expect("Should export");

    // With default scale (1000), padding (10):
    // p1: (1.0, 2.0) -> SVG (1000, -2000)
    // p2: (3.0, 4.0) -> SVG (3000, -4000)
    // min_x = 1000, max_x = 3000, width = 2000 + 20 = 2020
    // min_y = -4000, max_y = -2000, height = 2000 + 20 = 2020
    // viewBox should be: "990 -4010 2020 2020"

    assert!(
        svg.contains("viewBox=\"990"),
        "ViewBox should start at min_x - padding"
    );
    assert!(
        svg.contains("-4010"),
        "ViewBox should include min_y - padding"
    );
    assert!(svg.contains("2020"), "ViewBox width should include padding");
}

#[test]
fn test_svg_viewbox_padding() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(None);
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));

    let solution = sketch.solve_and_extract().expect("Should solve");

    let exporter = SVGExporter::new();
    let svg = exporter.export(&sketch, &solution).expect("Should export");

    // Single point at origin with padding 10
    // Should have viewBox with padding on all sides
    assert!(svg.contains("viewBox=\"-10"));
}

// ============================================================================
// Coordinate Transformation Tests
// ============================================================================

#[test]
fn test_coordinate_transformation_in_export() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(None);
    let p2 = sketch.add_point(None);

    // Test positive and negative coordinates
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(-1.0),
        Length::meters(2.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(1.0),
        Length::meters(-2.0),
    ));

    sketch.add_line(p1, p2, None);

    let solution = sketch.solve_and_extract().expect("Should solve");

    let exporter = SVGExporter::new();
    let svg = exporter.export(&sketch, &solution).expect("Should export");

    // p1: (-1.0, 2.0) -> SVG (-1000, -2000)
    // p2: (1.0, -2.0) -> SVG (1000, 2000)
    assert!(svg.contains("x1=\"-1000.00\""));
    assert!(svg.contains("y1=\"-2000.00\""));
    assert!(svg.contains("x2=\"1000.00\""));
    assert!(svg.contains("y2=\"2000.00\""));
}

#[test]
fn test_y_axis_flip_in_export() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(None);
    let p2 = sketch.add_point(None);

    // Points with different Y values
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(0.0),
        Length::meters(1.0), // Positive Y
    ));

    sketch.add_line(p1, p2, None);

    let solution = sketch.solve_and_extract().expect("Should solve");

    let exporter = SVGExporter::new();
    let svg = exporter.export(&sketch, &solution).expect("Should export");

    // p1: (0, 0) -> SVG (0, 0)
    // p2: (0, 1.0) -> SVG (0, -1000) - Y is flipped
    assert!(svg.contains("y1=\"0.00\""));
    assert!(
        svg.contains("y2=\"-1000.00\""),
        "Y should be flipped in SVG"
    );
}

// ============================================================================
// Formatting and Precision Tests
// ============================================================================

#[test]
fn test_coordinate_decimal_precision() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(None);
    let p2 = sketch.add_point(None);

    // Use values that will test decimal precision
    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.123456),
        Length::meters(0.789012),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(1.111111),
        Length::meters(2.222222),
    ));

    sketch.add_line(p1, p2, None);

    let solution = sketch.solve_and_extract().expect("Should solve");

    let exporter = SVGExporter::new();
    let svg = exporter.export(&sketch, &solution).expect("Should export");

    // Should format to 2 decimal places as per implementation
    assert!(svg.contains(".00\"") || svg.contains(".11\"") || svg.contains(".12\""));

    // Should not have excessive precision
    assert!(!svg.contains("123456"));
}

// ============================================================================
// Regression Tests
// ============================================================================

#[test]
fn test_line_length_constraint_export() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(None);
    let p2 = sketch.add_point(None);

    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));

    let line = sketch.add_line(p1, p2, None);
    sketch.add_constraint(LineLengthConstraint::new(line, Length::meters(2.0)));

    // Don't fix p2, let the solver determine it based on length constraint

    let solution = sketch.solve_and_extract().expect("Should solve");

    let exporter = SVGExporter::new();
    let svg = exporter.export(&sketch, &solution).expect("Should export");

    // Should successfully export even with unconstrained endpoint
    assert!(svg.contains("<line"));

    // Verify the exported line has valid coordinates
    assert!(svg.contains("x1=\""));
    assert!(svg.contains("y1=\""));
    assert!(svg.contains("x2=\""));
    assert!(svg.contains("y2=\""));
}

#[test]
fn test_multiple_circles_export() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create two circles
    let c1 = sketch.add_point(Some("center1".to_string()));
    let c2 = sketch.add_point(Some("center2".to_string()));

    sketch.add_constraint(FixedPositionConstraint::new(
        c1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        c2,
        Length::meters(2.0),
        Length::meters(0.0),
    ));

    let circle1 = sketch.add_circle(c1, Some("circle1".to_string()));
    let circle2 = sketch.add_circle(c2, Some("circle2".to_string()));

    sketch.add_constraint(CircleRadiusConstraint::new(circle1, Length::meters(0.5)));
    sketch.add_constraint(CircleRadiusConstraint::new(circle2, Length::meters(0.8)));

    let solution = sketch.solve_and_extract().expect("Should solve");

    let exporter = SVGExporter::new();
    let svg = exporter.export(&sketch, &solution).expect("Should export");

    // Should have two circles
    let circle_count = svg.matches("<circle").count();
    assert_eq!(circle_count, 2, "Should have exactly 2 circles");

    // Verify both circles have different radii
    assert!(svg.contains("r=\"500.00\""));
    assert!(svg.contains("r=\"800.00\""));
}

// ============================================================================
// Property-Based Tests
// ============================================================================

proptest! {
    /// Property: Exported SVG always contains valid XML structure
    #[test]
    fn prop_svg_always_has_valid_structure(
        x1 in -10.0f64..10.0f64,
        y1 in -10.0f64..10.0f64,
        x2 in -10.0f64..10.0f64,
        y2 in -10.0f64..10.0f64
    ) {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        let p1 = sketch.add_point(None);
        let p2 = sketch.add_point(None);

        sketch.add_constraint(FixedPositionConstraint::new(
            p1,
            Length::meters(x1),
            Length::meters(y1),
        ));
        sketch.add_constraint(FixedPositionConstraint::new(
            p2,
            Length::meters(x2),
            Length::meters(y2),
        ));

        sketch.add_line(p1, p2, None);

        if let Ok(solution) = sketch.solve_and_extract() {
            let exporter = SVGExporter::new();
            if let Ok(svg) = exporter.export(&sketch, &solution) {
                // SVG should always have opening and closing tags
                prop_assert!(svg.starts_with("<svg"));
                prop_assert!(svg.ends_with("</svg>\n"));

                // Should have namespace
                prop_assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));

                // Should have viewBox
                prop_assert!(svg.contains("viewBox="));

                // Should be properly formatted
                prop_assert!(svg.contains("<line") || svg.contains("<circle"));
            }
        }
    }

    /// Property: Coordinate transformation is consistent
    #[test]
    fn prop_coordinate_transformation_is_consistent(
        x in -100.0f64..100.0f64,
        y in -100.0f64..100.0f64
    ) {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        let p1 = sketch.add_point(None);
        let p2 = sketch.add_point(None);

        // Point at origin
        sketch.add_constraint(FixedPositionConstraint::new(
            p1,
            Length::meters(0.0),
            Length::meters(0.0),
        ));

        // Point at test coordinates
        sketch.add_constraint(FixedPositionConstraint::new(
            p2,
            Length::meters(x),
            Length::meters(y),
        ));

        sketch.add_line(p1, p2, None);

        if let Ok(solution) = sketch.solve_and_extract() {
            let exporter = SVGExporter::new();
            if let Ok(svg) = exporter.export(&sketch, &solution) {
                // Expected SVG coordinates
                let svg_x = x * 1000.0;
                let svg_y = -y * 1000.0; // Y is flipped

                let svg_x_str = format!("{:.2}", svg_x);
                let svg_y_str = format!("{:.2}", svg_y);

                // SVG should contain the transformed coordinates
                prop_assert!(
                    svg.contains(&svg_x_str),
                    "SVG should contain X coordinate: {}", svg_x_str
                );
                prop_assert!(
                    svg.contains(&svg_y_str),
                    "SVG should contain Y coordinate: {}", svg_y_str
                );
            }
        }
    }

    /// Property: Bounding box always contains all entities
    #[test]
    fn prop_bounding_box_contains_all_points(
        x1 in -50.0f64..50.0f64,
        y1 in -50.0f64..50.0f64,
        x2 in -50.0f64..50.0f64,
        y2 in -50.0f64..50.0f64,
        x3 in -50.0f64..50.0f64,
        y3 in -50.0f64..50.0f64
    ) {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        let p1 = sketch.add_point(None);
        let p2 = sketch.add_point(None);
        let p3 = sketch.add_point(None);

        sketch.add_constraint(FixedPositionConstraint::new(
            p1,
            Length::meters(x1),
            Length::meters(y1),
        ));
        sketch.add_constraint(FixedPositionConstraint::new(
            p2,
            Length::meters(x2),
            Length::meters(y2),
        ));
        sketch.add_constraint(FixedPositionConstraint::new(
            p3,
            Length::meters(x3),
            Length::meters(y3),
        ));

        sketch.add_line(p1, p2, None);
        sketch.add_line(p2, p3, None);

        if let Ok(solution) = sketch.solve_and_extract() {
            let exporter = SVGExporter::new();
            if let Ok(svg) = exporter.export(&sketch, &solution) {
                // Extract viewBox values
                if let Some(viewbox_start) = svg.find("viewBox=\"") {
                    let viewbox_str = &svg[viewbox_start + 9..];
                    if let Some(viewbox_end) = viewbox_str.find("\"") {
                        let viewbox = &viewbox_str[..viewbox_end];
                        let parts: Vec<&str> = viewbox.split_whitespace().collect();

                        if parts.len() == 4 {
                            if let (Ok(vb_x), Ok(vb_y), Ok(vb_w), Ok(vb_h)) = (
                                parts[0].parse::<f64>(),
                                parts[1].parse::<f64>(),
                                parts[2].parse::<f64>(),
                                parts[3].parse::<f64>(),
                            ) {
                                // Transform all points to SVG coordinates
                                let svg_points = vec![
                                    (x1 * 1000.0, -y1 * 1000.0),
                                    (x2 * 1000.0, -y2 * 1000.0),
                                    (x3 * 1000.0, -y3 * 1000.0),
                                ];

                                // All points should be within viewBox (including padding)
                                for (px, py) in svg_points {
                                    prop_assert!(
                                        px >= vb_x && px <= vb_x + vb_w,
                                        "Point X {} should be within viewBox [{}, {}]",
                                        px, vb_x, vb_x + vb_w
                                    );
                                    prop_assert!(
                                        py >= vb_y && py <= vb_y + vb_h,
                                        "Point Y {} should be within viewBox [{}, {}]",
                                        py, vb_y, vb_y + vb_h
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Property: Circle radius is always scaled correctly
    #[test]
    fn prop_circle_radius_scaling(
        cx in -10.0f64..10.0f64,
        cy in -10.0f64..10.0f64,
        radius in 0.1f64..5.0f64
    ) {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut sketch = Sketch::new(&ctx);

        let center = sketch.add_point(None);
        sketch.add_constraint(FixedPositionConstraint::new(
            center,
            Length::meters(cx),
            Length::meters(cy),
        ));

        let circle = sketch.add_circle(center, None);
        sketch.add_constraint(CircleRadiusConstraint::new(circle, Length::meters(radius)));

        if let Ok(solution) = sketch.solve_and_extract() {
            let exporter = SVGExporter::new();
            if let Ok(svg) = exporter.export(&sketch, &solution) {
                // Expected SVG radius
                let svg_radius = radius * 1000.0;
                let radius_str = format!("r=\"{:.2}\"", svg_radius);

                prop_assert!(
                    svg.contains(&radius_str),
                    "SVG should contain radius: {}", radius_str
                );
            }
        }
    }
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_export_with_very_small_coordinates() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(None);
    let p2 = sketch.add_point(None);

    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0001),
        Length::meters(0.0001),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(0.0002),
        Length::meters(0.0002),
    ));

    sketch.add_line(p1, p2, None);

    let solution = sketch.solve_and_extract().expect("Should solve");

    let exporter = SVGExporter::new();
    let svg = exporter.export(&sketch, &solution).expect("Should export");

    // Should handle very small values correctly
    assert!(svg.contains("<line"));
}

#[test]
fn test_export_with_very_large_coordinates() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    let p1 = sketch.add_point(None);
    let p2 = sketch.add_point(None);

    sketch.add_constraint(FixedPositionConstraint::new(
        p1,
        Length::meters(0.0),
        Length::meters(0.0),
    ));
    sketch.add_constraint(FixedPositionConstraint::new(
        p2,
        Length::meters(100.0),
        Length::meters(100.0),
    ));

    sketch.add_line(p1, p2, None);

    let solution = sketch.solve_and_extract().expect("Should solve");

    let exporter = SVGExporter::new();
    let svg = exporter.export(&sketch, &solution).expect("Should export");

    // Should handle large values correctly
    assert!(svg.contains("<line"));
    assert!(svg.contains("100000.00")); // 100m * 1000 = 100000
}

#[test]
fn test_export_preserves_circle_count() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create exactly 5 circles
    for i in 0..5 {
        let center = sketch.add_point(Some(format!("c{}", i)));
        sketch.add_constraint(FixedPositionConstraint::new(
            center,
            Length::meters(i as f64),
            Length::meters(0.0),
        ));

        let circle = sketch.add_circle(center, Some(format!("circle{}", i)));
        sketch.add_constraint(CircleRadiusConstraint::new(circle, Length::meters(0.5)));
    }

    let solution = sketch.solve_and_extract().expect("Should solve");

    let exporter = SVGExporter::new();
    let svg = exporter.export(&sketch, &solution).expect("Should export");

    // Should have exactly 5 circle elements
    let circle_count = svg.matches("<circle").count();
    assert_eq!(circle_count, 5, "Should have exactly 5 circles");
}

#[test]
fn test_export_preserves_line_count() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Create exactly 7 lines
    for i in 0..7 {
        let p1 = sketch.add_point(Some(format!("p{}a", i)));
        let p2 = sketch.add_point(Some(format!("p{}b", i)));

        sketch.add_constraint(FixedPositionConstraint::new(
            p1,
            Length::meters(i as f64),
            Length::meters(0.0),
        ));
        sketch.add_constraint(FixedPositionConstraint::new(
            p2,
            Length::meters(i as f64 + 0.5),
            Length::meters(0.5),
        ));

        sketch.add_line(p1, p2, Some(format!("line{}", i)));
    }

    let solution = sketch.solve_and_extract().expect("Should solve");

    let exporter = SVGExporter::new();
    let svg = exporter.export(&sketch, &solution).expect("Should export");

    // Should have exactly 7 line elements
    let line_count = svg.matches("<line").count();
    assert_eq!(line_count, 7, "Should have exactly 7 lines");
}
