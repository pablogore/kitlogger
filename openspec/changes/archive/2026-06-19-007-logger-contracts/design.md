# Design: Logger and LoggerFactory Contracts (AS-03)

## Technical Approach

Define two object-safe trait abstractions, `Logger` and `LoggerFactory`, in the
existing `kitlogger-log-domain` crate. They are pure domain contracts: transport,
exporter, and storage agnostic. `Logger` emits `LogRecord` instances built from
severity, message, attributes, and an inherited `LogContext`. `LoggerFactory`
creates named, context-seeded loggers. Concrete implementations (e.g. one bridging
to `console-exporter`) live outside this crate. New modules: `logger.rs`,
`logger_factory.rs`, `emit_error.rs`.

## Architecture Decisions

| Decision | Choice | Rejected | Rationale |
|----------|--------|----------|-----------|
| Object safety | Trait methods take `&str` / owned args, never `impl Into<String>`; factory returns `Arc<dyn Logger>` | `impl Into<String>` in methods (the 005 draft signature) | Generic method params make a trait NOT object-safe — `Box<dyn Logger>` would fail to compile. `&str` keeps `dyn Logger` usable, which the factory pattern requires. |
| Dispatch model | Trait objects (`Arc<dyn Logger>`) | Generic `<L: Logger>` everywhere | Spec requires runtime-named loggers from a factory and DI; static dispatch cannot return heterogeneous loggers from one method. |
| Sharing type | `Arc<dyn Logger>` | `Box<dyn Logger>` | Spec demands thread-safe concurrent reuse and "retrieve by name for reuse"; `Arc` allows shared ownership across threads. |
| Error type | Dedicated `EmitError` wrapping `ValidationError` | Reuse `ValidationError` directly | Emission can fail for reasons beyond record validation (closed/transport-down). `EmitError` keeps the domain open while embedding `ValidationError` for record-level faults. |
| Severity API | One `log(severity, msg, attrs)` plus convenience `trace/debug/info/warn/error/fatal` | Only per-level methods | A generic `log` covers all six `Severity` variants (incl. `Fatal`, which the 005 draft omitted) and lets convenience methods delegate. |
| Timestamp | `Logger` stamps `SystemTime::now()` at emit | Caller supplies timestamp | `LogRecord::new` requires a `SystemTime`; centralizing it in the logger keeps the public API minimal and consistent. |
| Context merge | Factory default context + logger-local context merged at construction; per-emit attributes appended | Merge on every emit call | Spec marks context immutable and inherited; merging once at creation honors immutability and meets the 1ms emit budget. |

## Data Flow

```
App ─create_logger(name, ctx?, cfg?)─▶ LoggerFactory ─▶ Arc<dyn Logger>
                                                          │ (holds merged LogContext + min severity)
App ─info("msg", attrs)──────────────▶ Logger.log(Info, "msg", attrs)
                                          │ severity < min? ─▶ drop (Ok)
                                          ▼
                       LogRecord::new(now, sev, msg, ctx.attrs ++ attrs)
                                          │ EmptyMessage ─▶ Err(EmitError::Validation)
                                          ▼
                       (impl) hand off to transport/exporter — outside this crate
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `crates/kitlogger-log-domain/src/logger.rs` | Create | `Logger` trait |
| `crates/kitlogger-log-domain/src/logger_factory.rs` | Create | `LoggerFactory` trait |
| `crates/kitlogger-log-domain/src/emit_error.rs` | Create | `EmitError` enum |
| `crates/kitlogger-log-domain/src/lib.rs` | Modify | Add modules + re-exports |

## Interfaces / Contracts

```rust
// emit_error.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmitError {
    Validation(ValidationError), // record construction failed
    LoggerClosed,                // emit after shutdown
}
impl From<ValidationError> for EmitError { /* ... */ }
impl std::error::Error for EmitError {}

// logger.rs — object-safe (no generic method params)
pub trait Logger: Send + Sync {
    fn name(&self) -> &str;
    fn log(&self, severity: Severity, message: &str, attributes: &[LogAttribute])
        -> Result<(), EmitError>;
    fn trace(&self, m: &str, a: &[LogAttribute]) -> Result<(), EmitError> { self.log(Severity::Trace, m, a) }
    fn debug(&self, m: &str, a: &[LogAttribute]) -> Result<(), EmitError> { self.log(Severity::Debug, m, a) }
    fn info (&self, m: &str, a: &[LogAttribute]) -> Result<(), EmitError> { self.log(Severity::Info,  m, a) }
    fn warn (&self, m: &str, a: &[LogAttribute]) -> Result<(), EmitError> { self.log(Severity::Warn,  m, a) }
    fn error(&self, m: &str, a: &[LogAttribute]) -> Result<(), EmitError> { self.log(Severity::Error, m, a) }
    fn fatal(&self, m: &str, a: &[LogAttribute]) -> Result<(), EmitError> { self.log(Severity::Fatal, m, a) }
}

// logger_factory.rs
pub trait LoggerFactory: Send + Sync {
    fn create_logger(&self, name: &str) -> Arc<dyn Logger>;
    fn create_logger_with_context(&self, name: &str, default_context: Option<LogContext>)
        -> Arc<dyn Logger>;
}
```

`LoggingConfiguration` (AS-05) is not yet in the workspace; the factory accepts it
via a future `create_logger_with_config` method. This change exposes a
`min_severity` hook on implementations only — the trait stays config-type-free to
avoid a forward dependency, satisfying the "no breaking changes / additive" constraint.

## Testing Strategy (Strict TDD — `cargo test`)

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | `EmitError` From/Display, object safety | Compile-time `let _: Arc<dyn Logger>`; assert `From<ValidationError>` |
| Unit | Convenience methods delegate to `log` | Mock `Logger` recording `(severity, msg)` tuples |
| Unit | Empty message → `EmitError::Validation` | Mock logger calling `LogRecord::new` |
| Integration | Context inheritance + per-emit attribute merge | Mock factory with default `LogContext`, assert merged attrs on record |
| Integration | Bridge to `ConsoleExporter` (005) | Test `Logger` impl forwarding formatted record + `Severity` to exporter |

## Migration / Rollout

No migration required — additive contracts. The 005 design's `impl Into<String>`
signatures are superseded by the object-safe `&str` forms here; no concrete logger
was shipped against them.

## Open Questions

- [ ] Should `LoggerFactory` add `get_logger(name)` retrieval/registry now or defer to the implementing crate? (Spec mentions "retrieve by name for reuse".)
- [ ] Lifecycle: does `Logger` need `flush`/`shutdown`, or is that solely the exporter's responsibility (currently owned by `ConsoleExporter`)?
- [ ] Final `LoggingConfiguration` shape (AS-05) — confirm `min_severity` is the only behavior-affecting field.
