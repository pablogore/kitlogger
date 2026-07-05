# Tasks: Crate Removal

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 150–250 (mostly deletion; the `Transport` relocation is a near-verbatim move) |
| 400-line budget risk | None |
| Chained PRs recommended | No |
| Suggested split | Single PR |

## Phase 1: Verification Before Deleting

- [x] 1.1 Confirm `sampling.rs`, `redaction.rs`, `rotation.rs` each have their behavior fully covered by `kitlogger-sampling`, `kitlogger-redaction`, and `file-exporter` (rotation) respectively (changes 013/014) — re-confirm the "subsumed, not silently dropped" checks already done in those changes' own tasks. **Corrected before implementation**: this task originally also listed `formatter.rs`, `output.rs`, `buffering.rs`, `provider.rs`, `logger.rs` — re-verified against the current tree and confirmed change 016 already deleted all five; nothing to re-check for them here.
- [x] 1.2 Confirm (again) no crate outside `telemetry-transport-contract` references any of the three remaining modules above.
- [x] 1.3 Confirm `telemetry_types::BackpressureSignal` is what `error.rs`'s `Backpressure` variant already uses (post change-017) — the relocation in Phase 3 below must not reintroduce a local copy.

## Phase 2: Delete Absorbed Modules

- [x] 2.1 Delete `crates/telemetry-transport-contract/src/sampling.rs`.
- [x] 2.2 Delete `crates/telemetry-transport-contract/src/redaction.rs`.
- [x] 2.3 Delete `crates/telemetry-transport-contract/src/rotation.rs`.
- [x] 2.4 Remove all three corresponding `mod`/`pub use` declarations from `crates/telemetry-transport-contract/src/lib.rs`. (`formatter`/`output`/`buffering`/`provider`/`logger` declarations no longer exist — removed by change 016.)

## Phase 3: Relocate Transport

- [x] 3.1 Create `crates/telemetry-adapter-contracts/src/transport.rs`, moving `Transport`, `DeliveryMode`, `TransportResult` from the orphaned `transport.rs` verbatim (shape unchanged, per FR-001/002/003).
- [x] 3.2 Move `error.rs`'s `TransportError` (post change-017: `Timeout`, `Unavailable`, `Backpressure(telemetry_types::BackpressureSignal)`, `PayloadTooLarge`, `UnsupportedTransport`) into the new `transport.rs` module (or a sibling within `telemetry-adapter-contracts`, kept distinct from `AdapterError` per `design.md`). Satisfies FR-004.
- [x] 3.3 Add `pub mod transport;` and its re-exports to `crates/telemetry-adapter-contracts/src/lib.rs`.
- [x] 3.4 Port the orphaned crate's own `transport.rs`/`error.rs` unit tests into the new location, unchanged in intent. **Note**: this task's originally-named tests (`test_delivery_mode_serialization`, `test_backpressure_signal`) are stale — change 017 already deleted `test_backpressure_signal` (it referenced the local `BackpressureSignal` change 017 removed) and added `backpressure_variant_survives_the_canonical_type_repoint` (in `error.rs`) and `transport_trait_is_implementable_with_the_canonical_envelope_type` (in `transport.rs`) in its place. Port the three tests that currently exist: `test_delivery_mode_serialization`, `backpressure_variant_survives_the_canonical_type_repoint`, `transport_trait_is_implementable_with_the_canonical_envelope_type`.
- [x] 3.5 Confirm `Transport`/`TransportError` are reachable from `telemetry-adapter-contracts`'s crate root without requiring any `Adapter` implementer to depend on them. Satisfies FR-002.
- [x] 3.6 Run `cargo test -p telemetry-adapter-contracts` — confirm the ported tests pass and no existing test broke.

## Phase 4: Supersede `exporter-registry`

- [x] 4.1 Confirm the `## REMOVED Requirements` delta (already drafted in this change's `specs/exporter-registry/spec.md`) is ready to merge into the canonical spec at archive time.
- [x] 4.2 No code change required — `exporter-registry` was never implemented.

## Phase 5: Delete the Crate

- [x] 5.1 Confirmed before deletion: `crates/telemetry-transport-contract/src/` contained only `lib.rs`, `transport.rs`, and `error.rs` — every other module was either deleted (Phase 2, this change; five more by change 016) or relocated (Phase 3, this change). `transport.rs`/`error.rs` were not separately emptied in place — their content was moved to `telemetry-adapter-contracts/src/transport.rs` and the whole crate directory was then removed in one step (5.2), which is equivalent and avoids an intermediate half-emptied state.
- [x] 5.2 Deleted `crates/telemetry-transport-contract/` entirely (all remaining source files, `Cargo.toml`; `tests/` no longer existed — already fully removed by change 017).
- [x] 5.3 Remove `"crates/telemetry-transport-contract"` from the workspace root `Cargo.toml`'s `members`.

## Phase 6: Verification

- [x] 6.1 Run `cargo build --workspace` and `cargo test --workspace` — builds and passes with `telemetry-transport-contract` gone.
- [x] 6.2 Run `rg -rl "telemetry_transport_contract" crates` — one match: a historical doc comment in `crates/file-exporter/src/rotation.rs` ("ported from the orphaned `telemetry_transport_contract::rotation::RotationManager`") — provenance documentation, not a code reference or dependency; the workspace builds and tests green with the crate gone. No other matches.
- [x] 6.3 Run `rg -rl "telemetry-transport-contract"` across the workspace `Cargo.toml` files — zero matches.
- [x] 6.4 Confirm `kit-config` and `fastrand` remain workspace dependencies via `kitlogger` and `kitlogger-sampling` respectively — not orphaned by this crate's removal.
- [x] 6.5 Confirm no source file defines a second `Transport`, `DeliveryMode`, `TransportResult`, or `TransportError`.
- [x] 6.6 Confirm this closes every row of ADR-008's original Component Inventory — no component remains unresolved except the explicitly-tracked `kit_config::OutputTarget::File` gap (change 015) and ADR-009's correlation-id unification (Phase 9, not yet done).
