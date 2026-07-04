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

### Suggested Work Units

| Unit | Goal | Likely PR | Depends on |
|------|------|-----------|------------|
| 1 | Enabled gate + level filter | PR 1 | None |
| 2 | Sample + redact wiring | PR 2 | Unit 1 |
| 3 | Buffer wiring + flush/shutdown drainage | PR 3 | Unit 2 |
| 4 | Format selection + dispatch + default console registration | PR 4 | Unit 3 |

---

## Phase 1: Enabled Gate + Level Filter

- [ ] 1.1 **RED** — Write failing test `disabled_config_performs_no_processing` asserting zero downstream stage invocations (via spies/counters) when `LoggingConfig.enabled = false`. Satisfies FR-001.
- [ ] 1.2 **GREEN** — Implement the enabled short-circuit at the top of `log`/`log_record`. Run — 1.1 passes.
- [ ] 1.3 **RED** — Write failing table-driven test over `(LogLevel, Severity)` pairs, including `Fatal` at `LogLevel::Error`. Satisfies FR-002.
- [ ] 1.4 **GREEN** — Implement level filtering, with `Severity::Fatal` unconditionally passing. Run — 1.3 passes.

## Phase 2: Sample + Redact Wiring

- [ ] 2.1 Add `kitlogger-sampling` and `kitlogger-redaction` as dependencies of `kitlogger`.
- [ ] 2.2 **RED** — Write failing test `sampled_out_record_does_not_reach_later_stages` using a spy on redaction/buffering/formatting/dispatch. Satisfies FR-003.
- [ ] 2.3 **GREEN** — Wire the `Sampler` call after level filtering, short-circuiting on a negative decision. Run — 2.2 passes.
- [ ] 2.4 **RED** — Write failing test `buffered_record_reflects_redaction` asserting the buffered content has sensitive attributes already redacted. Satisfies FR-004.
- [ ] 2.5 **GREEN** — Wire the `Redactor` call after sampling, before buffering. Run — 2.4 passes.

## Phase 3: Buffer Wiring + Flush/Shutdown Drainage

- [ ] 3.1 **RED** — Write failing test `buffering_defers_format_and_dispatch` (batch size > 1, one record added, assert zero format/dispatch calls yet). Satisfies FR-005.
- [ ] 3.2 **GREEN** — Wire the redacted record into the internal `Buffer` module. Run — 3.1 passes.
- [ ] 3.3 **RED** — Write failing test `disabled_buffering_is_synchronous` (buffering disabled, assert immediate format+dispatch). Satisfies FR-005's disabled-passthrough scenario.
- [ ] 3.4 **GREEN** — Confirm the disabled-buffering path from `kitlogger-buffering`'s own FR-003 flows through correctly here. Run — 3.3 passes.
- [ ] 3.5 **RED** — Write failing test `shutdown_drains_buffered_records` (records added below the flush threshold, call shutdown, assert all were dispatched). Satisfies FR-006.
- [ ] 3.6 **GREEN** — Extend `KITLogger`'s existing `flush()`/`shutdown()` (`LifecycleAdapter`) to force-drain the buffer through formatting and dispatch. Run — 3.5 passes.

## Phase 4: Format Selection + Dispatch + Default Registration

- [ ] 4.1 Add `output-adapter-contracts`, `console-exporter`, and `file-exporter` (built, not registered) as dependencies of `kitlogger`, if not already present from change 014's own tasks.
- [ ] 4.2 **RED** — Write failing test `flushed_records_are_formatted_before_dispatch`. Satisfies FR-007 and FR-008.
- [ ] 4.3 **GREEN** — Wire buffer-flush output through the `kitlogger-format-selection` mapping and the selected `Formatter`, then into `output-adapter-contracts`'s dispatch. Run — 4.2 passes.
- [ ] 4.4 **RED** — Write failing test `console_registered_by_default_no_file`. Satisfies FR-009.
- [ ] 4.5 **GREEN** — At construction, translate `LoggingConfig.output.targets` into `console-exporter`'s stream-routing configuration and register it; explicitly do not register `file-exporter`. Run — 4.4 passes.
- [ ] 4.6 **RED** — Write failing test `exactly_one_dispatch_mechanism_exists` (structural/inspection-level, confirming no second registry or provider-shaped type exists). Satisfies FR-010.
- [ ] 4.7 **GREEN** — Confirm by construction; no `LoggerProvider`-equivalent type is introduced. Run — 4.6 passes.

## Phase 5: Verification

- [ ] 5.1 Run `cargo test --workspace` — all tests pass, no regressions.
- [ ] 5.2 Run `cargo clippy -p kitlogger -- -D warnings` and `cargo fmt --package kitlogger -- --check`.
- [ ] 5.3 Confirm `console-exporter`'s own `FlushStrategy` and the new pipeline `Buffer` both operate correctly under at least one combined configuration (e.g. `BatchFlush` + buffering enabled) without one masking the other.
- [ ] 5.4 Confirm `file-exporter` remains completely unreferenced from `KITLogger`'s construction or execution path.
- [ ] 5.5 Confirm no source file in `kitlogger` reintroduces a `LoggerProvider`-shaped type or a second dispatch registry.
- [ ] 5.6 Confirm `kitlogger-config-integration`'s FR-002 (construction-time validation) and FR-004/FR-005 (removed `with_config`, direct `kit-config` dependency) from change 012 are still true — this phase must not regress them.
