# Design: Record Model Retirement (Migration Plan Phase 6)

## Why this document exists

`tasks.md` originally stated "No `design.md` for this change — it is a verified deletion with no new architecture decision to make." That held for `event.rs` alone. It stopped holding once implementation began: deleting `event.rs` in isolation does not compile, because five other modules in `telemetry-transport-contract` depend on `LogEvent` directly. This document records the verification that justifies deleting those five modules too, in this same phase — a scope correction discovered during implementation, before any code was touched, per this repo's Architecture Conflict Procedure (`AGENTS.md`).

## The five modules

`logger.rs`, `output.rs`, `buffering.rs`, `formatter.rs`, `provider.rs`.

## Verification: are these independent bounded contexts, or `LogEvent`'s satellites?

Four conditions were checked, with evidence, before concluding they are satellites (not independent contexts, per ADR-010's ownership rule: a module with no purpose independent of a retired canonical type is not itself a canonical owner of anything — it retires with the type):

### 1. Each module's core responsibility is defined in terms of `LogEvent`, not incidentally

```
logger.rs:    pub fn log(&self, event: LogEvent)
provider.rs:  fn log(&self, event: LogEvent);
buffering.rs: sender: Sender<LogEvent>; batch: &[LogEvent]
formatter.rs: fn format(&self, event: &LogEvent) -> String;
output.rs:    fn write(&self, event: &LogEvent, formatter: &dyn Formatter) -> io::Result<()>;
```

`LogEvent` is every one of these traits'/structs' primary data type, not a passed-through detail. There is no method on any of the five that operates on something other than `LogEvent` (or a collection of it).

### 2. No independent architectural responsibility — verified by cross-reference, not assumption

Grepped every *other* module in the crate (`batch.rs`, `payload.rs`, `transport.rs`, `error.rs`, `redaction.rs`, `rotation.rs`, `sampling.rs`) for any reference to `LogEvent` or to `logger`/`output`/`buffering`/`formatter`/`provider`:

```
$ grep -n "LogEvent\|crate::logger\|crate::output\|crate::buffering\|crate::formatter\|crate::provider" \
    batch.rs payload.rs transport.rs error.rs redaction.rs rotation.rs sampling.rs
(no matches)
```

Zero. The five modules form a fully self-contained cluster (`logger.rs` orchestrates `provider.rs`, `buffering.rs`, `formatter.rs`, `output.rs` — confirmed by their own internal `use crate::...` lines) with no edge into or out of the rest of the crate. Nothing else in `telemetry-transport-contract` would notice or be affected by their removal.

### 3. Zero production consumers outside the subsystem — verified against the whole workspace, not just these modules

```
$ grep -rln "telemetry-transport-contract" --include="Cargo.toml" crates/ \
    | grep -v "telemetry-transport-contract/Cargo.toml"
(no matches)
```

No crate in the workspace depends on `telemetry-transport-contract` at all — not the five modules specifically, the entire crate. This was already established in the broader migration's earlier architecture review (the crate is orphaned, slated for full removal in Phase 8) and is reconfirmed here.

### 4. Will become dead code immediately once `LogEvent` disappears

Since `LogEvent` is each module's primary operand (condition 1) and each module has zero callers outside this cluster (conditions 2–3), there is no remaining reachable functionality in any of the five once `LogEvent` no longer exists — every public item either takes a `LogEvent`, returns a `LogEvent`, or exists solely to be called by another module in the same cluster that does.

All four conditions hold, with evidence, for all five modules.

## Resulting decision

Delete all six files (`event.rs` plus the five satellites) in this phase, together — not sequentially, and not behind a compatibility shim. A shim would give something a chance to depend on the old shape mid-migration, which nothing does and nothing should; per this migration's established discipline (ADR-010, and the "no speculative capability" principle already invoked earlier in this proposal for `LogEvent`'s unused fields), introducing one here would be manufacturing risk this deletion doesn't have.

This does **not** expand the proposal's architectural scope: no new capability is introduced, no existing capability (`kitlogger-emission-pipeline`, `output-adapter-contracts`, etc.) changes, and `kitlogger_log_domain::LogRecord`/`LogContext` are untouched. It only corrects which files closure-and-deletion actually requires touching to leave `telemetry-transport-contract` compiling with `LogEvent` gone.

## What remains explicitly out of scope

`batch.rs`, `payload.rs`, `transport.rs`, `error.rs`, `redaction.rs`, `rotation.rs`, `sampling.rs` — verified above to have zero coupling to `LogEvent` or to the deleted cluster. These are Phase 7's and Phase 8's concern, unaffected by this phase's expanded scope.
