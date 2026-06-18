# Feature Specification: JSON Formatter

**SPEC_ID**: `004-formatting-pipeline-as-03-json-formatter`

**PARENT_SPEC_ID**: `004-formatting-pipeline`

**Candidate Key**: AS-03

**Created**: 2026-06-18

**Status**: Draft

## Scope

Implement a concrete JsonFormatter that renders a LogRecord as a JSON object with typed attribute values, proper JSON escaping, and configurable field ordering.

## Non-Scope

- Text or logfmt output formats
- Formatter trait definition (owned by AS-01)
- Output delivery, transport, or persistence
- LogRecord creation, validation, or enrichment

## Responsibility

Implement JSON object formatting of LogRecord instances with typed attribute values, proper JSON escaping and encoding, and configurable field ordering.

## Dependencies

- `004-formatting-pipeline-as-01-formatter-contract` — Formatter trait, FormattedRecord type

## User Scenarios & Testing

### Scenario 1: Application exports logs to a JSON consumer

A developer configures JSON output. LogRecord instances are serialized as JSON objects that can be consumed by log aggregation or analysis tools.

### Scenario 2: Structured attributes are preserved as typed JSON values

A LogRecord with integer, float, boolean, string, and array attributes produces a JSON object where each attribute value uses the appropriate JSON type.

### Testing

- JsonFormatter implements the Formatter trait
- Output is valid JSON according to the JSON specification
- Severity, timestamp, message, and all attributes appear as JSON object fields
- Attribute types are preserved: String → JSON string, Integer → JSON number, Float → JSON number, Boolean → JSON boolean, Array → JSON array
- Field ordering matches the configured order
- Identical records produce identical output

## Requirements

### Functional Requirements

1. JsonFormatter MUST implement the Formatter trait from AS-01.
2. Output MUST be valid JSON.
3. Output MUST include severity, timestamp, message, and all attributes as JSON object fields.
4. Attribute value types MUST map to corresponding JSON types: string → JSON string, integer → JSON number, float → JSON number, boolean → JSON boolean, array → JSON array.
5. Field ordering MUST be configurable at construction time with a sensible default order.
6. Output MUST be deterministic: identical inputs and configuration produce identical output.

### Key Entities

- **JsonFormatter** — Concrete Formatter implementation for JSON output.

## Success Criteria

### Measurable Outcomes

1. A LogRecord can be rendered as valid JSON.
2. All structured attributes appear as typed JSON values in the output.
3. Severity and timestamp are included as JSON fields.
4. Field ordering matches the configured default or custom order.
5. Output is deterministic for identical records.

## Assumptions

1. JSON output uses double-quoted field names and string values.
2. Special characters in string values are JSON-escaped.
3. Timestamps are rendered as ISO 8601 strings within JSON strings.
4. The default field order for JSON is: timestamp, severity, message, attributes.
5. Array attributes are rendered as JSON arrays.
