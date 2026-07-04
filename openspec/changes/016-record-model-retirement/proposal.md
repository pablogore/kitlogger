# Proposal: Record Model Retirement (Migration Plan Phase 6)

## Intent

ADR-008's original migration sequencing described Phase 6 as "retire `event::LogEvent`, re-pointing everything migrated above onto `kitlogger_log_domain::LogRecord`/`LogContext`." Verified against the actual specs produced by changes 013, 014, and 015: **there is nothing left to re-point.** Every capability built in those three phases (`kitlogger-redaction`, `kitlogger-sampling`, `output-adapter-contracts`, `file-exporter`, `kitlogger-buffering`, `kitlogger-format-selection`, `kitlogger-emission-pipeline`) was designed from the start against the already-canonical `kitlogger_log_domain::LogRecord` (per ADR-008 §3), and none of them ever referenced `telemetry_transport_contract::event::LogEvent`. Change 013's own proposal already refers to it in passing as "the now-retired `LogEvent`."

This proposal is therefore a **closure and deletion** phase, not a re-pointing phase. No new capability is introduced or modified.

## Verification Performed

`rg -il "LogEvent" openspec/changes/013-redaction-sampling openspec/changes/014-output-consolidation openspec/changes/015-orchestration-fold` returns exactly one match, and it is a contrastive historical note in change 013's proposal.md explaining why `Redactor` operates on `LogRecord` instead of `LogEvent` — not a dependency.

## Field Parity (the original audit's flagged concern)

`LogEvent` carried fields `level` (→ `Severity`, already on `LogRecord`), `message` (→ already on `LogRecord`), `timestamp` (→ already on `LogRecord`), `fields: HashMap<String, serde_json::Value>` (→ `attributes: Vec<LogAttribute>`, already on `LogRecord`), and `correlation_id` (→ already on `LogContext`). All of these already have a canonical home.

`LogEvent` also carried `target`, `module`, `file: Option<String>`, and `line: Option<u32>` — source-location/logger-name metadata with no dedicated field on `LogRecord` or `LogContext`. No capability built in Phases 3–5 requires any of these. Per the structured-logging-core domain model's own design (attributes are the general enrichment mechanism — see `LogAttribute`), a host needing this metadata attaches it as an ordinary attribute; this is not a gap requiring new fields, and inventing dedicated fields for a need nothing in this migration has would be exactly the kind of speculative, unrequested capability ADR-010's spirit argues against.

## Scope

### In Scope

- Delete `telemetry_transport_contract::event::LogEvent` and its module (`crates/telemetry-transport-contract/src/event.rs`).
- Remove `event`'s `mod`/`pub use` declarations from `telemetry-transport-contract/src/lib.rs`.
- Formal record, in this proposal, that field parity was checked (not assumed) and found already satisfied.

### Out of Scope

- Any change to `kitlogger_log_domain::LogRecord`/`LogContext` — no new fields are added.
- Deleting any other module of `telemetry-transport-contract` (Phase 7 handles `batch.rs`/`payload.rs`/the duplicate `BackpressureSignal`; Phase 8 removes the crate once every module, including this one, is empty).
- Anything related to Phase 7's transport/envelope cleanup — independent, parallel track per change 012's design.md.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- None.

## Approach

This is a deletion, not a design. `event.rs` is removed because it has already been fully superseded by decisions made in earlier, already-accepted phases (ADR-008 §3, and the actual specs of changes 013–015) — not because this proposal makes a new decision about it.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/telemetry-transport-contract/src/event.rs` | Deleted | `LogEvent` and its builder methods |
| `crates/telemetry-transport-contract/src/lib.rs` | Modified | Remove `event` module declaration and re-export |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| A future spec assumes `target`/`module`/`file`/`line` should be first-class `LogRecord` fields, contradicting this proposal's "use attributes instead" resolution | Low | Explicitly recorded here as the closure decision; a future spec proposing dedicated fields would need to justify why the existing attribute mechanism is insufficient |
| `telemetry-transport-contract`'s own tests reference `LogEvent` and would fail to compile | Medium | These tests are not migrated or preserved — the crate's own test suite is being dismantled module by module across Phases 6–8, not maintained mid-dismantling |

## Rollback Plan

`event.rs` deletion is isolated to a crate with zero remaining workspace dependents. Reverting restores the file and its `lib.rs` declarations — no other crate is affected either way.

## Dependencies

- ADR-008 (Migration Plan Phase 6), ADR-010.
- Changes 013, 014, 015 (verified to have zero dependency on `LogEvent`).

## Success Criteria

- [ ] `telemetry_transport_contract::event::LogEvent` no longer exists.
- [ ] `telemetry-transport-contract/src/lib.rs` has no reference to the deleted module.
- [ ] No other crate in the workspace references `LogEvent` (already true; confirmed again after deletion).
- [ ] Field parity is recorded as checked, not assumed, per the original audit's flagged concern.
