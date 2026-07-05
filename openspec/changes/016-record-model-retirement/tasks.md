# Tasks: Record Model Retirement

`design.md` now exists for this change — added after implementation began, when deleting `event.rs` alone was found not to compile. See `design.md` for the verification evidence behind this expanded scope. No new `spec.md`: still no capability is introduced or modified, only deletion.

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | < 150 (six file deletions + `lib.rs` edit; no new code) |
| 400-line budget risk | None |
| Chained PRs recommended | No |
| Suggested split | Single PR |

## Phase 1: Verification

- [x] 1.1 Run `rg -il "LogEvent" openspec/changes/archive/*013-redaction-sampling openspec/changes/archive/*014-output-consolidation openspec/changes/archive/*015-orchestration-fold crates` — confirm the only matches are the historical/contrastive mention in change 013's `proposal.md` and `telemetry-transport-contract`'s own source.
- [x] 1.2 Confirm no crate other than `telemetry-transport-contract` depends on `LogEvent` (already re-verified in `design.md` condition 3 — this task re-runs the same check after deletion, not before).
- [x] 1.3 Confirm `batch.rs`/`payload.rs`/`transport.rs`/`error.rs`/`redaction.rs`/`rotation.rs`/`sampling.rs` have zero references to `LogEvent` or to `logger`/`output`/`buffering`/`formatter`/`provider` (already verified in `design.md` condition 2 — re-run to confirm nothing changed since).

## Phase 2: Deletion

- [x] 2.1 Delete `crates/telemetry-transport-contract/src/event.rs`.
- [x] 2.2 Delete `crates/telemetry-transport-contract/src/logger.rs`.
- [x] 2.3 Delete `crates/telemetry-transport-contract/src/provider.rs`.
- [x] 2.4 Delete `crates/telemetry-transport-contract/src/buffering.rs`.
- [x] 2.5 Delete `crates/telemetry-transport-contract/src/formatter.rs`.
- [x] 2.6 Delete `crates/telemetry-transport-contract/src/output.rs`.
- [x] 2.7 Remove all six modules' `mod`/`pub use` declarations from `crates/telemetry-transport-contract/src/lib.rs`.
- [x] 2.8 Run `cargo check -p telemetry-transport-contract` — confirm it compiles with no compatibility stub, no placeholder type, and no temporary re-export standing in for any of the six deleted modules. Its remaining modules (`batch`, `payload`, `transport`, `error`, `redaction`, `rotation`, `sampling`) were already verified in Phase 1 to have no reference to the deleted cluster, so this should pass without further changes — if it doesn't, that reveals another undiscovered coupling and implementation must stop again, not route around it.

## Phase 3: Verification

- [x] 3.1 Run `cargo test --workspace` — no regressions.
- [x] 3.2 Confirm `LogEvent` is absent from the workspace entirely (`rg -l "LogEvent"` returns no matches under `crates/`).
- [x] 3.3 Confirm `logger`/`output`/`buffering`/`formatter`/`provider` are absent from `telemetry-transport-contract` entirely (`rg -l "mod (logger|output|buffering|formatter|provider)"` under `crates/telemetry-transport-contract/src/lib.rs` returns no matches).
- [x] 3.4 Run `cargo clippy -p telemetry-transport-contract -- -D warnings` and `cargo fmt --package telemetry-transport-contract -- --check`. Clippy clean. `cargo fmt --check` shows pre-existing, unrelated drift in `redaction.rs`/`sampling.rs` (untouched by this change — confirmed via `git diff --stat` showing zero changes to either file); `lib.rs`, the only file this change modifies, is clean.
