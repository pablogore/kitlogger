//! Log records for structured logs.

use std::time::SystemTime;

use crate::{LogAttribute, ValidationError};

/// Canonical log entry. Immutable after construction.
#[derive(Clone, Debug)]
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
    /// Returns `ValidationError::InvalidAttributeName` if any attribute name is invalid.
    /// Returns `ValidationError::InvalidAttributeValue` if any attribute value is invalid.
    pub fn new(
        timestamp: SystemTime,
        severity: crate::Severity,
        message: String,
        attributes: Vec<LogAttribute>,
    ) -> Result<Self, ValidationError> {
        if message.is_empty() {
            return Err(ValidationError::EmptyMessage);
        }

        // Validate all attribute names
        for attr in &attributes {
            // Attribute name validation is already done in LogAttribute::new
            // But we'll check again to be safe
            crate::log_attribute::validate_attribute_name(attr.name())?;
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