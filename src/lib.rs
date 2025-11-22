//! TextCAD: A constraint-based 2D CAD system built in Rust using Z3.
//!
//! TextCAD provides a declarative API for geometric constraint specification
//! while leveraging Z3 as the constraint solver for determining concrete
//! geometric configurations.

pub mod constraint;
pub mod entity;
pub mod error;
pub mod units;

// Re-export commonly used types
pub use constraint::{Constraint, ConstraintFactory, SketchQuery};
pub use entity::{PointId, LineId, CircleId};
pub use error::{TextCadError, Result, SolverResult};
pub use units::{Length, Angle, Area};
