use generational_arena::Index;

/// Strongly-typed identifier for Line entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineId(pub Index);

impl From<Index> for LineId {
    fn from(index: Index) -> Self {
        LineId(index)
    }
}

impl From<LineId> for Index {
    fn from(id: LineId) -> Self {
        id.0
    }
}

/// Strongly-typed identifier for Circle entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CircleId(pub Index);

impl From<Index> for CircleId {
    fn from(index: Index) -> Self {
        CircleId(index)
    }
}

impl From<CircleId> for Index {
    fn from(id: CircleId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_id_creation() {
        let idx1 = Index::from_raw_parts(0, 0);
        let idx2 = Index::from_raw_parts(1, 0);
        let id1 = LineId::from(idx1);
        let id2 = LineId::from(idx2);
        assert_ne!(id1, id2);

        // Test conversion back to index
        let back1: Index = id1.into();
        assert_eq!(back1, idx1);
    }

    #[test]
    fn test_circle_id_creation() {
        let idx1 = Index::from_raw_parts(0, 0);
        let idx2 = Index::from_raw_parts(1, 0);
        let id1 = CircleId::from(idx1);
        let id2 = CircleId::from(idx2);
        assert_ne!(id1, id2);

        // Test conversion back to index
        let back1: Index = id1.into();
        assert_eq!(back1, idx1);
    }

    #[test]
    fn test_ids_are_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<LineId>();
        assert_send_sync::<CircleId>();
    }
}
