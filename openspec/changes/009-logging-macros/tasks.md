# Tasks: Logging Macros (KIT-015)

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 280–380 |
| 400-line budget risk | Medium |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | single-pr |
| Chain strategy | size-exception |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: size-exception
400-line budget risk: Medium

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Full `kitlogger-macros` crate | PR 1 | Foundation → impl → tests in one PR; single-pr strategy |

---

## Phase 1: Crate Scaffold & Foundation

- [ ] 1.1 Add `"crates/kitlogger-macros"` to `members` in `Cargo.toml` (workspace root). Satisfies CR-002.
- [ ] 1.2 Create `crates/kitlogger-macros/Cargo.toml` — `name = "kitlogger-macros"`, `edition = "2021"`, single dep `kitlogger-log-domain = { path = "../kitlogger-log-domain" }`. No serde_json, no thiserror. Satisfies CR-002.
- [ ] 1.3 Create `crates/kitlogger-macros/src/lib.rs` with crate-level doc comment and `pub use kitlogger_log_domain::{LogAttribute, LogAttributeValue, EmitError, Logger, LogContext, CorrelationId, TraceId, SpanId};` re-exports. Satisfies CR-003 (`$crate::` hygiene).
- [ ] 1.4 **RED** — Write unit test `into_attribute_value_str` in `lib.rs` `#[cfg(test)]` asserting `(&"prod").into_attribute_value() == LogAttributeValue::String("prod".to_string())`. Run `cargo test -p kitlogger-macros` — expect compile error (trait not defined yet).
- [ ] 1.5 **RED** — Write unit tests for all five `IntoAttributeValue` impls: `String`, `i64`, `f64`, `bool`. Run — expect same compile error.
- [ ] 1.6 **GREEN** — Define `pub trait IntoAttributeValue { fn into_attribute_value(self) -> LogAttributeValue; }` and five `impl` blocks (`&str`, `String`, `i64`, `f64`, `bool`) in `lib.rs`. Run `cargo test -p kitlogger-macros` — all five trait tests pass. Satisfies FR-013.

---

## Phase 2: `info!` Macro — RED → GREEN (reference macro)

- [ ] 2.1 **RED** — Write `MockLogger` struct (records `Vec<(Severity, String, Vec<LogAttribute>)>`) inside `#[cfg(test)]` in `lib.rs`. Write failing test `info_bare_message` calling `info!(logger, "hello")` expecting `Ok(())` and recorded severity `Severity::Info`. Run — expect compile error (macro not defined).
- [ ] 2.2 **RED** — Write failing tests for all remaining `info!` forms: `info_format_args`, `info_single_attr`, `info_multi_attr`, `info_ctx_bare`, `info_ctx_with_attr`. Run — same compile error. Satisfies FR-002.
- [ ] 2.3 **GREEN arm 1** — Implement `macro_rules! info` arm `($l:expr, $msg:literal)` expanding to `$l.info($msg, &[])`. Run — `info_bare_message` passes.
- [ ] 2.4 **GREEN arm 2** — Add format arm `($l:expr, $fmt:literal, $($a:expr),+ $(,)?)` expanding to `$l.info(&format!($fmt, $($a),+), &[])`. Run — `info_format_args` passes. Satisfies FR-002 formatted-message scenario. NOTE: arm order must come BEFORE attr arm to avoid ambiguity.
- [ ] 2.5 **GREEN arm 3** — Add attr arm `($l:expr, $msg:literal, $($k:ident => $v:expr),+ $(,)?)` building `[LogAttribute::new(stringify!($k).to_string(), IntoAttributeValue::into_attribute_value($v))?]` array and calling `$l.info($msg, &__attrs)`. Use `$crate::` paths throughout. Run — `info_single_attr` and `info_multi_attr` pass. Satisfies CR-001, FR-002, CR-003.
- [ ] 2.6 **GREEN arm 4** — Add ctx-bare arm `($l:expr, $ctx:expr, $msg:literal)` with context fold logic: `ctx.attributes().to_vec()` → push `correlation_id` if `Some` → push `trace_id` if `Some` → push `span_id` if `Some`. Run — `info_ctx_bare` passes. Satisfies FR-008 fold order.
- [ ] 2.7 **GREEN arm 5** — Add ctx+attr arm `($l:expr, $ctx:expr, $msg:literal, $($k:ident => $v:expr),+ $(,)?)` extending the fold vec with inline attr pairs, then calling `$l.info($msg, &__v)`. Run — `info_ctx_with_attr` passes. Satisfies FR-008 fully.

---

## Phase 3: Context Fold Unit Tests

- [ ] 3.1 **RED** — Write `fold_ctx_correlation_id_only` test: `LogContext` with `correlation_id = Some("req-1")`, no other ids, no attrs. Assert `logger.info` receives slice `[("correlation_id", String("req-1"))]`. Run — red (test helpers missing).
- [ ] 3.2 **RED** — Write `fold_ctx_all_three_ids` test: all three ids set, no attrs. Assert order: `correlation_id`, `trace_id`, `span_id`. Satisfies FR-008 scenario.
- [ ] 3.3 **RED** — Write `fold_ctx_attrs_only` test: two attrs, no ids. Assert only those two attrs, no id keys. Satisfies FR-008 scenario.
- [ ] 3.4 **RED** — Write `fold_ctx_full_plus_inline` test: `[env="prod"]` + all three ids + inline `region => "eu"`. Assert five attrs in order: `env`, `correlation_id`, `trace_id`, `span_id`, `region`. Satisfies FR-008 fully-populated scenario.
- [ ] 3.5 **GREEN** — All fold tests pass with the Phase 2 context arm implementation. If any fail, fix the context fold order in arms 4–5. No new code needed if Phase 2 was correct.

---

## Phase 4: Remaining Four Macros (debug!, trace!, warn!, error!)

- [ ] 4.1 **RED** — Write parallel tests for `debug!`, `trace!`, `warn!`, `error!` covering: bare message (severity check), format args, single attr, ctx+attr. Run — compile error (macros not defined). Satisfies FR-001 severity mapping.
- [ ] 4.2 **GREEN** — Implement `macro_rules! debug`, `trace`, `warn`, `error` — identical arm structure to `info!`, substituting the severity method name. Mark all five macros `#[macro_export]`. Run `cargo test -p kitlogger-macros` — all tests pass. Satisfies FR-001.

---

## Phase 5: Return Type & Error Propagation Tests

- [ ] 5.1 **RED** — Write `returns_ok_on_success`: logger emits ok, `info!(l, "msg")` returns `Ok(())`. Satisfies FR-003 success path.
- [ ] 5.2 **RED** — Write `propagates_logger_err`: mock returns `Err(EmitError::LoggerClosed)`, assert macro returns same. Satisfies FR-003 error path.
- [ ] 5.3 **RED** — Write `propagates_validation_err_empty_message`: validating mock, `info!(l, "")` returns `Err(EmitError::Validation(ValidationError::EmptyMessage))`. Satisfies FR-003 validation path.
- [ ] 5.4 **GREEN** — All three tests must pass with existing implementation. If any fail, fix `?`-propagation in the macro arms.

---

## Phase 6: Integration Tests — Equivalence with Direct Logger API

- [ ] 6.1 Create `crates/kitlogger-macros/tests/integration.rs`. Add `use kitlogger_macros::{info, warn, LogAttribute, LogAttributeValue, Logger, Severity};`.
- [ ] 6.2 **RED** — Write `macro_no_attrs_equiv_direct`: capturing logger, compare `info!(l, "hello")` record against manually built `l.info("hello", &[])` record. Assert tuples equal. Satisfies FR-009.
- [ ] 6.3 **RED** — Write `macro_with_attrs_equiv_direct`: `info!(l, "hello", k => "v")` vs hand-built `&[LogAttribute::new("k".to_string(), LogAttributeValue::String("v".to_string()))?]`. Assert equal. Satisfies FR-009.
- [ ] 6.4 **RED** — Write `hygiene_no_domain_imports`: crate imports only `kitlogger_macros::info`, calls `info!(l, "msg")` — assert it compiles without explicit `LogAttribute`/`LogAttributeValue` import. Satisfies CR-003.
- [ ] 6.5 **GREEN** — Run `cargo test -p kitlogger-macros` — all integration tests pass. Fix any `$crate::` path issues found.

---

## Phase 7: Verification

- [ ] 7.1 Run `cargo clippy -p kitlogger-macros -- -D warnings` — zero warnings. Fix any flagged issues.
- [ ] 7.2 Run `cargo fmt --package kitlogger-macros -- --check` — passes.
- [ ] 7.3 Run `cargo test --workspace` — all workspace tests pass (no regressions in existing crates).
- [ ] 7.4 Verify `crates/kitlogger-macros/Cargo.toml` has no dependency other than `kitlogger-log-domain`. Satisfies CR-002.
