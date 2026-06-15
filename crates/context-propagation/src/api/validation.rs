//! Validation rules and error handling for observability telemetry
//!
//! This module defines validation rules for telemetry entities and error types
//! that can occur during telemetry operations.

use std::collections::HashMap;
use std::fmt;

/// An error that can occur during telemetry operations
#[derive(Debug, Clone, PartialEq)]
pub enum TelemetryError {
    /// An invalid attribute value was provided
    InvalidAttributeValue(String),
    /// An invalid trace ID was provided
    InvalidTraceId(String),
    /// An invalid span ID was provided
    InvalidSpanId(String),
    /// An invalid correlation ID was provided
    InvalidCorrelationId(String),
    /// An invalid resource attribute was provided
    InvalidResourceAttribute(String),
    /// An invalid instrumentation scope name was provided
    InvalidInstrumentationScopeName(String),
    /// An invalid span name was provided
    InvalidSpanName(String),
    /// An invalid log record body was provided
    InvalidLogBody(String),
    /// An invalid metric name was provided
    InvalidMetricName(String),
    /// An invalid metric unit was provided
    InvalidMetricUnit(String),
    /// An invalid metric description was provided
    InvalidMetricDescription(String),
    /// A required field was missing
    MissingRequiredField(String),
}

impl fmt::Display for TelemetryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TelemetryError::InvalidAttributeValue(msg) => write!(f, "Invalid attribute value: {}", msg),
            TelemetryError::InvalidTraceId(msg) => write!(f, "Invalid trace ID: {}", msg),
            TelemetryError::InvalidSpanId(msg) => write!(f, "Invalid span ID: {}", msg),
            TelemetryError::InvalidCorrelationId(msg) => write!(f, "Invalid correlation ID: {}", msg),
            TelemetryError::InvalidResourceAttribute(msg) => write!(f, "Invalid resource attribute: {}", msg),
            TelemetryError::InvalidInstrumentationScopeName(msg) => write!(f, "Invalid instrumentation scope name: {}", msg),
            TelemetryError::InvalidSpanName(msg) => write!(f, "Invalid span name: {}", msg),
            TelemetryError::InvalidLogBody(msg) => write!(f, "Invalid log body: {}", msg),
            TelemetryError::InvalidMetricName(msg) => write!(f, "Invalid metric name: {}", msg),
            TelemetryError::InvalidMetricUnit(msg) => write!(f, "Invalid metric unit: {}", msg),
            TelemetryError::InvalidMetricDescription(msg) => write!(f, "Invalid metric description: {}", msg),
            TelemetryError::MissingRequiredField(msg) => write!(f, "Missing required field: {}", msg),
        }
    }
}

impl std::error::Error for TelemetryError {}

/// Validation functions for telemetry entities
pub mod validators {
    use super::*;
    use crate::models::{AttributeValue, InstrumentationScope, Span, LogRecord, Metric};

    /// Validates an attribute value
    pub fn validate_attribute_value(value: &AttributeValue) -> Result<(), TelemetryError> {
        match value {
            AttributeValue::String(s) => {
                if s.is_empty() {
                    return Err(TelemetryError::InvalidAttributeValue("String attribute value cannot be empty".to_string()));
                }
            }
            AttributeValue::StringArray(arr) => {
                for s in arr {
                    if s.is_empty() {
                        return Err(TelemetryError::InvalidAttributeValue("String array attribute value cannot contain empty strings".to_string()));
                    }
                }
            }
            _ => {} // Other types don't need validation
        }
        Ok(())
    }

    /// Validates a trace ID
    pub fn validate_trace_id(trace_id: &[u8; 16]) -> Result<(), TelemetryError> {
        // In a real implementation, we might check for all-zero trace IDs
        // For now, we'll just ensure it's not null
        if trace_id.iter().all(|&b| b == 0) {
            return Err(TelemetryError::InvalidTraceId("Trace ID cannot be all zeros".to_string()));
        }
        Ok(())
    }

    /// Validates a span ID
    pub fn validate_span_id(span_id: &[u8; 8]) -> Result<(), TelemetryError> {
        // In a real implementation, we might check for all-zero span IDs
        // For now, we'll just ensure it's not null
        if span_id.iter().all(|&b| b == 0) {
            return Err(TelemetryError::InvalidSpanId("Span ID cannot be all zeros".to_string()));
        }
        Ok(())
    }

    /// Validates a correlation ID
    pub fn validate_correlation_id(correlation_id: &str) -> Result<(), TelemetryError> {
        if correlation_id.is_empty() {
            return Err(TelemetryError::InvalidCorrelationId("Correlation ID cannot be empty".to_string()));
        }
        Ok(())
    }

    /// Validates a resource
    pub fn validate_resource(resource: &super::Resource) -> Result<(), TelemetryError> {
        for (key, value) in &resource.attributes {
            if key.is_empty() {
                return Err(TelemetryError::InvalidResourceAttribute("Resource attribute key cannot be empty".to_string()));
            }
            validate_attribute_value(value)?;
        }
        Ok(())
    }

    /// Validates an instrumentation scope
    pub fn validate_instrumentation_scope(scope: &InstrumentationScope) -> Result<(), TelemetryError> {
        if scope.name.is_empty() {
            return Err(TelemetryError::InvalidInstrumentationScopeName("Instrumentation scope name cannot be empty".to_string()));
        }
        Ok(())
    }

    /// Validates a span
    pub fn validate_span(span: &Span) -> Result<(), TelemetryError> {
        validate_trace_id(&span.context.trace_id)?;
        validate_span_id(&span.context.span_id)?;
        validate_instrumentation_scope(&span.instrumentation_scope)?;
        if span.name.is_empty() {
            return Err(TelemetryError::InvalidSpanName("Span name cannot be empty".to_string()));
        }
        validate_resource(&span.resource)?;
        Ok(())
    }

    /// Validates a log record
    pub fn validate_log_record(log_record: &LogRecord) -> Result<(), TelemetryError> {
        validate_trace_id(&log_record.context.trace_id)?;
        validate_span_id(&log_record.context.span_id)?;
        validate_instrumentation_scope(&log_record.instrumentation_scope)?;
        if log_record.body.is_empty() {
            return Err(TelemetryError::InvalidLogBody("Log record body cannot be empty".to_string()));
        }
        validate_resource(&log_record.resource)?;
        Ok(())
    }

    /// Validates a metric
    pub fn validate_metric(metric: &Metric) -> Result<(), TelemetryError> {
        validate_instrumentation_scope(&metric.instrumentation_scope)?;
        if metric.name.is_empty() {
            return Err(TelemetryError::InvalidMetricName("Metric name cannot be empty".to_string()));
        }
        if metric.description.is_empty() {
            return Err(TelemetryError::InvalidMetricDescription("Metric description cannot be empty".to_string()));
        }
        if metric.unit.is_empty() {
            return Err(TelemetryError::InvalidMetricUnit("Metric unit cannot be empty".to_string()));
        }
        validate_resource(&metric.resource)?;
        Ok(())
    }
}

/// Validation utilities for telemetry entities
pub mod validation {
    use super::*;
    use crate::models::{AttributeValue, InstrumentationScope, Span, LogRecord, Metric};

    /// Validates an attribute value and returns an error if invalid
    pub fn validate_attribute_value(value: &AttributeValue) -> Result<(), TelemetryError> {
        validators::validate_attribute_value(value)
    }

    /// Validates a resource and returns an error if invalid
    pub fn validate_resource(resource: &super::Resource) -> Result<(), TelemetryError> {
        validators::validate_resource(resource)
    }

    /// Validates an instrumentation scope and returns an error if invalid
    pub fn validate_instrumentation_scope(scope: &InstrumentationScope) -> Result<(), TelemetryError> {
        validators::validate_instrumentation_scope(scope)
    }

    /// Validates a span and returns an error if invalid
    pub fn validate_span(span: &Span) -> Result<(), TelemetryError> {
        validators::validate_span(span)
    }

    /// Validates a log record and returns an error if invalid
    pub fn validate_log_record(log_record: &LogRecord) -> Result<(), TelemetryError> {
        validators::validate_log_record(log_record)
    }

    /// Validates a metric and returns an error if invalid
    pub fn validate_metric(metric: &Metric) -> Result<(), TelemetryError> {
        validators::validate_metric(metric)
    }
}