# Feature Specification: Structured Logging Core

**SPEC_ID**: `003-structured-logging-core`

**Parent**: KitLogger Telemetry Framework (`002-core-telemetry-domain-model`)

**Candidate Key**: KIT-005

**Created**: 2026-06-18

**Status**: Draft

## Scope

Define the canonical structured logging model, logging contracts, and logger APIs used by all KitLogger components.

In scope:
- Define the canonical LogRecord domain entity
- Define severity level semantics
- Define timestamp semantics
- Define message semantics
- Define structured attribute semantics
- Define attribute value type requirements
- Define correlation identifier integration
- Define trace identifier integration
- Define span identifier integration
- Define contextual metadata attachment semantics
- Define logger contracts
- Define logger factory contracts
- Define log enrichment contracts
- Define immutable log record creation semantics
- Define validation requirements
- Define serialization contracts required by downstream exporters
- Define integration contracts with Kit Config

Configuration shall be consumed exclusively through Kit Config contracts and shall not involve direct configuration file loading, parsing, or environment variable interpretation. The LoggingConfiguration entity shape is defined by KIT-CONFIG; KIT-005 consumes it through configuration contracts but does not define it.

## Non-Scope

The following are explicitly out of scope:
- Formatting pipelines
- Console rendering
- File rendering
- JSON formatting implementation
- Exporters
- Storage systems
- Audit logging
- Security logging
- PII redaction
- HTTP middleware
- gRPC middleware
- OpenTelemetry exporters
- Runtime configuration loading
- Environment variable parsing
- Configuration file parsing
- Configuration hot reload implementation
- Any runtime implementation details

## Responsibility

The structured logging core specification owns the canonical logging domain model, contracts, and primary API. It is responsible for defining what a log record is, how it is structured, how attributes are typed, and how loggers are created and used. Logger and LoggerFactory are canonical domain contracts owned by KIT-005. KIT-003 Pluggable Architecture may define adapter patterns for specialized loggers but does not own the Logger or LoggerFactory interfaces. KIT-005 is not responsible for any runtime behavior, formatting, transport, or storage of log records.

## Dependencies

- KIT-002 Core Telemetry Domain Model — canonical telemetry type system and shared domain primitives
- KIT-003 Pluggable Architecture — adapter and extension point contracts
- KIT-CONFIG Configuration Contracts — configuration consumption contracts (no direct config loading)

## User Scenarios & Testing

### Scenario 1: Application developers create structured logs using strongly typed attributes

A developer creates a log record with a message and structured attributes (e.g., user ID, order amount, request duration). The log record preserves type information for each attribute value without losing semantic meaning.

### Scenario 2: Applications attach contextual metadata to all emitted logs

An application establishes a logging context with service name, environment, and version. Every log record emitted within that context automatically includes the contextual metadata without requiring per-record specification.

### Scenario 3: Middleware enriches logs with request metadata

HTTP middleware attaches request-scoped metadata (method, path, status code) to log records emitted during request processing. The enrichment does not modify existing log record content but adds to the record's context.

### Scenario 4: Distributed systems attach correlation and tracing identifiers

A distributed tracing system attaches correlation, trace, and span identifiers to log records. Log consumers can correlate logs across service boundaries using these identifiers without depending on the tracing implementation.

### Scenario 5: Exporters consume canonical LogRecord instances

Downstream exporter components receive LogRecord instances through a defined contract. Exporters read fields from the immutable record and serialize them according to their own format requirements.

### Scenario 6: Logging behavior is configured through Kit Config contracts

An operator configures log severity thresholds, attribute filtering rules, and context propagation settings through Kit Config. The logging core reads the resolved configuration values and adjusts behavior accordingly without parsing configuration files directly.

### Testing

- LogRecord creation with all field types is verified through canonical construction
- Immutability is verified by confirming that no mutation methods exist on LogRecord
- Attribute type preservation is verified across all supported value types
- Context attachment is verified by confirming contextual fields appear on emitted records
- Correlation and tracing identifier fields are verified as present and correctly typed
- Configuration consumption is verified using mock Kit Config contract implementations

## Requirements

### Functional Requirements

1. Severity MUST be one of six canonical levels: Trace, Debug, Info, Warn, Error, Fatal (ordered least to most severe).
2. Every emitted log MUST be represented as a canonical LogRecord.
3. Log records MUST be immutable after creation.
4. Structured attributes MUST support strongly typed values.
5. LogRecord MUST support at minimum the following attribute value types: string, integer, floating-point number, boolean, timestamp, and array of these types. Nested object values are prohibited; all attribute values MUST be flat and belong to one of the supported scalar or homogeneous-array types.
6. Correlation metadata MUST be attachable without exporter-specific behavior.
7. Trace metadata MUST be attachable without tracing implementation dependencies.
8. Span metadata MUST be attachable without span implementation dependencies.
9. Logger contracts MUST remain transport agnostic.
10. Logger contracts MUST remain exporter agnostic.
11. Logger contracts MUST remain storage agnostic.
12. Logging configuration MUST be supplied through Kit Config contracts.
13. The logging core MUST NOT load configuration files directly.
14. The logging core MUST NOT parse TOML, YAML, JSON, or environment variables.
15. LoggerFactory MUST support creating named loggers with optional default context.
16. Loggers MUST support emitting log records with a severity level, message, and optional attributes.
17. Log enrichment contracts MUST support adding attributes to a logging context without modifying previously emitted records.
18. Serialization contracts MUST define the minimum field set that downstream exporters can rely on being present in every LogRecord.
19. Validation MUST reject LogRecord construction with an empty message.
20. Validation MUST reject LogRecord construction with an unrecognized severity level.
21. Attribute names MUST be lowercase, use letters/numbers/underscores/dots, be at most 64 characters, and not conflict with reserved LogRecord field names (pattern: `[a-z][a-z0-9._]{0,63}`).
22. Validation MUST reject LogRecord construction containing an attribute with a name that violates the naming constraints.

### Key Entities

- **LogRecord** — Canonical log entry with immutable fields: timestamp, severity, message, attributes.
- **Severity** — Enumeration with six canonical levels: Trace, Debug, Info, Warn, Error, Fatal (ordered Trace < Debug < Info < Warn < Error < Fatal). Rejecting unrecognized levels enforces model completeness.
- **LogAttribute** — A named key-value pair where the value is a strongly typed LogAttributeValue.
- **LogAttributeValue** — Strongly typed wrapper supporting string, integer, float, boolean, timestamp, and homogeneous-array variants. Nested objects are not supported; all values are flat.
- **LogContext** — Immutable set of attributes and metadata that applies to all log records emitted within a scope.
- **CorrelationId** — Identifier type for correlating log records across service boundaries.
- **TraceId** — Identifier type for associating log records with a distributed trace.
- **SpanId** — Identifier type for associating log records with a specific span within a trace.
- **Logger** — Canonical domain contract (owned by KIT-005) for emitting structured log records with severity level, message, optional attributes, and context.
- **LoggerFactory** — Canonical domain contract (owned by KIT-005) for creating Logger instances, optionally pre-configured with context and LoggingConfiguration.
- **LoggingConfiguration** — Configuration contract defined by KIT-CONFIG and consumed through Kit Config contracts for controlling logging behavior (severity thresholds, attribute filtering, context propagation). The entity shape is owned by KIT-CONFIG; KIT-005 defines consumption points only.

## Success Criteria

### Measurable Outcomes

1. A canonical LogRecord model exists with all required fields (timestamp, severity, message, attributes).
2. The Severity enum contains exactly six canonical levels: Trace, Debug, Info, Warn, Error, Fatal.
3. Structured attributes support at least 6 value types (string, integer, float, boolean, timestamp, array).
4. Logging contracts are framework agnostic — no dependency on any specific logging framework appears in the public API.
5. Logging contracts are transport agnostic — no transport-specific types appear in logger or log record interfaces.
6. Logging contracts are exporter agnostic — no exporter-specific behavior is embedded in log record or logger contracts.
7. Logging contracts are storage agnostic — no storage-specific concerns appear in domain entities.
8. Correlation and tracing metadata are consistently represented across all log records without requiring tracing library dependencies.
9. Configuration is consumed exclusively through Kit Config contracts — no configuration file parsing code exists in the logging core.
10. Downstream components consume logs through canonical LogRecord instances without modifying domain entities.
11. Immutability is enforced — no public mutation methods exist on LogRecord, LogAttribute, LogAttributeValue, or LogContext.
12. An empty message is rejected at LogRecord construction time.
13. An unrecognized severity level is rejected at LogRecord construction time.
14. Attribute names violating the naming pattern `[a-z][a-z0-9._]{0,63}` or conflicting with reserved LogRecord fields are rejected at LogRecord construction time.

## Assumptions

1. The Kit Config framework provides typed configuration contracts that the logging core can consume as dependencies.
2. Timestamps use UTC as the reference timezone; formatting for display is a downstream concern.
3. Severity level ordering: Trace < Debug < Info < Warn < Error < Fatal. All six levels are canonical; no unrecognized severities may appear in LogRecord construction.
4. CorrelationId, TraceId, and SpanId are opaque string identifiers with no required internal structure.
5. Array attribute values contain elements of a single type (homogeneous arrays). Nested objects are not supported as attribute values.
6. Log enrichment middleware receives the Logger or LogContext as input and produces an enriched Logger or LogContext as output.
7. The LoggingConfiguration entity is consumed at LoggerFactory creation time and may be scoped to individual loggers.
8. Serialization contracts define the minimum field contract; exporters may require additional fields through extension contracts.

## Clarifications

### Session 2026-06-18

- Q: What attribute naming constraints apply? → A: Attribute names must be lowercase, use `[a-z0-9._]`, max 64 chars, no reserved LogRecord field name conflicts. Pattern: `[a-z][a-z0-9._]{0,63}`.
- Q: Are nested object values allowed in LogAttribute? → A: No; attribute values are flat scalars or homogeneous arrays only (no nested objects).
- Q: What is the complete canonical set of severity levels? → A: Six levels: Trace, Debug, Info, Warn, Error, Fatal (ordered Trace < Debug < Info < Warn < Error < Fatal).
- Q: Are Logger and LoggerFactory canonical domain contracts or adapter contracts? → A: Logger and LoggerFactory are canonical domain contracts owned by KIT-005. KIT-003 may define adapter patterns but does not own these interfaces.
- Q: Where is the LoggingConfiguration entity shape defined? → A: KIT-CONFIG defines LoggingConfiguration; KIT-005 consumes it through configuration contracts.
- Q: Does AS-04 Serialization Contracts depend on AS-03 Logger Contracts? → A: No; AS-04 depends only on AS-01 (LogRecord entity fields). Serialization operates on canonical domain entities, not on logger APIs, enabling architectural parallelism.

### Session 2026-06-18 (Architectural Consistency Review)

- Q: Which identifier convention is canonical? → A: Parent capability prefix reused for all atomic specifications. Children of `003-structured-logging-core` use `003-` prefix: `003-structured-logging-core-as-01-...` through `003-structured-logging-core-as-05-...`.
- Q: Where does LogContext belong in the canonical model? → A: LogContext is a separate entity owned by AS-02 and attached during emission. LogRecord does not own LogContext. LogContext is defined by AS-02 and is applied during logger emission and enrichment workflows.
