# Design: Logging Macros (KIT-015)

## Technical Approach

New crate `kitlogger-macros` (single `src/lib.rs`) exporting five `macro_rules!`
macros — `trace!`, `debug!`, `info!`, `warn!`, `error!` — that expand to the
matching `Logger` severity method. Macros are thin wrappers: they build a
`&[LogAttribute]` slice and forward to `logger.<severity>(msg, &attrs)`, returning
its `Result<(), EmitError>`. The crate depends only on `kitlogger-log-domain`.

Two non-obvious facts from the code drive the design:
1. `LogAttribute::new(name: String, value: LogAttributeValue)` — name is `String`,
   so `stringify!(key).to_string()` is required, and `new` returns `Result`.
2. There is **no** `From<T> for LogAttributeValue` in the domain crate. The orphan
   rule forbids adding `From` (foreign trait) for a foreign type from this crate.
   So value conversion uses a **local** trait `IntoAttributeValue` (local trait on
   foreign type is allowed) implemented for `&str`, `String`, `i64`, `f64`, `bool`.

## Architecture Decisions

| Decision | Choice | Rejected | Rationale |
|----------|--------|----------|-----------|
| Macro engine | `macro_rules!` | proc-macro | 5 simple wrappers; no parsing/codegen need; no extra crate, no `syn`/`quote` build cost |
| File layout | single `src/lib.rs` | module per macro | 5 short arms + 1 trait; submodules add noise; `#[macro_export]` is crate-root anyway |
| Value conversion | local `IntoAttributeValue` trait | `Into`/`From` (proposal) | orphan rule blocks `From<T> for LogAttributeValue` here; local trait is legal and keeps domain pure |
| Context fold order | attrs, then correlation_id, trace_id, span_id | arbitrary | deterministic, matches `LogContext::Display` order; reproducible records |
| Attr-name validation | runtime via `LogAttribute::new?` | compile-time | domain owns the rule; macro propagates `ValidationError` → `EmitError` |
| Error handling | return `Result<(), EmitError>` | swallow / panic | caller uses `?` or `.ok()`; preserves Logger contract |

## Interfaces / Contracts

Local conversion trait (declared and impl'd inside `kitlogger-macros`):

```rust
pub trait IntoAttributeValue { fn into_attribute_value(self) -> LogAttributeValue; }
impl IntoAttributeValue for &str   { /* String(self.to_string()) */ }
impl IntoAttributeValue for String { /* String(self) */ }
impl IntoAttributeValue for i64    { /* Integer(self) */ }
impl IntoAttributeValue for f64    { /* Float(self) */ }
impl IntoAttributeValue for bool   { /* Boolean(self) */ }
```

Canonical expansion — `info!` with attributes (others are identical bar severity):

```rust
// info!(logger, "u={}", id, user => name, count => 3i64)  expands to:
{
    let __attrs = [
        $crate::LogAttribute::new(
            "user".to_string(),
            $crate::IntoAttributeValue::into_attribute_value(name),
        )?,
        $crate::LogAttribute::new(
            "count".to_string(),
            $crate::IntoAttributeValue::into_attribute_value(3i64),
        )?,
    ];
    $crate::__logger_method!(logger, info, &format!("u={}", id), &__attrs)
}
```

Context form folds `LogContext` into a `Vec<LogAttribute>` before the call:

```rust
// info!(logger, ctx, "msg", k => v)  fold step:
let mut __v: Vec<$crate::LogAttribute> = ctx.attributes().to_vec();          // 1. context attrs
if let Some(id) = ctx.correlation_id() {                                      // 2. correlation_id
    __v.push($crate::LogAttribute::new("correlation_id".to_string(),
        $crate::LogAttributeValue::String(id.to_string()))?);
}
if let Some(id) = ctx.trace_id()  { /* push "trace_id"  String(id.to_string()) */ }  // 3.
if let Some(id) = ctx.span_id()   { /* push "span_id"   String(id.to_string()) */ }  // 4.
__v.extend([ /* key => val attrs as above */ ]);                              // 5. inline attrs
logger.info("msg", &__v)
```

`correlation_id` / `trace_id` / `span_id` are NOT reserved (reserved =
`timestamp`, `severity`, `message`, `attributes`), so the folded names validate.

## Macro Arms (per macro, e.g. `info!`)

```
($l:expr, $msg:literal)                              // bare message
($l:expr, $fmt:literal, $($a:expr),+ $(,)?)          // format args  → format!(...)
($l:expr, $msg:literal, $($k:ident => $v:expr),+ $(,)?)        // attrs
($l:expr, $ctx:expr, $msg:literal)                            // ctx, bare
($l:expr, $ctx:expr, $msg:literal, $($k:ident => $v:expr),+ $(,)?)  // ctx + attrs
```

Disambiguation: `key => value` uses the `=>` separator (no other arm has it), and
the context form is detected by a second `$expr` before the literal. The format
arm and the ctx-bare arm are distinct because format args follow a literal with
`,` while ctx precedes the literal.

## Hygiene

- Every domain type is referenced via `$crate::` (`$crate::LogAttribute`,
  `$crate::LogAttributeValue`, `$crate::IntoAttributeValue`), re-exported from
  `kitlogger-macros` so call sites need no `use`.
- `kitlogger-macros/src/lib.rs` adds `pub use kitlogger_log_domain::{LogAttribute,
  LogAttributeValue, EmitError, Logger, LogContext};` so `$crate::` paths resolve.
- Internal binding names use `__`-prefixed identifiers to avoid capture.
- All 5 macros carry `#[macro_export]`.

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `crates/kitlogger-macros/Cargo.toml` | Create | `name = "kitlogger-macros"`, `edition = "2021"`, dep `kitlogger-log-domain` only; no `serde_json`, no `thiserror` |
| `crates/kitlogger-macros/src/lib.rs` | Create | re-exports, `IntoAttributeValue` trait + impls, 5 `#[macro_export]` macros, unit tests |
| `Cargo.toml` (workspace) | Modify | add `"crates/kitlogger-macros"` to `members` |

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | Each macro form (bare, format, attrs, ctx, ctx+attrs) expands and emits | `MockLogger` recording (severity, message, attrs) |
| Unit | All value types (&str/String/i64/f64/bool) → correct `LogAttributeValue` | assert variant per type |
| Unit | Context fold order + folded id names present and correct | build `LogContext` with attrs + all 3 ids, assert slice order |
| Unit | Invalid attr name → `Err(EmitError::Validation(...))` propagated | `info!(l, "m", bad_NAME => 1i64)` is impossible (ident lowercased by author); test runtime path via reserved-like name documented |
| Integration | Records equivalent to direct Logger API (FR-009) | compare macro emit vs hand-built `&[LogAttribute]` |

## Migration / Rollout

No migration required. Additive new crate; nothing depends on it. Rollback = remove
the crate dir and the workspace member line.

## Open Questions

- [ ] Should `IntoAttributeValue` also cover `i32`/`u32`/`&String` for ergonomics?
  Deferred to tasks; minimum viable set is the five confirmed types.
