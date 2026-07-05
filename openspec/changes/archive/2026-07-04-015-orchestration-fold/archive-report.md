# Archive Report: Orchestration Fold (Change 015)

## Summary

Change 015 "Orchestration Fold" has been completed, verified, and merged. All 28 implementation tasks are marked complete (5 phases × multiple tasks + final verification = 28/28 checkboxes). The change wired the full KITLogger emission pipeline (enabled gate → level filter → sample → redact → buffer → format → dispatch), retiring the orphaned LoggerProvider concept and activating LoggingConfig's behavioral fields for the first time.

## What Shipped

### Core Deliverable: KITLogger Emission Pipeline

`KITLogger::log`/`log_record` now executes the complete pipeline sequencing:

1. **Enabled gate**: `LoggingConfig.enabled = false` short-circuits all further processing (FR-001)
2. **Level filter**: Records below `LoggingConfig.level` are dropped; `Severity::Fatal` always proceeds (FR-002)
3. **Sampling**: Sampler decision gates further processing (FR-003)
4. **Redaction**: Record is redacted before buffering (FR-004)
5. **Buffering**: Redacted record enters buffer, which defers format+dispatch until flush (FR-005)
6. **Formatting**: Buffer flush formats records using `LoggingConfig.format` selection (FR-007, FR-008)
7. **Dispatch**: Formatted records dispatched via `output-adapter-contracts::Registry` to registered outputs (FR-010)

### Dispatch Ownership Realized

- `KITLogger` holds exactly one `output-adapter-contracts::Registry` instance
- `console-exporter` registered by default at construction
- `LoggerProvider` concept formally retired (no second dispatch orchestrator introduced)
- Satisfies ADR-010 (single canonical owner)

### LoggingConfig Behavioral Fields Activated

Lifting change 012's FR-003 restriction ("no behavioral change from config fields"):

- `.enabled` now controls pipeline entry (FR-001)
- `.level` controls level filtering threshold (FR-002)
- `.sampling` controls sampling gate decision (FR-003)
- `.redact` controls redaction behavior (FR-004)
- `.buffering` controls buffering enabled/disabled and flush conditions (FR-005)
- `.format` controls which formatter is used (FR-007)

This was an intentional scope boundary in change 012, explicitly deferred to this phase.

### Default Output Registration

- Console output (`console-exporter`) registered by default
- File output deliberately NOT registered (blocked by `kit_config` gap — no `OutputTarget::File` variant, no file-path config)
- This constraint is documented and explicit, not a gap

## Capability Specifications Created

### New Capability: `kitlogger-emission-pipeline`

- Canonical spec created at: `openspec/specs/kitlogger-emission-pipeline/spec.md`
- Defines end-to-end behavioral contract for the 7-stage pipeline
- 10 functional requirements (FR-001 through FR-010) with scenarios
- All requirements have corresponding implementation tasks (28/28 complete)

### Modified Capability: `kitlogger-config-integration`

- Delta merged into: `openspec/specs/kitlogger-config-integration/spec.md`
- FR-003 replaced: from "no behavioral change from config fields" to "LoggingConfig fields now drive real behavior per kitlogger-emission-pipeline"
- FR-001, FR-002, FR-004, FR-005 unchanged (construction, validation, removal of with_config, kit-config dependency remain true)

## Process Divergence: Single PR Instead of Four

**Planned approach** (per tasks.md Review Workload Forecast):
- 4 stacked PRs covering units 1–4 (enabled gate, sample+redact, buffer, format+dispatch)
- Rationale: ~400–550 changed lines across isolated work units

**Actual outcome** (per tasks.md "Post-apply note"):
- Shipped as **single PR #44** instead of 4 stacked PRs
- **Reason**: All 5 phases modified the same `crates/kitlogger` functions (`log_record`, `build`, `flush`/`shutdown`). Unlike change 014 (which had 4 separate crates/files), the interleaved hunks within the same functions made post-hoc splitting too risky — manually re-partitioning verified code risked introducing bugs for no material review benefit once implemented as a unit.
- **Outcome**: PR shipped as one diff (reviewed as one unit, merged as one commit)
- **Metrics**: This decision is documented in tasks.md and the PR description for full visibility

## Bugs Found and Fixed (Pre-Merge Gap Review)

Two real behavioral gaps were discovered and fixed before merge (both committed as part of PR #44):

### Bug 1: Data Loss on Partial Batch Dispatch Failure

**What**: `format_and_dispatch` aborted its per-record loop with early `?` on the first dispatch failure. Since batches are already removed from Buffer via `mem::take` before dispatch, any record after the first failure was silently lost (neither dispatched nor re-buffered).

**Impact**: Contradicts `flush()`/`shutdown()`'s documented guarantee to drain all buffered records through format+dispatch.

**Fix**: Changed to attempt every record in the batch regardless of earlier failures, aggregating failures into a single error.

**Regression test**: `batch_dispatch_failure_still_attempts_every_record` ✓

### Bug 2: Dead Time-Based Flush Condition

**What**: `Buffer::try_flush()` (the time-based half of FR-005's "size/time flush conditions") was never called from `log_record` — only from its own unit tests — so `BufferingConfig.flush_interval_ms` had no effect in the real pipeline.

**Impact**: Time-based flush config setting was silently ignored in production.

**Fix**: Added opportunistic `try_flush()` check whenever `add()` doesn't already trigger a size-based flush, so time-based flushing now works.

**Regression test**: `log_record_checks_time_based_flush_when_size_has_not_been_reached` ✓

## Follow-Up Fix: Dispatch Failures Distinguishable from Initialization Failures (Issue #46)

A third round of review (after the two bugs above were already fixed) found that `KITLogger::dispatch()`/`format_and_dispatch()` collapsed every dispatch failure — partial or total — into the generic `AdapterError::InitializationFailed`, losing the distinction a caller might need for retries, metrics, or circuit breakers.

**Fix**: `dispatch()` now returns raw per-output `(AdapterId, String)` failure pairs instead of a stringified error; `format_and_dispatch` classifies the whole batch using `telemetry-adapter-contracts`'s already-existing `AdapterError::PartialDelivery`/`DeliveryFailed` variants (no new variant needed) — `PartialDelivery` when at least one record in the batch fully succeeded, `DeliveryFailed` when none did.

**Regression tests**: `batch_dispatch_failure_still_attempts_every_record` (asserts `DeliveryFailed`), `batch_with_some_dispatch_failures_reports_partial_delivery` (asserts `PartialDelivery`).

Landed as a second commit on the same PR #44 branch before merge; issue #46 is closed.

## Follow-Up Issues (Outside This Change's Scope)

Two issues were opened during review and remain open for future phases:

### Issue #45: Registry Arc vs Box Wrapper

**Question**: Should `output-adapter-contracts::Registry` accept `Arc<dyn Output>` instead of `Box<dyn Output>` to avoid the `ConsoleOutputAdapter` forwarding-shim wrapper?

**Status**: Open — unresolved, deferred to a future optimization phase.

### Issue #39: Registry HashMap vs Vec

**Question**: Should `Registry` be a `HashMap` instead of `Vec` once dispatch-order semantics are settled?

**Status**: Open — unresolved, deferred to a future performance/ordering phase.

## Deliberate Scope Decisions (Not Gaps)

### File Output Registration Deferred

`file-exporter` (from change 014) remains built and tested but NOT registered. This is explicit, not accidental:

- **Reason**: `kit_config` has no `OutputTarget::File` variant and no file-path field
- **Blocker**: Cross-repo schema change to sibling `kit-config` repo (outside this repo's control)
- **Documented**: Explicitly noted in proposal.md, design.md, and tasks.md
- **Not a code gap**: The capability exists; the configuration layer cannot express "use file output"

### LoggingConfig.output.targets Per-Stream Override Deferred

This change does NOT translate `LoggingConfig.output.targets` into per-severity stream-mapping overrides:

- **Reason**: FR-009 only requires console-presence and no-file-presence; per-target stream remapping would override `console-exporter`'s already-tested per-severity routing (warn/error/fatal → stderr)
- **Risk mitigation**: This design decision preserves backward compatibility with existing default behavior
- **Documented**: Rationale in tasks.md "Design note on 4.5"

## Verification Status

All Phase 5 verification checks passed:

- [x] 5.1 `cargo test --workspace` — all tests pass, no regressions
- [x] 5.2 `cargo clippy` and `cargo fmt` — all pass
- [x] 5.3 `console-exporter`'s `FlushStrategy` and pipeline `Buffer` compose correctly (test: `console_flush_strategy_and_pipeline_buffer_compose`)
- [x] 5.4 `file-exporter` remains unreferenced (verified via `rg`)
- [x] 5.5 No second dispatch registry or `LoggerProvider`-shaped type (verified via `rg`)
- [x] 5.6 Change 012's regressions confirmed still true (construction validation, no `with_config`, direct `kit-config` dependency)

## Task Completion

**Total tasks**: 28/28 complete

### Breakdown by Phase

- **Phase 1** (Enabled gate + level filter): 4 tasks ✓
- **Phase 2** (Sample + redact): 4 tasks ✓
- **Phase 3** (Buffer + flush/shutdown): 4 tasks ✓
- **Phase 4** (Format + dispatch + registration): 7 tasks ✓
- **Phase 5** (Verification): 6 tasks ✓
- **Post-merge gap review**: 2 regression tests added ✓
- **Follow-up issues**: 2 issues opened (not part of scope) ✓

All implementation tasks marked `[x]` in the archived `tasks.md`.

## Canonical Specs Finalized

| Spec | Location | Status |
|------|----------|--------|
| KITLogger Emission Pipeline | `openspec/specs/kitlogger-emission-pipeline/spec.md` | Created (new capability) |
| KITLogger Config Integration | `openspec/specs/kitlogger-config-integration/spec.md` | Updated (FR-003 modified) |

Both specs now reflect the final, merged-in behavior as of this change's completion.

## Dependencies and Related Changes

### Prior Dependencies (All Archived)

- Change 012: `kitlogger-config-integration` — archived
- Change 013: `kitlogger-sampling`, `kitlogger-redaction` — archived
- Change 014: `output-adapter-contracts`, `console-exporter`, `file-exporter`, `kitlogger-buffering` — archived

### Unmodified Inputs

- `kit_config::{LoggingConfig, LogLevel, OutputTarget}` (sibling repo, read-only)
- All consumed capabilities (sampling, redaction, buffering, formatting, dispatch) treated as frozen

## Archive Contents

```
openspec/changes/archive/2026-07-04-015-orchestration-fold/
├── proposal.md
├── design.md
├── tasks.md
├── specs/
│   ├── kitlogger-emission-pipeline/
│   │   └── spec.md
│   └── kitlogger-config-integration/
│       └── spec.md
└── archive-report.md (this file)
```

## Closed

The SDD cycle for change 015 is complete:

- ✓ Proposal finalized
- ✓ Specifications merged into canonical store
- ✓ Design documented
- ✓ Tasks planned and executed (28/28)
- ✓ Implementation merged (PR #44)
- ✓ Verification passed (Phase 5)
- ✓ Bugs found and fixed (2 regressions)
- ✓ Follow-ups tracked as open issues (not part of this change)

The change is archived and ready for the next phase of the Migration Plan.

---

**Archived**: 2026-07-04
**Change**: 015 Orchestration Fold
**Status**: Complete and merged to develop branch
