# Kit Framework Observability Contracts

This document defines the interface contracts for the Kit framework's observability abstractions. These contracts are designed to be backend-agnostic, vendor-neutral, and OpenTelemetry compatible.

## Context Contract

### Interface
```rust
struct Context {
    trace_id: TraceId,
    span_id: SpanId,
    correlation_id: Option<String>,
    attributes: HashMap<String, String>,
}
```

### Operations
- `create_root_context() -> Context`: Create a new root context with fresh trace and span IDs
- `create_child_context(parent: &Context) -> Context`: Create a child context inheriting trace_id, correlation_id, and attributes
- `with_correlation_id(self, correlation_id: String) -> Context`: Create a new context with correlation_id

**Implementation Notes:**
The specification does not mandate any specific context propagation mechanism. Implementations MAY use thread-local storage, task-local storage, explicit propagation, or other strategies. The core contract MUST be compatible with async runtimes and concurrent execution.

## Resource Contract

### Interface
```rust
struct Resource {
    attributes: HashMap<String, String>,
}
```

### Operations
- `new(attributes: HashMap<String, String>) -> Resource`: Create a new Resource with given attributes
- `merge_with(self, other: &Resource) -> Resource`: Merge this resource with another resource

**Attribute Resolution:**
The specification does not require resource attributes and telemetry attributes to be merged into a single attribute collection. Both MUST remain independently accessible. The core MUST NOT mandate merge behavior. Exporters, adapters, SDKs, or future runtime implementations MAY define projection or merge strategies appropriate for their backend.

## InstrumentationScope Contract

### Interface
```rust
struct InstrumentationScope {
    name: String,
    version: Option<String>,
}
```

### Operations
- `new(name: String, version: Option<String>) -> InstrumentationScope`: Create a new InstrumentationScope

**Default Behavior:**
When no explicit scope is provided, the default scope name MUST be "unknown" with no version specified.

## Span Contract

### Interface
```rust
struct Span {
    name: String,
    trace_id: TraceId,
    span_id: SpanId,
    parent_span_id: Option<SpanId>,
    start_time: Timestamp,
    end_time: Option<Timestamp>,
    status: SpanStatus,
    attributes: HashMap<String, String>,
    resource: Resource,
    instrumentation_scope: InstrumentationScope,
}
```

### Operations
- `start(name: String, context: &Context, resource: &Resource, scope: &InstrumentationScope) -> Span`: Create a new span
- `end(&mut self)`: End the span
- `add_attribute(&mut self, key: String, value: String)`: Add an attribute to the span
- `record_event(&mut self, name: String, timestamp: Timestamp)`: Record an event in the span

## LogRecord Contract

### Interface
```rust
struct LogRecord {
    timestamp: Timestamp,
    severity: Severity,
    message: String,
    trace_id: Option<TraceId>,
    span_id: Option<SpanId>,
    correlation_id: Option<String>,
    attributes: HashMap<String, String>,
    resource: Resource,
    instrumentation_scope: InstrumentationScope,
}
```

### Operations
- `emit(message: String, context: &Context, resource: &Resource, scope: &InstrumentationScope, severity: Severity) -> LogRecord`: Create and emit a log record

## Metric Contract

### Interface
```rust
enum InstrumentType {
    Counter,
    Gauge,
    Histogram,
    UpDownCounter,
}

struct Metric {
    instrument_name: String,
    instrument_type: InstrumentType,
    values: Vec<MetricValue>,
    timestamp: Timestamp,
    attributes: HashMap<String, String>,
    resource: Resource,
    instrumentation_scope: InstrumentationScope,
}

enum MetricValue {
    CounterValue(f64),
    GaugeValue(f64),
    HistogramValue(HistogramData),
    UpDownCounterValue(f64),
}

struct HistogramData {
    count: u64,
    sum: f64,
    min: f64,
    max: f64,
    bucket_boundaries: Vec<f64>,
    bucket_counts: Vec<u64>,
}
```

### Operations
- `create_counter(name: String, resource: &Resource, scope: &InstrumentationScope) -> Counter`: Create a counter instrument
- `create_gauge(name: String, resource: &Resource, scope: &InstrumentationScope) -> Gauge`: Create a gauge instrument
- `create_histogram(name: String, resource: &Resource, scope: &InstrumentationScope) -> Histogram`: Create a histogram instrument
- `create_up_down_counter(name: String, resource: &Resource, scope: &InstrumentationScope) -> UpDownCounter`: Create an up-down counter instrument
- `record_counter(counter: &mut Counter, value: f64)`: Record a value in a counter
- `record_gauge(gauge: &mut Gauge, value: f64)`: Record a value in a gauge
- `record_histogram(histogram: &mut Histogram, value: f64)`: Record a value in a histogram
- `record_up_down_counter(up_down_counter: &mut UpDownCounter, value: f64)`: Record a value in an up-down counter

**Semantic Requirements:**
- Counter: Monotonic, accepts only positive increments, represents cumulative increasing value
- UpDownCounter: Non-monotonic, accepts positive and negative increments
- Histogram: Records observations into a statistical distribution, supports count, sum, min, max, and bucketed aggregation semantics
- Gauge: Represents a point-in-time value, most recently recorded value is current observation

**Implementation Strategy:**
The specific implementation strategy for these instruments (including locking, concurrent access, atomic operations, memory layout, aggregation algorithms, etc.) is implementation-defined and outside the scope of KIT-001.

## NoOp Implementations

### NoOpLogger Contract
```rust
struct NoOpLogger;
impl Logger for NoOpLogger {
    fn emit(&self, _record: LogRecord) {}
}
```

### NoOpTracer Contract
```rust
struct NoOpTracer;
impl Tracer for NoOpTracer {
    fn start_span(&self, _name: String, _context: &Context, _resource: &Resource, _scope: &InstrumentationScope) -> Span {
        // Return no-op span
    }
}
```

### NoOpMeter Contract
```rust
struct NoOpMeter;
impl Meter for NoOpMeter {
    fn create_counter(&self, _name: String, _resource: &Resource, _scope: &InstrumentationScope) -> Counter {
        // Return no-op counter
    }
    // Similar implementations for gauge, histogram, up_down_counter
}
```

**Implementation Details:**
NoOp implementations MUST be available by default and MUST accept all valid API calls without error. The specific instantiation and access patterns are implementation-specific and outside the scope of KIT-001.

## Compatibility with OpenTelemetry

The Kit framework's observability contracts are designed to be structurally compatible with the OpenTelemetry specification. The core data models map cleanly to OpenTelemetry's data model without requiring any OpenTelemetry SDK dependencies.