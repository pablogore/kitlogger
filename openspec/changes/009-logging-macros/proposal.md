# Proposal: Logging Macros (KIT-015)

## Intent

The Structured Logging Core (Logger, LogRecord, LogContext, LogAttribute) requires
developers to build attribute slices and call `logger.info(msg, &attrs)` by hand for
every log line. This is verbose and error-prone. Provide ergonomic `macro_rules!`
macros (`trace!`, `debug!`, `info!`, `warn!`, `error!`) that compress logging call
sites while preserving every capability of the Logger API: structured attributes,
explicit context, severity mapping, validation, and formatter/exporter independence.

## Scope

### In Scope
- Five severity macros: `trace!`, `debug!`, `info!`, `warn!`, `error!`.
- Structured attributes via `key => value` pairs (zero, one, or many).
- Explicit `LogContext` association form (`info!(logger, ctx, "msg", ...)`).
- Formatted messages via `format!`-style args.
- Macro expansion to existing `Logger` severity methods — thin wrappers only.
- New crate `kitlogger-macros`, depending on `kitlogger-log-domain` only.

### Out of Scope
- Formatting, exporting, OpenTelemetry, HTTP/gRPC middleware.
- Audit/security logging macros, attribute redaction, compile-time filtering.
- `#[instrument]`, `#[audit]`, `#[security]` attribute macros.
- Global/thread-local logger — logger is always passed explicitly.

## Capabilities

### New Capabilities
- `logging-macros`: ergonomic `macro_rules!` severity macros that expand to Logger
  severity-method calls, supporting structured attributes, formatted messages, and
  explicit LogContext association.

### Modified Capabilities
- None.

## Approach

New crate `kitlogger-macros` (mirrors `kitlogger-formatter` / `console-exporter`
pattern), keeping `kitlogger-log-domain` pure and the macros independently importable.

Concrete invocation API:
```rust
info!(logger, "message");
info!(logger, "formatted {} message", arg);
info!(logger, "message", key => value, other => value2);
info!(logger, ctx, "message");
info!(logger, ctx, "message", key => value);
```

Expansion (thin wrapper, returns `Result<(), EmitError>` from the Logger call):
```rust
// info!(logger, "msg", k => v)  expands to:
logger.info("msg", &[LogAttribute::new("k".into(), v.into())?])
```

`LogContext` handling: `Logger::log` accepts NO context parameter — it takes only
`(severity, message, &[LogAttribute])`. So the context form merges
`ctx.attributes()` into the attribute slice the macro passes to the logger; the macro
introduces no behavior unavailable through the Logger API (CR-008 satisfied).

Macros invoke `logger.info/debug/...` so severity mapping (FR-010) and validation
(CR-001) flow entirely through the Logger. Values rely on a `From`/`Into`
conversion into `LogAttributeValue` covering all variants (FR-013).

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/kitlogger-macros/` | New | New crate with `macro_rules!` macros + tests |
| `Cargo.toml` (workspace) | Modified | Add `crates/kitlogger-macros` member |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Macro hygiene (captured idents, `$crate` paths) | Med | Use `$crate::` paths; hygiene tests |
| Attribute name validated at runtime, not compile time | High | Macro returns `Result`; document; spec covers error path |
| Context has no Logger method — must merge attributes | High | Spec the merge contract explicitly (FR-008) |
| `format!` message allocates a `String` per call | Low | Document; acceptable, matches Logger `&str` contract |

## Rollback Plan

Remove `crates/kitlogger-macros` and its workspace member entry. No other crate
depends on it; domain crate is untouched, so removal is isolated and non-breaking.

## Dependencies

- KIT-005 Structured Logging Core, KIT-006 Log Context, KIT-007 Logger Contracts
  (all present in `kitlogger-log-domain`).

## Success Criteria

- [ ] All five macros expand to the matching `Logger` severity method.
- [ ] Zero / one / many attributes and all `LogAttributeValue` variants supported.
- [ ] Context form merges `ctx.attributes()` into the emitted attribute slice.
- [ ] Records are equivalent to direct Logger API usage (FR-009).
- [ ] `kitlogger-macros` depends only on `kitlogger-log-domain`.
