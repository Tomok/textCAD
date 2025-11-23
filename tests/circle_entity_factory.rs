//! Tests for Circle entity factory methods
//!
//! Tests the entity-as-constraint-factory pattern for Circle entities.
//! These tests will be activated when Circle constraints are implemented.

use generational_arena::Index;
use textcad::entities::{Circle, PointId};
use textcad::entity::CircleId;
use z3::{Config, Context};

#[test]
fn test_circle_entity_creation_patterns() {
    // Test that Circle follows the same entity creation patterns as Point2D and Line
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let circle_id = CircleId::from(Index::from_raw_parts(0, 0));
    let center_id = PointId::from(Index::from_raw_parts(0, 0));

    let circle = Circle::new(circle_id, center_id, &ctx, Some("factory_test".to_string()));

    // Test that it follows the same patterns as other entities:
    // 1. Has an ID
    assert_eq!(circle.id, circle_id);

    // 2. Has a display name method
    assert_eq!(circle.display_name(), "factory_test");

    // Note: Circle doesn't implement Clone because it contains Z3 variables
    // This is consistent with Point2D which also doesn't clone Z3 variables

    // 3. Has consistent Debug representation
    let debug_str = format!("{:?}", circle);
    assert!(debug_str.contains("Circle"));
    assert!(debug_str.contains("center"));
    assert!(debug_str.contains("radius"));
}

#[test]
fn test_circle_follows_entity_id_patterns() {
    // Test that CircleId follows the same patterns as PointId and LineId

    // Test conversion from Index
    let index = Index::from_raw_parts(42, 7);
    let circle_id = CircleId::from(index);
    assert_eq!(circle_id.0, index);

    // Test conversion back to Index
    let back_to_index: Index = circle_id.into();
    assert_eq!(back_to_index, index);

    // Test that different indices create different IDs
    let id1 = CircleId::from(Index::from_raw_parts(0, 0));
    let id2 = CircleId::from(Index::from_raw_parts(1, 0));
    let id3 = CircleId::from(Index::from_raw_parts(0, 1));

    assert_ne!(id1, id2);
    assert_ne!(id1, id3);
    assert_ne!(id2, id3);

    // Test that IDs are orderable (for use in collections)
    let mut ids = vec![id3, id1, id2];
    ids.sort();
    assert_eq!(ids, vec![id1, id3, id2]); // Specific order depends on Index ordering
}

#[test]
fn test_circle_entity_consistency_with_existing_patterns() {
    // Ensure Circle entity follows the same patterns as Point2D and Line
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let center_id = PointId::from(Index::from_raw_parts(0, 0));

    // Test creation with and without names (like Point2D and Line)
    let named_circle = Circle::new(
        CircleId::from(Index::from_raw_parts(0, 0)),
        center_id,
        &ctx,
        Some("named".to_string()),
    );

    let unnamed_circle = Circle::new(
        CircleId::from(Index::from_raw_parts(1, 0)),
        center_id,
        &ctx,
        None,
    );

    assert_eq!(named_circle.display_name(), "named");
    assert!(unnamed_circle.display_name().starts_with("Circle"));
    assert!(unnamed_circle.display_name() != "Circle"); // Should include ID info

    // Test that different circles have different display names when unnamed
    let another_circle = Circle::new(
        CircleId::from(Index::from_raw_parts(2, 0)),
        center_id,
        &ctx,
        None,
    );

    assert_ne!(unnamed_circle.display_name(), another_circle.display_name());
}

// Future tests for constraint factory methods
// These will be uncommented when Circle constraints are implemented:

/*
#[test]
fn test_circle_radius_constraint_factory() {
    // When CircleRadiusConstraint is implemented, test:
    // let constraint = circle.radius_equals(Length::meters(10.0));
}

#[test]
fn test_circle_tangent_constraint_factory() {
    // When tangent constraints are implemented, test:
    // let constraint = circle.tangent_to(&other_circle);
    // let constraint = circle.tangent_to_line(&line);
}

#[test]
fn test_circle_concentric_constraint_factory() {
    // When concentric constraints are implemented, test:
    // let constraint = circle.concentric_with(&other_circle);
}

#[test]
fn test_point_on_circle_constraint_factory() {
    // When point-on-circle constraints are implemented, test:
    // let constraint = circle.contains_point(point_id);
}
*/

#[test]
fn test_circle_z3_integration_consistency() {
    // Test that Circle uses Z3 variables consistently with the rest of the project
    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    let circle_id = CircleId::from(Index::from_raw_parts(0, 0));
    let center_id = PointId::from(Index::from_raw_parts(0, 0));

    let circle = Circle::new(circle_id, center_id, &ctx, Some("test_circle".to_string()));

    // The radius should be a Z3 Real variable
    let radius_str = circle.radius.to_string();
    assert!(radius_str.contains("test_circle_radius"));

    // Different circles should have different radius variables
    let circle2 = Circle::new(
        CircleId::from(Index::from_raw_parts(1, 0)),
        center_id,
        &ctx,
        Some("test_circle2".to_string()),
    );

    assert_ne!(circle.radius.to_string(), circle2.radius.to_string());
}

#[test]
fn test_circle_debug_representation_quality() {
    // Test that Circle has a useful Debug representation (important for debugging constraints)
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

#[test]
fn test_circle_center_point_access() {
    // Test that center point access is consistent
    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    let center_id = PointId::from(Index::from_raw_parts(42, 7));
    let circle_id = CircleId::from(Index::from_raw_parts(0, 0));

    let circle = Circle::new(circle_id, center_id, &ctx, Some("center_test".to_string()));

    // Both methods should return the same center
    assert_eq!(circle.center_point(), center_id);
    assert_eq!(circle.center, center_id);
}
