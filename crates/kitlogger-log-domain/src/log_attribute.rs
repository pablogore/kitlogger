//! Log attributes for structured logs.

use crate::{LogAttributeValue, ValidationError};

/// A named key-value pair of structured data.
///
/// Derive exclusions:
/// - `Eq` not derived: contains `LogAttributeValue` which has `f64` (no `Eq`).
/// - `Hash` not derived: contains `LogAttributeValue` which has `f64` (no `Hash`).
#[derive(Clone, Debug, PartialEq)]
pub struct LogAttribute {
    name: String,
    value: LogAttributeValue,
}

impl LogAttribute {
    /// Creates a new log attribute.
    ///
    /// # Errors
    ///
    /// Returns `ValidationError::InvalidAttributeName` if the name is invalid.
    pub fn new(name: String, value: LogAttributeValue) -> Result<Self, ValidationError> {
        validate_attribute_name(&name)?;
        Ok(LogAttribute { name, value })
    }

    /// Returns the attribute name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the attribute value.
    pub fn value(&self) -> &LogAttributeValue {
        &self.value
    }
}

/// Validates an attribute name against the naming constraints.
///
/// # Errors
///
/// Returns `ValidationError::InvalidAttributeName` if the name is invalid.
pub fn validate_attribute_name(name: &str) -> Result<(), ValidationError> {
    // Check for empty name
    if name.is_empty() {
        return Err(ValidationError::InvalidAttributeName(
            "Name cannot be empty".to_string(),
        ));
    }

    // Check max length (64 characters)
    if name.len() > 64 {
        return Err(ValidationError::InvalidAttributeName(
            "Name exceeds 64 characters".to_string(),
        ));
    }

    // Check pattern: ^[a-z][a-z0-9._]{0,63}$
    if !name.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
        return Err(ValidationError::InvalidAttributeName(
            "Name must start with lowercase letter".to_string(),
        ));
    }

    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_')
    {
        return Err(ValidationError::InvalidAttributeName(
            "Name contains invalid characters".to_string(),
        ));
    }

    // Check reserved field names
    let reserved = ["timestamp", "severity", "message", "attributes"];
    if reserved.contains(&name) {
        return Err(ValidationError::InvalidAttributeName(
            "Name is reserved".to_string(),
        ));
    }

    Ok(())
}
