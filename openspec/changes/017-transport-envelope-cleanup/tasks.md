# Tasks: Transport/Envelope Cleanup

`design.md` now exists for this change — added after implementation was attempted and the original "exact duplicate, mechanical repoint" premise was found false. See `design.md` for the Canonical Concept vs. Concrete Representation distinction that resolves this without changing the implementation scope below. No new `spec.md` — no capability introduced or modified.

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | < 100 |
| 400-line budget risk | None |
| Chained PRs recommended | No |
| Suggested split | Single PR |

## Phase 1: Verification Before Deleting

- [x] 1.1 Confirm `telemetry_types::{TelemetryBatch, PayloadEnvelope, TransportMetadata, BackpressureSignal}` represent the same concept as `telemetry-transport-contract`'s local versions per ADR-010 (they do NOT field-for-field match — see `design.md` — this task confirms conceptual ownership, not shape equivalence, and confirms `telemetry_types` has no `TelemetryBatchError` counterpart to repoint to at all).
- [x] 1.2 Confirm no crate other than `telemetry-transport-contract` depends on `context-propagation` (already verified during the original audit — `context-propagation` is a leaf crate with `telemetry-transport-contract` as its only in-workspace consumer; re-confirm before removing the dependency).

## Phase 2: Delete and Repoint

- [x] 2.1 Delete `crates/telemetry-transport-contract/src/batch.rs`.
- [x] 2.2 Delete `crates/telemetry-transport-contract/src/payload.rs`.
- [x] 2.3 In `transport.rs`: delete the local `BackpressureSignal` struct; change `Transport::send()`'s parameter type to `telemetry_types::PayloadEnvelope`; update the `use` statement accordingly.
- [x] 2.4 In `error.rs`: change `TransportError::Backpressure`'s field type to `telemetry_types::BackpressureSignal`; update the `use` statement accordingly.
- [x] 2.5 In `lib.rs`: remove the `pub mod payload;`/`mod batch;` declarations and their `pub use batch::*;`/`pub use payload::*;` re-exports; remove the `pub use context_propagation::models::{...}` and `pub use context_propagation::propagation_metadata::PropagationMetadata;` re-exports.
- [x] 2.6 In `Cargo.toml`: remove the `context-propagation` dependency; add `telemetry-types = { path = "../telemetry-types" }`.
- [x] 2.7 Delete `tests/batch_test.rs` and `tests/payload_test.rs` (or whichever existing test files exercise the deleted modules) — not migrated, per `proposal.md`'s Risks. Also deleted `tests/integration_tests.rs` and `tests/transport_test.rs`: both constructed literals in the local (now-deleted) shape (`TelemetryBatch::new(Resource::new(), ...)`, `BackpressureSignal { retry_after: None }` with no `attributes` field, etc.) — the "or whichever" in this task's own wording already anticipated more than the two named files.
- [x] 2.8 Run `cargo check -p telemetry-transport-contract` — confirm it compiles with only `transport`, `error`, `redaction`, `rotation`, `sampling` remaining (`formatter`, `output`, `buffering`, `provider`, `logger` no longer exist, deleted by change 016).

## Phase 3: Verification

- [x] 3.1 Run `cargo test --workspace` — no regressions.
- [x] 3.2 Confirm `context-propagation` has zero workspace dependents (`rg -l "context-propagation" crates/*/Cargo.toml` returns no matches) — expected and documented, not a silent orphan (see `proposal.md`'s Risks; ADR-009/Phase 9 gives it a role again).
- [x] 3.3 Confirm no crate defines a second `TelemetryBatch`, `PayloadEnvelope`, `TransportMetadata`, or `BackpressureSignal` (`rg -l` for each type name across `crates/`, expecting exactly one definition site — `telemetry_types` — for each). Confirmed: one definition site each, in `crates/telemetry-types/src/lib.rs`.

**Post-implementation note**: `TransportError` lost its `PartialEq`/`Eq` derive (it kept `Debug, Clone, Serialize, Deserialize`). `telemetry_types::BackpressureSignal` (now `TransportError::Backpressure`'s field type) doesn't implement `PartialEq`/`Eq`, unlike the local version this crate used to define — a necessary consequence of the repoint, not caught by the original task list. No other crate compares `TransportError` values, so nothing else was affected.

**Post-review fix**: code review found that deleting `tests/{transport_test,integration_tests}.rs` (task 2.7) discarded real coverage of `Transport`/`TransportError` — types that remain in this crate, not deleted by this change — not just coverage of the deleted `TelemetryBatch`/`PayloadEnvelope`/`TransportMetadata`. Restored the minimum needed to cover the surface this change actually touched, without recreating the ~1000 lines of deleted tests (those covered the now-gone types, correctly not migrated):
- `error.rs`: `backpressure_variant_survives_the_canonical_type_repoint` — `TransportError::Backpressure` still constructs, serde-round-trips, and `Display`s correctly with `telemetry_types::BackpressureSignal` in place of the former local type.
- `transport.rs`: `transport_trait_is_implementable_with_the_canonical_envelope_type` — a mock `Transport` impl still compiles and `send()` still works with `telemetry_types::PayloadEnvelope` in place of the former local type.

Also added a Risks row to `proposal.md` explicitly naming the type deletions as a public API break, mitigated by the crate's zero workspace dependents and lack of external publication — matching change 012's precedent for documenting this (a Risks table row, not a new "Breaking Changes" section).
