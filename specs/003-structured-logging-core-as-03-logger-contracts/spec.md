# Feature Specification: Logger Contracts

**SPEC_ID**: `003-structured-logging-core-as-03-logger-contracts`

**Parent**: Structured Logging Core (`003-structured-logging-core`)

**Candidate Key**: AS-03

**Created**: 2026-06-18

**Status**: Draft

## Scope

Define the canonical Logger and LoggerFactory domain contracts owned by KIT-005. These are the primary API interfaces for emitting structured log records and creating logger instances.

In scope:
- Define the Logger interface for emitting structured LogRecords with severity level, message, optional attributes, and context
- Define the LoggerFactory interface for creating named Logger instances
- Define optional default context support on LoggerFactory
- Define that Logger contracts are transport, exporter, and storage agnostic
- Define that Logger and LoggerFactory are canonical domain contracts owned by KIT-005

## Non-Scope

- LogRecord entity definition (covered by AS-01)
- LogContext entity definition (covered by AS-02)
- Log enrichment contracts (covered by AS-02)
- Serialization contracts (covered by AS-04)
- Configuration integration (covered by AS-05)
- Adapter patterns for specialized loggers (owned by KIT-003)
- Any runtime formatting, transport, or storage implementation
- Middleware or exporter implementations

## Responsibility

Define the Logger interface for emitting structured LogRecords with severity level, message, optional attributes, and context. Define the LoggerFactory interface for creating named Logger instances with optional default context and LoggingConfiguration. Both contracts are canonical domain contracts owned by KIT-005.

## Dependencies

- `003-structured-logging-core-as-01-structured-log-domain-model` (AS-01) — LogRecord, Severity
- `003-structured-logging-core-as-02-log-context-enrichment` (AS-02) — LogContext

## Requirements

### Functional Requirements

1. Logger MUST support emitting a LogRecord with a severity level, message, and optional attributes.
2. Logger MUST support emission with an associated LogContext.
3. Logger contracts MUST be transport agnostic.
4. Logger contracts MUST be exporter agnostic.
5. Logger contracts MUST be storage agnostic.
6. LoggerFactory MUST support creating named Logger instances.
7. LoggerFactory MUST support creating Logger instances with an optional default LogContext.
8. Logger and LoggerFactory are canonical domain contracts owned by KIT-005, not adapter contracts defined by KIT-003.

### Key Entities

- **Logger** — Canonical domain contract for emitting structured log records. Accepts severity, message, optional attributes, and optional context.
- **LoggerFactory** — Canonical domain contract for creating named Logger instances. Supports optional default context provisioning.

## User Scenarios & Testing

### Scenario 1: Application emits a structured log record

A developer obtains a Logger from LoggerFactory and emits a log record with Info severity, a message, and typed attributes.

### Scenario 2: Logger carries default context

A Logger created with a default LogContext automatically includes contextual attributes on every emitted record without per-record specification.

### Scenario 3: Multiple named loggers

An application creates multiple named Logger instances, each with potentially different default contexts. Loggers are identified by name and independently operable.

### Testing

- Logger emits LogRecord with severity + message
- Logger emits LogRecord with severity + message + attributes
- Logger emits LogRecord with associated context
- LoggerFactory creates named Logger instances
- LoggerFactory creates Logger with default context
- Contracts contain no transport, exporter, or storage types

## Success Criteria

### Measurable Outcomes

1. A Logger interface exists for emitting structured LogRecords.
2. A LoggerFactory interface exists for creating named Logger instances.
3. Logger supports severity, message, optional attributes, and optional context.
4. LoggerFactory supports named creation with optional default context.
5. No transport-specific, exporter-specific, or storage-specific types appear in Logger or LoggerFactory contracts.

## Assumptions

1. LoggerFactory consumes LoggingConfiguration at creation time (fully specified in AS-05).
2. LoggerFactory may be configured with a default LogContext that applies to all loggers it creates.
3. KIT-003 may define adapter patterns for specialized loggers but does not own the Logger or LoggerFactory interfaces.
