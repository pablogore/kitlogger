# Architecture: Structured Logging Core

**SPEC_ID**: `003-structured-logging-core`

**Candidate Key**: KIT-005

**Status**: Draft

---

## Capability and Domain Boundaries

The Structured Logging Core capability defines the canonical structured logging model, logging contracts, and logger APIs used by all KitLogger components. It is the foundation for all log emission within the KitLogger framework.

### Domain Scope

| Boundary | Includes | Excludes |
|----------|----------|----------|
| **Data Model** | LogRecord, Severity, LogAttribute, LogAttributeValue, LogContext, CorrelationId, TraceId, SpanId | Formatting pipelines, JSON representation, storage schemas |
| **Contracts** | Logger, LoggerFactory, enrichment contracts, serialization contracts | Transport protocols, exporter implementations, middleware implementations |
| **Configuration** | Consumption contracts with Kit Config | Configuration file loading, parsing (TOML/YAML/JSON/env) |
| **Validation** | Immutability, severity level, attribute naming, message constraints | Runtime validation of emitted records |

### External Boundaries

- **KIT-002 Core Telemetry Domain Model**: Provides shared domain primitives and canonical type system
- **KIT-003 Pluggable Architecture**: Defines adapter patterns for specialized loggers (KIT-005 owns Logger/LoggerFactory interfaces, not KIT-003)
- **KIT-CONFIG Configuration Contracts**: Owns the LoggingConfiguration entity shape; KIT-005 defines consumption points only

## Concepts, Constraints, and Ownership Boundaries

### Key Concepts

- **LogRecord**: Immutable canonical log entry with timestamp, severity, message, and attributes
- **Severity**: Six canonical levels (Trace < Debug < Info < Warn < Error < Fatal); unrecognized levels rejected
- **LogAttribute/LogAttributeValue**: Strongly typed key-value pairs; flat values only (no nested objects); homogeneous arrays supported
- **LogContext**: Immutable scope-bound metadata set defined by AS-02. Attached to all log records during logger emission and enrichment workflows. Not a field of LogRecord.
- **CorrelationId/TraceId/SpanId**: Opaque string identifiers for distributed system correlation
- **Logger**: Domain contract for emitting LogRecords with severity, message, and attributes
- **LoggerFactory**: Domain contract for creating Logger instances with optional context and configuration

### Constraints

1. All log records MUST be immutable after creation
2. Attribute values MUST be flat (no nested objects); only scalar and homogeneous-array types supported
3. Attribute names MUST match `[a-z][a-z0-9._]{0,63}` and not conflict with reserved LogRecord fields
4. Severity MUST be one of six canonical levels; unrecognized levels rejected
5. Empty messages MUST be rejected at construction time
6. Configuration MUST be consumed through Kit Config contracts only — no direct file loading, parsing, or environment variable interpretation
7. Logger and LoggerFactory contracts MUST be transport agnostic, exporter agnostic, and storage agnostic
8. Correlation and tracing identifiers MUST be attachable without tracing implementation dependencies

### Ownership Boundaries

- **KIT-005 owns**: LogRecord (AS-01), Severity (AS-01), LogAttribute (AS-01), LogAttributeValue (AS-01), LogContext (AS-02), CorrelationId (AS-01/AS-02), TraceId (AS-01/AS-02), SpanId (AS-01/AS-02), Logger (AS-03), LoggerFactory (AS-03)
- **KIT-CONFIG owns**: LoggingConfiguration entity shape
- **KIT-003 may extend**: Adapter patterns for specialized loggers (but does not own Logger/LoggerFactory interfaces)

## Decomposition Strategy

The capability is decomposed along the natural seams of the logging domain:

1. **Data foundation first**: The domain model (entities, validation, attribute system) must be defined before any contract or integration work
2. **Context separate from emission**: LogContext is a distinct concern from Logger — context can be defined, attached, and enriched independently of how loggers emit records
3. **Contracts after data**: Logger and LoggerFactory interfaces depend on the domain model entities being fully defined
4. **Serialization as a contract layer**: The minimum field set for exporters is a separate contract concern that builds on the domain model
5. **Configuration integration last**: Kit Config integration consumes all previously defined contracts and entities

## Atomic Specification Dependency Graph

```
AS-01 (Log Domain Model)
  │
  ├─────────────────────────┐
  ▼                         ▼
AS-02 (Log Context)      AS-04 (Serialization)
  │
  ▼
AS-03 (Logger Contracts)
  │
  ▼
AS-05 (Configuration Integration)
```

### Dependency Rules

- AS-02 depends on AS-01 (uses Severity, LogAttribute, LogAttributeValue, CorrelationId, TraceId, SpanId)
- AS-03 depends on AS-01 and AS-02 (uses LogRecord, LogContext)
- AS-04 depends on AS-01 (serializes LogRecord fields only; no dependency on Logger or LoggerFactory)
- AS-05 depends on AS-03 (LoggerFactory consumes configuration) and AS-04 (serialization config)
- AS-04 and AS-03 can be implemented in parallel after AS-01 is complete
- AS-04 and AS-05 can be partially parallel: AS-04 after AS-01, AS-05 after AS-03 and AS-04

## Atomic Specification Candidates

### AS-01: Structured Log Domain Model

- **Local Key**: AS-01
- **Name**: Structured Log Domain Model
- **Responsibility**: Define the canonical LogRecord entity with severity levels, attribute types, attribute naming constraints, identifier types (CorrelationId, TraceId, SpanId), timestamp semantics, message semantics, and all validation rules (immutability, empty message rejection, severity level rejection, attribute name validation).
- **Dependencies**: KIT-002 (shared domain primitives)
- **Ownership Boundary**: KIT-005 owns all entities; no dependency on Logger or LoggerFactory

### AS-02: Log Context & Enrichment

- **Local Key**: AS-02
- **Name**: Log Context & Enrichment
- **Responsibility**: Define the LogContext entity and contextual metadata attachment semantics. Define log enrichment contracts for adding attributes and identifiers (correlation, trace, span) to logging scopes without modifying previously emitted records.
- **Dependencies**: AS-01 (LogRecord, LogAttribute, CorrelationId, TraceId, SpanId)
- **Ownership Boundary**: KIT-005 owns LogContext and enrichment contracts

### AS-03: Logger Contracts

- **Local Key**: AS-03
- **Name**: Logger Contracts
- **Responsibility**: Define the Logger interface for emitting structured LogRecords with severity level, message, optional attributes, and context. Define the LoggerFactory interface for creating named Logger instances with optional default context and LoggingConfiguration. Both contracts are canonical domain contracts owned by KIT-005, not adapter contracts.
- **Dependencies**: AS-01 (LogRecord, Severity), AS-02 (LogContext)
- **Ownership Boundary**: KIT-005 owns Logger and LoggerFactory as canonical domain contracts

### AS-04: Serialization Contracts

- **Local Key**: AS-04
- **Name**: Serialization Contracts
- **Responsibility**: Define the minimum field set that downstream exporters can rely on being present in every LogRecord. Establish serialization contracts for exporting LogRecord instances without coupling to specific serialization formats, transport protocols, or exporter implementations.
- **Dependencies**: AS-01 (LogRecord entity fields)
- **Ownership Boundary**: KIT-005 owns serialization contracts; exporter implementations are separate

### AS-05: Configuration Integration

- **Local Key**: AS-05
- **Name**: Configuration Integration
- **Responsibility**: Define how KIT-005 consumes LoggingConfiguration (owned by KIT-CONFIG) through Kit Config contracts for controlling logging behavior — severity thresholds, attribute filtering, context propagation settings, and serialization preferences. No direct configuration file loading, parsing, or environment variable interpretation.
- **Dependencies**: AS-03 (LoggerFactory consumes configuration), AS-04 (serialization configuration contracts)
- **Ownership Boundary**: KIT-005 defines consumption points; KIT-CONFIG owns the LoggingConfiguration entity shape
