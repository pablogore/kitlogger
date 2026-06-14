# Data Model: KIT-001 Foundational Observability Abstractions

## Overview

This document defines the core data models for the Kit framework's observability abstractions. These models are designed to be backend-agnostic, vendor-neutral, domain-agnostic, and OpenTelemetry compatible.

## Core Entities

### Context
A Context carries trace context information, optional correlation identifiers, and arbitrary attributes. It serves as the foundation for all telemetry creation.

**Fields:**
- `trace_id`: Unique identifier for a trace
- `span_id`: Unique identifier for a span within a trace
- `correlation_id`: Optional string identifier for correlating telemetry across service, system, workflow, and request boundaries
- `attributes`: Collection of arbitrary key-value pairs

### Resource
A Resource represents metadata describing the running service instance. It carries arbitrary attributes that describe the service environment.

**Fields:**
- `attributes`: Collection of arbitrary key-value pairs describing the service instance

### InstrumentationScope
An InstrumentationScope identifies the logical component or library producing telemetry.

**Fields:**
- `name`: String identifier for the component/library
- `version`: Optional string identifier for the component/library version

**Default Behavior:**
When no explicit scope is provided, the default scope name MUST be "unknown" with no version specified.

### Span
A Span represents a single unit of work in a distributed trace.

**Fields:**
- `name`: String name of the span
- `trace_id`: Unique identifier for the trace
- `span_id`: Unique identifier for the span
- `parent_span_id`: Optional identifier for the parent span
- `start_time`: Timestamp when the span started
- `end_time`: Timestamp when the span ended
- `status`: Status of the span (e.g., OK, ERROR)
- `attributes`: Collection of arbitrary key-value pairs
- `resource`: Associated Resource entity
- `instrumentation_scope`: Associated InstrumentationScope entity

### LogRecord
A LogRecord represents a single structured log entry.

**Fields:**
- `timestamp`: Timestamp of the log record
- `severity`: Severity level (e.g., DEBUG, INFO, WARN, ERROR)
- `message`: Log message body
- `trace_id`: Optional trace identifier
- `span_id`: Optional span identifier
- `correlation_id`: Optional correlation identifier
- `attributes`: Collection of arbitrary key-value pairs
- `resource`: Associated Resource entity
- `instrumentation_scope`: Associated InstrumentationScope entity

### Metric
A Metric represents a measured value or distribution.

**Fields:**
- `instrument_name`: Name of the instrument
- `instrument_type`: Type of instrument (Counter, Gauge, Histogram, UpDownCounter)
- `values`: Recorded value(s) for the metric
- `timestamp`: Timestamp when the value was recorded
- `attributes`: Collection of arbitrary key-value pairs
- `resource`: Associated Resource entity
- `instrumentation_scope`: Associated InstrumentationScope entity

## Metric Instrument Types

### Counter
A Counter supports only additive (positive) increments and exposes a monotonic sum.

### Gauge
A Gauge supports recording point-in-time values, replacing the previous value on each recording.

### Histogram
A Histogram supports recording observations into a distribution, capturing count, sum, min, max, and configurable bucket boundaries.

### UpDownCounter
An UpDownCounter supports both positive and negative increments, exposing a non-monotonic sum.

**Semantic Requirements:**
- Counter: Monotonic, accepts only positive increments, represents cumulative increasing value
- UpDownCounter: Non-monotonic, accepts positive and negative increments
- Histogram: Records observations into a statistical distribution, supports count, sum, min, max, and bucketed aggregation semantics
- Gauge: Represents a point-in-time value, most recently recorded value is current observation

**Implementation Strategy:**
The specific implementation strategy for these instruments (including locking, concurrent access, atomic operations, memory layout, aggregation algorithms, etc.) is implementation-defined and outside the scope of KIT-001.

## NoOp Implementations

The framework provides NoOp implementations for all core abstractions:
- NoOpLogger: Accepts all log record emission requests without error and silently discards them
- NoOpTracer: Accepts all span creation requests without error and returns no-op spans that silently discard all operations
- NoOpMeter: Accepts all metric instrument creation and recording requests without error and silently discards all values

**Implementation Details:**
NoOp implementations MUST be available by default and MUST accept all valid API calls without error. The specific instantiation and access patterns are implementation-specific and outside the scope of KIT-001.

## Validation Rules

- All core entities must be backend agnostic
- No SDK or API dependencies on any vendor's libraries
- No domain-specific fields as first-class properties
- All attributes collections must support arbitrary key-value pairs
- Resource attributes must be user-defined, not hardcoded infrastructure provider attributes
- Correlation_id must be usable independently of trace context

## Attribute Data Model

Attribute values MUST support a typed value model compatible with OpenTelemetry-style attributes.

Minimum supported types:
- String
- Bool
- I64
- F64

Additionally, the model SHOULD support homogeneous arrays of primitive values:
- Vec<String>
- Vec<bool>
- Vec<i64>
- Vec<f64>

The core MUST NOT support arbitrary nested maps or arbitrary JSON structures as first-class attribute values.