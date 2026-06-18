//! Validation errors for structured logs.

use std::fmt::{Display, Formatter, Result as FmtResult};

/// Errors that can occur during log record construction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValidationError {
    /// Message string is empty
    EmptyMessage,
    /// Severity level not recognized
    InvalidSeverity,
    /// Attribute name violates naming constraints
    InvalidAttributeName(String),
    /// Attribute value violates type constraints
    InvalidAttributeValue(String),
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ValidationError::EmptyMessage => write!(f, "Message cannot be empty"),
            ValidationError::InvalidSeverity => write!(f, "Invalid severity level"),
            ValidationError::InvalidAttributeName(name) => {
                write!(f, "Invalid attribute name: {}", name)
            }
            ValidationError::InvalidAttributeValue(msg) => {
                write!(f, "Invalid attribute value: {}", msg)
            }
        }
    }
}

impl std::error::Error for ValidationError {}
