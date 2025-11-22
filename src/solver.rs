//! Solver abstraction traits and types
//!
//! This module defines abstract interfaces for constraint solvers,
//! allowing for future extensibility to support different solving engines.

use crate::error::Result;

/// A solution extracted from a constraint solver
///
/// This trait provides a common interface for solution types
/// regardless of the underlying solver implementation.
pub trait Solution {
    /// Type representing variable values in the solution
    type Value;

    /// Extract the value of a variable by name
    fn get_value(&self, variable_name: &str) -> Option<Self::Value>;

    /// Check if the solution is complete (all variables have values)
    fn is_complete(&self) -> bool;
}

/// Abstract interface for constraint solvers
///
/// This trait allows TextCAD to potentially support multiple
/// constraint solving backends in the future.
pub trait ConstraintSolver {
    /// Type representing the solution returned by this solver
    type Solution: Solution;

    /// Type representing assertions/constraints for this solver
    type Assertion;

    /// Add a constraint/assertion to the solver
    fn add_assertion(&mut self, assertion: Self::Assertion) -> Result<()>;

    /// Attempt to solve the current constraint system
    fn solve(&mut self) -> Result<Self::Solution>;

    /// Check satisfiability without extracting a solution
    fn check_satisfiable(&mut self) -> Result<bool>;

    /// Reset the solver, clearing all constraints
    fn reset(&mut self);

    /// Get the number of active constraints
    fn constraint_count(&self) -> usize;
}

/// Marker trait for solvers that support incremental solving
///
/// Incremental solvers can push/pop constraint contexts,
/// allowing efficient exploration of constraint variations.
pub trait IncrementalSolver: ConstraintSolver {
    /// Push a new constraint context onto the stack
    fn push(&mut self);

    /// Pop the most recent constraint context from the stack
    fn pop(&mut self) -> Result<()>;

    /// Get the current context depth
    fn context_depth(&self) -> usize;
}

/// Marker trait for solvers that support optimization objectives
///
/// Some constraint solvers can optimize objectives (minimize/maximize)
/// in addition to finding satisfying solutions.
pub trait OptimizingSolver: ConstraintSolver {
    /// Type representing optimization objectives
    type Objective;

    /// Add a minimization objective
    fn minimize(&mut self, objective: Self::Objective) -> Result<()>;

    /// Add a maximization objective  
    fn maximize(&mut self, objective: Self::Objective) -> Result<()>;

    /// Clear all optimization objectives
    fn clear_objectives(&mut self);
}

/// Metadata about a solver implementation
#[derive(Debug, Clone)]
pub struct SolverInfo {
    /// Human-readable name of the solver
    pub name: String,
    /// Version string of the solver
    pub version: String,
    /// Whether the solver supports real arithmetic
    pub supports_reals: bool,
    /// Whether the solver supports integer arithmetic
    pub supports_integers: bool,
    /// Whether the solver supports incremental solving
    pub supports_incremental: bool,
    /// Whether the solver supports optimization
    pub supports_optimization: bool,
}

/// Trait for querying solver capabilities
pub trait SolverMetadata {
    /// Get information about the solver implementation
    fn solver_info(&self) -> SolverInfo;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementations for testing trait definitions

    struct MockSolution {
        values: std::collections::HashMap<String, f64>,
    }

    impl Solution for MockSolution {
        type Value = f64;

        fn get_value(&self, variable_name: &str) -> Option<Self::Value> {
            self.values.get(variable_name).copied()
        }

        fn is_complete(&self) -> bool {
            true // For simplicity in tests
        }
    }

    struct MockAssertion {
        _constraint: String,
    }

    struct MockSolver {
        assertions: Vec<MockAssertion>,
    }

    impl ConstraintSolver for MockSolver {
        type Solution = MockSolution;
        type Assertion = MockAssertion;

        fn add_assertion(&mut self, assertion: Self::Assertion) -> Result<()> {
            self.assertions.push(assertion);
            Ok(())
        }

        fn solve(&mut self) -> Result<Self::Solution> {
            let mut values = std::collections::HashMap::new();
            values.insert("x".to_string(), 42.0);
            Ok(MockSolution { values })
        }

        fn check_satisfiable(&mut self) -> Result<bool> {
            Ok(true)
        }

        fn reset(&mut self) {
            self.assertions.clear();
        }

        fn constraint_count(&self) -> usize {
            self.assertions.len()
        }
    }

    #[test]
    fn test_mock_solver() {
        let mut solver = MockSolver {
            assertions: Vec::new(),
        };

        assert_eq!(solver.constraint_count(), 0);

        let assertion = MockAssertion {
            _constraint: "x + 1 = 2".to_string(),
        };

        solver.add_assertion(assertion).unwrap();
        assert_eq!(solver.constraint_count(), 1);

        let solution = solver.solve().unwrap();
        assert_eq!(solution.get_value("x"), Some(42.0));
        assert!(solution.is_complete());

        solver.reset();
        assert_eq!(solver.constraint_count(), 0);
    }

    #[test]
    fn test_solver_info() {
        let info = SolverInfo {
            name: "MockSolver".to_string(),
            version: "1.0.0".to_string(),
            supports_reals: true,
            supports_integers: false,
            supports_incremental: false,
            supports_optimization: false,
        };

        assert_eq!(info.name, "MockSolver");
        assert!(info.supports_reals);
        assert!(!info.supports_integers);
    }
}
