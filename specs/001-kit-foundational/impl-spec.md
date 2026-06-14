# Implementation Specification: KIT-001 Foundational Observability Abstractions

**Feature Branch**: `001-kit-foundational`
**Created**: 2026-06-09
**Status**: Draft
**Input**: Foundational observability abstractions — define the core data model for traces, logs, and metrics that is backend agnostic, vendor neutral, domain agnostic, and OpenTelemetry compatible.

## 1. Public Data Model

### Core Entities

#### Context
```rust
struct Context {
    trace_id: TraceId,
    span_id: SpanId,
    correlation_id: Option<String>,
    attributes: HashMap<String, AttributeValue>,
}
```

#### Resource
```rust
struct Resource {
    attributes: HashMap<String, AttributeValue>,
}
```

#### InstrumentationScope
```rust
struct InstrumentationScope {
    name: String,
    version: Option<String>,
}
```

#### Span
```rust
struct Span {
    name: String,
    trace_id: TraceId,
    span_id: SpanId,
    parent_span_id: Option<SpanId>,
    start_time: Timestamp,
    end_time: Option<Timestamp>,
    status: SpanStatus,
    attributes: HashMap<String, AttributeValue>,
    resource: Resource,
    instrumentation_scope: InstrumentationScope,
}
```

#### LogRecord
```rust
struct LogRecord {
    timestamp: Timestamp,
    severity: Severity,
    message: String,
    trace_id: Option<TraceId>,
    span_id: Option<SpanId>,
    correlation_id: Option<String>,
    attributes: HashMap<String, AttributeValue>,
    resource: Resource,
    instrumentation_scope: InstrumentationScope,
}
```

#### Metric
```rust
struct Metric {
    instrument_name: String,
    instrument_type: InstrumentType,
    values: Vec<MetricValue>,
    timestamp: Timestamp,
    attributes: HashMap<String, AttributeValue>,
    resource: Resource,
    instrumentation_scope: InstrumentationScope,
}
```

#### AttributeValue
```rust
enum AttributeValue {
    String(String),
    Bool(bool),
    I64(i64),
    F64(f64),
    StringArray(Vec<String>),
    BoolArray(Vec<bool>),
    I64Array(Vec<i64>),
    F64Array(Vec<f64>),
}
```

#### InstrumentType
```rust
enum InstrumentType {
    Counter,
    Gauge,
    Histogram,
    UpDownCounter,
}
```

#### MetricValue
```rust
enum MetricValue {
    CounterValue(f64),
    GaugeValue(f64),
    HistogramValue(HistogramData),
    UpDownCounterValue(f64),
}
```

#### HistogramData
```rust
struct HistogramData {
    count: u64,
    sum: f64,
    min: f64,
    max: f64,
    bucket_boundaries: Vec<f64>,
    bucket_counts: Vec<u64>,
}
```

## 2. Public Traits and Abstraction Boundaries

### Logger Trait
```rust
trait Logger {
    fn emit(&self, record: LogRecord);
}
```

### Tracer Trait
```rust
trait Tracer {
    fn start_span(
        &self, 
        name: String, 
        context: &Context, 
        resource: &Resource, 
        scope: &InstrumentationScope
    ) -> Span;
}
```

### Meter Trait
```rust
trait Meter {
    fn create_counter(
        &self, 
        name: String, 
        resource: &Resource, 
        scope: &InstrumentationScope
    ) -> Counter;
    
    fn create_gauge(
        &self, 
        name: String, 
        resource: &Resource, 
        scope: &InstrumentationScope
    ) -> Gauge;
    
    fn create_histogram(
        &self, 
        name: String, 
        resource: &Resource, 
        scope: &InstrumentationScope
    ) -> Histogram;
    
    fn create_up_down_counter(
        &self, 
        name: String, 
        resource: &Resource, 
        scope: &InstrumentationScope
    ) -> UpDownCounter;
}
```

### Counter Trait
```rust
trait Counter {
    fn record(&mut self, value: f64);
}
```

### Gauge Trait
```rust
trait Gauge {
    fn record(&mut self, value: f64);
}
```

### Histogram Trait
```rust
trait Histogram {
    fn record(&mut self, value: f64);
}
```

### UpDownCounter Trait
```rust
trait UpDownCounter {
    fn record(&mut self, value: f64);
}
```

## 3. Required Entities

### Context Operations
- `create_root_context() -> Context`: Create a new root context with fresh trace and span IDs
- `create_child_context(parent: &Context) -> Context`: Create a child context inheriting trace_id, correlation_id, and attributes
- `with_correlation_id(self, correlation_id: String) -> Context`: Create a new context with correlation_id

### Resource Operations
- `new(attributes: HashMap<String, AttributeValue>) -> Resource`: Create a new Resource with given attributes
- `merge_with(self, other: &Resource) -> Resource`: Merge this resource with another resource

### InstrumentationScope Operations
- `new(name: String, version: Option<String>) -> InstrumentationScope`: Create a new InstrumentationScope

### Span Operations
- `new(name: String, trace_id: TraceId, span_id: SpanId, parent_span_id: Option<SpanId>, start_time: Timestamp, attributes: HashMap<String, AttributeValue>, resource: Resource, instrumentation_scope: InstrumentationScope) -> Span`: Create a new Span

### LogRecord Operations
- `new(timestamp: Timestamp, severity: Severity, message: String, trace_id: Option<TraceId>, span_id: Option<SpanId>, correlation_id: Option<String>, attributes: HashMap<String, AttributeValue>, resource: Resource, instrumentation_scope: InstrumentationScope) -> LogRecord`: Create a new LogRecord

### Metric Operations
- `new(instrument_name: String, instrument_type: InstrumentType, values: Vec<MetricValue>, timestamp: Timestamp, attributes: HashMap<String, AttributeValue>, resource: Resource, instrumentation_scope: InstrumentationScope) -> Metric`: Create a new Metric

## 4. Validation Rules

- All core entities must be backend agnostic
- No SDK or API dependencies on any vendor's libraries
- No domain-specific fields as first-class properties
- All attributes collections must support arbitrary key-value pairs
- Resource attributes must be user-defined, not hardcoded infrastructure provider attributes
- Correlation_id must be usable independently of trace context
- Attribute values MUST support a typed value model compatible with OpenTelemetry-style attributes
- Minimum supported types: String, Bool, I64, F64
- Additionally, the model SHOULD support homogeneous arrays of primitive values: Vec<String>, Vec<bool>, Vec<i64>, Vec<f64>
- The core MUST NOT support arbitrary nested maps or arbitrary JSON structures as first-class attribute values

## 5. Error Handling Requirements

- All API operations must be safe and not panic
- Invalid attribute values should be rejected gracefully
- All operations must be thread-safe where applicable
- No operation should cause memory leaks or resource exhaustion
- All operations must be non-blocking where possible

## 6. NoOp Behavior Requirements

### NoOpLogger
```rust
struct NoOpLogger;
impl Logger for NoOpLogger {
    fn emit(&self, _record: LogRecord) {}
}
```

### NoOpTracer
```rust
struct NoOpTracer;
impl Tracer for NoOpTracer {
    fn start_span(&self, _name: String, _context: &Context, _resource: &Resource, _scope: &InstrumentationScope) -> Span {
        // Return no-op span
    }
}
```

### NoOpMeter
```rust
struct NoOpMeter;
impl Meter for NoOpMeter {
    fn create_counter(&self, _name: String, _resource: &Resource, _scope: &InstrumentationScope) -> Counter {
        // Return no-op counter
    }
    fn create_gauge(&self, _name: String, _resource: &Resource, _scope: &InstrumentationScope) -> Gauge {
        // Return no-op gauge
    }
    fn create_histogram(&self, _name: String, _resource: &Resource, _scope: &InstrumentationScope) -> Histogram {
        // Return no-op histogram
    }
    fn create_up_down_counter(&self, _name: String, _resource: &Resource, _scope: &InstrumentationScope) -> UpDownCounter {
        // Return no-op up-down counter
    }
}
```

NoOp implementations MUST be available by default and MUST accept all valid API calls without error. The specific instantiation and access patterns are implementation-specific and outside the scope of KIT-001.

## 7. OpenTelemetry Compatibility Requirements

The Kit framework's observability contracts are designed to be structurally compatible with the OpenTelemetry specification. The core data models map cleanly to OpenTelemetry's data model without requiring any OpenTelemetry SDK dependencies.

## 8. Test Scenarios and Acceptance Criteria

### Core Functionality Tests
1. **Context Creation**: Verify root context creation generates new trace and span IDs
2. **Context Inheritance**: Verify child context inherits trace_id, correlation_id, and attributes
3. **Resource Association**: Verify Resource attributes are correctly associated with telemetry signals
4. **InstrumentationScope**: Verify InstrumentationScope is mandatory and defaults to name "unknown" with no version when not explicitly provided
5. **Metric Instruments**: Verify all four instrument types (Counter, Gauge, Histogram, UpDownCounter) work correctly with OpenTelemetry-compatible semantics
6. **NoOp Implementations**: Verify all NoOp implementations accept all valid API calls without error
7. **Attribute Support**: Verify all supported attribute types (String, Bool, I64, F64, arrays) work correctly
8. **Correlation ID**: Verify correlation_id works independently of trace context

### Integration Tests
1. **End-to-End Flow**: Verify complete telemetry flow from context creation to log emission to metric recording
2. **Async Compatibility**: Verify context propagation works correctly in concurrent async environments
3. **NoOp Default**: Verify application compiles and runs without errors when no backend is configured

## 9. Rust-Specific API Expectations

- All public APIs must be thread-safe where applicable
- Use standard Rust idioms and conventions
- Leverage Rust's type system for compile-time safety
- Provide clear error handling through Result types where appropriate
- Use appropriate lifetimes and ownership patterns
- Follow Rust naming conventions (snake_case for functions, PascalCase for types)
- Provide documentation for all public APIs
- Use standard collections (HashMap, Vec) for data structures
- Support async operations where needed

## 10. Success Criteria

### Measurable Outcomes
- A developer can create a root context, create child spans, emit log records, and record metrics using all four instrument types — all without configuring any exporter or backend
- A test can verify that no domain-specific fields exist as first-class properties on any core entity
- A Resource can be created with arbitrary attributes, associated with a span, a log record, and a metric datapoint
- A correlation_id can be attached to a log record or metric datapoint without an active trace context
- All four metric instrument types can be created and exercised through the core API without any backend or exporter configured
- The core API surface can be used in the pattern that macro expansion would produce
- The core abstractions compile and operate correctly in a multi-threaded or async execution context
- The core API surface maps cleanly to the OpenTelemetry specification's data model
- An application linking only against the core compiles and runs without errors