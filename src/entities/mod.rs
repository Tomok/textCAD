//! Geometric entity implementations
//!
//! This module contains implementations of geometric entities (Point2D, Line, Circle)
//! that integrate with Z3 for constraint-based modeling.

pub mod circle;
pub mod line;
pub mod point;

pub use circle::Circle;
pub use line::Line;
pub use point::{Point2D, PointId};
