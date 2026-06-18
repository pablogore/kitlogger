# Feature Specification: Structured Log Domain Model

**SPEC_ID**: `003-structured-logging-core-as-01-structured-log-domain-model`

**PARENT_SPEC_ID**: `003-structured-logging-core`

**Parent**: Structured Logging Core (`003-structured-logging-core`)

**Candidate Key**: AS-01

**Created**: 2026-06-18

**Status**: Draft

## Scope

Define the canonical LogRecord entity, its immutable fields, and all value types that form the foundation of the structured logging domain.

In scope:
- Define the canonical LogRecord entity with immutable fields: timestamp, severity, message, attributes
- Define Severity enum with six canonical levels: Trace, Debug, Info, Warn, Error, Fatal
- Define LogAttribute as a named key-value pair
- Define LogAttributeValue supporting strongly typed values: string, integer, float, boolean, timestamp, homogeneous array
- Define attribute naming constraints: lowercase, `[a-z][a-z0-9._]{0,63}`, max 64 characters, no reserved LogRecord field name conflicts
- Define CorrelationId, TraceId, SpanId as opaque string identifier types
- Define timestamp semantics (UTC reference)
- Define message semantics
- Define validation rules: immutability after creation, empty message rejection, unrecognized severity rejection, attribute name constraint enforcement
- Prohibit nested object attribute values — all values must be flat

## Non-Scope

- LogContext entity and contextual metadata attachment
- Logger and LoggerFactory interfaces
- Log enrichment contracts
- Serialization contracts
- Configuration integration
- Any runtime formatting, transport, or storage of log records
- Formatting pipelines, console/file rendering, JSON formatting
- Exporter or middleware behavior
- Configuration file loading or environment variable parsing

## Responsibility

Define the canonical LogRecord entity with severity levels, attribute types, attribute naming constraints, identifier types (CorrelationId, TraceId, SpanId), timestamp semantics, message semantics, and all validation rules. This is the foundational data layer upon which all other logging specifications depend.

## Dependencies

- `002-core-telemetry-domain-model` (KIT-002 Core Telemetry Domain Model) — shared domain primitives and canonical type system

## Requirements

### Functional Requirements

1. LogRecord MUST contain the following immutable fields: timestamp, severity, message, attributes.
2. Severity MUST be one of six canonical levels: Trace, Debug, Info, Warn, Error, Fatal (ordered least to most severe).
3. LogAttribute MUST be a named key-value pair with a string name and strongly typed LogAttributeValue.
4. LogAttributeValue MUST support at minimum: string, integer, floating-point number, boolean, timestamp, and homogeneous array of these types.
5. Nested object attribute values are prohibited; all attribute values MUST be flat.
6. Attribute names MUST match the pattern `[a-z][a-z0-9._]{0,63}`, be at most 64 characters, and not conflict with reserved LogRecord field names.
7. CorrelationId, TraceId, and SpanId MUST be opaque string identifiers with no required internal structure.
8. LogRecord MUST be immutable after creation — no public mutation methods.
9. LogRecord construction MUST reject an empty message.
10. LogRecord construction MUST reject an unrecognized severity level.
11. LogRecord construction MUST reject an attribute with a name that violates naming constraints.

### Key Entities

- **LogRecord** — Canonical log entry with immutable fields: timestamp, severity, message, attributes.
- **Severity** — Enumeration with six canonical levels: Trace, Debug, Info, Warn, Error, Fatal.
- **LogAttribute** — Named key-value pair with strongly typed LogAttributeValue.
- **LogAttributeValue** — Strongly typed wrapper supporting string, integer, float, boolean, timestamp, homogeneous-array variants.
- **CorrelationId** — Opaque string identifier for cross-service correlation.
- **TraceId** — Opaque string identifier for distributed trace association.
- **SpanId** — Opaque string identifier for span within a trace.

## User Scenarios & Testing

### Scenario 1: Application creates a structured log record

A developer creates a LogRecord with severity Info, a message, and a set of typed attributes (user_id string, amount float). The record preserves type information for each attribute.

### Scenario 2: Validation rejects invalid records

Attempting to create a LogRecord with an empty message, an unrecognized severity level, or an attribute with a name violating the naming pattern is rejected at construction time.

### Scenario 3: LogRecord is immutable after creation

Once constructed, a LogRecord has no mutation methods. All fields are set at construction and remain read-only.

### Testing

- LogRecord creation with valid inputs succeeds
- LogRecord creation with empty message fails
- LogRecord creation with invalid severity fails
- LogRecord creation with invalid attribute name fails
- No mutation methods exist on LogRecord

## Success Criteria

### Measurable Outcomes

1. A canonical LogRecord model exists with all required fields (timestamp, severity, message, attributes).
2. Severity enum contains exactly six canonical levels: Trace, Debug, Info, Warn, Error, Fatal.
3. Structured attributes support at least 6 value types (string, integer, float, boolean, timestamp, array).
4. Immutability is enforced — no public mutation methods on LogRecord, LogAttribute, or LogAttributeValue.
5. An empty message is rejected at LogRecord construction time.
6. An unrecognized severity level is rejected at LogRecord construction time.
7. Attribute names violating the naming pattern `[a-z][a-z0-9._]{0,63}` or conflicting with reserved LogRecord fields are rejected at LogRecord construction time.

## Assumptions

1. Timestamps use UTC as the reference timezone; formatting for display is a downstream concern.
2. Severity level ordering: Trace < Debug < Info < Warn < Error < Fatal.
3. CorrelationId, TraceId, and SpanId are opaque string identifiers with no required internal structure.
4. Array attribute values contain elements of a single type (homogeneous arrays).
5. Nested objects are not supported as attribute values.
