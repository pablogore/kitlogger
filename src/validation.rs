//! Validation utilities for telemetry data

use crate::models::AttributeValue;


/// An error that can occur during telemetry validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelemetryError {
    /// An invalid attribute value was provided
    InvalidAttributeValue,
    /// An invalid span name was provided
    InvalidSpanName,
    /// An invalid log record body was provided
    InvalidLogBody,
    /// An invalid metric name was provided
    InvalidMetricName,
    /// An invalid resource attribute was provided
    InvalidResourceAttribute,
    /// An invalid instrumentation scope name was provided
    InvalidInstrumentationScopeName,
    /// An invalid trace ID was provided
    InvalidTraceId,
    /// An invalid span ID was provided
    InvalidSpanId,
    /// An invalid timestamp was provided
    InvalidTimestamp,
    /// An invalid correlation ID was provided
    InvalidCorrelationId,
}

/// Validates an attribute value
pub fn validate_attribute_value(value: &AttributeValue) -> Result<(), TelemetryError> {
    match value {
        AttributeValue::String(_) => Ok(()),
        AttributeValue::Bool(_) => Ok(()),
        AttributeValue::I64(_) => Ok(()),
        AttributeValue::F64(_) => Ok(()),
        AttributeValue::StringArray(arr) => {
            for item in arr {
                if item.is_empty() {
                    return Err(TelemetryError::InvalidAttributeValue);
                }
            }
            Ok(())
        }
        AttributeValue::BoolArray(_) => Ok(()),
        AttributeValue::I64Array(_) => Ok(()),
        AttributeValue::F64Array(_) => Ok(()),
    }
}

/// Validates a span name
pub fn validate_span_name(name: &str) -> Result<(), TelemetryError> {
    if name.is_empty() {
        Err(TelemetryError::InvalidSpanName)
    } else {
        Ok(())
    }
}

/// Validates a log record body
pub fn validate_log_body(body: &str) -> Result<(), TelemetryError> {
    if body.is_empty() {
        Err(TelemetryError::InvalidLogBody)
    } else {
        Ok(())
    }
}

/// Validates a metric name
pub fn validate_metric_name(name: &str) -> Result<(), TelemetryError> {
    if name.is_empty() {
        Err(TelemetryError::InvalidMetricName)
    } else {
        Ok(())
    }
}

/// Validates a resource attribute
pub fn validate_resource_attribute(key: &str, value: &AttributeValue) -> Result<(), TelemetryError> {
    if key.is_empty() {
        Err(TelemetryError::InvalidResourceAttribute)
    } else {
        validate_attribute_value(value)
    }
}

/// Validates an instrumentation scope name
pub fn validate_instrumentation_scope_name(name: &str) -> Result<(), TelemetryError> {
    if name.is_empty() {
        Err(TelemetryError::InvalidInstrumentationScopeName)
    } else {
        Ok(())
    }
}

/// Validates a trace ID
pub fn validate_trace_id(trace_id: &[u8; 16]) -> Result<(), TelemetryError> {
    if trace_id.iter().all(|&b| b == 0) {
        Err(TelemetryError::InvalidTraceId)
    } else {
        Ok(())
    }
}

/// Validates a span ID
pub fn validate_span_id(span_id: &[u8; 8]) -> Result<(), TelemetryError> {
    if span_id.iter().all(|&b| b == 0) {
        Err(TelemetryError::InvalidSpanId)
    } else {
        Ok(())
    }
}

/// Validates a timestamp
pub fn validate_timestamp(timestamp: u64) -> Result<(), TelemetryError> {
    if timestamp == 0 {
        Err(TelemetryError::InvalidTimestamp)
    } else {
        Ok(())
    }
}

/// Validates a correlation ID
pub fn validate_correlation_id(correlation_id: &str) -> Result<(), TelemetryError> {
    if correlation_id.is_empty() {
        Err(TelemetryError::InvalidCorrelationId)
    } else {
        Ok(())
    }
}

/// A collection of validation functions
pub mod validators {
    use super::*;

    /// Validates an attribute value
    pub fn attribute_value(value: &AttributeValue) -> Result<(), TelemetryError> {
        validate_attribute_value(value)
    }

    /// Validates a span name
    pub fn span_name(name: &str) -> Result<(), TelemetryError> {
        validate_span_name(name)
    }

    /// Validates a log record body
    pub fn log_body(body: &str) -> Result<(), TelemetryError> {
        validate_log_body(body)
    }

    /// Validates a metric name
    pub fn metric_name(name: &str) -> Result<(), TelemetryError> {
        validate_metric_name(name)
    }

    /// Validates a resource attribute
    pub fn resource_attribute(key: &str, value: &AttributeValue) -> Result<(), TelemetryError> {
        validate_resource_attribute(key, value)
    }

    /// Validates an instrumentation scope name
    pub fn instrumentation_scope_name(name: &str) -> Result<(), TelemetryError> {
        validate_instrumentation_scope_name(name)
    }

    /// Validates a trace ID
    pub fn trace_id(trace_id: &[u8; 16]) -> Result<(), TelemetryError> {
        validate_trace_id(trace_id)
    }

    /// Validates a span ID
    pub fn span_id(span_id: &[u8; 8]) -> Result<(), TelemetryError> {
        validate_span_id(span_id)
    }

    /// Validates a timestamp
    pub fn timestamp(timestamp: u64) -> Result<(), TelemetryError> {
        validate_timestamp(timestamp)
    }

    /// Validates a correlation ID
    pub fn correlation_id(correlation_id: &str) -> Result<(), TelemetryError> {
        validate_correlation_id(correlation_id)
    }
}

/// A collection of validation functions that return a boolean
pub mod validation {
    use super::*;

    /// Validates an attribute value
    pub fn attribute_value(value: &AttributeValue) -> bool {
        validate_attribute_value(value).is_ok()
    }

    /// Validates a span name
    pub fn span_name(name: &str) -> bool {
        validate_span_name(name).is_ok()
    }

    /// Validates a log record body
    pub fn log_body(body: &str) -> bool {
        validate_log_body(body).is_ok()
    }

    /// Validates a metric name
    pub fn metric_name(name: &str) -> bool {
        validate_metric_name(name).is_ok()
    }

    /// Validates a resource attribute
    pub fn resource_attribute(key: &str, value: &AttributeValue) -> bool {
        validate_resource_attribute(key, value).is_ok()
    }

    /// Validates an instrumentation scope name
    pub fn instrumentation_scope_name(name: &str) -> bool {
        validate_instrumentation_scope_name(name).is_ok()
    }

    /// Validates a trace ID
    pub fn trace_id(trace_id: &[u8; 16]) -> bool {
        validate_trace_id(trace_id).is_ok()
    }

    /// Validates a span ID
    pub fn span_id(span_id: &[u8; 8]) -> bool {
        validate_span_id(span_id).is_ok()
    }

    /// Validates a timestamp
    pub fn timestamp(timestamp: u64) -> bool {
        validate_timestamp(timestamp).is_ok()
    }

    /// Validates a correlation ID
    pub fn correlation_id(correlation_id: &str) -> bool {
        validate_correlation_id(correlation_id).is_ok()
    }
}