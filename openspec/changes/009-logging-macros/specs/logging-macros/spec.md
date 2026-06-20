# Logging Macros Specification

## Purpose

Ergonomic `macro_rules!` severity macros (`trace!`, `debug!`, `info!`, `warn!`, `error!`)
that expand to `Logger` severity-method calls. Crate: `kitlogger-macros`, depends on
`kitlogger-log-domain` only. All macros are thin wrappers — they introduce no behavior
unavailable through the direct Logger API.

Traceability: proposal `openspec/changes/009-logging-macros/proposal.md`

---

## Requirements

### Requirement: Five Severity Macros (FR-001)

The crate MUST export exactly five `macro_rules!` macros: `trace!`, `debug!`, `info!`,
`warn!`, and `error!`. Each macro MUST expand to the matching `Logger` severity method
(`logger.trace`, `logger.debug`, `logger.info`, `logger.warn`, `logger.error`). No
`fatal!` macro SHALL be provided.

#### Scenario: Each macro maps to its severity method

- GIVEN a `Logger` implementation and a simple message string
- WHEN `trace!(logger, "msg")`, `debug!(logger, "msg")`, `info!(logger, "msg")`,
  `warn!(logger, "msg")`, and `error!(logger, "msg")` are each called
- THEN each call invokes `logger.log` with `Severity::Trace`, `Severity::Debug`,
  `Severity::Info`, `Severity::Warn`, and `Severity::Error` respectively
- AND the message passed to the logger equals `"msg"`

---

### Requirement: Supported Invocation Forms (FR-002)

Every macro MUST accept all six invocation forms:

| Form | Syntax |
|------|--------|
| Simple message | `macro!(logger, "literal")` |
| Formatted message | `macro!(logger, "fmt {}", arg)` |
| Single attribute | `macro!(logger, "msg", key => value)` |
| Multiple attributes | `macro!(logger, "msg", k1 => v1, k2 => v2)` |
| Context + message | `macro!(logger, ctx, "msg")` |
| Context + message + attrs | `macro!(logger, ctx, "msg", key => value)` |

#### Scenario: Simple literal message — no attributes, no context

- GIVEN a logger and the invocation `info!(logger, "user logged in")`
- WHEN the macro expands
- THEN `logger.info("user logged in", &[])` is called

#### Scenario: Formatted message with arguments

- GIVEN a logger, a format string, and an argument
- WHEN `info!(logger, "request {} failed", request_id)` is invoked
- THEN the macro calls `format!("request {} failed", request_id)` internally
- AND passes the resulting `String` as `&str` to `logger.info`

#### Scenario: Single structured attribute

- GIVEN a logger, a message, and one `key => value` pair
- WHEN `info!(logger, "login", user_id => 42u64)` is invoked
- THEN `logger.info("login", &[LogAttribute::new("user_id", 42u64.into())?])` is called

#### Scenario: Multiple structured attributes

- GIVEN a logger, a message, and two `key => value` pairs
- WHEN `info!(logger, "order placed", order_id => "abc", amount => 9.99f64)` is invoked
- THEN `logger.info` is called with a slice containing exactly two `LogAttribute` entries
- AND the entries preserve the declaration order

#### Scenario: Formatted message with attributes

- GIVEN a format string with args and one attribute pair
- WHEN `warn!(logger, "retry {}", n, attempt => n)` is invoked
- THEN the message is formatted first, then attributes are built, then `logger.warn` is called

---

### Requirement: Return Type (FR-003)

Every macro invocation MUST return `Result<(), EmitError>`. The result propagates the
`Result` returned by the underlying `Logger` method without wrapping or unwrapping.

#### Scenario: Success path propagates Ok

- GIVEN a logger that emits successfully
- WHEN `info!(logger, "msg")` is called
- THEN the return value is `Ok(())`

#### Scenario: Logger error propagates Err

- GIVEN a logger whose `info` method returns `Err(EmitError::LoggerClosed)`
- WHEN `info!(logger, "msg")` is called
- THEN the return value is `Err(EmitError::LoggerClosed)`

#### Scenario: Validation error on empty message propagates Err

- GIVEN a logger that validates messages
- WHEN `info!(logger, "")` is called
- THEN the return value is `Err(EmitError::Validation(ValidationError::EmptyMessage))`

---

### Requirement: Attribute Key Validation (CR-001)

Attribute key validation MUST occur at runtime (not compile-time). The macro MUST NOT
perform compile-time rejection of attribute keys. When `LogAttribute::new` returns an
error for an invalid key, the macro MUST return that error via `Result`.

#### Scenario: Invalid attribute key returns error at runtime

- GIVEN a logger and a key string that violates attribute validation rules
- WHEN a macro is invoked with that key
- THEN the macro returns `Err` containing the validation error
- AND the logger's `log` method is NOT called

#### Scenario: Valid attribute key proceeds normally

- GIVEN a logger and a well-formed attribute key
- WHEN `info!(logger, "msg", valid_key => "value")` is invoked
- THEN `LogAttribute::new` succeeds and the logger is called

---

### Requirement: Context Fold Contract (FR-008)

When a `LogContext` is passed as the second argument, the macro MUST fold ALL context
data into the attribute slice before calling the Logger. The fold order MUST be:

1. `ctx.attributes()` — copied as-is, in declaration order
2. `correlation_id` — only when `Some`; key `"correlation_id"`, value `LogAttributeValue::String`
3. `trace_id` — only when `Some`; key `"trace_id"`, value `LogAttributeValue::String`
4. `span_id` — only when `Some`; key `"span_id"`, value `LogAttributeValue::String`
5. Inline `key => value` pairs from the macro call site — appended after context data

The Logger trait accepts NO `LogContext` parameter; the macro is the ONLY place where
context is resolved into attributes.

#### Scenario: Context with correlation_id only

- GIVEN a `LogContext` where `correlation_id = Some("req-1")`, `trace_id = None`,
  `span_id = None`, and `attributes = []`
- WHEN `info!(logger, ctx, "msg")` is invoked
- THEN `logger.info` is called with a single-element slice containing
  `LogAttribute { name: "correlation_id", value: String("req-1") }`

#### Scenario: Context with all three IDs and no extra attributes

- GIVEN a `LogContext` with `correlation_id = Some("c")`, `trace_id = Some("t")`,
  `span_id = Some("s")`, and `attributes = []`
- WHEN `info!(logger, ctx, "msg")` is invoked
- THEN `logger.info` is called with exactly three attributes in order:
  `"correlation_id"`, `"trace_id"`, `"span_id"`

#### Scenario: Context with attributes only (no IDs)

- GIVEN a `LogContext` with two attributes `[env="prod", region="us-east"]` and no IDs
- WHEN `info!(logger, ctx, "msg")` is invoked
- THEN `logger.info` is called with exactly those two attributes
- AND no `"correlation_id"`, `"trace_id"`, or `"span_id"` attribute is present

#### Scenario: Context with IDs plus inline attribute pairs

- GIVEN a `LogContext` with `correlation_id = Some("c")` and `attributes = []`
- WHEN `info!(logger, ctx, "msg", user_id => 7u64)` is invoked
- THEN `logger.info` is called with two attributes: first `"correlation_id"`, then `"user_id"`

#### Scenario: Fully populated context with inline pairs

- GIVEN a `LogContext` with `attributes = [env="prod"]`, `correlation_id = Some("c")`,
  `trace_id = Some("t")`, `span_id = Some("s")`
- WHEN `warn!(logger, ctx, "degraded", region => "eu")` is invoked
- THEN `logger.warn` is called with five attributes in order:
  `"env"`, `"correlation_id"`, `"trace_id"`, `"span_id"`, `"region"`

#### Scenario: Empty context with no inline pairs

- GIVEN a `LogContext` with no attributes and no IDs
- WHEN `info!(logger, ctx, "msg")` is invoked
- THEN `logger.info` is called with an empty attribute slice

---

### Requirement: Equivalence with Direct Logger API (FR-009)

A macro invocation MUST produce a `LogRecord` that is observably identical to the record
produced by the equivalent direct `Logger` method call. The macro MUST introduce no
additional fields, timestamps, or metadata beyond what the Logger API itself provides.

#### Scenario: Macro output matches direct call — no attributes

- GIVEN a capturing logger that records `(Severity, message, attributes)` tuples
- WHEN `info!(logger, "hello")` is called
- THEN the recorded tuple equals the one produced by `logger.info("hello", &[])`

#### Scenario: Macro output matches direct call — with attributes

- GIVEN a capturing logger
- WHEN `info!(logger, "hello", k => "v")` is called
- THEN the recorded tuple equals
  `logger.info("hello", &[LogAttribute::new("k", "v".into())?])`

---

### Requirement: Crate Isolation (CR-002)

The `kitlogger-macros` crate MUST depend only on `kitlogger-log-domain`. It MUST NOT
introduce dependencies on formatting, exporting, or any runtime crate outside the domain.

#### Scenario: Dependency graph is minimal

- GIVEN the `Cargo.toml` of `kitlogger-macros`
- WHEN its dependency list is inspected
- THEN `kitlogger-log-domain` is the only workspace-internal dependency
- AND no formatter, exporter, or I/O crate appears in `[dependencies]`

---

### Requirement: Macro Hygiene (CR-003)

All internal identifiers and path references within macro expansions MUST use `$crate::`
qualification. Macros MUST NOT capture or shadow identifiers from the call site.

#### Scenario: Macro compiles in a crate that does not import domain types

- GIVEN a caller crate that imports only `kitlogger_macros::info`
- WHEN `info!(logger, "msg")` is invoked
- THEN the expansion compiles without requiring the caller to import
  `LogAttribute`, `LogAttributeValue`, or `EmitError` directly

---

### Requirement: LogAttributeValue Conversion (FR-013)

Values provided in `key => value` pairs MUST be converted to `LogAttributeValue` via
`Into<LogAttributeValue>`. All variants defined in `LogAttributeValue` MUST be usable
as macro attribute values.

#### Scenario: String value accepted

- GIVEN `info!(logger, "msg", env => "prod")`
- WHEN expanded
- THEN the attribute value is `LogAttributeValue::String("prod".to_string())`

#### Scenario: Integer value accepted

- GIVEN `info!(logger, "msg", count => 42i64)`
- WHEN expanded
- THEN the attribute value is `LogAttributeValue::Integer(42)`

#### Scenario: Boolean value accepted

- GIVEN `info!(logger, "msg", success => true)`
- WHEN expanded
- THEN the attribute value is `LogAttributeValue::Boolean(true)`

#### Scenario: Float value accepted

- GIVEN `info!(logger, "msg", latency => 0.42f64)`
- WHEN expanded
- THEN the attribute value is `LogAttributeValue::Float(0.42)`
