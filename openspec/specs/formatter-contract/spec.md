# Formatter Contract Specification

## Purpose

Defines the `Formatter` trait, `FormatError` type, `LogFormat` enum, and the
`formatter_from_config` factory function that together form the public contract
of the `kitlogger-formatter` crate.

## Requirements

### Requirement: Formatter Trait

The `kitlogger-formatter` crate MUST expose a `Formatter` trait with a single
required method:

```
fn format(&self, record: &LogRecord, context: Option<&LogContext>) -> Result<String, FormatError>
```

Every type implementing `Formatter` MUST be stateless and pure: the same inputs
MUST always produce the same output. The trait MUST be object-safe so callers can
store `Box<dyn Formatter>`.

#### Scenario: Successful format with context

- GIVEN a `LogRecord` with timestamp, severity INFO, and message "login ok"
- AND a `LogContext` with attribute `logger = "auth"`
- WHEN `format(record, Some(&context))` is called
- THEN it returns `Ok(String)` containing a non-empty formatted representation

#### Scenario: Successful format without context

- GIVEN a `LogRecord` with timestamp, severity WARN, and message "retry"
- AND `context` is `None`
- WHEN `format(record, None)` is called
- THEN it returns `Ok(String)` with no context-derived fields present
- AND the record's own fields (timestamp, severity, message, attributes) are rendered

#### Scenario: Format error propagation

- GIVEN a `LogRecord` whose attributes contain a value that cannot be serialized
- WHEN `format` is called
- THEN it returns `Err(FormatError)` — MUST NOT panic

---

### Requirement: FormatError Type

`FormatError` MUST be a public error type in `kitlogger-formatter`. It MUST cover
at minimum:

- Serialization failure (e.g. `serde_json` error during JSON or logfmt array output)
- Value-rendering failure (any attribute value that cannot be converted to a string)

`FormatError` MUST implement `std::error::Error`, `std::fmt::Display`, and
`std::fmt::Debug`. Panics on format failure are PROHIBITED.

#### Scenario: Serialization error is surfaced

- GIVEN a formatting path that encounters a serde_json serialization error
- WHEN `format` is called
- THEN it returns `Err(FormatError)` whose `Display` message is non-empty
- AND the error does not cause a panic

---

### Requirement: LogFormat Enum

The `kitlogger-formatter` crate MUST expose a `LogFormat` enum with exactly four
variants:

| Variant         | Selects formatter        |
|-----------------|--------------------------|
| `Json`          | `JsonFormatter`          |
| `HumanReadable` | `HumanReadableFormatter` |
| `Text`          | `TextFormatter`          |
| `Logfmt`        | `LogfmtFormatter`        |

`LogFormat` MUST derive or implement `Clone`, `Debug`, and `PartialEq`.

#### Scenario: All variants are distinct

- GIVEN the `LogFormat` enum
- WHEN each variant is compared with the others
- THEN no two variants are equal (`PartialEq`)

---

### Requirement: formatter_from_config Factory

`kitlogger-formatter` MUST expose a `formatter_from_config(format: LogFormat) -> Box<dyn Formatter>`
function. Given a `LogFormat` variant it MUST return the corresponding formatter
as a trait object with `'static` lifetime.

#### Scenario: Factory returns the correct formatter type

- GIVEN `LogFormat::Json`
- WHEN `formatter_from_config(LogFormat::Json)` is called
- THEN the returned `Box<dyn Formatter>` produces JSON output on subsequent `format` calls

#### Scenario: Every LogFormat variant is handled

- GIVEN each of the four `LogFormat` variants
- WHEN `formatter_from_config` is called with each variant
- THEN a non-null `Box<dyn Formatter>` is returned without panic or error for all four

---

### Requirement: Crate Dependencies

`kitlogger-formatter` MUST depend only on `kitlogger-log-domain`, `serde_json`, and
`thiserror` (proc-macro, zero runtime overhead). It MUST NOT depend on any exporter
crate or I/O crate.

#### Scenario: Dependency boundary is respected

- GIVEN the `kitlogger-formatter` crate manifest
- WHEN its dependency list is inspected
- THEN `kitlogger-log-domain` and `serde_json` are present
- AND no exporter or I/O crate is listed as a dependency

## Constraints

- The trait is object-safe: no generic methods, no `Self` in return positions.
- `formatter_from_config` uses `LogFormat` as selector; unknown variants are impossible (exhaustive enum).
- Timestamp values in `LogRecord.attributes` MUST be rendered as RFC3339 UTC strings by all formatters.

## Traceability

| Proposal section | Requirement |
|------------------|-------------|
| Scope — Formatter trait | Formatter Trait |
| Scope — FormatError | FormatError Type |
| Scope — LogFormat enum | LogFormat Enum |
| Approach — adapter selects formatter via LogFormat | formatter_from_config Factory |
| Dependencies — serde_json only | Crate Dependencies |
