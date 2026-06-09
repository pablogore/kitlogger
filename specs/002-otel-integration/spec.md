# Feature Specification: KIT-002 OpenTelemetry Integration

**Feature Branch**: `002-otel-integration`
**Created**: 2026-06-09
**Status**: Draft
**Input**: Implement OpenTelemetry integration for Kit Observability — provide interoperability between Kit's internal observability abstractions and the OpenTelemetry ecosystem without introducing OpenTelemetry dependencies into KIT-001.

## Overview

This feature provides interoperability between Kit's internal observability abstractions and the OpenTelemetry ecosystem. The integration allows Kit telemetry to be exported to any OTLP-compatible backend without introducing OpenTelemetry dependencies into KIT-001.

### Goals

- OpenTelemetry Trace Adapter
- OpenTelemetry Metric Adapter
- OpenTelemetry Log Adapter
- OTLP Export Support
- Resource Mapping
- Context Mapping
- InstrumentationScope Mapping

### Separation of Concerns

Maintain complete separation between:

- `kit-observability-core` (KIT-001)
- OpenTelemetry SDK

### Non-Goals

This feature does not implement:

- Prometheus exporter
- Loki exporter
- Tempo exporter
- Datadog exporter
- New Relic exporter

Those belong to separate specifications.

### Supported Backends (Examples)

- Grafana Cloud
- Tempo
- Loki
- Datadog
- New Relic
- SigNoz
- OpenObserve
- OpenTelemetry Collector

## User Scenarios & Testing _(mandatory)_

### User Story 1 — Export Traces to OpenTelemetry Collector (Priority: P1)

As a developer using Kit Observability, I want to export traces to an OpenTelemetry Collector so that my distributed tracing data is available in any OTLP-compatible backend without changing application code.

**Why this priority**: Trace export is the primary integration path. Without it, the observability data cannot reach external backends.

**Independent Test**: A Kit application configured with an OTLP endpoint creates a trace with parent-child spans. A test verifies that the trace data reaches a collector and parent-child relationships are preserved.

**Acceptance Scenarios**:

1. **Given** a Kit application configured with an OTLP exporter, **When** a trace is created with a parent span and a child span, **Then** the trace data reaches the collector with parent-child relationships preserved.
2. **Given** an active trace context with `trace_id`, `span_id`, and `correlation_id`, **When** the trace is exported via OTLP, **Then** all three identifiers are preserved in the exported telemetry.
3. **Given** a Kit application with no OTLP exporter configured, **When** traces are created, **Then** the application continues operating without errors and no data is exported.

---

### User Story 2 — Export Metrics Through OpenTelemetry (Priority: P1)

As a developer monitoring application behaviour, I want to export metrics through OpenTelemetry so that all four metric instrument types (Counter, Gauge, Histogram, UpDownCounter) are available in OTLP-compatible backends.

**Why this priority**: Metric export is essential for monitoring and alerting across all instrument types.

**Independent Test**: A Kit application creates one instrument of each type, records values, and verifies that each is correctly mapped to its OpenTelemetry counterpart and exported via OTLP.

**Acceptance Scenarios**:

1. **Given** a Counter instrument with recorded values, **When** exported via OTLP, **Then** the Counter is mapped to an OpenTelemetry Counter with correct sum.
2. **Given** a Gauge instrument with a current value, **When** exported via OTLP, **Then** the Gauge is mapped to an OpenTelemetry Gauge with the correct point-in-time value.
3. **Given** a Histogram instrument with recorded observations, **When** exported via OTLP, **Then** the Histogram is mapped to an OpenTelemetry Histogram with correct distribution data.
4. **Given** an UpDownCounter instrument with both positive and negative increments, **When** exported via OTLP, **Then** the UpDownCounter is mapped to an OpenTelemetry UpDownCounter with correct non-monotonic sum.

---

### User Story 3 — Export Structured Logs Through OpenTelemetry (Priority: P2)

As a developer debugging an application, I want to export structured logs through OpenTelemetry so that severity, attributes, and resource metadata are preserved in OTLP-compatible backends.

**Why this priority**: Structured log export provides consistent correlation between logs, traces, and metrics in the backend.

**Independent Test**: A Kit application emits a log record with severity, attributes, and resource metadata, and verifies that all three are preserved when exported via OTLP.

**Acceptance Scenarios**:

1. **Given** a log record with severity INFO, message body, and attributes, **When** exported via OTLP, **Then** the severity level and message body are preserved in the OpenTelemetry LogRecord.
2. **Given** a log record with custom attributes (e.g., `user.id`, `tenant.id`), **When** exported via OTLP, **Then** all attributes are preserved.
3. **Given** a log record associated with a Resource (`service.name`, `deployment.environment`), **When** exported via OTLP, **Then** the Resource metadata is preserved.

### Edge Cases

- **Missing Resource metadata**: When a telemetry record has no explicit Resource, the integration must use a default or skip Resource mapping without error.
- **Missing InstrumentationScope**: When a telemetry record has no explicit InstrumentationScope, the integration must use a default scope mapping without error.
- **Missing correlation_id**: When `correlation_id` is absent, the integration must not inject a default value or fail.
- **Empty attributes**: When the attributes collection is empty, the integration must produce a valid OpenTelemetry record with no attributes.
- **No configured exporter**: When no OTLP exporter is configured, the application must continue operating without errors.
- **Collector unavailable**: When the target collector is unreachable, the application must not crash or hang — export failures must be handled gracefully (e.g., retry, buffer, or drop according to configuration).

## Requirements _(mandatory)_

### Non-Functional Requirements

- **NFR-001**: The integration MUST NOT require modifications to KIT-001 public APIs. All mapping and export logic must be implemented as an adapter layer.
- **NFR-002**: OpenTelemetry support MUST remain optional. Applications not using OpenTelemetry must incur no additional dependency burden.
- **NFR-003**: The integration MUST remain transport agnostic — OTLP is the supported protocol, but the mapping layer must not assume a specific transport implementation.
- **NFR-004**: Export failures (e.g., collector unavailable, network errors) MUST NOT crash the application. The integration must degrade gracefully.
- **NFR-005**: The integration MUST preserve `trace_id`, `span_id`, `correlation_id`, and attributes when supported by the OpenTelemetry data model.

### Functional Requirements

#### Context Mapping

- **FR-001**: The integration MUST provide a mapping from Kit Context to OpenTelemetry Context.
- **FR-002**: The mapping MUST preserve `trace_id` and `span_id` from Kit Context to OpenTelemetry SpanContext.
- **FR-003**: The mapping MUST preserve `correlation_id` as an attribute or baggage item when supported.

#### Span Mapping

- **FR-004**: The integration MUST provide a mapping from Kit Span to OpenTelemetry Span.
- **FR-005**: The mapping MUST preserve span name, timestamps (start, end), status, attributes, Resource, and InstrumentationScope.
- **FR-006**: Parent-child span relationships from Kit MUST be preserved in the OpenTelemetry trace hierarchy.

#### Metric Mapping

- **FR-007**: The integration MUST provide a mapping from Kit Metric to OpenTelemetry Metric.
- **FR-008**: Kit Counter MUST map to OpenTelemetry Counter.
- **FR-009**: Kit Gauge MUST map to OpenTelemetry Gauge.
- **FR-010**: Kit Histogram MUST map to OpenTelemetry Histogram.
- **FR-011**: Kit UpDownCounter MUST map to OpenTelemetry UpDownCounter.
- **FR-012**: Metric attributes, Resource, and InstrumentationScope MUST be preserved in the mapping.

#### LogRecord Mapping

- **FR-013**: The integration MUST provide a mapping from Kit LogRecord to OpenTelemetry LogRecord.
- **FR-014**: Severity level MUST be mapped to OpenTelemetry SeverityNumber.
- **FR-015**: Message body, attributes, Resource, and InstrumentationScope MUST be preserved.

#### Resource Mapping

- **FR-016**: The integration MUST provide a mapping from Kit Resource to OpenTelemetry Resource.
- **FR-017**: All Resource attributes MUST be preserved in the mapping.

#### InstrumentationScope Mapping

- **FR-018**: The integration MUST provide a mapping from Kit InstrumentationScope to OpenTelemetry InstrumentationScope.
- **FR-019**: Scope name and optional version MUST be preserved.

#### OTLP Transport

- **FR-020**: The integration MUST support OTLP transport for exporting traces, metrics, and logs.
- **FR-021**: OTLP export configuration (endpoint, headers, compression, timeout) MUST be configurable without modifying application instrumentation code.

## Key Entities

- **OpenTelemetry Trace Adapter**: Converts Kit spans and trace contexts to OpenTelemetry spans and span contexts.
- **OpenTelemetry Metric Adapter**: Converts Kit metric instruments and datapoints to OpenTelemetry metrics.
- **OpenTelemetry Log Adapter**: Converts Kit log records to OpenTelemetry log records.
- **OTLP Exporter**: Exports OpenTelemetry telemetry data via the OTLP protocol to compatible backends.
- **Resource Mapping**: Converts Kit Resource to OpenTelemetry Resource.
- **Context Mapping**: Converts Kit Context (trace_id, span_id, correlation_id, attributes) to OpenTelemetry Context.
- **InstrumentationScope Mapping**: Converts Kit InstrumentationScope to OpenTelemetry InstrumentationScope.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: A Kit application can export traces to an OTLP-compatible collector without changing business logic. Verified by a test that creates a trace with parent-child spans, exports via OTLP, and confirms receipt at a collector.
- **SC-002**: A Kit application can export all four metric instrument types (Counter, Gauge, Histogram, UpDownCounter) through OpenTelemetry. Verified by a test that records values on each instrument type and confirms correct mapping.
- **SC-003**: A Kit application can export structured logs through OpenTelemetry with severity, attributes, and resource metadata preserved. Verified by a test that emits log records and confirms correct mapping.
- **SC-004**: No changes to KIT-001 APIs are required to support OpenTelemetry integration. Verified by a review that confirms all mapping logic lives in the adapter layer.
- **SC-005**: OpenTelemetry remains an optional integration. Verified by a test that KIT-001 applications link and run without the OpenTelemetry adapter present.

## Assumptions

- KIT-001 (Foundational Observability Abstractions) is implemented and provides the Context, Span, LogRecord, Metric, Resource, and InstrumentationScope entities that this integration maps from.
- The OpenTelemetry SDK (opentelemetry-rust or language-equivalent) is available as a separate dependency — not part of KIT-001.
- OTLP is the initial supported export protocol. Other protocols (Prometheus, Loki, etc.) belong to separate features.
- The integration targets the stable OpenTelemetry specification data model. Protocol-specific details (gRPC vs HTTP, protobuf serialization) are implementation concerns.
- Export retry, buffering, and batching logic is part of the integration implementation, not KIT-001.
