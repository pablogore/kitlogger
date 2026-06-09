# Feature Specification: KIT-001 Foundational Observability Abstractions

**Feature Branch**: `001-kit-foundational`
**Created**: 2026-06-09
**Status**: Draft
**Input**: Foundational observability abstractions — define the core data model for traces, logs, and metrics that is backend agnostic, vendor neutral, domain agnostic, and OpenTelemetry compatible.

## Overview

This feature defines the foundational observability abstractions for the Kit framework. It establishes a core data model for telemetry (traces, logs, metrics) that is:

- **Backend agnostic** — not coupled to any specific observability backend (console, file, network, OTLP, etc.)
- **Vendor neutral** — no dependency on or bias toward any vendor SDK
- **Domain agnostic** — the observability core does not encode business concepts; domain-specific attributes are carried in generic attribute maps
- **OpenTelemetry compatible** — structurally compatible with future OpenTelemetry adapters without requiring breaking API changes

The observability core intentionally excludes implementation details for exporters, sampling, batching, wire protocols, and macro expansion — those are the responsibility of subsequent feature work. This feature delivers the data model and creation API only.

## User Scenarios & Testing _(mandatory)_

### User Story 1 — Instrument Application Code With Structured Telemetry (Priority: P1)

As a developer using the Kit framework, I want to create and emit traces, logs, and metrics using a unified, domain-agnostic API so that I can observe the behaviour of my application without coupling to any specific backend or vendor.

**Why this priority**: The ability to produce structured telemetry is the foundational capability. Without it, no downstream observation, export, or analysis is possible.

**Independent Test**: A standalone test creates a span, attaches a log record to it, records a metric, and verifies that all three telemetry signals carry the expected context (trace_id, span_id, resource metadata) and arbitrary attributes — all without configuring any exporter or backend.

**Acceptance Scenarios**:

1. **Given** a new trace context, **When** a span is created with a name and attributes, **Then** the span contains a valid `trace_id`, `span_id`, the provided attributes, and is associated with the active `Resource`.
2. **Given** a span context, **When** a log record is emitted through the logging abstraction, **Then** the log record carries the `trace_id`, `span_id`, optional `correlation_id`, a timestamp, severity level, message body, and arbitrary attributes.
3. **Given** a configured metric instrument, **When** a value is recorded, **Then** the metric datapoint carries the instrument type (Counter, Gauge, Histogram, or UpDownCounter), the recorded value, attributes, and the associated `Resource`.
4. **Given** an application that uses all three telemetry signals, **When** the application runs, **Then** no backend or exporter configuration is required — the core abstractions operate without external dependencies.

---

### User Story 2 — Attach Resource Metadata to Telemetry (Priority: P1)

As a platform engineer, I want every telemetry record to carry metadata about the running service instance (service name, version, deployment environment, host, etc.) so that I can filter, route, and correlate observability data across a distributed system without hardcoding infrastructure provider details.

**Why this priority**: Resource association is fundamental to distributed observability. Without it, telemetry from different services and environments is indistinguishable.

**Independent Test**: A test creates a Resource with custom attributes (service.name, deployment.environment, cloud.region), creates a span and a log record associated with that Resource, and verifies that both carry the Resource attributes — without any infrastructure provider being hardcoded.

**Acceptance Scenarios**:

1. **Given** a `Resource` instance with attributes `{service.name: "api-gateway", deployment.environment: "production"}`, **When** a span is created and associated with that resource, **Then** the span's associated resource contains both attributes.
2. **Given** a `Resource` instance with arbitrary custom attributes (e.g., `datacenter: "us-east-1a"`), **When** telemetry is produced, **Then** the custom attributes are preserved without validation or rejection.
3. **Given** no explicit Resource is provided, **When** telemetry is produced, **Then** a default Resource with at minimum `service.name` is used.

---

### User Story 3 — Correlate Telemetry Across Systems via Correlation ID (Priority: P2)

As an operator debugging a cross-service workflow, I want to set a `correlation_id` on telemetry records independently of tracing so that I can correlate logs, metrics, events, and messages across service boundaries without requiring an active trace context or relying on business-specific identifiers.

**Why this priority**: Correlation identifiers are widely used across logs, traces, events, workflows, and messaging systems. Making them a first-class concept that works independently of tracing ensures consistent cross-system correlation without domain coupling.

**Independent Test**: A test creates a log record and a metric datapoint with a `correlation_id` but no trace context, and verifies that both carry the `correlation_id` correctly. A second test verifies that when a trace context is also present, both the correlation_id and trace identifiers are preserved.

**Acceptance Scenarios**:

1. **Given** no active trace context, **When** a log record is emitted with `correlation_id = "corr-abc-123"`, **Then** the log record carries `correlation_id = "corr-abc-123"` and has no `trace_id` or `span_id`.
2. **Given** no active trace context, **When** a metric value is recorded with `correlation_id = "corr-abc-123"`, **Then** the metric datapoint carries `correlation_id = "corr-abc-123"`.
3. **Given** an active trace context with `trace_id = "trc-xyz"` and a `correlation_id = "corr-abc-123"`, **When** a span is created, **Then** the span carries both `trace_id = "trc-xyz"` and `correlation_id = "corr-abc-123"`.
4. **Given** a telemetry record without a `correlation_id`, **Then** the `correlation_id` field is absent (no default value is injected).

---

### User Story 4 — Use All Four Metric Instrument Types (Priority: P2)

As a developer monitoring application behaviour, I want to use Counter, Gauge, Histogram, and UpDownCounter instruments so that I can accurately model additive, point-in-time, distributional, and incrementally-varying measurements.

**Why this priority**: Four instrument types cover the standard observability modelling patterns. Adding UpDownCounter (a counter that can increase or decrease) fills a gap in the original model.

**Independent Test**: A test creates one instrument of each type (Counter, Gauge, Histogram, UpDownCounter), records values on each, and verifies that each instrument preserves the correct semantic type and recorded data points.

**Acceptance Scenarios**:

1. **Given** a Counter instrument, **When** a value is recorded, **Then** the value is added to the counter's monotonic sum.
2. **Given** a Gauge instrument, **When** a value is recorded, **Then** the gauge records the current value as a point-in-time sample.
3. **Given** a Histogram instrument, **When** a value is recorded, **Then** the histogram records the value in its distribution of observations.
4. **Given** an UpDownCounter instrument, **When** a positive or negative value is recorded, **Then** the up-down counter adjusts its non-monotonic sum accordingly.

---

### User Story 5 — Attribute Telemetry to an Instrumentation Scope (Priority: P3)

As a library author integrating with Kit, I want to tag the telemetry my library produces with an `InstrumentationScope` (e.g., "auth", "persistence", "runtime", "scheduler", "workflow") so that operators can identify the source component of each telemetry record without inspecting application code.

**Why this priority**: Scope attribution enables fine-grained filtering and cost allocation. It is a standard concept in OpenTelemetry and essential for library instrumentation.

**Independent Test**: A test creates two instrumentation scopes ("auth" and "persistence"), emits a span under each, and verifies that each span carries the correct scope name.

**Acceptance Scenarios**:

1. **Given** an `InstrumentationScope` with name "auth" and version "1.0.0", **When** a span is created under that scope, **Then** the span carries the scope name "auth" and version "1.0.0".
2. **Given** an `InstrumentationScope` with only a name, **When** a log record is emitted under that scope, **Then** the log record carries the scope name (version is optional).
3. **Given** no explicit scope is provided, **When** telemetry is produced, **Then** a default scope (e.g., name: "unknown") is assigned.

---

### User Story 6 — Macro-Based Instrumentation (Priority: P3)

As a developer writing instrumentation code, I want to use concise macros (`kit_trace!`, `kit_debug!`, `kit_info!`, `kit_warn!`, `kit_error!`) so that common telemetry operations are ergonomic and reduce boilerplate compared to manual API calls.

**Why this priority**: Macros are a first-class user experience expectation. The core design must be compatible with future macro definitions without requiring API redesign.

**Independent Test**: A test exercises the underlying core API in the same pattern that each macro would expand to, and verifies the resulting telemetry is structurally identical to what a macro would produce — proving the core API supports macro expansion without modification.

**Acceptance Scenarios**:

1. **Given** a macro invocation such as `kit_info!("user registered", user.id = "abc")`, **When** expanded, **Then** the resulting log record carries severity INFO, the message "user registered", and the attribute `user.id = "abc"` in the attributes map.
2. **Given** a macro invocation within an active span, **When** expanded, **Then** the resulting telemetry record inherits the active `trace_id`, `span_id`, and `correlation_id`.
3. **Given** a macro invocation outside any active context, **When** expanded, **Then** the resulting telemetry record is created without a trace context (no `trace_id` or `span_id`), but still carries any explicitly provided attributes and correlation_id.
4. **Given** a macro that records a metric value, **When** expanded, **Then** the metric datapoint preserves the instrument type, value, and attributes exactly as provided to the macro.

---

### User Story 7 — Async Compatibility for Concurrent Runtimes (Priority: P3)

As a developer using an async runtime, I want to create and propagate trace contexts across concurrent tasks and async boundaries so that observability works correctly in asynchronous and concurrent execution environments.

**Why this priority**: Async runtimes and concurrent execution models are the dominant paradigm in modern services. The abstractions must not assume synchronous, single-threaded execution.

**Independent Test**: A test spawns multiple concurrent async tasks, creates a parent trace context, propagates it into each task, and verifies that child spans within each task correctly reference the parent trace and maintain independent span_ids.

**Acceptance Scenarios**:

1. **Given** an active trace context, **When** a new async task is spawned, **Then** the context (trace_id, span_id, correlation_id, attributes) is correctly propagated to the task.
2. **Given** multiple concurrent async tasks sharing the same trace_id, **When** each task creates its own child span, **Then** each child span has a unique span_id and all share the same trace_id.
3. **Given** an async runtime, **When** context is not explicitly propagated, **Then** no context leaks across tasks (isolation is maintained).

### Edge Cases

- **Empty attributes**: When a Context, Resource, or InstrumentationScope is created with an empty or null attributes map, the system must accept it gracefully and produce valid telemetry.
- **Missing trace context**: When creating a span without an active parent context, the system must generate a new trace_id and span_id (root span behaviour).
- **Concurrent scope access**: When multiple threads or tasks read/write context attributes concurrently, behaviour must be well-defined (immutable snapshots or explicit clone semantics).
- **High-cardinality attributes**: The core must not enforce cardinality limits on attribute values; limits are the responsibility of exporters.
- **Resource attribute conflicts**: When a telemetry record has an attribute with the same key as a Resource attribute, the telemetry-record-level attribute takes precedence for that specific record.
- **Correlation ID without tracing**: A correlation_id must be usable on any telemetry record without requiring a trace context, and must not imply or require trace_id or span_id generation.

## Requirements _(mandatory)_

### Non-Functional Requirements

- **NFR-001**: All core abstractions (Context, Resource, InstrumentationScope, Span, LogRecord, Metric) MUST be backend agnostic — they must not reference any specific exporter, transport, or storage format.
- **NFR-002**: The observability core MUST remain vendor neutral — no SDK or API dependency on any vendor's libraries.
- **NFR-003**: The observability core MUST remain domain agnostic — it must not define or reserve domain-specific fields such as `tenant_id`, `user_id`, `organization_id`, `customer_id`, `country`, `order_id`, or `workflow_id` as first-class fields. These MUST be represented as generic attributes.
- **NFR-004**: The core abstractions MUST be structurally compatible with future OpenTelemetry adapters without requiring breaking API changes. The data model (Context, Resource, InstrumentationScope, Span, LogRecord, Metric) must map cleanly to the OpenTelemetry specification's data model.
- **NFR-005**: No OpenTelemetry SDK dependencies or protocol dependencies (e.g., OTLP protobufs) may be introduced into the core.
- **NFR-006**: The observability abstractions MUST be usable in async runtimes and concurrent execution environments. Context propagation must support non-blocking, concurrent, and task-based execution models. No synchronous-only assumptions may be encoded.
- **NFR-007**: The core MUST remain runtime agnostic — no dependency on any specific async runtime is permitted.
- **NFR-008**: The core design MUST remain compatible with future macro-based instrumentation (e.g., `kit_trace!`, `kit_debug!`, `kit_info!`, `kit_warn!`, `kit_error!`) without requiring core API changes. Macros are defined as a first-class user experience expectation.

### Functional Requirements

#### Context Model

- **FR-001**: Context MUST contain `trace_id`, `span_id`, optional `correlation_id`, and an `attributes` collection supporting arbitrary key-value pairs.
- **FR-002**: Context MUST support creating child contexts that inherit the parent's `trace_id`, `correlation_id`, and attributes.
- **FR-003**: Context MUST support creating a root context (no parent) that generates a new `trace_id` and `span_id`.
- **FR-004**: `correlation_id` MUST be a first-class, optional field on Context. It is a string identifier for correlating telemetry across service, system, and workflow boundaries.
- **FR-005**: The `attributes` collection MUST support arbitrary key-value pairs. The framework MUST NOT define, reserve, or validate specific attribute keys. Examples of valid attributes (illustrative only, not a prescriptive schema): `tenant.id`, `user.id`, `organization.id`, `workflow.id`, `country`, `region`.

#### Resource Model

- **FR-006**: A `Resource` entity MUST represent metadata describing the running service instance.
- **FR-007**: Resource MUST support arbitrary key-value attributes. Examples of valid resource attributes (illustrative only): `service.name`, `service.version`, `deployment.environment`, `host.name`, `host.id`, `container.id`, `cloud.provider`, `cloud.region`.
- **FR-008**: The framework MUST NOT hardcode or require any specific infrastructure provider attributes. All infrastructure attributes are user-defined. The framework remains infrastructure agnostic.
- **FR-009**: Spans, LogRecords, and Metrics MUST support association with a `Resource`.

#### Correlation Support

- **FR-010**: `correlation_id` MUST be supportable independently of a trace context. A LogRecord or Metric datapoint may carry a `correlation_id` without a `trace_id` or `span_id`, and a Context may carry a `correlation_id` without a trace context.
- **FR-011**: When a Context carries both a trace context and a `correlation_id`, telemetry created from that Context MUST inherit both.

#### Telemetry Signals

- **FR-012**: A `Span` MUST carry a name, `trace_id`, `span_id`, optional parent `span_id`, timestamps (start, end), status, attributes, an associated `Resource`, and an associated `InstrumentationScope`.
- **FR-013**: A `LogRecord` MUST carry a timestamp, severity level, message body, optional `trace_id`, optional `span_id`, optional `correlation_id`, attributes, an associated `Resource`, and an associated `InstrumentationScope`.
- **FR-014**: A `Metric` datapoint MUST carry an instrument name, instrument type, recorded value(s), timestamp, attributes, an associated `Resource`, and an associated `InstrumentationScope`.

#### Metric Instruments

- **FR-015**: The framework MUST support four metric instrument types: `Counter`, `Gauge`, `Histogram`, and `UpDownCounter`.
- **FR-016**: A `Counter` MUST support only additive (positive) increments and expose a monotonic sum.
- **FR-017**: An `UpDownCounter` MUST support both positive and negative increments, exposing a non-monotonic sum.
- **FR-018**: A `Gauge` MUST support recording point-in-time values, replacing the previous value on each recording.
- **FR-019**: A `Histogram` MUST support recording observations into a distribution, capturing count, sum, min, max, and configurable bucket boundaries.

#### InstrumentationScope

- **FR-020**: An `InstrumentationScope` entity MUST represent the component or library producing telemetry.
- **FR-021**: InstrumentationScope MUST carry a `name` (string) and an optional `version` (string).
- **FR-022**: InstrumentationScope MUST remain generic — it must not be tied to any specific framework, library, or internal component.
- **FR-023**: All telemetry signals (Spans, LogRecords, Metrics) MUST carry an associated InstrumentationScope.

### Key Entities

- **Context**: Carries trace context (`trace_id`, `span_id`), optional cross-system `correlation_id`, and arbitrary `attributes`. Used as the foundation for all telemetry creation. Supports root context generation and child context inheritance.
- **Resource**: Metadata describing the running service instance. Carries arbitrary attributes (e.g., service.name, service.version, deployment.environment, host.name, cloud.provider). Associated with every telemetry record. The framework remains infrastructure agnostic — no provider attributes are hardcoded.
- **InstrumentationScope**: Identifies the component or library that produced the telemetry (e.g., "auth", "runtime", "persistence", "workflow", "scheduler"). Remains generic and not tied to any specific framework internals. Carries a name and optional version.
- **Span**: Represents a single unit of work in a distributed trace. Carries trace context, timing, status, attributes, Resource, and InstrumentationScope.
- **LogRecord**: Represents a single structured log entry. Carries severity, message, optional trace context, optional correlation_id, attributes, Resource, and InstrumentationScope.
- **Metric**: Represents a measured value or distribution. Carries instrument type (Counter, Gauge, Histogram, UpDownCounter), values, attributes, Resource, and InstrumentationScope.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: A developer can create a root context, create child spans, emit log records, and record metrics using all four instrument types — all without configuring any exporter or backend. Verified by a test that exercises all core abstractions without external transport or storage.
- **SC-002**: A test can verify that no domain-specific fields (`tenant_id`, `user_id`, `organization_id`, `customer_id`, `country`, `order_id`, `workflow_id`) exist as first-class properties on any core entity. All such values are carried in the generic attributes map.
- **SC-003**: A `Resource` can be created with arbitrary attributes, associated with a span, a log record, and a metric datapoint, and the associated resource attributes are retrievable from each telemetry signal. Verified by a single test that exercises all three associations.
- **SC-004**: A `correlation_id` can be attached to a log record or metric datapoint without an active trace context. Verified by a test that creates telemetry with a correlation_id but no trace context, and asserts the correlation_id is present while trace_id and span_id are absent.
- **SC-005**: All four metric instrument types (Counter, Gauge, Histogram, UpDownCounter) can be created and exercised through the core API without any backend or exporter configured. Verified by a test that records values on each instrument type and reads back the recorded data.
- **SC-006**: The core API surface can be used in the pattern that macro expansion would produce (e.g., create context, create span/log/metric via the API with the same parameters a macro would supply). Verified by a test that exercises each macro-equivalent API pattern and validates the resulting telemetry structure.
- **SC-007**: The core abstractions compile and operate correctly in a multi-threaded or async execution context. Verified by a test that creates and propagates contexts across concurrent tasks without data races or context leaks.
- **SC-008**: The core API surface (Context, Resource, InstrumentationScope, Span, LogRecord, Metric, Counter, Gauge, Histogram, UpDownCounter) maps cleanly to the OpenTelemetry specification's data model without requiring any OpenTelemetry SDK dependency. Verified by a mapping review against the OpenTelemetry specification.

## Assumptions

- The core abstractions are library/types only — no export, transport, sampling, batching, or macro expansion logic is included in this feature. Those concerns belong to subsequent feature work.
- The concrete programming language for implementation is determined by the project's broader technology choices and is not constrained by this specification.
- Macro implementation is a separate feature. This specification defines only the compatibility requirement: the core API must support the shapes that macro expansion would produce.
- The default Resource (when none is explicitly provided) will use minimal reasonable defaults such as `service.name` derived from the application or environment context.
- The core defines the data model and creation API. Thread safety and context propagation mechanisms are implementation concerns that must satisfy the async compatibility requirement (NFR-006).
- Correlation identifiers are free-form strings. No format, length, or uniqueness constraints are imposed by the core — validation (if any) belongs in specific adapter or exporter implementations.
