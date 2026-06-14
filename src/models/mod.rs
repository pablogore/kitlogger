//! Core data models for observability telemetry
//!
//! This module contains the foundational data structures for all telemetry types:
//! - Context: For correlation and propagation
//! - Resource: For service metadata
//! - InstrumentationScope: For library identification
//! - Span: For trace telemetry
//! - LogRecord: For log telemetry
//! - Metric: For metric telemetry

use std::collections::HashMap;

/// A unique identifier for a trace
pub type TraceId = [u8; 16];

/// A unique identifier for a span
pub type SpanId = [u8; 8];

/// A unique identifier for a correlation
pub type CorrelationId = String;

/// A timestamp in nanoseconds since Unix epoch
pub type Timestamp = u64;

/// A context for telemetry correlation and propagation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Context {
    /// The trace identifier
    pub trace_id: TraceId,
    /// The span identifier
    pub span_id: SpanId,
    /// The correlation identifier
    pub correlation_id: Option<CorrelationId>,
}

impl Context {
    /// Creates a new root context with a new trace ID
    pub fn new_root() -> Self {
        let trace_id = [0u8; 16];
        let span_id = [0u8; 8];
        Self {
            trace_id,
            span_id,
            correlation_id: None,
        }
    }

    /// Creates a child context with the same trace ID but a new span ID
    pub fn new_child(&self) -> Self {
        let span_id = [0u8; 8]; // In a real implementation, this would be generated
        Self {
            trace_id: self.trace_id,
            span_id,
            correlation_id: self.correlation_id.clone(),
        }
    }

    /// Creates a context with a correlation ID
    pub fn with_correlation_id(&self, correlation_id: CorrelationId) -> Self {
        Self {
            correlation_id: Some(correlation_id),
            ..self.clone()
        }
    }
}

/// A resource represents the entity producing telemetry
#[derive(Debug, Clone, PartialEq)]
pub struct Resource {
    /// The resource attributes
    pub attributes: HashMap<String, AttributeValue>,
}

impl Resource {
    /// Creates a new resource with no attributes
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }

    /// Creates a new resource with the given attributes
    pub fn with_attributes(attributes: HashMap<String, AttributeValue>) -> Self {
        Self { attributes }
    }

    /// Merges this resource with another resource
    pub fn merge_with(&self, other: &Resource) -> Self {
        let mut merged = self.attributes.clone();
        for (key, value) in &other.attributes {
            merged.insert(key.clone(), value.clone());
        }
        Self { attributes: merged }
    }
}

/// An instrumentation scope represents the library or component that generated telemetry
#[derive(Debug, Clone, PartialEq)]
pub struct InstrumentationScope {
    /// The instrumentation scope name
    pub name: String,
    /// The instrumentation scope version
    pub version: Option<String>,
}

impl InstrumentationScope {
    /// Creates a new instrumentation scope with the given name
    pub fn new(name: String) -> Self {
        Self {
            name,
            version: None,
        }
    }

    /// Creates a new instrumentation scope with the given name and version
    pub fn with_version(name: String, version: String) -> Self {
        Self {
            name,
            version: Some(version),
        }
    }
}

/// An attribute value that can be used in telemetry
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    /// A string attribute value
    String(String),
    /// A boolean attribute value
    Bool(bool),
    /// An i64 attribute value
    I64(i64),
    /// An f64 attribute value
    F64(f64),
    /// An array of string attribute values
    StringArray(Vec<String>),
    /// An array of boolean attribute values
    BoolArray(Vec<bool>),
    /// An array of i64 attribute values
    I64Array(Vec<i64>),
    /// An array of f64 attribute values
    F64Array(Vec<f64>),
}

/// A span represents a single operation within a trace
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    /// The span name
    pub name: String,
    /// The span attributes
    pub attributes: HashMap<String, AttributeValue>,
    /// The context for the span
    pub context: Context,
    /// The resource for the span
    pub resource: Resource,
    /// The instrumentation scope for the span
    pub instrumentation_scope: InstrumentationScope,
    /// The start time of the span
    pub start_time: Timestamp,
    /// The end time of the span
    pub end_time: Option<Timestamp>,
    /// The status of the span
    pub status: Option<SpanStatus>,
}

impl Span {
    /// Creates a new span
    pub fn new(
        context: Context,
        resource: Resource,
        instrumentation_scope: InstrumentationScope,
        name: String,
        start_time: Timestamp,
    ) -> Self {
        Self {
            context,
            resource,
            instrumentation_scope,
            name,
            start_time,
            end_time: None,
            attributes: HashMap::new(),
            status: None,
        }
    }
}

/// The status of a span
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpanStatus {
    /// The span has completed successfully
    Ok,
    /// The span has completed with an error
    Error(String),
}

/// A log record represents a single log entry
#[derive(Debug, Clone, PartialEq)]
pub struct LogRecord {
    /// The log record body
    pub body: String,
    /// The log record attributes
    pub attributes: HashMap<String, AttributeValue>,
    /// The context for the log record
    pub context: Context,
    /// The resource for the log record
    pub resource: Resource,
    /// The instrumentation scope for the log record
    pub instrumentation_scope: InstrumentationScope,
    /// The timestamp of the log record
    pub timestamp: Timestamp,
    /// The severity of the log record
    pub severity: LogSeverity,
}

impl LogRecord {
    /// Creates a new log record
    pub fn new(
        context: Context,
        resource: Resource,
        instrumentation_scope: InstrumentationScope,
        timestamp: Timestamp,
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
}

/// The severity of a log record
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogSeverity {
    /// Trace severity
    Trace,
    /// Debug severity
    Debug,
    /// Info severity
    Info,
    /// Warn severity
    Warn,
    /// Error severity
    Error,
    /// Fatal severity
    Fatal,
}

/// A metric represents a single metric value
#[derive(Debug, Clone, PartialEq)]
pub struct Metric {
    /// The metric name
    pub name: String,
    /// The metric attributes
    pub attributes: HashMap<String, AttributeValue>,
    /// The resource for the metric
    pub resource: Resource,
    /// The instrumentation scope for the metric
    pub instrumentation_scope: InstrumentationScope,
    /// The description of the metric
    pub description: String,
    /// The unit of the metric
    pub unit: String,
}

impl Metric {
    /// Creates a new metric
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
}
