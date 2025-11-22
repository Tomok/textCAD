/// Strongly-typed identifier for Point2D entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PointId(pub u32);

/// Strongly-typed identifier for Line entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineId(pub u32);

/// Strongly-typed identifier for Circle entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CircleId(pub u32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_id_creation() {
        let id1 = PointId(0);
        let id2 = PointId(1);
        assert_ne!(id1, id2);
        assert!(id1 < id2);
    }

    #[test]
    fn test_line_id_creation() {
        let id1 = LineId(0);
        let id2 = LineId(1);
        assert_ne!(id1, id2);
        assert!(id1 < id2);
    }

    #[test]
    fn test_circle_id_creation() {
        let id1 = CircleId(0);
        let id2 = CircleId(1);
        assert_ne!(id1, id2);
        assert!(id1 < id2);
    }

    #[test]
    fn test_ids_are_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PointId>();
        assert_send_sync::<LineId>();
        assert_send_sync::<CircleId>();
    }
}