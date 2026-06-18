# Feature Specification: Logfmt Formatter

**SPEC_ID**: `004-formatting-pipeline-as-04-logfmt-formatter`

**PARENT_SPEC_ID**: `004-formatting-pipeline`

**Candidate Key**: AS-04

**Created**: 2026-06-18

**Status**: Draft

## Scope

Implement a concrete LogFmtFormatter that renders a LogRecord as logfmt key=value tokens with proper encoding for values containing spaces or special characters.

## Non-Scope

- Text or JSON output formats
- Formatter trait definition (owned by AS-01)
- Output delivery, transport, or persistence
- LogRecord creation, validation, or enrichment

## Responsibility

Implement logfmt key=value formatting of LogRecord instances with space-separated token generation, proper encoding of values, and configurable field ordering.

## Dependencies

- `004-formatting-pipeline-as-01-formatter-contract` — Formatter trait, FormattedRecord type

## User Scenarios & Testing

### Scenario 1: Application exports logs for aggregation tools

A developer configures logfmt output. LogRecord instances are rendered as logfmt tokens suitable for ingestion by tools like Logstash, Promtail, or custom log processors.

### Scenario 2: Values containing spaces are properly encoded

A LogRecord with a message containing spaces or special characters produces correctly quoted logfmt values.

### Testing

- LogFmtFormatter implements the Formatter trait
- Output is space-separated key=value tokens
- Severity, timestamp, message, and all attributes appear as key=value tokens
- Values containing spaces are quoted
- Field ordering matches the configured order
- Identical records produce identical output

## Requirements

### Functional Requirements

1. LogFmtFormatter MUST implement the Formatter trait from AS-01.
2. Output MUST be space-separated key=value tokens.
3. Output MUST include severity, timestamp, message, and all attributes as tokens.
4. Values containing spaces or special characters MUST be quoted according to logfmt conventions.
5. Field ordering MUST be configurable at construction time with a sensible default order.
6. Output MUST be deterministic: identical inputs and configuration produce identical output.

### Key Entities

- **LogFmtFormatter** — Concrete Formatter implementation for logfmt output.

## Success Criteria

### Measurable Outcomes

1. A LogRecord can be rendered as logfmt key=value output.
2. All structured attributes appear as key=value tokens in the output.
3. Severity and timestamp are included as tokens.
4. Values containing spaces are properly quoted.
5. Field ordering matches the configured default or custom order.
6. Output is deterministic for identical records.

## Assumptions

1. Logfmt follows the standard convention: `key=value` tokens separated by spaces.
2. Values containing spaces or special characters are wrapped in double quotes.
3. Keys are not quoted.
4. Timestamps are rendered as ISO 8601 strings.
5. The default field order for logfmt is: timestamp, severity, message, attributes.
6. Array attributes are rendered as comma-separated values within brackets.
