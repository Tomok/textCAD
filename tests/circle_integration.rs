//! Integration tests for Circle entity
//!
//! Tests complete workflows including circle creation, arena operations,
//! and basic sketch integration for Circle entities.

use generational_arena::Index;
use textcad::entities::{Circle, PointId};
use textcad::entity::CircleId;
use textcad::sketch::Sketch;
use z3::{Config, Context};

#[test]
fn test_circle_creation_and_basic_properties() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let _sketch = Sketch::new(&ctx);

    // Create a circle entity directly using Z3 context
    let circle_id = CircleId::from(Index::from_raw_parts(0, 0));
    let center_id = PointId::from(Index::from_raw_parts(0, 0));

    let circle = Circle::new(circle_id, center_id, &ctx, Some("test_circle".to_string()));

    // Test basic properties
    assert_eq!(circle.id, circle_id);
    assert_eq!(circle.center_point(), center_id);
    assert_eq!(circle.display_name(), "test_circle");

    // Test that Z3 variables are properly created
    assert!(circle.radius.to_string().contains("test_circle_radius"));
}

#[test]
fn test_circle_with_sketch_integration() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Add a center point to the sketch
    let center = sketch.add_point(Some("center".to_string()));

    // Add a circle to the sketch via the sketch API
    let circle_id = sketch.add_circle(center, Some("circle1".to_string()));

    // Get the circle back from the sketch
    let circle = sketch.get_circle(circle_id).unwrap();

    assert_eq!(circle.center, center);
    assert_eq!(circle.display_name(), "circle1");
    assert!(circle.radius.to_string().contains("circle1_radius"));
}

#[test]
fn test_multiple_circles_in_sketch() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let mut sketch = Sketch::new(&ctx);

    // Add center points
    let center1 = sketch.add_point(Some("center1".to_string()));
    let center2 = sketch.add_point(Some("center2".to_string()));

    // Add circles
    let circle1 = sketch.add_circle(center1, Some("circle1".to_string()));
    let circle2 = sketch.add_circle(center2, Some("circle2".to_string()));

    // Verify circles are different and properly stored
    assert_ne!(circle1, circle2);

    let c1 = sketch.get_circle(circle1).unwrap();
    let c2 = sketch.get_circle(circle2).unwrap();

    assert_eq!(c1.center, center1);
    assert_eq!(c2.center, center2);
    assert_ne!(c1.display_name(), c2.display_name());
    assert_ne!(c1.radius.to_string(), c2.radius.to_string());
}

#[test]
fn test_circle_display_names() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let center_id = PointId::from(Index::from_raw_parts(0, 0));

    // Circle with custom name
    let circle_id1 = CircleId::from(Index::from_raw_parts(0, 0));
    let named_circle = Circle::new(circle_id1, center_id, &ctx, Some("MyCircle".to_string()));
    assert_eq!(named_circle.display_name(), "MyCircle");

    // Circle without name (should get default)
    let circle_id2 = CircleId::from(Index::from_raw_parts(1, 0));
    let unnamed_circle = Circle::new(circle_id2, center_id, &ctx, None);
    assert!(unnamed_circle.display_name().starts_with("Circle"));
    assert!(unnamed_circle.display_name().contains("1")); // Should contain the index

    // Different circles should have different default names
    let circle_id3 = CircleId::from(Index::from_raw_parts(5, 3));
    let another_circle = Circle::new(circle_id3, center_id, &ctx, None);
    assert_ne!(unnamed_circle.display_name(), another_circle.display_name());
}

#[test]
fn test_circle_id_uniqueness() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let center_id = PointId::from(Index::from_raw_parts(0, 0));

    // Create circles with different IDs
    let circle_ids = [
        CircleId::from(Index::from_raw_parts(0, 0)),
        CircleId::from(Index::from_raw_parts(1, 0)),
        CircleId::from(Index::from_raw_parts(0, 1)),
        CircleId::from(Index::from_raw_parts(42, 7)),
    ];

    let circles: Vec<Circle> = circle_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| Circle::new(id, center_id, &ctx, Some(format!("circle_{}", i))))
        .collect();

    // All circles should have unique IDs
    for i in 0..circles.len() {
        for j in (i + 1)..circles.len() {
            assert_ne!(
                circles[i].id, circles[j].id,
                "Circles {} and {} should have different IDs",
                i, j
            );
            assert_ne!(
                circles[i].display_name(),
                circles[j].display_name(),
                "Circles {} and {} should have different names",
                i,
                j
            );
        }
    }
}

#[test]
fn test_circle_z3_variable_uniqueness() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let center_id = PointId::from(Index::from_raw_parts(0, 0));

    // Create multiple circles and verify their Z3 variables are unique
    let circles: Vec<_> = (0..5)
        .map(|i| {
            let circle_id = CircleId::from(Index::from_raw_parts(i, 0));
            Circle::new(circle_id, center_id, &ctx, Some(format!("c{}", i)))
        })
        .collect();

    // All radius variables should be distinct
    for i in 0..circles.len() {
        for j in (i + 1)..circles.len() {
            assert_ne!(
                circles[i].radius.to_string(),
                circles[j].radius.to_string(),
                "Circle {} and {} should have different radius variables",
                i,
                j
            );
        }
    }
}

#[test]
fn test_circle_center_point_references() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let circle_id = CircleId::from(Index::from_raw_parts(0, 0));

    // Test circles with different center points
    let center_ids = [
        PointId::from(Index::from_raw_parts(0, 0)),
        PointId::from(Index::from_raw_parts(1, 0)),
        PointId::from(Index::from_raw_parts(5, 3)),
        PointId::from(Index::from_raw_parts(100, 50)),
    ];

    for (i, &center_id) in center_ids.iter().enumerate() {
        let circle = Circle::new(circle_id, center_id, &ctx, Some(format!("circle_{}", i)));

        assert_eq!(circle.center_point(), center_id);
        assert_eq!(circle.center, center_id);

        // Radius variable should be the same regardless of center
        assert!(
            circle
                .radius
                .to_string()
                .contains(&format!("circle_{}_radius", i))
        );
    }
}

#[test]
fn test_circle_debug_representation_quality() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let circle_id = CircleId::from(Index::from_raw_parts(5, 3));
    let center_id = PointId::from(Index::from_raw_parts(10, 2));

    let circle = Circle::new(circle_id, center_id, &ctx, Some("debug_test".to_string()));
    let debug_output = format!("{:?}", circle);

    // Should contain key information for debugging
    assert!(debug_output.contains("Circle"));
    assert!(debug_output.contains("id"));
    assert!(debug_output.contains("center"));
    assert!(debug_output.contains("radius"));
    assert!(debug_output.contains("name"));
    assert!(debug_output.contains("debug_test"));

    // Test unnamed circle debug output
    let unnamed_circle = Circle::new(circle_id, center_id, &ctx, None);
    let unnamed_debug = format!("{:?}", unnamed_circle);
    assert!(unnamed_debug.contains("None"));
}

// Future integration tests will be added here when Circle constraints are implemented
// These would test:
// - Circle constraint application and solving
// - Circle radius constraints
// - Point-on-circle constraints
// - Circle-circle tangency
// - Circle-line tangency
// - Solution extraction for circles
