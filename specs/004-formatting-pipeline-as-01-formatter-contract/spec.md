# Feature Specification: Formatter Contract

**SPEC_ID**: `004-formatting-pipeline-as-01-formatter-contract`

**PARENT_SPEC_ID**: `004-formatting-pipeline`

**Candidate Key**: AS-01

**Created**: 2026-06-18

**Status**: Draft

## Scope

Define the Formatter trait that all concrete formatters implement, the FormattedRecord output type, and the formatting pipeline abstraction that connects LogRecord instances to formatted output strings suitable for exporter consumption.

## Non-Scope

- Concrete formatter implementations (Text, JSON, Logfmt)
- LogRecord creation, validation, or enrichment
- Output delivery, transport, or persistence
- Configuration loading or environment variable parsing
- Output encoding or content-type negotiation

## Responsibility

Define the Formatter trait, FormattedRecord output type, and the formatting pipeline abstraction that concrete formatters implement and exporters consume.

## Dependencies

- `003-structured-logging-core` — LogRecord, Severity, LogAttribute, LogAttributeValue

## User Scenarios & Testing

### Scenario 1: A concrete formatter implements the Formatter trait

A developer creates a new formatter by implementing the Formatter trait. The trait contract guarantees that the formatter receives an immutable LogRecord reference and returns a FormattedRecord.

### Scenario 2: An exporter consumes formatted output

An exporter receives a FormattedRecord without needing to know which formatter produced it or how formatting was performed.

### Testing

- The Formatter trait is object-safe or otherwise usable through trait objects
- A Formatter implementation can format any valid LogRecord
- A Formatter implementation does not mutate the input LogRecord
- FormattedRecord is a concrete type that can be passed between components

## Requirements

### Functional Requirements

1. A Formatter trait MUST define at least one method that accepts an immutable LogRecord reference and returns a FormattedRecord.
2. The Formatter trait MUST be implementable without knowledge of other formatters.
3. FormattedRecord MUST be a concrete, owned type suitable for inter-component transfer.
4. The trait MUST be usable through trait objects or equivalent dispatch mechanism.
5. The formatting pipeline MUST accept any Formatter implementation polymorphically.

### Key Entities

- **Formatter** — Trait that defines the formatting contract.
- **FormattedRecord** — Concrete output type produced by formatters.
- **FormattingPipeline** — Abstraction that dispatches formatting to a Formatter.

## Success Criteria

### Measurable Outcomes

1. A concrete formatter can implement the Formatter trait and format LogRecord instances.
2. An exporter can consume FormattedRecord without knowledge of the formatter implementation.
3. The LogRecord is not mutated during formatting.
4. The Formatter trait can be dispatched through trait objects.

## Assumptions

1. FormattedRecord wraps a string or byte buffer; its internal representation is an implementation detail.
2. The Formatter trait uses `&self` receiver; interior mutability is the implementor's responsibility.
3. Field ordering configuration is defined at the trait level (e.g., via constructor parameters or builder), not as a trait method.
