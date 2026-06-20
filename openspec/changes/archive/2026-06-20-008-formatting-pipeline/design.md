# Design: Formatting Pipeline (KIT-006)

## Technical Approach

New crate `kitlogger-formatter` provides a `RecordFormatter` trait and 4 pure,
stateless implementations that turn `(&LogRecord, Option<&LogContext>)` into a
`String`. A `LogFormat` enum plus a `formatter_from_config` factory selects the
implementation. The top-level `kitlogger` adapter formats a record, then hands
the `&str` to `ConsoleExporter::export(msg, severity)`. Purely additive: no
domain or exporter type changes. Closes the `LogRecord -> String` gap.

## Crate Structure

```
crates/kitlogger-formatter/
├── Cargo.toml          deps: kitlogger-log-domain (path), serde_json
└── src/
    ├── lib.rs          trait, LogFormat, factory, FormatError, re-exports, shared helpers
    ├── error.rs        FormatError (thiserror)
    ├── json.rs         JsonFormatter
    ├── human.rs        HumanReadableFormatter
    ├── text.rs         TextFormatter
    └── logfmt.rs       LogfmtFormatter
```

One file per formatter (mirrors `console-exporter`'s module-per-concern style).
Shared logic (logger-name extraction, RFC3339 rendering, severity label) lives in
`lib.rs` as private free functions.

## Architecture Decisions

| Decision | Choice | Alternatives rejected | Rationale |
|----------|--------|-----------------------|-----------|
| Trait name | `RecordFormatter` | `Formatter` (proposal text) | `std::fmt::Formatter` is ubiquitous in this codebase (`use std::fmt::Formatter`); `Formatter` would collide/confuse. `RecordFormatter` states the input domain. |
| Crate boundary | New crate `kitlogger-formatter` | Module in `console-exporter`; module in `kitlogger` | Formatting is exporter-agnostic and reusable by future exporters (File/OTLP). Keeps `console-exporter` consuming `&str` only. |
| Logger name source | `LogContext` attr key `logger` | Add `logger_name` to `LogRecord` | Out of scope per proposal; avoids domain mutation. Omitted when absent. |
| Error model | `thiserror` enum `FormatError` | `Box<dyn Error>`; `String` | Matches `console-exporter`/`ExportError` convention; typed variants, no panics. |
| Timestamp policy | RFC3339 UTC, computed in-crate | `chrono` dependency | Avoid new heavy dep; format via `SystemTime` epoch math. Single dep stays `serde_json`. |
| Logfmt arrays | inline JSON via `serde_json` | drop arrays; comma-join | No silent data loss; `tags=["api","auth"]` is parseable. |

## Interfaces / Contracts

```rust
// lib.rs
pub trait RecordFormatter: Send + Sync {
    fn format(&self, record: &LogRecord, context: Option<&LogContext>)
        -> Result<String, FormatError>;
}

pub enum LogFormat { Json, HumanReadable, Text, Logfmt }

pub fn formatter_from_config(format: LogFormat) -> Box<dyn RecordFormatter> {
    match format {
        LogFormat::Json          => Box::new(JsonFormatter),
        LogFormat::HumanReadable => Box::new(HumanReadableFormatter),
        LogFormat::Text          => Box::new(TextFormatter),
        LogFormat::Logfmt        => Box::new(LogfmtFormatter),
    }
}
```

Object safety: `format` takes `&self`, no generics, no `Self` return — trait is
object-safe, usable as `Box<dyn RecordFormatter>`. `Send + Sync` because all
formatters are zero-state unit structs; the adapter holds one across threads.

```rust
// error.rs
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("serialization failed: {0}")]
    SerializationError(String),
    #[error("value rendering failed: {0}")]
    RenderError(String),
}
// serde_json::Error mapped via .map_err(|e| FormatError::SerializationError(e.to_string()))
```

### Shared helpers (private, in `lib.rs`)
- `severity_label(&Severity) -> &'static str` → `TRACE|DEBUG|INFO|WARN|ERROR|FATAL` (uppercase; `Severity::Display` yields title-case, so map explicitly).
- `rfc3339_utc(SystemTime) -> String` → `YYYY-MM-DDTHH:MM:SSZ` (UTC, seconds precision).
- `logger_name<'a>(ctx: Option<&'a LogContext>) -> Option<&'a str>` → first attribute named `logger` whose value is `String`.

## Per-Formatter Design

Field order is fixed per formatter. Record attributes render after core fields;
context attributes (excluding `logger`) follow record attributes. `correlation_id`,
`trace_id`, `span_id` from context render as their own keys when present.

| Formatter | Output |
|-----------|--------|
| `JsonFormatter` | `serde_json::json!` object: `ts`, `level`, `msg`, then `logger` (if any), then record attrs, then context attrs/ids. Serialized via `serde_json::to_string`. |
| `HumanReadableFormatter` | `2026-06-20T10:00:00Z  INFO [auth] message  key=val ...`; `[logger]` omitted when no logger; attrs space-joined `key=val`. |
| `TextFormatter` | `[INFO] auth: message`; no timestamp, no attrs. `logger:` prefix omitted when no logger (`[INFO] message`). |
| `LogfmtFormatter` | `ts=… level=INFO msg="…" logger=auth key=val ...` |

### Logfmt value rendering (`LogAttributeValue` → token)
| Variant | Rendering |
|---------|-----------|
| `String(s)` | bare if no space/`=`/`"`; else JSON-quoted (`serde_json::to_string(s)`) |
| `Integer(n)` / `Float(n)` | `n.to_string()` |
| `Boolean(b)` | `true` / `false` |
| `Timestamp(t)` | `rfc3339_utc(t)` |
| `Array(a)` | inline JSON: `serde_json::to_string(&a)` → `["api","auth"]` |

`msg` is always quoted in logfmt. JSON formatter delegates all quoting to `serde_json`.

## Data Flow

```
LogRecord ─┐
           ├─→ RecordFormatter::format() ─→ String ─→ ConsoleExporter::export(&str, severity)
LogContext ┘   (selected via LogFormat)
(optional)
```

## Pipeline Wiring in `kitlogger`

`crates/kitlogger/src/lib.rs`: `KITLogger` gains a `formatter: Box<dyn RecordFormatter>`
field, defaulted by `formatter_from_config(LogFormat::Json)` in `new()` (add a
`with_format` constructor for selection). New method:

```rust
pub fn log_record(&self, record: &LogRecord, context: Option<&LogContext>)
    -> Result<(), AdapterError>
{
    let formatted = self.formatter.format(record, context)
        .map_err(|e| AdapterError::InitializationFailed(e.to_string()))?;
    self.exporter.export(&formatted, *record.severity())
        .map_err(|e| AdapterError::InitializationFailed(e.to_string()))
}
```

Existing `log(severity, &str)` stays for back-compat (raw passthrough).
`AdapterError` has no format-specific variant; reuse `InitializationFailed`
(error-variant cleanup is a separate change).

## Workspace Cargo.toml

Add member `"crates/kitlogger-formatter"` to `members`. New crate `Cargo.toml`:
`edition = "2021"`, deps `kitlogger-log-domain = { path = "../kitlogger-log-domain" }`
and `serde_json = "1.0"`. Add `kitlogger-formatter` path dep to `crates/kitlogger`.

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `crates/kitlogger-formatter/Cargo.toml` | Create | New crate manifest |
| `crates/kitlogger-formatter/src/lib.rs` | Create | Trait, `LogFormat`, factory, helpers, re-exports |
| `crates/kitlogger-formatter/src/error.rs` | Create | `FormatError` |
| `crates/kitlogger-formatter/src/{json,human,text,logfmt}.rs` | Create | 4 formatters |
| `Cargo.toml` (workspace) | Modify | Register member |
| `crates/kitlogger/Cargo.toml` | Modify | Add formatter dep |
| `crates/kitlogger/src/lib.rs` | Modify | Add formatter field + `log_record` |

## Testing Strategy

| Layer | What | Approach |
|-------|------|----------|
| Unit | Each formatter's exact output, logger present/absent, every `LogAttributeValue` variant, empty attrs/context | Per-formatter `#[cfg(test)]` with literal-string assertions |
| Unit | `formatter_from_config` returns correct impl | Format a known record, assert dialect |
| Unit | Logfmt quoting + inline-JSON arrays; RFC3339 rendering | Fixed `SystemTime` epoch inputs |
| Integration | `LogRecord → formatter → ConsoleExporter` via `log_record` | `set_writers` capture buffer in `kitlogger` test |

## Migration / Rollout

No migration required. Additive crate + adapter wiring; rollback = remove crate,
revert member entry and adapter commit.

## Open Questions

- [ ] `Float` rendering for non-finite values (NaN/Inf): JSON forbids them — emit `RenderError` or stringify? Default: stringify in logfmt/text, `RenderError` in JSON.
- [ ] RFC3339 sub-second precision: design fixes seconds-only; confirm acceptable for first slice.
