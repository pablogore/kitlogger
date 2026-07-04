# Proposal: Orchestration Fold (Migration Plan Phase 5)

## Intent

Phases 3 (change 013 — Redaction, Sampling) and 4 (change 014 — Output Port, `file-exporter`, Buffer, format selection) each landed their capability standalone, deliberately called from nowhere. This proposal folds them all into `KITLogger`'s actual `log`/`log_record` execution path, in the order ADR-008 §5 already fixed: **filter → sample → redact → buffer → format → dispatch**. It also lifts the "no behavioral change" restriction change 012 placed on `LoggingConfig`'s fields, and retires the orphaned `LoggerProvider` concept by making `KITLogger` the sole owner of multi-output dispatch.

This is architecture-freezing only, matching every prior phase in this initiative — no code is written here.

## A Gap Found Before Scoping This Phase

`kit_config::OutputTarget` has exactly three variants — `Console`, `Stdout`, `Stderr` — confirmed by reading the sibling repo's source directly. There is no `File` variant, and `LoggingConfig` has no file-path field anywhere. `file-exporter` (change 014) is complete and tested, but **`kit_config` cannot express "use file output" today.**

Per explicit decision: this phase wires **Console output only**. `file-exporter`'s registration into `KITLogger`'s dispatch remains an open, explicitly documented follow-up, blocked on `kit_config` gaining an `OutputTarget::File` variant and a file-path field — a cross-repo schema change outside this repository's control. `file-exporter` itself is unaffected and remains exactly as change 014 left it.

## ADR-010 Domain Model Declaration

No new public domain model is introduced by this change — it wires together models that already exist (`Sampler`, `Redactor`, `Buffer`, the format-selection mapping, `output-adapter-contracts`'s registry). Per ADR-010's Enforcement section, this is noted explicitly because it's the exception, not because a declaration is required: nothing here is a Canonical Owner/Model/Consumer/Existing-Competing-Model question. The one prior-art concept this change formally retires is `telemetry_transport_contract::provider::LoggerProvider` — its multi-output dispatch role is absorbed into `KITLogger` (Migrate, per ADR-008), not kept as a second type.

## Scope

### In Scope

- `KITLogger`'s `log`/`log_record` execution path gains the full sequence: enabled gate → level filter → sample → redact → buffer → format → dispatch.
- `KITLogger` registers `console-exporter` by default at construction, per `LoggingConfig.output.targets` (`Console`/`Stdout`/`Stderr` — mapped onto `console-exporter`'s existing stream-routing configuration).
- `KITLogger` owns exactly one `output-adapter-contracts` registry instance; this is the sole dispatch mechanism — no second one is introduced.
- `LoggingConfig.enabled`, `.level`, `.sampling`, `.redact`, `.buffering`, `.format` all become behaviorally active for the first time.
- Lifting change 012's `kitlogger-config-integration` FR-003 restriction ("LoggingConfig fields MUST NOT gate or alter behavior") — that restriction existed only until this phase.

### Out of Scope

- Registering `file-exporter` (see "A Gap Found" above) — blocked on an external `kit_config` schema change.
- Any change to `kit_config`, `output-adapter-contracts`, `console-exporter`'s Port implementation, `kitlogger-sampling`, `kitlogger-redaction`, `kitlogger-formatter`, or `formatter-contract` — all consumed as already-frozen, unmodified capabilities.
- OTLP/Loki/Sentry/etc. — unaffected, still fully out of scope per change 014.
- Deleting `telemetry-transport-contract` (Phase 8) or its `LoggerProvider`/`logger.rs` source — this change only stops that concept from being the canonical dispatch owner; physical deletion of the orphaned crate remains Phase 8's job.

## Capabilities

### New Capabilities

- `kitlogger-emission-pipeline`: the end-to-end sequencing behavior of `KITLogger::log`/`log_record`.

### Modified Capabilities

- `kitlogger-config-integration` (change 012): FR-003 ("No Behavioral Change from LoggingConfig Fields") is superseded — `LoggingConfig`'s behavioral fields now drive real behavior, as specified by `kitlogger-emission-pipeline`.

## Approach

Every stage this phase wires already exists as a complete, tested, standalone capability (changes 013 and 014) — this phase's only new behavior is the *sequencing* and *default registration*, not any stage's internal logic. `Severity::Fatal` (six-variant domain enum) has no corresponding `kit_config::LogLevel` variant (five variants, no `Fatal`) to threshold against — resolved as a design decision, not a config gap: `Fatal` always proceeds regardless of the configured level, since no configurable threshold can exceed the domain's own most severe level.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/kitlogger/src/lib.rs` | Modified | `log`/`log_record` gain the full pipeline sequence; construction registers `console-exporter` by default |
| `crates/kitlogger/Cargo.toml` | Modified | Add `kitlogger-sampling`, `kitlogger-redaction`, `output-adapter-contracts`, `console-exporter`, `file-exporter` (built, not yet registered) as dependencies |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `file-exporter` sits built-and-tested but permanently unreachable if the `kit_config` gap is never closed | Medium | Explicitly documented as a follow-up dependency in this proposal and design.md, not silently dropped |
| Buffering's deferred flush changes `log()`'s observable timing (a call may return before the record is actually formatted/dispatched) | Medium | Explicit requirement (FR-006) states this; `KITLogger`'s existing `flush()`/`shutdown()` lifecycle methods (already implemented per `LifecycleAdapter`) must guarantee all buffered records are flushed — verified by this change's own tests, not assumed |
| `console-exporter`'s own `FlushStrategy` (I/O-level) and the new pipeline `Buffer` (application-level batching) could be conflated if not composed carefully | Low | Already flagged and scoped correctly in changes 013/014's design docs; this phase's tests must confirm both operate at their own level without one silently disabling the other |
| Two divergent severity-threshold schemes (5-variant `LogLevel` vs. 6-variant `Severity`) could be handled inconsistently by a future contributor | Low | Explicit FR (FR-002) states the resolution; not left to implementer discretion |

## Rollback Plan

`KITLogger`'s pipeline wiring is entirely internal to the `kitlogger` crate. Reverting restores `log`/`log_record` to their pre-Phase-5 behavior (format + dispatch to console unconditionally, no filter/sample/redact/buffer) and removes the new dependency edges — isolated to one crate, no external consumers affected (nothing outside this workspace consumes `kitlogger` as a published dependency yet).

## Dependencies

- ADR-008 (Migration Plan Phase 5), ADR-010.
- Change 012 (`kitlogger-config-integration`), change 013 (`kitlogger-sampling`, `kitlogger-redaction`), change 014 (`output-adapter-contracts`, `console-exporter`'s Port implementation, `file-exporter`, `kitlogger-buffering`, `kitlogger-format-selection`) — all treated as complete and accepted inputs. If any of these three changes have not yet been archived into `openspec/specs/` by the time this change is applied, their content as frozen in their own change folders is the authoritative reference.
- `kit_config::{LoggingConfig, LogLevel, OutputTarget}` (external, sibling repo, read-only reference).

## Success Criteria

- [ ] `KITLogger::log`/`log_record` perform, in order: enabled gate, level filter, sample, redact, buffer, format, dispatch.
- [ ] `LoggingConfig.enabled = false` results in zero further pipeline processing.
- [ ] `Severity::Fatal` always proceeds regardless of the configured `LogLevel`.
- [ ] `console-exporter` is registered by default at construction; `file-exporter` is not.
- [ ] `KITLogger` holds exactly one dispatch registry; `LoggerProvider` is not reintroduced in any form.
- [ ] `KITLogger`'s `flush()`/`shutdown()` guarantee all buffered records are flushed before returning.
- [ ] `kitlogger-config-integration`'s FR-003 is formally superseded, not silently ignored.
- [ ] No two canonical models remain for the same concept within the scope of this change (ADR-010).
