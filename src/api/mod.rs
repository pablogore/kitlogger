//! Creation APIs for observability telemetry
//!
//! This module provides factory functions and APIs for creating telemetry entities.

use crate::models::{
    Context, Resource, InstrumentationScope, Span, LogRecord, Metric, AttributeValue, 
    SpanStatus, LogSeverity
};
use std::collections::HashMap;

pub use crate::validation::{validators, validation, TelemetryError};

/// A builder for creating Context objects
pub struct ContextBuilder {
    trace_id: Option<[u8; 16]>,
    span_id: Option<[u8; 8]>,
    correlation_id: Option<String>,
}

impl ContextBuilder {
    /// Creates a new context builder
    pub fn new() -> Self {
        Self {
            trace_id: None,
            span_id: None,
            correlation_id: None,
        }
    }

    /// Sets the trace ID
    pub fn with_trace_id(mut self, trace_id: [u8; 16]) -> Self {
        self.trace_id = Some(trace_id);
        self
    }

    /// Sets the span ID
    pub fn with_span_id(mut self, span_id: [u8; 8]) -> Self {
        self.span_id = Some(span_id);
        self
    }

    /// Sets the correlation ID
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Builds the context
    pub fn build(self) -> Context {
        Context {
            trace_id: self.trace_id.unwrap_or([0u8; 16]),
            span_id: self.span_id.unwrap_or([0u8; 8]),
            correlation_id: self.correlation_id,
        }
    }
}

/// A builder for creating Resource objects
pub struct ResourceBuilder {
    attributes: HashMap<String, AttributeValue>,
}

impl ResourceBuilder {
    /// Creates a new resource builder
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }

    /// Adds an attribute to the resource
    pub fn with_attribute(mut self, key: String, value: AttributeValue) -> Self {
        self.attributes.insert(key, value);
        self
    }

    /// Builds the resource
    pub fn build(self) -> Resource {
        Resource {
            attributes: self.attributes,
        }
    }
}

/// A builder for creating InstrumentationScope objects
pub struct InstrumentationScopeBuilder {
    name: String,
    version: Option<String>,
}

impl InstrumentationScopeBuilder {
    /// Creates a new instrumentation scope builder
    pub fn new(name: String) -> Self {
        Self {
            name,
            version: None,
        }
    }

    /// Sets the version
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    /// Builds the instrumentation scope
    pub fn build(self) -> InstrumentationScope {
        InstrumentationScope {
            name: self.name,
            version: self.version,
        }
    }
}

/// A builder for creating Span objects
pub struct SpanBuilder {
    context: Context,
    resource: Resource,
    instrumentation_scope: InstrumentationScope,
    name: String,
    start_time: u64,
    attributes: HashMap<String, AttributeValue>,
    status: Option<SpanStatus>,
}

impl SpanBuilder {
    /// Creates a new span builder
    pub fn new(
        context: Context,
        resource: Resource,
        instrumentation_scope: InstrumentationScope,
        name: String,
        start_time: u64,
    ) -> Self {
        Self {
            context,
            resource,
            instrumentation_scope,
            name,
            start_time,
            attributes: HashMap::new(),
            status: None,
        }
    }

    /// Adds an attribute to the span
    pub fn with_attribute(mut self, key: String, value: AttributeValue) -> Self {
        self.attributes.insert(key, value);
        self
    }

    /// Sets the status of the span
    pub fn with_status(mut self, status: SpanStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Builds the span
    pub fn build(self) -> Span {
        Span {
            context: self.context,
            resource: self.resource,
            instrumentation_scope: self.instrumentation_scope,
            name: self.name,
            start_time: self.start_time,
            end_time: None,
            attributes: self.attributes,
            status: self.status,
        }
    }
}

/// A builder for creating LogRecord objects
pub struct LogRecordBuilder {
    context: Context,
    resource: Resource,
    instrumentation_scope: InstrumentationScope,
    timestamp: u64,
    severity: LogSeverity,
    body: String,
    attributes: HashMap<String, AttributeValue>,
}

impl LogRecordBuilder {
    /// Creates a new log record builder
    pub fn new(
        context: Context,
        resource: Resource,
        instrumentation_scope: InstrumentationScope,
        timestamp: u64,
        severity: LogSeverity,
        body: String,
    ) -> Self {
        Self {
            context,
            resource,
            instrumentation_scope,
            timestamp,
            severity,
            body,
            attributes: HashMap::new(),
        }
    }

    /// Adds an attribute to the log record
    pub fn with_attribute(mut self, key: String, value: AttributeValue) -> Self {
        self.attributes.insert(key, value);
        self
    }

    /// Builds the log record
    pub fn build(self) -> LogRecord {
        LogRecord {
            context: self.context,
            resource: self.resource,
            instrumentation_scope: self.instrumentation_scope,
            timestamp: self.timestamp,
            severity: self.severity,
            body: self.body,
            attributes: self.attributes,
        }
    }
}

/// A builder for creating Metric objects
pub struct MetricBuilder {
    resource: Resource,
    instrumentation_scope: InstrumentationScope,
    name: String,
    description: String,
    unit: String,
    attributes: HashMap<String, AttributeValue>,
}

impl MetricBuilder {
    /// Creates a new metric builder
    pub fn new(
        resource: Resource,
        instrumentation_scope: InstrumentationScope,
        name: String,
        description: String,
        unit: String,
    ) -> Self {
        Self {
            resource,
            instrumentation_scope,
            name,
            description,
            unit,
            attributes: HashMap::new(),
        }
    }

    /// Adds an attribute to the metric
    pub fn with_attribute(mut self, key: String, value: AttributeValue) -> Self {
        self.attributes.insert(key, value);
        self
    }

    /// Builds the metric
    pub fn build(self) -> Metric {
        Metric {
            resource: self.resource,
            instrumentation_scope: self.instrumentation_scope,
            name: self.name,
            description: self.description,
            unit: self.unit,
            attributes: self.attributes,
        }
    }
}

/// Utility functions for creating common telemetry entities
pub mod factory {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Creates a new root context
    pub fn new_root_context() -> Context {
        Context::new_root()
    }

    /// Creates a new child context
    pub fn new_child_context(parent: &Context) -> Context {
        parent.new_child()
    }

    /// Creates a new context with a correlation ID
    pub fn new_context_with_correlation_id(correlation_id: String) -> Context {
        Context::new_root().with_correlation_id(correlation_id)
    }

    /// Creates a new resource
    pub fn new_resource() -> Resource {
        Resource::new()
    }

    /// Creates a new instrumentation scope
    pub fn new_instrumentation_scope(name: String) -> InstrumentationScope {
        InstrumentationScope::new(name)
    }

    /// Creates a new instrumentation scope with version
    pub fn new_instrumentation_scope_with_version(name: String, version: String) -> InstrumentationScope {
        InstrumentationScope::with_version(name, version)
    }

    /// Creates a new span
    pub fn new_span(
        context: Context,
        resource: Resource,
        instrumentation_scope: InstrumentationScope,
        name: String,
    ) -> Span {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Span::new(context, resource, instrumentation_scope, name, start_time)
    }

    /// Creates a new log record
    pub fn new_log_record(
        context: Context,
        resource: Resource,
        instrumentation_scope: InstrumentationScope,
        severity: LogSeverity,
        body: String,
    ) -> LogRecord {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        LogRecord::new(context, resource, instrumentation_scope, timestamp, severity, body)
    }

    /// Creates a new metric
    pub fn new_metric(
        resource: Resource,
        instrumentation_scope: InstrumentationScope,
        name: String,
        description: String,
        unit: String,
    ) -> Metric {
        Metric::new(resource, instrumentation_scope, name, description, unit)
    }
}