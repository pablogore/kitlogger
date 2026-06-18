//! Log attribute values for structured logs.

use std::time::SystemTime;

use crate::ValidationError;

/// Strongly typed wrapper for attribute values.
///
/// Supports flat scalar types only (no nested objects).
///
/// Derive exclusions:
/// - `Eq` not derived: `Float(f64)` variant contains `f64` which does not implement `Eq`.
/// - `Hash` not derived: `Float(f64)` variant contains `f64` which does not implement `Hash`.
#[derive(Clone, Debug, PartialEq)]
pub enum LogAttributeValue {
    /// UTF-8 string value
    String(String),
    /// Signed 64-bit integer
    Integer(i64),
    /// 64-bit floating point
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Point in time (UTC)
    Timestamp(SystemTime),
    /// Homogeneous array of values
    Array(Vec<LogAttributeValue>),
}

impl LogAttributeValue {
    /// Creates a new String variant.
    pub fn string(value: String) -> Self {
        LogAttributeValue::String(value)
    }

    /// Creates a new Integer variant.
    pub fn integer(value: i64) -> Self {
        LogAttributeValue::Integer(value)
    }

    /// Creates a new Float variant.
    pub fn float(value: f64) -> Self {
        LogAttributeValue::Float(value)
    }

    /// Creates a new Boolean variant.
    pub fn boolean(value: bool) -> Self {
        LogAttributeValue::Boolean(value)
    }

    /// Creates a new Timestamp variant.
    pub fn timestamp(value: SystemTime) -> Self {
        LogAttributeValue::Timestamp(value)
    }

    /// Creates a new Array variant.
    ///
    /// Enforces homogeneous element types at construction.
    pub fn array(values: Vec<LogAttributeValue>) -> Result<Self, ValidationError> {
        if values.is_empty() {
            return Ok(LogAttributeValue::Array(values));
        }

        let first_type = &values[0];
        for value in &values {
            if std::mem::discriminant(value) != std::mem::discriminant(first_type) {
                return Err(ValidationError::InvalidAttributeValue(
                    "Array elements must be of the same type".to_string(),
                ));
            }
        }

        Ok(LogAttributeValue::Array(values))
    }
}
