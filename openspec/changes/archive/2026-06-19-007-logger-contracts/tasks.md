# Tasks: Logger and LoggerFactory Contracts (AS-03)

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 220–310 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | EmitError + Logger + LoggerFactory traits + lib.rs wiring | PR 1 | Additive; all in `kitlogger-log-domain` |

---

## Phase 1: Foundation — EmitError type

- [x] 1.1 **[RED]** Write failing tests in `crates/kitlogger-log-domain/src/emit_error.rs` (new file): `EmitError::Validation` wraps `ValidationError`, `EmitError::LoggerClosed` exists, `From<ValidationError>` converts correctly, `Display` formats both variants, `std::error::Error` is implemented.
- [x] 1.2 **[GREEN]** Create `crates/kitlogger-log-domain/src/emit_error.rs`: define `EmitError` enum with `Validation(ValidationError)` and `LoggerClosed` variants; impl `From<ValidationError>`, `Display`, `std::error::Error`, `Debug`, `Clone`, `PartialEq`, `Eq`.
- [x] 1.3 **[GREEN]** Add `pub mod emit_error;` and `pub use emit_error::EmitError;` to `crates/kitlogger-log-domain/src/lib.rs`.
- [x] 1.4 Run `cargo test -p kitlogger-log-domain` — all Phase 1 tests must pass.

## Phase 2: Core Logger Trait (TDD)

- [x] 2.1 **[RED]** Write failing tests in `crates/kitlogger-log-domain/src/logger.rs` (new file): `Logger` is object-safe (compile-time check: `let _: Box<dyn Logger>`), `Arc<dyn Logger>` compiles, convenience methods `trace/debug/info/warn/error/fatal` each call `log` with the correct `Severity`, `name()` returns the logger name.
- [x] 2.2 **[RED]** Add test: mock `Logger` impl that records `(Severity, String)` tuples; assert each convenience method records the right severity.
- [x] 2.3 **[RED]** Add test: mock `Logger` impl that delegates to `LogRecord::new`; assert empty message returns `Err(EmitError::Validation(ValidationError::EmptyMessage))`.
- [x] 2.4 **[GREEN]** Create `crates/kitlogger-log-domain/src/logger.rs`: define `pub trait Logger: Send + Sync` with `name(&self) -> &str`, `log(&self, severity: Severity, message: &str, attributes: &[LogAttribute]) -> Result<(), EmitError>`, and default convenience methods for all six severity levels.
- [x] 2.5 **[GREEN]** Add `pub mod logger;` and `pub use logger::Logger;` to `crates/kitlogger-log-domain/src/lib.rs`.
- [x] 2.6 Run `cargo test -p kitlogger-log-domain` — all Phase 2 tests must pass.

## Phase 3: LoggerFactory Trait (TDD)

- [x] 3.1 **[RED]** Write failing tests in `crates/kitlogger-log-domain/src/logger_factory.rs` (new file): `LoggerFactory` is object-safe (`let _: Box<dyn LoggerFactory>`), `Arc<dyn LoggerFactory>` compiles, `create_logger` returns `Arc<dyn Logger>` with the correct name, `create_logger_with_context(name, None)` returns logger with empty context, `create_logger_with_context(name, Some(ctx))` returns logger whose emitted records carry factory context attributes.
- [x] 3.2 **[RED]** Add test: mock `LoggerFactory` with a default `LogContext` containing one attribute; assert the returned logger's emitted `LogRecord` includes that attribute without modifying the factory's context.
- [x] 3.3 **[GREEN]** Create `crates/kitlogger-log-domain/src/logger_factory.rs`: define `pub trait LoggerFactory: Send + Sync` with `create_logger(&self, name: &str) -> Arc<dyn Logger>` and `create_logger_with_context(&self, name: &str, default_context: Option<LogContext>) -> Arc<dyn Logger>`.
- [x] 3.4 **[GREEN]** Add `pub mod logger_factory;` and `pub use logger_factory::LoggerFactory;` to `crates/kitlogger-log-domain/src/lib.rs`.
- [x] 3.5 Run `cargo test -p kitlogger-log-domain` — all Phase 3 tests must pass.

## Phase 4: Integration Tests

- [x] 4.1 **[RED]** Add integration test in `crates/kitlogger-log-domain/tests/integration_tests.rs`: mock `LoggerFactory` with default context `{"service": "my-svc"}`; call `create_logger_with_context("auth", Some(extra_ctx))`; emit `info("login", &[])` on returned logger; assert emitted `LogRecord` contains both factory and extra-context attributes (merge semantics).
- [x] 4.2 **[RED]** Add integration test: per-emit attributes passed to `info("login", &[attr])` are appended after context attributes in the emitted `LogRecord`.
- [x] 4.3 **[RED]** Add integration test: `Logger` trait is `Send + Sync` — wrap `Arc<dyn Logger>` in a `std::thread::spawn` closure and call `info`.
- [x] 4.4 **[RED]** Add integration test: `LoggerFactory` trait is `Send + Sync` — wrap `Arc<dyn LoggerFactory>` in a `std::thread::spawn` closure and call `create_logger`.
- [x] 4.5 **[GREEN]** Implement mock logger and factory in `tests/integration_tests.rs` sufficient to make all Phase 4 tests pass (no production code changes needed).
- [x] 4.6 Run `cargo test -p kitlogger-log-domain` — full test suite must pass.

## Phase 5: Verification and Cleanup

- [x] 5.1 Run `cargo clippy -p kitlogger-log-domain -- -D warnings`; resolve all warnings.
- [x] 5.2 Run `cargo fmt -p kitlogger-log-domain`; confirm no diffs.
- [x] 5.3 Verify `lib.rs` public surface: `EmitError`, `Logger`, `LoggerFactory` are all re-exported alongside existing types (`LogRecord`, `LogContext`, `Severity`, `LogAttribute`, `ValidationError`, etc.).
- [x] 5.4 Confirm open design questions are documented: (a) `get_logger` registry deferred to implementing crate, (b) `flush`/`shutdown` lifecycle deferred to exporter, (c) `LoggingConfiguration` (AS-05) integration deferred to future `create_logger_with_config` method.
- [x] 5.5 Mark spec success criteria as satisfied: `[ ]` → `[x]` in `openspec/changes/007-logger-contracts/specs/logger-contracts/spec.md` and `specs/logger-factory/spec.md` for each completed criterion.
