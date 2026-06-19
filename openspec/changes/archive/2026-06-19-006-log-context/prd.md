# Product Requirements: Log Context & Enrichment

## Problem

Applications and middleware need to attach contextual metadata to log records without coupling to specific loggers, exporters, or tracing implementations. Today, KITLogger logs raw strings — there is no concept of a "logging scope" that carries contextual attributes (service name, environment, request ID, correlation identifiers) across log emissions.

Without this, every caller must manually add the same attributes to every log call, leading to repetitive code, inconsistent metadata, and tight coupling between business logic and logging infrastructure.

## User Stories

### Story 1: Application establishes logging context with service metadata

An application creates a LogContext with service name, environment, and version attributes. Every log record emitted within that scope automatically includes these attributes without per-record repetition.

### Story 2: Middleware enriches logs with request metadata

HTTP middleware creates an enriched LogContext by adding request-scoped metadata (method, path, status code). Previously emitted records remain unaffected — enrichment only applies to future records.

### Story 3: Distributed tracing attaches correlation identifiers

A tracing system attaches correlation, trace, and span identifiers to the LogContext without depending on any tracing implementation. The identifiers are opaque strings at this layer.

## Functional Requirements

1. LogContext MUST be an immutable set of attributes and metadata applying to all log records emitted within a scope
2. LogContext MUST support attaching LogAttribute values
3. LogContext MUST support attaching CorrelationId for cross-service correlation
4. LogContext MUST support attaching TraceId for distributed trace association
5. LogContext MUST support attaching SpanId for span-level identification
6. Enrichment MUST produce a new LogContext without modifying the original
7. Enrichment MUST NOT modify previously emitted log records
8. Contextual metadata MUST be attachable without exporter-specific behavior
9. Trace and span metadata MUST be attachable without tracing implementation dependencies
10. Duplicate attribute names MUST be rejected with an error
11. LogContext MUST implement Display for debugging output
12. LogContext MUST implement Default for empty context creation

## Non-Goals

- Logger or LoggerFactory interfaces (covered by AS-03)
- Serialization or formatting (covered by AS-04)
- OTel/W3C Baggage integration
- Exporter-specific behavior
- Configuration or runtime scope management

## Success Criteria

1. A LogContext entity exists with LogAttribute attachment support
2. CorrelationId, TraceId, SpanId are attachable without implementation dependencies
3. Enrichment produces a new context without modifying original
4. All enrichment paths return Result (not panics)
5. Duplicate attribute names produce an error
6. LogContext can be displayed for debugging
7. Default empty context is creatable
8. All tests pass: `cargo test -p kitlogger-log-domain`
