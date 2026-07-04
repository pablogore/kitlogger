# Tasks: Record Model Retirement

No `design.md` for this change — it is a verified deletion with no new architecture decision to make (see `proposal.md`'s "Verification Performed" and "Field Parity" sections). No new `spec.md` either — no capability is introduced or modified.

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | < 50 |
| 400-line budget risk | None |
| Chained PRs recommended | No |
| Suggested split | Single PR |

## Phase 1: Verification

- [ ] 1.1 Run `rg -il "LogEvent" openspec/changes/013-redaction-sampling openspec/changes/014-output-consolidation openspec/changes/015-orchestration-fold crates` — confirm the only matches are the historical/contrastive mention in change 013's `proposal.md` and `telemetry-transport-contract`'s own source.
- [ ] 1.2 Confirm no crate other than `telemetry-transport-contract` depends on `LogEvent`.

## Phase 2: Deletion

- [ ] 2.1 Delete `crates/telemetry-transport-contract/src/event.rs`.
- [ ] 2.2 Remove the `event` module declaration and its `pub use` re-export from `crates/telemetry-transport-contract/src/lib.rs`.
- [ ] 2.3 Run `cargo check -p telemetry-transport-contract` — confirm it still compiles (its own remaining modules — `batch`, `payload`, `transport`, `error`, `formatter`, `output`, `sampling`, `redaction`, `rotation`, `buffering`, `provider`, `logger` — do not reference `event::LogEvent`; if any do, that reference is itself dead code scheduled for removal in Phases 5/7, not a reason to keep `event.rs`).

## Phase 3: Verification

- [ ] 3.1 Run `cargo test --workspace` — no regressions.
- [ ] 3.2 Confirm `LogEvent` is absent from the workspace entirely (`rg -l "LogEvent"` returns no matches under `crates/`).
