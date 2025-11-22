use crate::error::Result;
use crate::entity::PointId;
use z3::ast::Real;

/// Trait for constraints that can be applied to a Z3 solver context.
/// Each constraint knows how to translate itself into Z3 assertions.
pub trait Constraint: Send + Sync + std::fmt::Debug {
    /// Apply this constraint to the solver by adding the necessary Z3 assertions.
    /// 
    /// # Arguments
    /// * `context` - The Z3 context for creating expressions
    /// * `solver` - The Z3 solver to add assertions to
    /// * `sketch` - Reference to the sketch containing entities
    fn apply(&self, context: &z3::Context, solver: &z3::Solver, sketch: &dyn SketchQuery) -> Result<()>;
    
    /// Get a human-readable description of this constraint for debugging
    fn description(&self) -> String;
}

/// Trait for querying sketch state during constraint application.
/// This allows constraints to access entity data without requiring
/// the full sketch type as a generic parameter.
pub trait SketchQuery {
    /// Get the Z3 Real variables for a point's coordinates
    fn point_variables(&self, point_id: PointId) -> Result<(Real<'_>, Real<'_>)>;
    
    /// Get the Z3 Real variable for a length/distance value
    fn length_variable(&self, name: &str) -> Result<Real<'_>>;
    
    /// Get the Z3 Real variable for an angle value
    fn angle_variable(&self, name: &str) -> Result<Real<'_>>;
}

/// Trait for entities that can generate constraints involving themselves
pub trait ConstraintFactory {
    /// Generate constraints that can be applied to the sketch
    fn constraints(&self) -> Vec<Box<dyn Constraint>>;
}

