# Feature Specification: Text Formatter

**SPEC_ID**: `004-formatting-pipeline-as-02-text-formatter`

**PARENT_SPEC_ID**: `004-formatting-pipeline`

**Candidate Key**: AS-02

**Created**: 2026-06-18

**Status**: Draft

## Scope

Implement a concrete TextFormatter that renders a LogRecord as human-readable plain text with ordered fields and inline structured attribute rendering.

## Non-Scope

- JSON or logfmt output formats
- Formatter trait definition (owned by AS-01)
- Output delivery, transport, or persistence
- LogRecord creation, validation, or enrichment

## Responsibility

Implement human-readable plain text formatting of LogRecord instances with configurable field ordering and inline attribute rendering.

## Dependencies

- `004-formatting-pipeline-as-01-formatter-contract` — Formatter trait, FormattedRecord type

## User Scenarios & Testing

### Scenario 1: Developer views logs in a terminal

A developer configures text output. LogRecord instances are rendered as readable lines with severity prefix, timestamp, message, and attributes inline.

### Scenario 2: Text output is deterministic

The same LogRecord formatted twice with the same configuration produces identical output.

### Testing

- TextFormatter implements the Formatter trait
- Output includes severity, timestamp, message, and all attributes
- Attributes are rendered inline in human-readable format
- Field ordering matches the configured order
- Identical records produce identical output
- Empty attributes collection does not produce empty field tokens

## Requirements

### Functional Requirements

1. TextFormatter MUST implement the Formatter trait from AS-01.
2. Output MUST include severity, timestamp, message, and all attributes.
3. Attributes MUST be rendered inline as comma-separated or space-separated key=value pairs.
4. Field ordering MUST be configurable at construction time with a sensible default order.
5. Output MUST be deterministic: identical inputs and configuration produce identical output.

### Key Entities

- **TextFormatter** — Concrete Formatter implementation for plain text output.

## Success Criteria

### Measurable Outcomes

1. A LogRecord can be rendered as human-readable plain text.
2. All structured attributes appear in the text output.
3. Severity and timestamp are included in the output.
4. Field ordering matches the configured default or custom order.
5. Output is deterministic for identical records.

## Assumptions

1. Attribute values are rendered inline; multi-line attribute content is not expected.
2. Array attributes are rendered as comma-separated lists within brackets.
3. Timestamps are rendered as ISO 8601 strings.
4. The default field order for text is: timestamp, severity, message, attributes.
