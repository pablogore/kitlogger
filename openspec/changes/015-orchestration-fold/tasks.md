# Tasks: Orchestration Fold

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 400–550 (single crate, `kitlogger`, but touches its core execution path) |
| 400-line budget risk | High |
| Chained PRs recommended | Yes — stacked, since each stage depends on the previous being wired |
| Suggested split | PR 1: enabled gate + level filter; PR 2: sample + redact; PR 3: buffer + flush/shutdown drainage; PR 4: format selection + dispatch + default registration |
| Delivery strategy | ask-on-risk (already resolved: split, stacked) |
| Chain strategy | stacked-to-main |

Decision needed before apply: No (already resolved)
Chained PRs recommended: Yes
400-line budget risk: High combined; each individual PR is within budget

**Post-apply note**: the recommended 4-way stacked split assumed each phase's
work would land in isolated files/crates, as change 014's four units did.
In practice, all 5 phases were implemented as one pass over `crates/kitlogger`
(the same `build()`, `log_record`, `flush`/`shutdown` functions accumulate
fields and branches across phases, unlike 014's separate crates). Splitting
this after the fact into 4 clean, independently-reviewable diffs would
require manually re-partitioning interleaved hunks within the same
functions — real risk of introducing a bug while re-slicing working,
verified code, for a benefit (four smaller diffs) that no longer matches how
the change was actually built. Shipped as a single PR instead; see PR
description for the full diff summary.

### Suggested Work Units

| Unit | Goal | Likely PR | Depends on |
|------|------|-----------|------------|
| 1 | Enabled gate + level filter | PR 1 | None |
| 2 | Sample + redact wiring | PR 2 | Unit 1 |
| 3 | Buffer wiring + flush/shutdown drainage | PR 3 | Unit 2 |
| 4 | Format selection + dispatch + default console registration | PR 4 | Unit 3 |

---

## Phase 1: Enabled Gate + Level Filter

- [x] 1.1 **RED** — Write failing test `disabled_config_performs_no_processing` asserting zero downstream stage invocations (via spies/counters) when `LoggingConfig.enabled = false`. Satisfies FR-001.
- [x] 1.2 **GREEN** — Implement the enabled short-circuit at the top of `log`/`log_record`. Run — 1.1 passes.
- [x] 1.3 **RED** — Write failing table-driven test over `(LogLevel, Severity)` pairs, including `Fatal` at `LogLevel::Error`. Satisfies FR-002.
- [x] 1.4 **GREEN** — Implement level filtering, with `Severity::Fatal` unconditionally passing. Run — 1.3 passes.

## Phase 2: Sample + Redact Wiring

- [x] 2.1 Add `kitlogger-sampling` and `kitlogger-redaction` as dependencies of `kitlogger`.
- [x] 2.2 **RED** — Write failing test `sampled_out_record_does_not_reach_later_stages` using a spy on redaction/buffering/formatting/dispatch. Satisfies FR-003.
- [x] 2.3 **GREEN** — Wire the `Sampler` call after level filtering, short-circuiting on a negative decision. Run — 2.2 passes.
- [x] 2.4 **RED** — Write failing test `dispatched_record_reflects_redaction` (buffering not yet wired at this point in the sequence — buffer-integrated wording deferred to Phase 3) asserting the dispatched content has sensitive attributes already redacted. Satisfies FR-004.
- [x] 2.5 **GREEN** — Wire the `Redactor` call after sampling, before formatting/dispatch. Run — 2.4 passes.

## Phase 3: Buffer Wiring + Flush/Shutdown Drainage

- [x] 3.1 **RED** — Write failing test `buffering_defers_format_and_dispatch` (batch size > 1, one record added, assert zero format/dispatch calls yet). Satisfies FR-005.
- [x] 3.2 **GREEN** — Wire the redacted record into the internal `Buffer` module. Run — 3.1 passes.
- [x] 3.3 **RED** — Write failing test `disabled_buffering_is_synchronous` (buffering disabled, assert immediate format+dispatch). Satisfies FR-005's disabled-passthrough scenario.
- [x] 3.4 **GREEN** — Confirm the disabled-buffering path from `kitlogger-buffering`'s own FR-003 flows through correctly here. Run — 3.3 passes.
- [x] 3.5 **RED** — Write failing test `shutdown_drains_buffered_records` (records added below the flush threshold, call shutdown, assert all were dispatched). Satisfies FR-006. (Also added `Buffer::drain()`'s own RED unit tests in `buffer.rs`, required scaffolding for this task, not previously present.)
- [x] 3.6 **GREEN** — Extend `KITLogger`'s existing `flush()`/`shutdown()` (`LifecycleAdapter`, plus new inherent `flush()`/existing inherent `shutdown()`) to force-drain the buffer through formatting and dispatch via `Buffer::drain()`. Run — 3.5 passes.

## Phase 4: Format Selection + Dispatch + Default Registration

- [x] 4.1 Add `output-adapter-contracts`, `console-exporter`, and `file-exporter` (built, not registered) as dependencies of `kitlogger`, if not already present from change 014's own tasks.
- [x] 4.2 **RED** — Write failing test `flushed_records_use_the_configured_format` (renamed from `flushed_records_are_formatted_before_dispatch` for clarity: format-before-dispatch is structurally guaranteed by this code's control flow — the meaningful new behavior under test is that the *configured* format, not a hardcoded default, is what gets used). Satisfies FR-007 and FR-008.
- [x] 4.3 **GREEN** — Wire buffer-flush output through the `kitlogger-format-selection` mapping and the selected `Formatter`, then into `output-adapter-contracts`'s dispatch (`Registry`). Run — 4.2 passes.
- [x] 4.4 **RED** — Write failing test `console_registered_by_default_no_file` (required adding a new `KITLogger::registered_output_ids()` inspection accessor, since `Registry` itself has no enumeration API and is not modified per proposal.md's "Out of Scope"). Satisfies FR-009.
- [x] 4.5 **GREEN** — At construction, register `console-exporter` (via a local `ConsoleOutputAdapter` Output-Port wrapper) into the registry under `"console"`; explicitly do not register `file-exporter`. (Console's own existing per-severity `LevelStreamMapping` is left untouched by `LoggingConfig.output.targets` in this phase — see rationale below.) Run — 4.4 passes.
- [x] 4.6 **RED** — Write failing test `dispatch_delivers_exactly_once` (renamed from `exactly_one_dispatch_mechanism_exists`: a true structural/reflection-level check isn't expressible in a Rust unit test, so this asserts the behavioral proxy — a dispatched record is delivered exactly once, evidence against a second, parallel dispatch mechanism). Satisfies FR-010.
- [x] 4.7 **GREEN** — Confirmed by construction: `KITLogger` holds exactly one `registry: Registry` field; no `LoggerProvider`-equivalent type is introduced anywhere in `crates/kitlogger/src/`. Run — 4.6 passes.

**Design note on 4.5**: this phase deliberately does NOT translate `LoggingConfig.output.targets` into a `console-exporter` `LevelStreamMapping` override. `OutputConfig::default().targets == [OutputTarget::Stdout]` is kit_config's own default — if that were read as "force every severity to stdout," it would silently override `console-exporter`'s existing, already-tested per-severity routing (warn/error/fatal → stderr) for every default-constructed `KITLogger`, breaking `pipeline_integration.rs`'s `log_record_error_severity_goes_to_stderr` and `log_record_logfmt_format_produces_kv_pairs` (which route via that same default mapping). FR-009's own scenarios only require console-presence / no-file-presence, not per-target stream remapping, so registering `console-exporter` unconditionally (once `kit_config`'s own validation already guarantees `output.targets` is non-empty whenever `enabled`) fully satisfies FR-009 without that risk. This is the same category of implementation-detail latitude design.md's "Open Questions" grants elsewhere (e.g. the buffer's flush-driving mechanism).

**Post-merge fix (final gap review)**: two real gaps were found and fixed before merge:
1. `format_and_dispatch` used to abort its per-record loop with `?` on the first dispatch failure — since batches are already removed from `Buffer` via `mem::take` before this point, any record after the first failure was silently lost (neither dispatched nor re-buffered), contradicting `flush`/`shutdown`'s own documented guarantee. Fixed to attempt every record in the batch regardless of earlier failures, aggregating failures into one error. Regression test: `batch_dispatch_failure_still_attempts_every_record`.
2. `Buffer::try_flush()` (the time-based half of FR-005's "size/time flush conditions") was never called from `log_record` — only from its own unit tests — so `BufferingConfig.flush_interval_ms` had no effect in the real pipeline. Fixed by checking `try_flush()` opportunistically whenever `add()` doesn't already trigger a size-based flush. Regression test: `log_record_checks_time_based_flush_when_size_has_not_been_reached`.

## Phase 5: Verification

- [x] 5.1 Run `cargo test --workspace` — all tests pass, no regressions.
- [x] 5.2 Run `cargo clippy -p kitlogger -- -D warnings` and `cargo fmt --package kitlogger -- --check`.
- [x] 5.3 Confirm `console-exporter`'s own `FlushStrategy` and the new pipeline `Buffer` both operate correctly under at least one combined configuration (e.g. `BatchFlush` + buffering enabled) without one masking the other. Added `console_flush_strategy_and_pipeline_buffer_compose` — passed without further production-code changes.
- [x] 5.4 Confirm `file-exporter` remains completely unreferenced from `KITLogger`'s construction or execution path. Verified via `rg` — only the module-level comment documenting the dependency edge mentions it; no `use`/call sites.
- [x] 5.5 Confirm no source file in `kitlogger` reintroduces a `LoggerProvider`-shaped type or a second dispatch registry. Verified via `rg` — `KITLogger` has exactly one `registry: Registry` field; the only `LoggerProvider` mention is the doc comment stating it is retired.
- [x] 5.6 Confirm `kitlogger-config-integration`'s FR-002 (construction-time validation) and FR-004/FR-005 (removed `with_config`, direct `kit-config` dependency) from change 012 are still true — this phase must not regress them. Verified: `from_logging_config`/`from_logging_config_with_exporter` still call `config.validate()?` (FR-002); no `with_config` method exists anywhere in `kitlogger` (FR-004); `kit-config` remains a direct `Cargo.toml` dependency (FR-005). `logging_config_test.rs` and `with_config_test.rs` still pass.
