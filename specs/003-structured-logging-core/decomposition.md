# Decomposition: Structured Logging Core

**SPEC_ID**: `003-structured-logging-core`

**Candidate Key**: KIT-005

**Status**: Draft

**Expansion**: 2026-06-18 — 5 atomic specifications expanded: `003-structured-logging-core-as-01` through `003-structured-logging-core-as-05`

---

## Atomic Specifications

### AS-01: Structured Log Domain Model

| Field | Value |
|-------|-------|
| **Local Key** | AS-01 |
| **SPEC_ID** | `003-structured-logging-core-as-01-structured-log-domain-model` |
| **Name** | Structured Log Domain Model |
| **Responsibility** | Define the canonical LogRecord entity with severity levels, attribute types, attribute naming constraints, identifier types (CorrelationId, TraceId, SpanId), timestamp semantics, message semantics, and all validation rules |
| **Dependencies** | KIT-002 (shared domain primitives) |
| **Ownership Boundary** | KIT-005 owns all entities; no dependency on Logger or LoggerFactory |

### AS-02: Log Context & Enrichment

| Field | Value |
|-------|-------|
| **Local Key** | AS-02 |
| **SPEC_ID** | `003-structured-logging-core-as-02-log-context-enrichment` |
| **Name** | Log Context & Enrichment |
| **Responsibility** | Define the LogContext entity, contextual metadata attachment semantics, and log enrichment contracts for adding attributes and identifiers to logging scopes without modifying previously emitted records |
| **Dependencies** | AS-01 (`003-structured-logging-core-as-01-structured-log-domain-model`) |
| **Ownership Boundary** | KIT-005 owns LogContext and enrichment contracts |

### AS-03: Logger Contracts

| Field | Value |
|-------|-------|
| **Local Key** | AS-03 |
| **SPEC_ID** | `003-structured-logging-core-as-03-logger-contracts` |
| **Name** | Logger Contracts |
| **Responsibility** | Define the Logger interface for emitting structured LogRecords and the LoggerFactory interface for creating named Logger instances with optional context and configuration |
| **Dependencies** | AS-01 (`003-structured-logging-core-as-01-structured-log-domain-model`), AS-02 (`003-structured-logging-core-as-02-log-context-enrichment`) |
| **Ownership Boundary** | KIT-005 owns Logger and LoggerFactory as canonical domain contracts |

### AS-04: Serialization Contracts

| Field | Value |
|-------|-------|
| **Local Key** | AS-04 |
| **SPEC_ID** | `003-structured-logging-core-as-04-serialization-contracts` |
| **Name** | Serialization Contracts |
| **Responsibility** | Define the minimum field set and serialization contracts for exporting LogRecord instances without coupling to specific formats, transport, or exporter implementations |
| **Dependencies** | AS-01 (`003-structured-logging-core-as-01-structured-log-domain-model`) |
| **Ownership Boundary** | KIT-005 owns serialization contracts; exporter implementations are separate |

### AS-05: Configuration Integration

| Field | Value |
|-------|-------|
| **Local Key** | AS-05 |
| **SPEC_ID** | `003-structured-logging-core-as-05-configuration-integration` |
| **Name** | Configuration Integration |
| **Responsibility** | Define how KIT-005 consumes LoggingConfiguration (owned by KIT-CONFIG) through Kit Config contracts for controlling logging behavior — severity thresholds, attribute filtering, context propagation, serialization preferences |
| **Dependencies** | AS-03 (`003-structured-logging-core-as-03-logger-contracts`), AS-04 (`003-structured-logging-core-as-04-serialization-contracts`), KIT-CONFIG |
| **Ownership Boundary** | KIT-005 defines consumption points; KIT-CONFIG owns LoggingConfiguration entity shape |

## Dependency Graph

```
AS-01 ──┬──► AS-02 ──► AS-03 ──► AS-05
        └──► AS-04 ──────────────────┘
```

- AS-03 and AS-04 can be implemented in parallel after AS-01
- AS-05 depends on AS-03 and AS-04
