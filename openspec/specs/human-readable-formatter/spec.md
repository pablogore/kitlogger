# HumanReadableFormatter Specification

## Purpose

Specifies the output contract for `HumanReadableFormatter`: a formatter targeting
developer consoles and log viewers, producing a human-friendly single-line string
with aligned fields and key=value attribute pairs.

## Requirements

### Requirement: Human-Readable Output Structure

`HumanReadableFormatter` MUST produce a single-line string (no trailing newline)
with fields in this order:

```
<timestamp>  <LEVEL> [<logger>] <message>  <key>=<value> ...
```

Field rules:

| Field        | Source                                       | Always present?         |
|--------------|----------------------------------------------|-------------------------|
| `<timestamp>`| `LogRecord.timestamp` as RFC3339 UTC string  | Yes                     |
| `<LEVEL>`    | `LogRecord.severity` as uppercase string     | Yes                     |
| `[<logger>]` | `LogContext` attribute `"logger"`            | No — omit bracket+name when absent |
| `<message>`  | `LogRecord.message`                          | Yes                     |
| key=value... | `LogRecord.attributes` then remaining `LogContext.attributes` (excluding `"logger"`) | No — omit when empty |

Separator between timestamp and level: two spaces. Separator between message and
attributes: two spaces.

`LogAttributeValue` MUST render as follows:

| Variant     | Rendering                            |
|-------------|--------------------------------------|
| `String`    | bare value (no quotes)               |
| `Integer`   | decimal string                       |
| `Float`     | decimal string                       |
| `Boolean`   | `true` or `false`                    |
| `Timestamp` | RFC3339 UTC string                   |
| `Array`     | inline JSON array (e.g. `["a","b"]`) |

#### Scenario: Full record with logger and attributes

- GIVEN a `LogRecord` with timestamp `2026-06-20T10:00:00Z`, severity INFO, message `"login ok"`, attribute `service=api`
- AND a `LogContext` with attribute `logger="auth"`
- WHEN `HumanReadableFormatter.format(record, Some(&context))` is called
- THEN it returns `"2026-06-20T10:00:00Z  INFO [auth] login ok  service=api"`

#### Scenario: Record without context

- GIVEN a `LogRecord` with timestamp, severity WARN, message `"slow query"`, no attributes
- AND context is `None`
- WHEN `format` is called
- THEN the result contains the timestamp, `WARN`, and `"slow query"`
- AND no `[...]` bracket is present

#### Scenario: Context present but no logger attribute

- GIVEN a `LogContext` that has attribute `env="prod"` but no `logger` key
- WHEN `format` is called
- THEN no `[...]` bracket appears
- AND `env=prod` appears after the message

#### Scenario: No attributes on either side

- GIVEN a `LogRecord` with no attributes and a `LogContext` with no attributes (or `None`)
- WHEN `format` is called
- THEN the output is `"<ts>  <LEVEL> <message>"` with no trailing spaces or separators

#### Scenario: Array attribute renders as inline JSON

- GIVEN a `LogRecord` attribute `tags` with `LogAttributeValue::Array(["api", "auth"])`
- WHEN `format` is called
- THEN the attribute renders as `tags=["api","auth"]`

## Constraints

- No trailing newline.
- String attribute values are NOT quoted (unlike JSON).
- The `[logger]` bracket pair is present only when the logger name is available.
- Severity MUST be uppercase ASCII.

## Traceability

| Proposal section | Requirement |
|------------------|-------------|
| Approach — HumanReadableFormatter | Human-Readable Output Structure |
| Architectural decision — logger from LogContext | Logger field sourcing |
| Architectural decision — Timestamp as RFC3339 UTC | Timestamp rendering |
| Output example in orchestrator prompt | Field order and separators |
