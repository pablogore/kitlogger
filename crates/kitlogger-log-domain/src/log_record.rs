//! Log records for structured logs.

use std::time::SystemTime;

use crate::{LogAttribute, ValidationError};

/// Canonical log entry. Immutable after construction.
///
/// Derive exclusions:
/// - `Eq` not derived: contains `LogAttribute` -> `LogAttributeValue` which has `f64` (no `Eq`).
/// - `Hash` not derived: contains `LogAttribute` -> `LogAttributeValue` which has `f64` (no `Hash`).
#[derive(Clone, Debug, PartialEq)]
pub struct LogRecord {
    timestamp: SystemTime,
    severity: crate::Severity,
    message: String,
    attributes: Vec<LogAttribute>,
}

impl LogRecord {
    /// Creates a new log record.
    ///
    /// # Errors
    ///
    /// Returns `ValidationError::EmptyMessage` if the message is empty.
    pub fn new(
        timestamp: SystemTime,
        severity: crate::Severity,
        message: String,
        attributes: Vec<LogAttribute>,
    ) -> Result<Self, ValidationError> {
        if message.is_empty() {
            return Err(ValidationError::EmptyMessage);
        }

        Ok(LogRecord {
            timestamp,
            severity,
            message,
            attributes,
        })
    }

    /// Returns the timestamp.
    pub fn timestamp(&self) -> &SystemTime {
        &self.timestamp
    }

    /// Returns the severity.
    pub fn severity(&self) -> &crate::Severity {
        &self.severity
    }

    /// Returns the message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the attributes.
    pub fn attributes(&self) -> &[LogAttribute] {
        &self.attributes
    }
}
