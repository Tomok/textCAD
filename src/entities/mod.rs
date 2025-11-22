//! Geometric entity implementations
//!
//! This module contains implementations of geometric entities (Point2D, Line, Circle)
//! that integrate with Z3 for constraint-based modeling.

pub mod point;

pub use point::{Point2D, PointId};
