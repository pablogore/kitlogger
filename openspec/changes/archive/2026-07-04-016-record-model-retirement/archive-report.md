# Archive Report: Record Model Retirement (Change 016)

## Status

Shipped. All 15 tasks in `tasks.md` complete (`[x]`). Merged into `develop` via PR #48.

## What shipped

Deleted `telemetry_transport_contract::event::LogEvent` and its five satellite modules — `logger.rs`, `output.rs`, `buffering.rs`, `formatter.rs`, `provider.rs` — from the orphaned `telemetry-transport-contract` crate. No new capability, no capability modified: this is pure closure/deletion. No canonical spec was created or updated — there was never a `specs/` delta for this change, since nothing here is a behavioral capability.

## Scope divergence, found and corrected before implementation

The original proposal scoped this change to `event.rs` alone, on the premise (from an earlier architecture review) that no other module in the crate referenced `LogEvent`. During implementation, verification found that premise false: `logger.rs`, `output.rs`, `buffering.rs`, and `formatter.rs` reference `LogEvent` heavily as their primary operand, and `provider.rs` is `logger.rs`'s own direct dependency. Deleting `event.rs` alone would not have compiled.

Per this repo's Architecture Conflict Procedure (`AGENTS.md`), implementation stopped before any code was touched. `design.md` was added (there was none originally — this change was planned as "a verified deletion with no new architecture decision to make") to record the verification evidence for expanding the scope to all six files:

1. Each of the five modules' core methods take/return `LogEvent` as their primary type, not incidentally.
2. Zero cross-references from the crate's other modules (`batch`, `payload`, `transport`, `error`, `redaction`, `rotation`, `sampling`) — confirmed via grep, no matches.
3. Zero dependents anywhere else in the workspace — no crate lists `telemetry-transport-contract` as a `Cargo.toml` dependency at all.
4. All five modules become unreachable dead code the moment `LogEvent` is gone, since their public surface exists only to operate on it or to be called by another module in the same cluster.

`proposal.md` and `tasks.md` were updated to reflect the corrected scope before implementation began — no code was written against the stale premise.

## Implementation

All six files deleted together, in one phase — no compatibility stub, placeholder type, or temporary re-export was introduced standing in for any of them, per the corrected proposal's explicit constraint. `crates/telemetry-transport-contract/src/lib.rs`'s six `mod`/`pub use` declarations for the deleted modules were removed in the same change.

## Verification

- `cargo check --workspace` — clean immediately after deletion, no compatibility stub needed.
- `cargo test -p telemetry-transport-contract` — 26/26 passing, confirming no accidental reference survived (doc tests, integration tests, `lib.rs` re-exports).
- `cargo test --workspace` — all green, zero regressions elsewhere.
- `cargo clippy -p telemetry-transport-contract -- -D warnings` — clean.
- `cargo fmt --package telemetry-transport-contract -- --check` — clean for `lib.rs`, the only file this change modified; pre-existing, unrelated formatting drift in `redaction.rs`/`sampling.rs` (confirmed untouched by this change via `git diff --stat`) was left alone, consistent with how pre-existing drift has been handled elsewhere in this migration.
- Confirmed `LogEvent` and all five module names are absent from the entire workspace (`rg` — no matches).

## Out of scope, unaffected

`batch.rs`, `payload.rs`, `transport.rs`, `error.rs`, `redaction.rs`, `rotation.rs`, `sampling.rs` — verified to have zero coupling to the deleted cluster. These remain `telemetry-transport-contract`'s only remaining modules, for Phase 7 (`batch.rs`/`payload.rs`/the duplicate `BackpressureSignal`) and Phase 8 (removing the crate entirely once every module, including these, is empty) to handle.
