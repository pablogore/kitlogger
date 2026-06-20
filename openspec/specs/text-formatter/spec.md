# TextFormatter Specification

## Purpose

Specifies the output contract for `TextFormatter`: a minimal formatter that
produces a compact, human-readable string suitable for simple log sinks that do
not require timestamps or structured attributes.

## Requirements

### Requirement: Text Output Structure

`TextFormatter` MUST produce a single-line string (no trailing newline) with this
fixed structure:

```
[<LEVEL>] <logger>: <message>
```

Field rules:

| Field       | Source                                      | Always present?                        |
|-------------|---------------------------------------------|----------------------------------------|
| `[<LEVEL>]` | `LogRecord.severity` as uppercase string    | Yes                                    |
| `<logger>:` | `LogContext` attribute `"logger"`           | No — omit `logger:` when absent        |
| `<message>` | `LogRecord.message`                         | Yes                                    |

`TextFormatter` MUST NOT include timestamp, record attributes, or context attributes
(other than `logger` for the name prefix).

#### Scenario: Record with logger in context

- GIVEN a `LogRecord` with severity INFO and message `"login ok"`
- AND a `LogContext` with attribute `logger="auth"`
- WHEN `TextFormatter.format(record, Some(&context))` is called
- THEN it returns `"[INFO] auth: login ok"`

#### Scenario: Record without context

- GIVEN a `LogRecord` with severity WARN and message `"slow query"`
- AND context is `None`
- WHEN `format` is called
- THEN it returns `"[WARN] slow query"`

#### Scenario: Context present but no logger attribute

- GIVEN a `LogContext` with attribute `env="prod"` but no `logger` key
- WHEN `format` is called
- THEN it returns `"[<LEVEL>] <message>"` — no `env` attribute appears, no colon prefix

#### Scenario: Attributes are ignored

- GIVEN a `LogRecord` with attributes `service="api"` and `retries=3`
- AND a `LogContext` with attribute `logger="auth"` and `env="prod"`
- WHEN `format` is called
- THEN none of `service`, `retries`, or `env` appear in the output

#### Scenario: All severity levels

- GIVEN `LogRecord` instances with each severity variant (TRACE, DEBUG, INFO, WARN, ERROR, FATAL)
- WHEN `format` is called for each
- THEN the severity appears as uppercase wrapped in brackets (e.g. `[TRACE]`, `[FATAL]`)

## Constraints

- No timestamp in output.
- No attribute key/value pairs in output.
- Only the `logger` attribute from `LogContext` affects the output (as a name prefix).
- No trailing newline.
- Severity MUST be uppercase ASCII.

## Traceability

| Proposal section | Requirement |
|------------------|-------------|
| Approach — TextFormatter: simple `[LEVEL] message`, no timestamp | Text Output Structure |
| Architectural decision — logger from LogContext | Logger field sourcing |
| Output example in orchestrator prompt | `[INFO] auth: login ok` |
