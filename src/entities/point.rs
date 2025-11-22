//! Point2D entity implementation
//!
//! Provides Point2D structure with Z3 integration for constraint-based 2D CAD modeling.

use generational_arena::Index;
use z3::{Context, ast::Real};

/// Strongly-typed identifier for Point2D entities using generational arena
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PointId(pub Index);

impl From<Index> for PointId {
    fn from(index: Index) -> Self {
        PointId(index)
    }
}

impl From<PointId> for Index {
    fn from(id: PointId) -> Self {
        id.0
    }
}

/// 2D point with x, y coordinates represented as Z3 Real variables
///
/// Point2D provides the foundation for geometric constraint modeling by
/// representing point coordinates as Z3 symbolic variables rather than
/// concrete values.
#[derive(Debug)]
pub struct Point2D<'ctx> {
    /// Unique identifier for this point
    pub id: PointId,
    /// X coordinate as Z3 Real variable
    pub x: Real<'ctx>,
    /// Y coordinate as Z3 Real variable  
    pub y: Real<'ctx>,
    /// Optional name for debugging and display
    pub name: Option<String>,
}

impl<'ctx> Point2D<'ctx> {
    /// Create a new Point2D with Z3 Real variables for coordinates
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this point
    /// * `ctx` - Z3 context for creating symbolic variables
    /// * `name` - Optional name for debugging (affects Z3 variable names)
    ///
    /// # Example
    /// ```
    /// use z3::{Config, Context};
    /// use generational_arena::Index;
    /// use textcad::entities::{Point2D, PointId};
    ///
    /// let cfg = Config::new();
    /// let ctx = Context::new(&cfg);
    /// let id = PointId::from(Index::from_raw_parts(0, 0));
    /// let point = Point2D::new(id, &ctx, Some("p1".to_string()));
    /// ```
    pub fn new(id: PointId, ctx: &'ctx Context, name: Option<String>) -> Self {
        let base_name = name.as_deref().unwrap_or("p");
        let x = Real::new_const(ctx, format!("{}_x", base_name));
        let y = Real::new_const(ctx, format!("{}_y", base_name));

        Self { id, x, y, name }
    }

    /// Get the point's name, or a default if none was specified
    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| format!("Point{:?}", self.id.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z3::{Config, Context};

    #[test]
    fn test_point_id_from_index() {
        let index = Index::from_raw_parts(42, 7);
        let point_id = PointId::from(index);
        assert_eq!(point_id.0, index);

        let back_to_index: Index = point_id.into();
        assert_eq!(back_to_index, index);
    }

    #[test]
    fn test_point_creation_with_name() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let id = PointId::from(Index::from_raw_parts(0, 0));

        let point = Point2D::new(id, &ctx, Some("test_point".to_string()));

        assert_eq!(point.id, id);
        assert_eq!(point.name, Some("test_point".to_string()));
        assert_eq!(point.display_name(), "test_point");

        // Verify Z3 variables have correct names
        assert!(point.x.to_string().contains("test_point_x"));
        assert!(point.y.to_string().contains("test_point_y"));
    }

    #[test]
    fn test_point_creation_without_name() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let id = PointId::from(Index::from_raw_parts(1, 0));

        let point = Point2D::new(id, &ctx, None);

        assert_eq!(point.id, id);
        assert_eq!(point.name, None);
        assert!(point.display_name().starts_with("Point"));

        // Verify Z3 variables have default names
        assert!(point.x.to_string().contains("p_x"));
        assert!(point.y.to_string().contains("p_y"));
    }

    #[test]
    fn test_multiple_points_have_distinct_variables() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);

        let id1 = PointId::from(Index::from_raw_parts(0, 0));
        let id2 = PointId::from(Index::from_raw_parts(1, 0));

        let point1 = Point2D::new(id1, &ctx, Some("p1".to_string()));
        let point2 = Point2D::new(id2, &ctx, Some("p2".to_string()));

        assert_ne!(point1.id, point2.id);

        // Z3 variables should be distinct
        assert_ne!(point1.x.to_string(), point2.x.to_string());
        assert_ne!(point1.y.to_string(), point2.y.to_string());

        // Names should be different
        assert!(point1.x.to_string().contains("p1_x"));
        assert!(point2.x.to_string().contains("p2_x"));
    }

    #[test]
    fn test_point_id_ordering() {
        let id1 = PointId::from(Index::from_raw_parts(0, 0));
        let id2 = PointId::from(Index::from_raw_parts(1, 0));
        let id3 = PointId::from(Index::from_raw_parts(0, 1));

        // Test basic inequality
        assert_ne!(id1, id2);
        assert_ne!(id1, id3);
        assert_ne!(id2, id3);
    }

    #[test]
    fn test_point_ids_are_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PointId>();
    }
}
