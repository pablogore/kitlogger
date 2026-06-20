# LogfmtFormatter Specification

## Purpose

Specifies the output contract for `LogfmtFormatter`: a formatter that produces
[logfmt](https://brandur.org/logfmt)-style `key=value` pairs on a single line.
Key requirements are zero silent data loss and deterministic field ordering.

## Requirements

### Requirement: Logfmt Output Structure

`LogfmtFormatter` MUST produce a single-line string (no trailing newline) with
`key=value` pairs separated by single spaces, in this field order:

| Position | Key       | Source                                      | Always present?                     |
|----------|-----------|---------------------------------------------|-------------------------------------|
| 1        | `ts`      | `LogRecord.timestamp` as RFC3339 UTC string | Yes                                 |
| 2        | `level`   | `LogRecord.severity` as uppercase string    | Yes                                 |
| 3        | `msg`     | `LogRecord.message` (quoted if contains space or `=`) | Yes               |
| 4        | `logger`  | `LogContext` attribute `"logger"`           | No — omit when absent               |
| 5+       | *(attrs)* | `LogRecord.attributes` key=value pairs      | No — omit when empty                |
| last     | *(ctx)*   | Remaining `LogContext.attributes` (excluding `"logger"`) | No — omit when absent |

#### Scenario: Full record with logger in context

- GIVEN a `LogRecord` with timestamp `2026-06-20T10:00:00Z`, severity INFO, message `"login ok"`, attribute `service=api`
- AND a `LogContext` with attribute `logger="auth"`
- WHEN `LogfmtFormatter.format(record, Some(&context))` is called
- THEN it returns `"ts=2026-06-20T10:00:00Z level=INFO msg=\"login ok\" logger=auth service=api"`

#### Scenario: Message with spaces is quoted

- GIVEN a `LogRecord` with message `"user logged in"`
- WHEN `format` is called
- THEN `msg` renders as `msg="user logged in"` (double-quoted)

#### Scenario: Record without context

- GIVEN a `LogRecord` with severity WARN, message `"retry"`, no attributes
- AND context is `None`
- WHEN `format` is called
- THEN the result is `"ts=<ts> level=WARN msg=retry"` with no `logger` key

---

### Requirement: Logfmt Value Quoting Rules

Values that contain spaces, `=`, or `"` MUST be double-quoted. Values that contain
double-quote characters inside the value MUST have those characters escaped as `\"`.
Simple scalar values with no whitespace or special characters MUST NOT be quoted.

#### Scenario: Value with equals sign is quoted

- GIVEN an attribute with value `"k=v"` (String containing `=`)
- WHEN rendered in logfmt
- THEN it appears as `key="k=v"` (double-quoted)

#### Scenario: Value with embedded quote is escaped

- GIVEN an attribute with value `say "hello"` (String containing double-quote)
- WHEN rendered in logfmt
- THEN it appears as `key="say \"hello\""` (escaped inner quotes)

#### Scenario: Simple value is bare

- GIVEN an attribute with value `"prod"` (no spaces or special chars)
- WHEN rendered in logfmt
- THEN it appears as `key=prod` (no quotes)

---

### Requirement: Array Serialization Policy

`LogAttributeValue::Array` MUST serialize as inline JSON (produced by `serde_json`),
NOT as a space-separated list or other format. Silent data loss or flattening of
array elements is PROHIBITED.

#### Scenario: String array renders as inline JSON

- GIVEN a `LogRecord` attribute `tags` with `LogAttributeValue::Array(["api", "auth"])`
- WHEN `LogfmtFormatter.format` is called
- THEN the output contains `tags=["api","auth"]`

#### Scenario: Integer array renders as inline JSON numbers

- GIVEN a `LogRecord` attribute `codes` with `LogAttributeValue::Array([200, 201, 204])`
- WHEN `format` is called
- THEN the output contains `codes=[200,201,204]`

#### Scenario: Array serialization failure returns FormatError

- GIVEN a value path that triggers a `serde_json` serialization error for an array
- WHEN `format` is called
- THEN it returns `Err(FormatError)` — MUST NOT panic

---

### Requirement: Timestamp Attribute Rendering

`LogAttributeValue::Timestamp` values that appear in `LogRecord.attributes` or
`LogContext.attributes` MUST render as RFC3339 UTC strings (not debug output, not
Unix epoch integers).

#### Scenario: Timestamp attribute renders as RFC3339

- GIVEN a `LogRecord` attribute `started_at` with `LogAttributeValue::Timestamp(t)` where `t` is `2026-01-01T12:00:00Z`
- WHEN `format` is called
- THEN the output contains `started_at=2026-01-01T12:00:00Z`

---

### Requirement: No Silent Data Loss

Every attribute from `LogRecord.attributes` and every attribute from `LogContext.attributes`
(except `logger`) MUST appear in the output. Omitting or silently dropping any attribute
is PROHIBITED.

#### Scenario: All record and context attributes are present

- GIVEN a `LogRecord` with attributes `a=1` and `b=2`
- AND a `LogContext` with attributes `logger="svc"`, `env="prod"`, `region="us-east-1"`
- WHEN `format` is called
- THEN all of `a=1`, `b=2`, `env=prod`, `region=us-east-1` appear in the output
- AND `logger=svc` appears as the `logger` field
- AND no attribute is missing

## Constraints

- No trailing newline.
- `logger` field appears only when the named attribute exists in `LogContext`.
- Array values use `serde_json` serialization; the resulting inline JSON MAY be further quoted if it contains logfmt-significant chars.
- Severity MUST be uppercase ASCII.
- `msg` without spaces MUST NOT be quoted.

## Traceability

| Proposal section | Requirement |
|------------------|-------------|
| Approach — LogfmtFormatter, no silent data loss | Logfmt Output Structure, No Silent Data Loss |
| Approach — Array as inline JSON | Array Serialization Policy |
| Architectural decision — Timestamp as RFC3339 UTC | Timestamp Attribute Rendering |
| Architectural decision — logger from LogContext | Logger field sourcing |
| Scope — FormatError, no panics | Array serialization failure |
