//! Constraint implementations for geometric relationships
//!
//! This module contains specific constraint types that can be applied to
//! geometric entities to define their relationships and properties.

pub mod basic;

#[cfg(test)]
mod property_tests;

// Re-export commonly used constraint types
pub use basic::{CoincidentPointsConstraint, FixedPositionConstraint};
