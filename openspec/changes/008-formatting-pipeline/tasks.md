# Tasks: Formatting Pipeline (KIT-006)

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 550–750 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1: new crate (foundation + 4 formatters + tests) → PR 2: pipeline wiring in `kitlogger` |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: pending
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | `kitlogger-formatter` crate: trait, error, 4 formatters, all tests | PR 1 | Self-contained; no changes outside new crate |
| 2 | Pipeline wiring: `kitlogger/src/lib.rs` + integration test | PR 2 | Depends on PR 1; touches existing crate |

---

## Phase 1: Foundation — Crate Scaffold and Contract Types

- [x] 1.1 Add `"crates/kitlogger-formatter"` to `members` in workspace `Cargo.toml`.
- [x] 1.2 Create `crates/kitlogger-formatter/Cargo.toml` with `edition = "2021"`, dep `kitlogger-log-domain = { path = "../kitlogger-log-domain" }`, dep `serde_json = "1.0"`, dep `thiserror = "1"`.
- [x] 1.3 Create `crates/kitlogger-formatter/src/error.rs`: `FormatError` enum with `SerializationError(String)` and `RenderError(String)` via `thiserror`; must implement `std::error::Error + Display + Debug`.
- [x] 1.4 Create `crates/kitlogger-formatter/src/lib.rs` skeleton: declare modules (`error`, `json`, `human`, `text`, `logfmt`); define `RecordFormatter` trait (`fn format(&self, record: &LogRecord, context: Option<&LogContext>) -> Result<String, FormatError>`); define `LogFormat` enum (`Json | HumanReadable | Text | Logfmt`) with `#[derive(Clone, Debug, PartialEq)]`; stub `formatter_from_config`; private helpers `severity_label`, `rfc3339_utc`, `logger_name`.
- [x] 1.5 RED: write `#[cfg(test)]` tests in `lib.rs` for: `LogFormat` variants are all distinct; `formatter_from_config` returns without panic for each variant; `severity_label` returns uppercase; `rfc3339_utc` formats a known epoch. Run `cargo test --workspace` — expect compile errors or failures.
- [x] 1.6 GREEN: implement helpers `severity_label`, `rfc3339_utc`, `logger_name`; implement `formatter_from_config` (stubs returning unit-struct placeholders suffice until formatters exist). Run `cargo test --workspace` — Phase 1 tests pass.

## Phase 2: JsonFormatter (TDD)

- [x] 2.1 Create `crates/kitlogger-formatter/src/json.rs` with empty `pub struct JsonFormatter;`.
- [x] 2.2 RED: write tests in `json.rs` covering: full record with logger in context matches spec scenario exactly; record without context has no `"logger"` key; context with no logger omits key; Boolean and Integer attrs produce native JSON types; Timestamp attr renders as RFC3339 string; Array attr produces JSON array; NaN/Inf Float returns `Err(FormatError)`. Run `cargo test` — expect failures.
- [x] 2.3 GREEN: implement `RecordFormatter` for `JsonFormatter` using ordered `Vec<(key, Value)>` + manual serialization to preserve field order; map `LogAttributeValue` variants; check NaN/Inf and return `FormatError::RenderError`. Run `cargo test` — all `json` tests pass.

## Phase 3: HumanReadableFormatter (TDD)

- [x] 3.1 Create `crates/kitlogger-formatter/src/human.rs` with empty `pub struct HumanReadableFormatter;`.
- [x] 3.2 RED: write tests for: full record with logger matches spec literal (`"2026-06-20T10:00:00Z  INFO [auth] login ok  service=api"`); record without context has no `[...]`; context without logger has no `[...]`; no attrs = no trailing spaces; Array attr renders as inline JSON (`tags=["api","auth"]`); NaN/Inf Float renders as string literal `"NaN"` / `"Inf"`. Run `cargo test` — expect failures.
- [x] 3.3 GREEN: implement `RecordFormatter` for `HumanReadableFormatter`: two-space separators, `[logger]` bracket only when present, collect record then context attrs (exclude `logger`), space-join `key=val`. Run `cargo test` — all `human` tests pass.

## Phase 4: TextFormatter (TDD)

- [x] 4.1 Create `crates/kitlogger-formatter/src/text.rs` with empty `pub struct TextFormatter;`.
- [x] 4.2 RED: write tests for: logger present → `"[INFO] auth: login ok"`; no context → `"[WARN] slow query"`; context without logger → `"[INFO] message"` (no colon prefix); all six severity variants render uppercase in brackets; attrs are NOT present in output. Run `cargo test` — expect failures.
- [x] 4.3 GREEN: implement `RecordFormatter` for `TextFormatter`: format `[LEVEL]`, conditionally prepend `logger: ` prefix, append message. Run `cargo test` — all `text` tests pass.

## Phase 5: LogfmtFormatter (TDD)

- [x] 5.1 Create `crates/kitlogger-formatter/src/logfmt.rs` with empty `pub struct LogfmtFormatter;`.
- [x] 5.2 RED: write tests for: full record with logger matches spec literal; message with spaces is quoted; no context → no `logger` field; value with `=` is quoted; value with embedded `"` is escaped; simple bare value is unquoted; String array renders as `tags=["api","auth"]`; Integer array renders as `codes=[200,201,204]`; Timestamp attr renders as RFC3339; all record + context attrs (minus `logger`) present in output; array serialization failure → `Err(FormatError)`; NaN/Inf Float renders as literal string. Run `cargo test` — expect failures.
- [x] 5.3 GREEN: implement `RecordFormatter` for `LogfmtFormatter`: build ordered `ts=`, `level=`, `msg=`, optional `logger=`, then record attrs, then context attrs (exclude `logger`); apply quoting rules; serialize arrays via `serde_json::to_string`. Run `cargo test` — all `logfmt` tests pass.

## Phase 6: Pipeline Wiring

- [x] 6.1 Add `kitlogger-formatter = { path = "../kitlogger-formatter" }` to `crates/kitlogger/Cargo.toml`.
- [x] 6.2 RED: write integration test in `crates/kitlogger/tests/` (or `#[cfg(test)]`) that calls `KITLogger::new()`, calls `log_record(&record, Some(&ctx))`, and asserts `ConsoleExporter` output buffer contains expected formatted string. Run `cargo test` — expect compile/runtime failure.
- [x] 6.3 Modify `crates/kitlogger/src/lib.rs`: import `kitlogger_formatter::{RecordFormatter, LogFormat, formatter_from_config}`; add `formatter: Box<dyn RecordFormatter>` field; default to `formatter_from_config(LogFormat::Json)` in `new()`; add `with_format(format: LogFormat)` constructor; add `log_record(&self, record: &LogRecord, context: Option<&LogContext>) -> Result<(), AdapterError>` method using design contract; keep existing `log()` for back-compat.
- [x] 6.4 GREEN: run `cargo test --workspace` — integration test passes; no regressions in existing tests.

## Phase 7: Verification

- [x] 7.1 Run `cargo clippy --workspace -- -D warnings`; fix all warnings.
- [x] 7.2 Run `cargo fmt --all -- --check`; apply fixes if needed.
- [x] 7.3 Run full `cargo test --workspace`; all tests pass with zero failures.
- [x] 7.4 Smoke-check: confirm `kitlogger-formatter` has no dependency on any exporter or I/O crate (inspect `Cargo.toml`).
