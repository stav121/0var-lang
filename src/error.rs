//! Error types for the zvar language compiler

use crate::span::Span;
use thiserror::Error;

/// Main error type for the zvar language
#[derive(Error, Debug)]
pub enum ZvarError {
    // Lexer errors
    #[error("Invalid number '{value}' at {span}")]
    InvalidNumber { span: Span, value: String },

    #[error("Unknown identifier '{name}' at {span}")]
    UnknownIdentifier { span: Span, name: String },

    #[error("Invalid entity number in '{entity}' at {span}")]
    InvalidEntityNumber { span: Span, entity: String },

    #[error("Unexpected character '{character}' at {span}")]
    UnexpectedCharacter { span: Span, character: char },

    // Parser errors
    #[error("Expected {expected}, found {found} at {span}")]
    UnexpectedToken {
        span: Span,
        expected: String,
        found: String,
    },

    #[error("Missing semicolon at {span}")]
    MissingSemicolon { span: Span },

    #[error("Invalid assignment target at {span}")]
    InvalidAssignmentTarget { span: Span },

    #[error("Undefined entity '{name}' at {span}")]
    UndefinedEntity { span: Span, name: String },

    #[error("Entity '{name}' already defined at {span}")]
    EntityAlreadyDefined {
        span: Span,
        name: String,
        previous_span: Option<Span>,
    },

    #[error("Type mismatch at {span}: expected {expected}, found {found}")]
    TypeMismatch {
        span: Span,
        expected: String,
        found: String,
    },

    #[error(
        "Wrong number of arguments for '{name}' at {span}: expected {expected}, found {found}"
    )]
    WrongArgumentCount {
        span: Span,
        name: String,
        expected: usize,
        found: usize,
    },

    // Codegen errors
    #[error("Code generation failed: {message}")]
    CodegenError { message: String },

    // Runtime errors
    #[error("Runtime error: {message}")]
    RuntimeError { message: String },

    #[error("Stack overflow")]
    StackOverflow,

    #[error("Stack underflow")]
    StackUnderflow,

    #[error("Division by zero{}", span.map_or(String::new(), |s| format!(" at {}", s)))]
    DivisionByZero { span: Option<Span> },

    #[error("Cannot assign to constant '{name}' at {span}")]
    CannotAssignToConstant { span: Span, name: String },

    // IO errors
    #[error("IO error: {message}")]
    IoError { message: String },

    #[error("File error: {message}")]
    FileError { message: String },
}

impl ZvarError {
    /// Get the span associated with this error, if any
    pub fn span(&self) -> Option<Span> {
        match self {
            ZvarError::InvalidNumber { span, .. } => Some(*span),
            ZvarError::UnknownIdentifier { span, .. } => Some(*span),
            ZvarError::InvalidEntityNumber { span, .. } => Some(*span),
            ZvarError::UnexpectedCharacter { span, .. } => Some(*span),
            ZvarError::UnexpectedToken { span, .. } => Some(*span),
            ZvarError::MissingSemicolon { span } => Some(*span),
            ZvarError::InvalidAssignmentTarget { span } => Some(*span),
            ZvarError::UndefinedEntity { span, .. } => Some(*span),
            ZvarError::EntityAlreadyDefined { span, .. } => Some(*span),
            ZvarError::TypeMismatch { span, .. } => Some(*span),
            ZvarError::WrongArgumentCount { span, .. } => Some(*span),
            ZvarError::CannotAssignToConstant { span, .. } => Some(*span),
            ZvarError::DivisionByZero { span, .. } => *span,
            _ => None,
        }
    }

    /// Check if this is a compile-time error
    pub fn is_compile_time(&self) -> bool {
        match self {
            ZvarError::RuntimeError { .. }
            | ZvarError::StackOverflow
            | ZvarError::StackUnderflow => false,
            _ => true,
        }
    }

    /// Create a simple runtime error
    pub fn runtime(message: impl Into<String>) -> Self {
        ZvarError::RuntimeError {
            message: message.into(),
        }
    }

    /// Create a file error
    pub fn file_error(message: impl Into<String>) -> Self {
        ZvarError::FileError {
            message: message.into(),
        }
    }
}

impl From<std::io::Error> for ZvarError {
    fn from(err: std::io::Error) -> Self {
        ZvarError::IoError {
            message: err.to_string(),
        }
    }
}

/// Result type alias for zvar operations
pub type ZvarResult<T> = Result<T, ZvarError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_span_extraction() {
        let span = Span::new(1, 5, 1, 10);
        let error = ZvarError::InvalidNumber {
            span,
            value: "abc".to_string(),
        };

        assert_eq!(error.span(), Some(span));
    }

    #[test]
    fn test_compile_time_error_detection() {
        let error = ZvarError::RuntimeError {
            message: "test".to_string(),
        };
        assert!(!error.is_compile_time());

        let error = ZvarError::UnexpectedToken {
            span: Span::new(1, 1, 1, 1),
            expected: "int".to_string(),
            found: "fn".to_string(),
        };
        assert!(error.is_compile_time());
    }

    #[test]
    fn test_error_creation_helpers() {
        let error = ZvarError::runtime("test message");
        match error {
            ZvarError::RuntimeError { message } => assert_eq!(message, "test message"),
            _ => panic!("Wrong error type"),
        }
    }
}
