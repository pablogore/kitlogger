# Feature Specification: Log Context & Enrichment

**SPEC_ID**: `log-context-enrichment`

**Parent**: Structured Logging Core (`003-structured-logging-core`)

**Candidate Key**: AS-02

**Created**: 2026-06-18

**Updated**: 2026-06-19

## Scope

Define the LogContext entity for scoped contextual metadata attachment and the enrichment contracts for adding attributes and identifiers to logging scopes without modifying previously emitted log records.

In scope:
- Define the LogContext entity as an immutable set of attributes and metadata
- Define contextual metadata attachment semantics
- Define correlation identifier attachment to logging scopes
- Define trace identifier attachment to logging scopes
- Define span identifier attachment to logging scopes
- Define enrichment contracts for adding attributes to a logging context
- Define enrichment contracts for adding correlation, trace, and span identifiers
- Define that enrichment does not modify previously emitted log records
- Define Display implementation for debugging
- Define Default implementation (empty context)
- Define duplicate attribute name rejection with error

## Non-Scope

- LogRecord entity definition (covered by AS-01)
- Logger and LoggerFactory interfaces (covered by AS-03)
- Serialization contracts (covered by AS-04)
- Configuration integration (covered by AS-05)
- Middleware implementation details
- OTel/W3C Baggage integration
- Formatting, transport, or storage

## Responsibility

Define the LogContext entity, contextual metadata attachment semantics, and log enrichment contracts for adding attributes and identifiers (correlation, trace, span) to logging scopes without modifying previously emitted records.

## Dependencies

- `003-structured-logging-core-as-01-structured-log-domain-model` (AS-01) — LogRecord, LogAttribute, LogAttributeValue, CorrelationId, TraceId, SpanId types

## Requirements

### Functional Requirements

1. LogContext MUST be an immutable set of attributes and metadata that applies to all log records emitted within a scope.
2. LogContext MUST support attaching LogAttribute values as contextual metadata.
3. LogContext MUST support attaching CorrelationId for cross-service correlation.
4. LogContext MUST support attaching TraceId for distributed trace association.
5. LogContext MUST support attaching SpanId for span-level identification within a trace.
6. Log enrichment MUST NOT modify previously emitted log records.
7. Enrichment contracts MUST support adding attributes to a LogContext without altering existing entries.
8. Contextual metadata MUST be attachable without exporter-specific behavior.
9. Trace and span metadata MUST be attachable without tracing implementation dependencies.
10. Enrichment operations MUST return a Result type for consistent error handling.
11. Duplicate attribute names in LogContext MUST be rejected with an error.
12. LogContext MUST implement Display for debugging output.
13. LogContext MUST implement Default, producing an empty context with no attributes or identifiers.
14. An empty LogContext (no attributes, no identifiers) MUST be a valid starting point.

### Requirement: LogContext Construction

LogContext MUST support construction via both `new()` and `Default`, each producing an immutable empty context with zero attributes and no identifiers set.

#### Scenario: Empty context via new()
- GIVEN a `LogContext::new()` call
- WHEN the context is created
- THEN `attributes()` SHALL return an empty slice
- AND `correlation_id()`, `trace_id()`, `span_id()` SHALL all return `None`

#### Scenario: Empty context via Default
- GIVEN a `LogContext::default()` call
- WHEN the context is created
- THEN the result MUST be equivalent to `LogContext::new()`

### Requirement: Display Implementation

LogContext MUST implement `Display` for human-readable debugging, showing attribute key/value pairs and any set identifiers.

#### Scenario: Display empty context
- GIVEN a default `LogContext`
- WHEN `to_string()` is called
- THEN the output MUST include `LogContext` and SHOW no attributes

#### Scenario: Display enriched context
- GIVEN a LogContext with an attribute and a correlation_id
- WHEN `to_string()` is called
- THEN the output MUST contain the attribute key/value and the correlation_id

### Requirement: Attribute Enrichment

`with_attribute` MUST create a new LogContext with the given `LogAttribute` appended, or return `Err(ValidationError::EnrichmentError)` if the attribute name already exists.

#### Scenario: Adding attribute to context
- GIVEN an empty `LogContext`
- WHEN `with_attribute(LogAttribute::new("env", "prod"))` is called
- THEN the returned context SHALL have one attribute with name "env"
- AND the original context SHALL remain empty

#### Scenario: Duplicate attribute name rejected
- GIVEN a LogContext with an attribute named "env"
- WHEN `with_attribute(LogAttribute::new("env", "staging"))` is called
- THEN the result SHALL be `Err(ValidationError::EnrichmentError(..))`

### Requirement: Identifier Enrichment

`with_correlation_id`, `with_trace_id`, and `with_span_id` MUST each produce a new LogContext with the identifier set, returning `Result<Self, ValidationError>`. Setting an already-set identifier SHALL replace the previous value.

#### Scenario: Attach correlation identifier
- GIVEN an empty `LogContext`
- WHEN `with_correlation_id(CorrelationId::new("req-1"))` is called
- THEN the returned context SHALL have `correlation_id()` returning `Some("req-1")`

#### Scenario: Attach trace and span identifiers
- GIVEN a LogContext with a correlation_id
- WHEN `with_trace_id(TraceId::new("trace-abc"))` and `with_span_id(SpanId::new("span-42"))` are called
- THEN the resulting context SHALL have all three identifiers set

#### Scenario: Identifier idempotency (last-wins)
- GIVEN a LogContext with `correlation_id` set to "req-1"
- WHEN `with_correlation_id(CorrelationId::new("req-2"))` is called
- THEN the returned context SHALL have `correlation_id()` returning "req-2"

### Requirement: Enrichment Immutability

All enrichment methods MUST return a new `Self` and MUST NOT mutate the original LogContext.

#### Scenario: Original unchanged after enrichment
- GIVEN a LogContext `ctx` with one attribute
- WHEN `ctx.with_attribute(new_attr)` is called and discarded
- THEN `ctx.attributes()` SHALL still contain only the original attribute

### Requirement: EnrichmentError Variant

`ValidationError` MUST expose an `EnrichmentError(String)` variant for enrichment failures, with a Display output prefixed by `"Enrichment error: "`.

#### Scenario: EnrichmentError display
- GIVEN `ValidationError::EnrichmentError("duplicate name: env".into())`
- WHEN `to_string()` is called
- THEN the output SHALL be `"Enrichment error: duplicate name: env"`

### Key Entities

- **LogContext** — Immutable set of attributes and metadata applicable to all log records emitted within a scope. Supports attachment of LogAttribute values, CorrelationId, TraceId, and SpanId. Enrichment produces a new instance without modifying the original.

### Error Handling

Enrichment failures (such as duplicate attribute names) MUST use a dedicated `ValidationError::EnrichmentError(String)` variant distinct from construction-time errors (empty message, invalid severity, attribute name validation).

### Enrichment API

```
/// Create a new LogContext with an additional attribute.
/// Returns Err(ValidationError::EnrichmentError) if the attribute name already exists.
fn with_attribute(&self, attr: LogAttribute) -> Result<Self, ValidationError>

/// Create a new LogContext with a correlation identifier.
fn with_correlation_id(&self, id: CorrelationId) -> Result<Self, ValidationError>

/// Create a new LogContext with a trace identifier.
fn with_trace_id(&self, id: TraceId) -> Result<Self, ValidationError>

/// Create a new LogContext with a span identifier.
fn with_span_id(&self, id: SpanId) -> Result<Self, ValidationError>
```

## User Scenarios & Testing

### Scenario 1: Application establishes logging context with service metadata

An application creates a LogContext with service name, environment, and version attributes. Every log record emitted within that scope automatically includes these attributes.

### Scenario 2: Middleware enriches logs with request metadata

HTTP middleware creates an enriched LogContext by adding request-scoped metadata (method, path, status code). Previously emitted records are unaffected — enrichment only affects the new context.

### Scenario 3: Distributed tracing attaches correlation identifiers

A tracing system attaches correlation, trace, and span identifiers to the LogContext without depending on the tracing implementation.

### Testing

- LogContext creation with attributes succeeds
- LogContext creation without attributes (Default) succeeds
- LogContext enrichment produces a new context without modifying the original
- LogContext enrichment with a duplicate attribute name returns an error
- LogContext enrichment with CorrelationId, TraceId, and SpanId succeeds
- LogContext Display output shows key/value pairs and identifiers for non-default context
- LogContext Default produces empty context
- Enriched context attribute isolation: original context unchanged after enrichment

## Success Criteria

### Measurable Outcomes

1. A LogContext entity exists supporting attachment of LogAttribute values.
2. CorrelationId, TraceId, and SpanId are attachable to LogContext without implementation dependencies.
3. Enrichment produces a new context without modifying the original LogContext.
4. Duplicate attribute names are rejected with ValidationError::EnrichmentError.
5. Contextual metadata is attachable without exporter-specific behavior.
6. LogContext implements Display and Default.
7. All enrichment paths return Result (no panics).
8. All tests pass: `cargo test -p kitlogger-log-domain`.

## Assumptions

1. Log enrichment operates on LogContext as a standalone entity — no dependency on Logger, LoggerFactory, or any emission concept.
2. Logger (AS-03) will consume LogContext at record emission time by merging context attributes with per-record attributes.
3. LogContext is passive — it holds data but does not know about scopes, middleware, or emission boundaries.
4. Middleware components create enriched contexts and pass them to Logger independently.
5. The small expected number of attributes (<10 per scope) makes O(n) duplicate-name checking acceptable.

## Changelog

### 2026-06-19 — Merged delta from 006-log-context

- Added structured Given/When/Then scenarios for all requirements (Construction, Display, Attribute Enrichment, Identifier Enrichment, Immutability, EnrichmentError)
- Updated Testing section to reflect key/value display format
- Changed Display output specification from "non-empty" to "shows key/value pairs and identifiers"
