//! Circle entity implementation
//!
//! Provides Circle structure with Z3 integration for constraint-based 2D CAD modeling.
//! Circles are composite entities defined by a center PointId and a radius as a Z3 symbolic variable.

use crate::entities::PointId;
use crate::entity::CircleId;
use z3::{Context, ast::Real};

/// 2D circle defined by a center point and radius
///
/// Circle provides a composite geometric entity that references a Point2D center
/// and stores its radius as a Z3 symbolic variable. This design supports the
/// constraint-based modeling approach where relationships between entities are
/// more important than concrete parameter values.
#[derive(Debug)]
pub struct Circle<'ctx> {
    /// Unique identifier for this circle
    pub id: CircleId,
    /// Center point of the circle
    pub center: PointId,
    /// Radius as Z3 Real variable  
    pub radius: Real<'ctx>,
    /// Optional name for debugging and display
    pub name: Option<String>,
}

impl<'ctx> Circle<'ctx> {
    /// Create a new Circle with a center point and symbolic radius
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this circle
    /// * `center` - PointId of the center point
    /// * `ctx` - Z3 context for creating symbolic variables
    /// * `name` - Optional name for debugging (affects Z3 variable names)
    ///
    /// # Example
    /// ```
    /// use z3::{Config, Context};
    /// use generational_arena::Index;
    /// use textcad::entities::{Circle, PointId};
    /// use textcad::entity::CircleId;
    ///
    /// let cfg = Config::new();
    /// let ctx = Context::new(&cfg);
    /// let circle_id = CircleId::from(Index::from_raw_parts(0, 0));
    /// let center_id = PointId::from(Index::from_raw_parts(0, 0));
    /// let circle = Circle::new(circle_id, center_id, &ctx, Some("c1".to_string()));
    /// ```
    pub fn new(id: CircleId, center: PointId, ctx: &'ctx Context, name: Option<String>) -> Self {
        let base_name = name.as_deref().unwrap_or("c");
        let radius = Real::new_const(ctx, format!("{}_radius", base_name));

        Self {
            id,
            center,
            radius,
            name,
        }
    }

    /// Get the circle's name, or a default if none was specified
    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| format!("Circle{:?}", self.id.0))
    }

    /// Get the center point ID
    pub fn center_point(&self) -> PointId {
        self.center
    }

    // Entity-as-constraint-factory methods will be added here when Circle constraints are implemented
    // These methods will return constraint objects that can be applied to the sketch
}

#[cfg(test)]
mod tests {
    use super::*;
    use generational_arena::Index;
    use z3::{Config, Context};

    #[test]
    fn test_circle_creation_with_name() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let circle_id = CircleId::from(Index::from_raw_parts(0, 0));
        let center_id = PointId::from(Index::from_raw_parts(0, 0));

        let circle = Circle::new(circle_id, center_id, &ctx, Some("test_circle".to_string()));

        assert_eq!(circle.id, circle_id);
        assert_eq!(circle.center, center_id);
        assert_eq!(circle.name, Some("test_circle".to_string()));
        assert_eq!(circle.display_name(), "test_circle");

        // Verify Z3 variable has correct name
        assert!(circle.radius.to_string().contains("test_circle_radius"));
    }

    #[test]
    fn test_circle_creation_without_name() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let circle_id = CircleId::from(Index::from_raw_parts(1, 0));
        let center_id = PointId::from(Index::from_raw_parts(2, 0));

        let circle = Circle::new(circle_id, center_id, &ctx, None);

        assert_eq!(circle.id, circle_id);
        assert_eq!(circle.center, center_id);
        assert_eq!(circle.name, None);
        assert!(circle.display_name().starts_with("Circle"));

        // Verify Z3 variable has default name
        assert!(circle.radius.to_string().contains("c_radius"));
    }

    #[test]
    fn test_circle_center_point() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let circle_id = CircleId::from(Index::from_raw_parts(0, 0));
        let center_id = PointId::from(Index::from_raw_parts(5, 0));

        let circle = Circle::new(circle_id, center_id, &ctx, None);

        assert_eq!(circle.center_point(), center_id);
    }

    #[test]
    fn test_multiple_circles_have_distinct_variables() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);

        let id1 = CircleId::from(Index::from_raw_parts(0, 0));
        let id2 = CircleId::from(Index::from_raw_parts(1, 0));
        let center_id = PointId::from(Index::from_raw_parts(0, 0));

        let circle1 = Circle::new(id1, center_id, &ctx, Some("c1".to_string()));
        let circle2 = Circle::new(id2, center_id, &ctx, Some("c2".to_string()));

        assert_ne!(circle1.id, circle2.id);

        // Z3 variables should be distinct
        assert_ne!(circle1.radius.to_string(), circle2.radius.to_string());

        // Names should be different
        assert!(circle1.radius.to_string().contains("c1_radius"));
        assert!(circle2.radius.to_string().contains("c2_radius"));
    }

    #[test]
    fn test_circle_id_ordering() {
        let id1 = CircleId::from(Index::from_raw_parts(0, 0));
        let id2 = CircleId::from(Index::from_raw_parts(1, 0));
        let id3 = CircleId::from(Index::from_raw_parts(0, 1));

        // Test basic inequality
        assert_ne!(id1, id2);
        assert_ne!(id1, id3);
        assert_ne!(id2, id3);
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
    fn test_circle_debug_representation() {
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
    }

    #[test]
    fn test_circles_are_send_sync() {
        // This won't work because Circle contains Z3 variables which are not Send/Sync
        // but we can test the CircleId
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CircleId>();

        // Note: Circle<'ctx> itself cannot be Send + Sync due to Z3 Real variables
        // This is expected and follows the same pattern as Point2D
    }
}
