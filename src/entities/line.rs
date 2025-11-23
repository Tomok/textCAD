//! Line entity implementation
//!
//! Provides Line structure with Z3 integration for constraint-based 2D CAD modeling.
//! Lines are composite entities defined by two endpoint PointIds.

use crate::constraints::{
    LineLengthConstraint, ParallelLinesConstraint, PerpendicularLinesConstraint,
};
use crate::entities::PointId;
use crate::entity::LineId;
use crate::units::Length;

/// 2D line defined by two endpoint points
///
/// Line provides a composite geometric entity that references two Point2D endpoints
/// rather than storing coordinates directly. This design supports the constraint-based
/// modeling approach where relationships between entities are more important than
/// concrete coordinate values.
#[derive(Debug, Clone)]
pub struct Line {
    /// Unique identifier for this line
    pub id: LineId,
    /// Starting point of the line
    pub start: PointId,
    /// Ending point of the line
    pub end: PointId,
    /// Optional name for debugging and display
    pub name: Option<String>,
}

impl Line {
    /// Create a new Line connecting two points
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this line
    /// * `start` - PointId of the starting point
    /// * `end` - PointId of the ending point
    /// * `name` - Optional name for debugging and display
    ///
    /// # Example
    /// ```
    /// use textcad::entities::{Line, PointId};
    /// use textcad::entity::LineId;
    /// use generational_arena::Index;
    ///
    /// let line_id = LineId::from(Index::from_raw_parts(0, 0));
    /// let start_id = PointId::from(Index::from_raw_parts(0, 0));
    /// let end_id = PointId::from(Index::from_raw_parts(1, 0));
    ///
    /// let line = Line::new(line_id, start_id, end_id, Some("line1".to_string()));
    /// assert_eq!(line.start, start_id);
    /// assert_eq!(line.end, end_id);
    /// ```
    pub fn new(id: LineId, start: PointId, end: PointId, name: Option<String>) -> Self {
        Self {
            id,
            start,
            end,
            name,
        }
    }

    /// Get the line's name, or a default if none was specified
    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| format!("Line{:?}", self.id.0))
    }

    /// Get both endpoint IDs as a tuple for convenience
    pub fn endpoints(&self) -> (PointId, PointId) {
        (self.start, self.end)
    }

    /// Check if this line contains a specific point as an endpoint
    pub fn contains_point(&self, point_id: PointId) -> bool {
        self.start == point_id || self.end == point_id
    }

    // Entity-as-constraint-factory methods
    // These methods return constraint objects that can be applied to the sketch

    /// Create a constraint that fixes this line to a specific length
    ///
    /// # Arguments
    /// * `length` - The target length for this line
    ///
    /// # Returns
    /// A LineLengthConstraint that can be added to the sketch
    ///
    /// # Example
    /// ```
    /// use textcad::{Line, LineId, PointId, LineLengthConstraint, Length};
    /// use generational_arena::Index;
    ///
    /// let line_id = LineId::from(Index::from_raw_parts(0, 0));
    /// let start_id = PointId::from(Index::from_raw_parts(0, 0));
    /// let end_id = PointId::from(Index::from_raw_parts(1, 0));
    /// let line = Line::new(line_id, start_id, end_id, None);
    ///
    /// let constraint = line.length_equals(Length::meters(5.0));
    /// // This constraint can now be added to a sketch
    /// ```
    pub fn length_equals(&self, length: Length) -> LineLengthConstraint {
        LineLengthConstraint::new(self.id, length)
    }

    /// Create a constraint that forces this line to be parallel to another line
    ///
    /// # Arguments
    /// * `other` - The other line to be parallel to
    ///
    /// # Returns
    /// A ParallelLinesConstraint that can be added to the sketch
    ///
    /// # Example
    /// ```
    /// use textcad::{Line, LineId, PointId, ParallelLinesConstraint};
    /// use generational_arena::Index;
    ///
    /// let line1_id = LineId::from(Index::from_raw_parts(0, 0));
    /// let line2_id = LineId::from(Index::from_raw_parts(1, 0));
    /// let start_id = PointId::from(Index::from_raw_parts(0, 0));
    /// let end_id = PointId::from(Index::from_raw_parts(1, 0));
    ///
    /// let line1 = Line::new(line1_id, start_id, end_id, None);
    /// let line2 = Line::new(line2_id, start_id, end_id, None);
    ///
    /// let constraint = line1.parallel_to(&line2);
    /// // This constraint can now be added to a sketch
    /// ```
    pub fn parallel_to(&self, other: &Line) -> ParallelLinesConstraint {
        ParallelLinesConstraint::new(self.id, other.id)
    }

    /// Create a constraint that forces this line to be perpendicular to another line
    ///
    /// # Arguments
    /// * `other` - The other line to be perpendicular to
    ///
    /// # Returns
    /// A PerpendicularLinesConstraint that can be added to the sketch
    ///
    /// # Example
    /// ```
    /// use textcad::{Line, LineId, PointId, PerpendicularLinesConstraint};
    /// use generational_arena::Index;
    ///
    /// let line1_id = LineId::from(Index::from_raw_parts(0, 0));
    /// let line2_id = LineId::from(Index::from_raw_parts(1, 0));
    /// let start_id = PointId::from(Index::from_raw_parts(0, 0));
    /// let end_id = PointId::from(Index::from_raw_parts(1, 0));
    ///
    /// let line1 = Line::new(line1_id, start_id, end_id, None);
    /// let line2 = Line::new(line2_id, start_id, end_id, None);
    ///
    /// let constraint = line1.perpendicular_to(&line2);
    /// // This constraint can now be added to a sketch
    /// ```
    pub fn perpendicular_to(&self, other: &Line) -> PerpendicularLinesConstraint {
        PerpendicularLinesConstraint::new(self.id, other.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constraint::Constraint;
    use generational_arena::Index;

    #[test]
    fn test_line_creation_with_name() {
        let line_id = LineId::from(Index::from_raw_parts(0, 0));
        let start_id = PointId::from(Index::from_raw_parts(0, 0));
        let end_id = PointId::from(Index::from_raw_parts(1, 0));

        let line = Line::new(line_id, start_id, end_id, Some("test_line".to_string()));

        assert_eq!(line.id, line_id);
        assert_eq!(line.start, start_id);
        assert_eq!(line.end, end_id);
        assert_eq!(line.name, Some("test_line".to_string()));
        assert_eq!(line.display_name(), "test_line");
    }

    #[test]
    fn test_line_creation_without_name() {
        let line_id = LineId::from(Index::from_raw_parts(1, 0));
        let start_id = PointId::from(Index::from_raw_parts(2, 0));
        let end_id = PointId::from(Index::from_raw_parts(3, 0));

        let line = Line::new(line_id, start_id, end_id, None);

        assert_eq!(line.id, line_id);
        assert_eq!(line.start, start_id);
        assert_eq!(line.end, end_id);
        assert_eq!(line.name, None);
        assert!(line.display_name().starts_with("Line"));
    }

    #[test]
    fn test_line_endpoints() {
        let line_id = LineId::from(Index::from_raw_parts(0, 0));
        let start_id = PointId::from(Index::from_raw_parts(5, 0));
        let end_id = PointId::from(Index::from_raw_parts(10, 0));

        let line = Line::new(line_id, start_id, end_id, None);
        let (start, end) = line.endpoints();

        assert_eq!(start, start_id);
        assert_eq!(end, end_id);
    }

    #[test]
    fn test_line_contains_point() {
        let line_id = LineId::from(Index::from_raw_parts(0, 0));
        let start_id = PointId::from(Index::from_raw_parts(1, 0));
        let end_id = PointId::from(Index::from_raw_parts(2, 0));
        let other_id = PointId::from(Index::from_raw_parts(3, 0));

        let line = Line::new(line_id, start_id, end_id, Some("test".to_string()));

        assert!(line.contains_point(start_id));
        assert!(line.contains_point(end_id));
        assert!(!line.contains_point(other_id));
    }

    #[test]
    fn test_line_clone() {
        let line_id = LineId::from(Index::from_raw_parts(0, 0));
        let start_id = PointId::from(Index::from_raw_parts(1, 0));
        let end_id = PointId::from(Index::from_raw_parts(2, 0));

        let line1 = Line::new(line_id, start_id, end_id, Some("original".to_string()));
        let line2 = line1.clone();

        assert_eq!(line1.id, line2.id);
        assert_eq!(line1.start, line2.start);
        assert_eq!(line1.end, line2.end);
        assert_eq!(line1.name, line2.name);
    }

    #[test]
    fn test_different_lines_have_different_ids() {
        let line_id1 = LineId::from(Index::from_raw_parts(0, 0));
        let line_id2 = LineId::from(Index::from_raw_parts(1, 0));
        let start_id = PointId::from(Index::from_raw_parts(0, 0));
        let end_id = PointId::from(Index::from_raw_parts(1, 0));

        let line1 = Line::new(line_id1, start_id, end_id, Some("line1".to_string()));
        let line2 = Line::new(line_id2, start_id, end_id, Some("line2".to_string()));

        assert_ne!(line1.id, line2.id);
        assert_ne!(line1.display_name(), line2.display_name());
    }

    // Tests for entity-as-constraint-factory methods
    #[test]
    fn test_line_length_equals_constraint() {
        let line_id = LineId::from(Index::from_raw_parts(0, 0));
        let start_id = PointId::from(Index::from_raw_parts(0, 0));
        let end_id = PointId::from(Index::from_raw_parts(1, 0));
        let line = Line::new(line_id, start_id, end_id, Some("test_line".to_string()));

        let target_length = Length::meters(5.0);
        let constraint = line.length_equals(target_length);

        assert_eq!(constraint.line, line_id);
        assert_eq!(constraint.length, target_length);
        assert!(constraint.description().contains("5.000m"));
    }

    #[test]
    fn test_line_length_constraint_with_different_units() {
        let line_id = LineId::from(Index::from_raw_parts(0, 0));
        let start_id = PointId::from(Index::from_raw_parts(0, 0));
        let end_id = PointId::from(Index::from_raw_parts(1, 0));
        let line = Line::new(line_id, start_id, end_id, None);

        // Test with millimeters
        let constraint_mm = line.length_equals(Length::millimeters(1000.0));
        assert_eq!(constraint_mm.length.to_meters(), 1.0);

        // Test with centimeters
        let constraint_cm = line.length_equals(Length::centimeters(100.0));
        assert_eq!(constraint_cm.length.to_meters(), 1.0);
    }

    #[test]
    fn test_line_parallel_to_constraint() {
        let line1_id = LineId::from(Index::from_raw_parts(0, 0));
        let line2_id = LineId::from(Index::from_raw_parts(1, 0));
        let start_id = PointId::from(Index::from_raw_parts(0, 0));
        let end_id = PointId::from(Index::from_raw_parts(1, 0));

        let line1 = Line::new(line1_id, start_id, end_id, Some("line1".to_string()));
        let line2 = Line::new(line2_id, start_id, end_id, Some("line2".to_string()));

        let constraint = line1.parallel_to(&line2);

        assert_eq!(constraint.line1, line1_id);
        assert_eq!(constraint.line2, line2_id);
        assert!(constraint.description().contains("parallel"));
    }

    #[test]
    fn test_line_perpendicular_to_constraint() {
        let line1_id = LineId::from(Index::from_raw_parts(0, 0));
        let line2_id = LineId::from(Index::from_raw_parts(1, 0));
        let start_id = PointId::from(Index::from_raw_parts(0, 0));
        let end_id = PointId::from(Index::from_raw_parts(1, 0));

        let line1 = Line::new(line1_id, start_id, end_id, Some("line1".to_string()));
        let line2 = Line::new(line2_id, start_id, end_id, Some("line2".to_string()));

        let constraint = line1.perpendicular_to(&line2);

        assert_eq!(constraint.line1, line1_id);
        assert_eq!(constraint.line2, line2_id);
        assert!(constraint.description().contains("perpendicular"));
    }

    #[test]
    fn test_line_constraint_factories_with_different_lines() {
        let line1_id = LineId::from(Index::from_raw_parts(0, 0));
        let line2_id = LineId::from(Index::from_raw_parts(1, 0));
        let line3_id = LineId::from(Index::from_raw_parts(2, 0));
        let start_id = PointId::from(Index::from_raw_parts(0, 0));
        let end_id = PointId::from(Index::from_raw_parts(1, 0));

        let line1 = Line::new(line1_id, start_id, end_id, None);
        let line2 = Line::new(line2_id, start_id, end_id, None);
        let line3 = Line::new(line3_id, start_id, end_id, None);

        // Test that different line combinations produce different constraints
        let parallel_1_2 = line1.parallel_to(&line2);
        let parallel_1_3 = line1.parallel_to(&line3);
        let perp_1_2 = line1.perpendicular_to(&line2);

        assert_ne!(parallel_1_2.line2, parallel_1_3.line2);
        assert_eq!(parallel_1_2.line1, perp_1_2.line1);
        assert_eq!(parallel_1_2.line2, perp_1_2.line2);
    }
}
