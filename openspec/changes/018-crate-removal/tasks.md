# Tasks: Crate Removal

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 150–250 (mostly deletion; the `Transport` relocation is a near-verbatim move) |
| 400-line budget risk | None |
| Chained PRs recommended | No |
| Suggested split | Single PR |

## Phase 1: Verification Before Deleting

- [ ] 1.1 Confirm `formatter.rs`, `output.rs`, `sampling.rs`, `redaction.rs`, `rotation.rs`, `buffering.rs`, `provider.rs`, `logger.rs` each have their behavior fully covered by `kitlogger-formatter`, `console-exporter`/`file-exporter`, `kitlogger-sampling`, `kitlogger-redaction`, (rotation is inside `file-exporter`), `kitlogger`'s `Buffer` module, and `kitlogger`'s emission pipeline (changes 013/014/015) respectively — re-confirm the "subsumed, not silently dropped" checks already done in those changes' own tasks.
- [ ] 1.2 Confirm (again) no crate outside `telemetry-transport-contract` references any of the eight modules above.
- [ ] 1.3 Confirm `telemetry_types::BackpressureSignal` is what `error.rs`'s `Backpressure` variant already uses (post change-017) — the relocation in Phase 3 below must not reintroduce a local copy.

## Phase 2: Delete Absorbed Modules

- [ ] 2.1 Delete `crates/telemetry-transport-contract/src/formatter.rs`.
- [ ] 2.2 Delete `crates/telemetry-transport-contract/src/output.rs`.
- [ ] 2.3 Delete `crates/telemetry-transport-contract/src/sampling.rs`.
- [ ] 2.4 Delete `crates/telemetry-transport-contract/src/redaction.rs`.
- [ ] 2.5 Delete `crates/telemetry-transport-contract/src/rotation.rs`.
- [ ] 2.6 Delete `crates/telemetry-transport-contract/src/buffering.rs`.
- [ ] 2.7 Delete `crates/telemetry-transport-contract/src/provider.rs`.
- [ ] 2.8 Delete `crates/telemetry-transport-contract/src/logger.rs`.
- [ ] 2.9 Remove all eight corresponding `mod`/`pub use` declarations from `crates/telemetry-transport-contract/src/lib.rs`.

## Phase 3: Relocate Transport

- [ ] 3.1 Create `crates/telemetry-adapter-contracts/src/transport.rs`, moving `Transport`, `DeliveryMode`, `TransportResult` from the orphaned `transport.rs` verbatim (shape unchanged, per FR-001/002/003).
- [ ] 3.2 Move `error.rs`'s `TransportError` (post change-017: `Timeout`, `Unavailable`, `Backpressure(telemetry_types::BackpressureSignal)`, `PayloadTooLarge`, `UnsupportedTransport`) into the new `transport.rs` module (or a sibling within `telemetry-adapter-contracts`, kept distinct from `AdapterError` per `design.md`). Satisfies FR-004.
- [ ] 3.3 Add `pub mod transport;` and its re-exports to `crates/telemetry-adapter-contracts/src/lib.rs`.
- [ ] 3.4 Port the orphaned crate's own `transport.rs`/`error.rs` unit tests (`test_delivery_mode_serialization`, `test_backpressure_signal`) into the new location, unchanged in intent.
- [ ] 3.5 Confirm `Transport`/`TransportError` are reachable from `telemetry-adapter-contracts`'s crate root without requiring any `Adapter` implementer to depend on them. Satisfies FR-002.
- [ ] 3.6 Run `cargo test -p telemetry-adapter-contracts` — confirm the ported tests pass and no existing test broke.

## Phase 4: Supersede `exporter-registry`

- [ ] 4.1 Confirm the `## REMOVED Requirements` delta (already drafted in this change's `specs/exporter-registry/spec.md`) is ready to merge into the canonical spec at archive time.
- [ ] 4.2 No code change required — `exporter-registry` was never implemented.

## Phase 5: Delete the Crate

- [ ] 5.1 Confirm `crates/telemetry-transport-contract/src/` now contains only `lib.rs`, `transport.rs` (emptied by 3.1–3.2's move), and `error.rs` (emptied by 3.2's move) — i.e. every module is either deleted or relocated.
- [ ] 5.2 Delete `crates/telemetry-transport-contract/` entirely (remaining source files, `Cargo.toml`, `tests/`).
- [ ] 5.3 Remove `"crates/telemetry-transport-contract"` from the workspace root `Cargo.toml`'s `members`.

## Phase 6: Verification

- [ ] 6.1 Run `cargo build --workspace` and `cargo test --workspace` — builds and passes with `telemetry-transport-contract` gone.
- [ ] 6.2 Run `rg -rl "telemetry_transport_contract" crates` — zero matches.
- [ ] 6.3 Run `rg -rl "telemetry-transport-contract"` across the workspace `Cargo.toml` files — zero matches.
- [ ] 6.4 Confirm `kit-config` and `fastrand` remain workspace dependencies via `kitlogger` and `kitlogger-sampling` respectively — not orphaned by this crate's removal.
- [ ] 6.5 Confirm no source file defines a second `Transport`, `DeliveryMode`, `TransportResult`, or `TransportError`.
- [ ] 6.6 Confirm this closes every row of ADR-008's original Component Inventory — no component remains unresolved except the explicitly-tracked `kit_config::OutputTarget::File` gap (change 015) and ADR-009's correlation-id unification (Phase 9, not yet done).
