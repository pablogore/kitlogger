# Feature Specification: Log Context & Enrichment

**SPEC_ID**: `003-structured-logging-core-as-02-log-context-enrichment`

**Parent**: Structured Logging Core (`003-structured-logging-core`)

**Candidate Key**: AS-02

**Created**: 2026-06-18

**Status**: Draft

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

## Non-Scope

- LogRecord entity definition (covered by AS-01)
- Logger and LoggerFactory interfaces (covered by AS-03)
- Serialization contracts (covered by AS-04)
- Configuration integration (covered by AS-05)
- Middleware implementation details
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

### Key Entities

- **LogContext** — Immutable set of attributes and metadata applicable to all log records emitted within a scope. Supports attachment of LogAttribute values, CorrelationId, TraceId, and SpanId.

## User Scenarios & Testing

### Scenario 1: Application establishes logging context with service metadata

An application creates a LogContext with service name, environment, and version attributes. Every log record emitted within that scope automatically includes these attributes.

### Scenario 2: Middleware enriches logs with request metadata

HTTP middleware creates an enriched LogContext by adding request-scoped metadata (method, path, status code). Previously emitted records are unaffected.

### Scenario 3: Distributed tracing attaches correlation identifiers

A tracing system attaches correlation, trace, and span identifiers to the LogContext without depending on the tracing implementation.

### Testing

- LogContext creation with attributes succeeds
- LogContext enrichment produces a new context without modifying the original
- Correlation, trace, and span identifiers are attachable to LogContext
- Enriched context attributes appear on emitted log records

## Success Criteria

### Measurable Outcomes

1. A LogContext entity exists supporting attachment of LogAttribute values.
2. CorrelationId, TraceId, and SpanId are attachable to LogContext without implementation dependencies.
3. Enrichment produces a new context without modifying the original LogContext or previously emitted records.
4. Contextual metadata is attachable without exporter-specific behavior.

## Assumptions

1. Log enrichment receives the Logger or LogContext as input and produces an enriched Logger or LogContext as output.
2. Middleware components consume enrichment contracts to modify contexts.
3. LogContext is consumed by Logger at record emission time (AS-03).
