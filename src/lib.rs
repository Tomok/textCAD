//! TextCAD: A constraint-based 2D CAD system built in Rust using Z3.
//!
//! TextCAD provides a declarative API for geometric constraint specification
//! while leveraging Z3 as the constraint solver for determining concrete
//! geometric configurations.

pub mod constraint;
pub mod constraints;
pub mod entities;
pub mod entity;
pub mod error;
pub mod sketch;
pub mod solution;
pub mod solver;
pub mod units;

// Re-export commonly used types
pub use constraint::{Constraint, ConstraintFactory, SketchQuery};
pub use constraints::{
    CoincidentPointsConstraint, FixedPositionConstraint, LineLengthConstraint,
    ParallelLinesConstraint, PerpendicularLinesConstraint,
};
pub use entities::{Line, Point2D, PointId};
pub use entity::{CircleId, LineId};
pub use error::{Result, SolverResult, TextCadError};
pub use sketch::Sketch;
pub use solution::{CircleParameters, LineParameters, Solution};
pub use units::{Angle, Area, Length};
