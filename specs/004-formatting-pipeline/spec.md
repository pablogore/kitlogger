# Feature Specification: Formatting Pipeline

**SPEC_ID**: `004-formatting-pipeline`

**Parent**: Structured Logging Core (`003-structured-logging-core`)

**Candidate Key**: N/A

**Created**: 2026-06-18

**Status**: Draft

## Scope

Define the formatting pipeline responsible for transforming canonical LogRecord instances into serialized output representations suitable for exporters. The formatting pipeline is transport-agnostic, sink-agnostic, and independent from output destinations.

In scope:
- Define a Formatter contract (trait) for converting LogRecord instances to formatted output
- Provide a TextFormatter implementation for human-readable plain text output
- Provide a JsonFormatter implementation for structured JSON output
- Provide a LogFmtFormatter implementation for logfmt key=value output
- Preserve all structured attributes, severity, and timestamp information during formatting
- Support deterministic output for identical records
- Support configurable field ordering for formatted output
- Structured attribute value rendering across all output formats

## Non-Scope

- Console, file, or HTTP output destinations
- OpenTelemetry exporters or integration
- Transport delivery mechanisms
- Log storage or persistence
- LogRecord creation, validation, or enrichment
- Configuration loading or environment variable parsing
- Output encoding negotiation or content-type handling
- Streaming or incremental formatting of partial records

## Responsibility

Own the transformation from structured LogRecord to serialized string representations. Provide the canonical set of output formatters so that exporters and consumers receive consistently formatted output without implementing rendering logic themselves. This specification is the exclusive owner of formatting concerns — no downstream component (exporter, transport, sink) shall implement its own formatting.

## Dependencies

- `003-structured-logging-core` — LogRecord, Severity, LogAttribute, LogAttributeValue, CorrelationId, TraceId, SpanId, ValidationError

## User Scenarios & Testing

### Scenario 1: Application exports a log record as JSON

A developer configures a JSON exporter. The formatting pipeline transforms a LogRecord into a JSON string. The exporter receives the formatted string and writes it to its output destination.

### Scenario 2: Application exports a log record as plain text

A developer configures a text exporter. The formatting pipeline transforms a LogRecord into a human-readable text string with all attributes rendered inline.

### Scenario 3: Application exports a log record as logfmt

A developer configures a logfmt exporter. The formatting pipeline transforms a LogRecord into a logfmt key=value string suitable for log aggregation tools.

### Scenario 4: Multiple records produce identical output

Given the same LogRecord and formatter configuration, repeated formatting produces byte-identical output.

### Testing

- A LogRecord formatted as JSON produces valid JSON output
- A LogRecord formatted as text produces expected human-readable output
- A LogRecord formatted as logfmt produces expected key=value output
- All structured attributes are present in the formatted output
- Severity and timestamp are preserved in the formatted output
- Identical records produce identical formatted output
- Field ordering matches the formatter's configured order
- Formatted output can be consumed by an exporter without knowledge of formatting internals

## Requirements

### Functional Requirements

1. The system MUST define a Formatter contract that accepts a LogRecord reference and produces a formatted string.
2. The system MUST provide a TextFormatter that renders LogRecord as human-readable text including timestamp, severity, message, and all attributes.
3. The system MUST provide a JsonFormatter that renders LogRecord as a JSON object including timestamp, severity, message, and all attributes.
4. The system MUST provide a LogFmtFormatter that renders LogRecord as space-separated key=value tokens including timestamp, severity, message, and all attributes.
5. All formatters MUST render the message text in every output format.
6. All formatters MUST render the severity level in every output format.
7. All formatters MUST render the timestamp in every output format.
8. All formatters MUST render every attribute present on the LogRecord.
9. Formatting output MUST be deterministic: identical LogRecord instances with the same formatter configuration MUST produce identical output on every invocation.
10. Each formatter SHOULD support configurable field ordering.

### Key Entities

- **Formatter** — Contract (trait) for converting a LogRecord reference into a formatted string representation.
- **TextFormatter** — Concrete formatter producing human-readable plain text output.
- **JsonFormatter** — Concrete formatter producing JSON object output.
- **LogFmtFormatter** — Concrete formatter producing logfmt key=value output.
- **FormattedRecord** — The output of a formatter; a string type suitable for export.

## Success Criteria

### Measurable Outcomes

1. A LogRecord can be rendered as valid JSON output.
2. A LogRecord can be rendered as human-readable plain text output.
3. A LogRecord can be rendered as logfmt key=value output.
4. Exporters can consume formatted output as a string without knowledge of formatting internals.
5. Formatting output is deterministic: identical inputs produce identical outputs.
6. All structured attributes, severity, and timestamp are present in every output format.
7. Field ordering reflects the configured order of each formatter.

## Assumptions

1. Formatters produce string output; further encoding (e.g., UTF-8 byte emission) is a downstream concern.
2. Timestamps are formatted using UTC ISO 8601 as the default format; custom format configuration is out of scope.
3. Nested attribute values (LogAttributeValue::Array) are rendered as JSON-like arrays in all output formats.
4. Field ordering defaults are formatter-specific; configuration mechanisms are defined at the point of integration, not by this specification.
5. Attribute values of type Timestamp are rendered as ISO 8601 strings in all output formats.
6. Float values are rendered using standard numeric representation (no scientific notation by default).
7. Empty attribute collections produce an empty attributes section in the formatted output, not an omission of the section.
