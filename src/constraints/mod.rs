//! Constraint implementations for geometric relationships
//!
//! This module contains specific constraint types that can be applied to
//! geometric entities to define their relationships and properties.

pub mod basic;
pub mod circle;
pub mod line;
pub mod parametric;

#[cfg(test)]
mod property_tests;

// Re-export commonly used constraint types
pub use basic::{CoincidentPointsConstraint, FixedPositionConstraint};
pub use circle::CircleRadiusConstraint;
pub use line::{LineLengthConstraint, ParallelLinesConstraint, PerpendicularLinesConstraint};
pub use parametric::PointOnLineConstraint;
