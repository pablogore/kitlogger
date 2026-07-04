# Tasks: Output Subsystem Consolidation

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 500–700 (across 4 work units) |
| 400-line budget risk | High — split required |
| Chained PRs recommended | Yes — one per work unit |
| Suggested split | PR 1: `output-adapter-contracts`; PR 2: `file-exporter`; PR 3: `console-exporter` Port implementation; PR 4: `kitlogger` internal modules (`Buffer`, format-selection) |
| Delivery strategy | ask-on-risk (already resolved: split) |
| Chain strategy | stacked-to-main for PR 1 → PR 2/PR 3 (both depend on PR 1's Port existing); PR 4 independent, can land in parallel |

Decision needed before apply: No (already resolved)
Chained PRs recommended: Yes
400-line budget risk: High per the combined estimate; each individual PR is within budget

### Suggested Work Units

| Unit | Goal | Likely PR | Depends on |
|------|------|-----------|------------|
| 1 | `output-adapter-contracts` (Port + registry) | PR 1 | None |
| 2 | `file-exporter` (implements the Port, owns rotation) | PR 2 | Unit 1 |
| 3 | `console-exporter` gains the Port implementation | PR 3 | Unit 1 |
| 4 | `kitlogger` internal modules: `Buffer`, format-selection | PR 4 | None — independent of Units 1–3 |

---

## Phase 1: `output-adapter-contracts`

- [ ] 1.1 Add `"crates/output-adapter-contracts"` to workspace `members`.
- [ ] 1.2 Create `crates/output-adapter-contracts/Cargo.toml` — deps: `kitlogger-log-domain` (path, for `Severity` only).
- [ ] 1.3 **RED** — Write failing test `conforming_output_receives_dispatch` asserting a fake output implementing the Port receives a formatted record + severity when dispatched to directly. Satisfies FR-001.
- [ ] 1.4 **GREEN** — Define the Output Port. Run `cargo test -p output-adapter-contracts` — 1.3 passes.
- [ ] 1.5 **RED** — Write failing test `duplicate_registration_rejected`. Satisfies FR-002.
- [ ] 1.6 **GREEN** — Implement the registry's `register` with duplicate-identifier rejection. Run — 1.5 passes.
- [ ] 1.7 **RED** — Write failing test `dispatch_reaches_all_registered_outputs` (3 fake outputs). Satisfies FR-003.
- [ ] 1.8 **GREEN** — Implement dispatch-to-all. Run — 1.7 passes.
- [ ] 1.9 **RED** — Write failing tests `partial_failure_does_not_block_others` and `total_failure_is_distinguishable`. Satisfies FR-004.
- [ ] 1.10 **GREEN** — Implement failure aggregation distinguishing partial vs. total failure. Run — 1.9 passes.
- [ ] 1.11 Confirm (by inspecting `Cargo.toml`) that `output-adapter-contracts` has no dependency on `telemetry-adapter-contracts` or `telemetry-types`. Satisfies FR-005.
- [ ] 1.12 Run `cargo clippy -p output-adapter-contracts -- -D warnings` and `cargo fmt --package output-adapter-contracts -- --check`.

## Phase 2: `file-exporter`

- [ ] 2.1 Add `"crates/file-exporter"` to workspace `members`.
- [ ] 2.2 Create `crates/file-exporter/Cargo.toml` — deps: `kit-config` (path, for `RotationConfig`), `kitlogger-log-domain` (path, for `Severity`), `output-adapter-contracts` (path, to implement the Port).
- [ ] 2.3 **RED** — Write failing test `dispatched_record_appended_to_file`. Satisfies FR-001.
- [ ] 2.4 **GREEN** — Implement the file writer. Run — 2.3 passes.
- [ ] 2.5 **RED** — Write failing test `rotation_triggers_at_size_boundary`, porting the orphaned `RotationManager`'s existing validated test cases where applicable. Satisfies FR-002.
- [ ] 2.6 **GREEN** — Port `RotationManager`'s numbered-backup-chain algorithm (the surviving one — the divergent inline single-backup version in the orphaned `FileOutput::rotate()` is not ported). Run — 2.5 passes.
- [ ] 2.7 **RED** — Write failing test `backups_beyond_max_are_discarded`. Satisfies FR-003.
- [ ] 2.8 **GREEN** — Implement backup-count bounding. Run — 2.7 passes.
- [ ] 2.9 **RED** — Write failing test `disabled_rotation_grows_file_unbounded`. Satisfies FR-004.
- [ ] 2.10 **GREEN** — Implement the enabled-check short-circuit. Run — 2.9 passes.
- [ ] 2.11 **RED** — Write failing test `file_exporter_conforms_to_output_port`, registering it into an `output-adapter-contracts` registry alongside a fake output. Satisfies FR-005.
- [ ] 2.12 **GREEN** — Implement the Port for `file-exporter`. Run — 2.11 passes.
- [ ] 2.13 Confirm the orphaned crate's rotation test coverage (`telemetry-transport-contract`'s existing rotation tests) is subsumed by 2.5–2.10, not silently dropped.
- [ ] 2.14 Run `cargo clippy -p file-exporter -- -D warnings` and `cargo fmt --package file-exporter -- --check`.

## Phase 3: `console-exporter` Output Port Implementation

- [ ] 3.1 Add `output-adapter-contracts` (path) as a dependency of `console-exporter`.
- [ ] 3.2 **RED** — Write failing test `console_exporter_conforms_to_output_port`, registering it into an `output-adapter-contracts` registry alongside a fake output.
- [ ] 3.3 **GREEN** — Implement the Port for `console-exporter`, reconciling it with the existing `ConsoleExporter` trait so exactly one write path underlies both (per the risk noted in `proposal.md` — no second, divergent console-write path).
- [ ] 3.4 Run `console-exporter`'s full existing test suite — confirm zero behavior change.
- [ ] 3.5 Run `cargo clippy -p console-exporter -- -D warnings` and `cargo fmt --package console-exporter -- --check`.

## Phase 4: `kitlogger` Internal Modules — `Buffer` and Format Selection

- [ ] 4.1 **RED** — Write failing test `batch_size_triggers_flush` for the new internal `Buffer` module. Satisfies FR-001 (`kitlogger-buffering`).
- [ ] 4.2 **GREEN** — Implement size-based flush. Run — 4.1 passes.
- [ ] 4.3 **RED** — Write failing test `flush_interval_triggers_flush_before_batch_size`, using `kitlogger_log_domain::FakeClock` (or equivalent) — no real sleeps. Satisfies FR-002 and FR-006.
- [ ] 4.4 **GREEN** — Implement time-based flush sourcing time exclusively through the injected clock. Run — 4.3 passes.
- [ ] 4.5 **RED** — Write failing test `disabled_buffering_passes_through_immediately`. Satisfies FR-003.
- [ ] 4.6 **GREEN** — Implement the enabled-check short-circuit. Run — 4.5 passes.
- [ ] 4.7 **RED** — Write failing test `flushed_batch_preserves_insertion_order`. Satisfies FR-004.
- [ ] 4.8 **GREEN** — Ensure ordering is preserved by construction. Run — 4.7 passes.
- [ ] 4.9 Confirm buffered content type is pre-format (`LogRecord`, not `String`) by inspection. Satisfies FR-005.
- [ ] 4.10 Grep the new `Buffer` module for direct `Instant::now()`/`SystemTime::now()` calls outside the clock abstraction — zero matches. Satisfies FR-006.
- [ ] 4.11 **RED** — Write failing test `every_log_format_variant_maps`, exhaustive over `kit_config::LogFormat`'s four variants, asserting the mapping table in `design.md` (`Json→Json`, `Text→Text`, `Pretty→HumanReadable`, `Compact→Logfmt`). Satisfies `kitlogger-format-selection` FR-001.
- [ ] 4.12 **GREEN** — Implement the mapping function as a new internal `kitlogger` module. Run — 4.11 passes.
- [ ] 4.13 **RED** — Write failing test `mapping_is_deterministic` (repeated calls, same result). Satisfies FR-002.
- [ ] 4.14 **GREEN** — Confirm by construction (pure function, no internal state). Run — 4.13 passes.
- [ ] 4.15 Diff `crates/kitlogger-formatter/Cargo.toml` before/after this change — confirm zero change. Satisfies FR-003.
- [ ] 4.16 Confirm neither new `kitlogger` module (`Buffer`, format-selection) is called from `KITLogger::log`/`log_record`/any constructor yet — both exist standalone, exercised only by their own unit tests.
- [ ] 4.17 Run `cargo clippy -p kitlogger -- -D warnings` and `cargo fmt --package kitlogger -- --check`.

## Phase 5: Verification

- [ ] 5.1 Run `cargo test --workspace` — all tests pass; no regressions elsewhere.
- [ ] 5.2 Confirm `output-adapter-contracts` has zero dependents outside `console-exporter`, `file-exporter`, and their own tests (not yet consumed by `kitlogger`).
- [ ] 5.3 Confirm `telemetry-adapter-contracts` gained no new dependents from this change.
- [ ] 5.4 Confirm no two crates define a competing Output Port, rotation algorithm, buffering mechanism, or `LogFormat`-mapping function (ADR-010 compliance check for this change's scope).
