# JsonFormatter Specification

## Purpose

Specifies the output contract for `JsonFormatter`: a stateless formatter that
converts a `LogRecord` (plus optional `LogContext`) into a single-line JSON
object string.

## Requirements

### Requirement: JSON Output Structure

`JsonFormatter` MUST produce a single-line JSON object (no trailing newline) with
fields in this order when present:

| JSON key   | Source                                      | Always present? |
|------------|---------------------------------------------|-----------------|
| `ts`       | `LogRecord.timestamp` as RFC3339 UTC string | Yes             |
| `level`    | `LogRecord.severity` as uppercase string    | Yes             |
| `msg`      | `LogRecord.message`                         | Yes             |
| `logger`   | `LogContext` attribute with key `"logger"`  | No — omit when absent |
| *(attrs)*  | `LogRecord.attributes` as key/value pairs   | No — omit when empty |
| *(ctx)*    | Remaining `LogContext.attributes` (excluding `"logger"`) | No — omit when absent |

`LogAttributeValue` MUST map to JSON types as follows:

| Variant     | JSON type   |
|-------------|-------------|
| `String`    | string      |
| `Integer`   | number      |
| `Float`     | number      |
| `Boolean`   | boolean     |
| `Timestamp` | string (RFC3339 UTC) |
| `Array`     | JSON array  |

#### Scenario: Full record with logger in context

- GIVEN a `LogRecord` with timestamp `2026-06-20T10:00:00Z`, severity INFO, message `"login ok"`, attribute `service="api"`
- AND a `LogContext` with attribute `logger="auth"`
- WHEN `JsonFormatter.format(record, Some(&context))` is called
- THEN it returns `Ok(r#"{"ts":"2026-06-20T10:00:00Z","level":"INFO","msg":"login ok","logger":"auth","service":"api"}"#)`

#### Scenario: Record without context

- GIVEN a `LogRecord` with timestamp, severity WARN, message `"slow query"`, no attributes
- AND context is `None`
- WHEN `JsonFormatter.format(record, None)` is called
- THEN the result contains `"level":"WARN"` and `"msg":"slow query"`
- AND no `"logger"` key is present

#### Scenario: Context with no logger attribute

- GIVEN a `LogRecord` and a `LogContext` that has attribute `env="prod"` but no `logger` key
- WHEN `format` is called
- THEN `"logger"` key is absent from the output
- AND `"env":"prod"` appears in the output

#### Scenario: Boolean and integer attributes

- GIVEN a `LogRecord` with attributes `retries=3` (Integer) and `cached=false` (Boolean)
- WHEN `format` is called
- THEN the output contains `"retries":3` and `"cached":false` as native JSON types

#### Scenario: Timestamp attribute value

- GIVEN a `LogRecord` attribute with value `LogAttributeValue::Timestamp(t)` where `t` represents `2026-01-01T00:00:00Z`
- WHEN `format` is called
- THEN the attribute renders as a JSON string `"2026-01-01T00:00:00Z"` (RFC3339 UTC)

#### Scenario: Array attribute value

- GIVEN a `LogRecord` attribute `tags` with value `LogAttributeValue::Array(["api", "auth"])`
- WHEN `format` is called
- THEN the output contains `"tags":["api","auth"]` as a native JSON array

#### Scenario: Serialization failure returns error

- GIVEN a formatting condition that triggers a `serde_json` serialization error
- WHEN `format` is called
- THEN it returns `Err(FormatError)` — MUST NOT panic

## Constraints

- Output MUST be a valid JSON object parseable by `serde_json::from_str`.
- No trailing newline.
- `logger` key appears only when `LogContext` is `Some` and contains an attribute with key `"logger"`.
- Severity MUST be uppercase ASCII (e.g. `"INFO"`, `"WARN"`, `"ERROR"`, `"DEBUG"`).

## Traceability

| Proposal section | Requirement |
|------------------|-------------|
| Approach — JsonFormatter | JSON Output Structure |
| Architectural decision — Timestamp as RFC3339 UTC | Timestamp rendering |
| Architectural decision — logger from LogContext | Logger field sourcing |
| Scope — FormatError, no panics | Serialization failure |
