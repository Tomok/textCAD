use thiserror::Error;

/// Main error type for TextCAD operations
#[derive(Error, Debug, Clone)]
pub enum TextCadError {
    /// Z3 solver related errors
    #[error("Solver error: {0}")]
    SolverError(String),
    
    /// Invalid constraint specification
    #[error("Invalid constraint: {0}")]
    InvalidConstraint(String),
    
    /// Entity reference errors (invalid ID, etc.)
    #[error("Entity error: {0}")]
    EntityError(String),
    
    /// Sketch is over-constrained (no solution exists)
    #[error("Sketch is over-constrained")]
    OverConstrained,
    
    /// Sketch is under-constrained (infinite solutions)
    #[error("Sketch is under-constrained")]
    UnderConstrained,
    
    /// Solution extraction failed
    #[error("Solution error: {0}")]
    SolutionError(String),
    
    /// Export/serialization errors
    #[error("Export error: {0}")]
    ExportError(String),
    
    /// Invalid input parameters
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Result type alias for TextCAD operations
pub type Result<T> = std::result::Result<T, TextCadError>;

/// Result type alias for solver operations specifically
pub type SolverResult<T> = std::result::Result<T, TextCadError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = TextCadError::SolverError("Z3 failed".to_string());
        assert_eq!(error.to_string(), "Solver error: Z3 failed");
        
        let error = TextCadError::OverConstrained;
        assert_eq!(error.to_string(), "Sketch is over-constrained");
    }

    #[test]
    fn test_error_debug() {
        let error = TextCadError::InvalidConstraint("test".to_string());
        assert!(format!("{:?}", error).contains("InvalidConstraint"));
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TextCadError>();
    }
}