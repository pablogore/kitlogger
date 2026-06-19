//! Error type for log emission failures.

use crate::validation::ValidationError;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Errors that can occur when emitting a log record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmitError {
    /// Log record construction failed due to a validation problem.
    Validation(ValidationError),
    /// Emit was attempted after the logger was shut down.
    LoggerClosed,
}

impl From<ValidationError> for EmitError {
    fn from(err: ValidationError) -> Self {
        EmitError::Validation(err)
    }
}

impl Display for EmitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            EmitError::Validation(e) => write!(f, "Emit validation error: {}", e),
            EmitError::LoggerClosed => write!(f, "Logger is closed"),
        }
    }
}

impl std::error::Error for EmitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EmitError::Validation(e) => Some(e),
            EmitError::LoggerClosed => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit_error_validation_wraps_validation_error() {
        let ve = ValidationError::EmptyMessage;
        let ee = EmitError::Validation(ve.clone());
        assert_eq!(ee, EmitError::Validation(ValidationError::EmptyMessage));
    }

    #[test]
    fn emit_error_logger_closed_exists() {
        let ee = EmitError::LoggerClosed;
        assert_eq!(ee, EmitError::LoggerClosed);
    }

    #[test]
    fn from_validation_error_converts_correctly() {
        let ve = ValidationError::EmptyMessage;
        let ee: EmitError = ve.clone().into();
        assert_eq!(ee, EmitError::Validation(ve));
    }

    #[test]
    fn display_formats_validation_variant() {
        let ee = EmitError::Validation(ValidationError::EmptyMessage);
        let s = format!("{}", ee);
        assert!(s.contains("Message cannot be empty"), "got: {}", s);
    }

    #[test]
    fn display_formats_logger_closed_variant() {
        let ee = EmitError::LoggerClosed;
        let s = format!("{}", ee);
        assert!(s.contains("closed"), "got: {}", s);
    }

    #[test]
    fn std_error_is_implemented() {
        let ee: &dyn std::error::Error = &EmitError::LoggerClosed;
        let _ = ee.to_string();
    }

    #[test]
    fn error_source_is_some_for_validation() {
        use std::error::Error;
        let ee = EmitError::Validation(ValidationError::EmptyMessage);
        assert!(ee.source().is_some());
    }

    #[test]
    fn error_source_is_none_for_closed() {
        use std::error::Error;
        let ee = EmitError::LoggerClosed;
        assert!(ee.source().is_none());
    }
}
