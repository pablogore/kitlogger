# Proposal: Crate Removal (Migration Plan Phase 8)

## Intent

This is the join point ADR-008 always intended: `telemetry-transport-contract` is removed once every module it owns has been absorbed, transferred, or deleted. Three things needed resolving before that could happen cleanly, none of them anticipated in full when Phases 3–7 were originally scoped:

1. **Orphaned source was never deleted.** Changes 013, 014, and 015 each absorbed a module's *behavior* into a new canonical home, but none of their `tasks.md` included deleting the original source (`formatter.rs`, `output.rs`, `sampling.rs`, `redaction.rs`, `rotation.rs`, `buffering.rs`, `provider.rs`, `logger.rs`). All eight still exist on disk.
2. **Phase 7's handoff was never executed.** Change 017 explicitly left `Transport`, `DeliveryMode`, `TransportResult`, and `TransportError` as "input to `telemetry-adapter-contracts`'s own future roadmap, not executed here." Nobody has picked it up since. Per explicit decision, this phase executes it rather than letting the crate's deletion silently drop it.
3. **A stale, unimplemented, competing spec was found.** `openspec/specs/exporter-registry/spec.md` describes a single-exporter-selection-by-name registry with a default fallback — a different model from `output-adapter-contracts`'s dispatch-to-all (ADR-008 §6). It was never mentioned in its originating change's (005-console-exporter) own proposal or design, no `ExporterRegistry` type was ever implemented, and that change's own verify-report already flagged it as "out of scope... consider documenting this explicitly." Per explicit decision, this phase formally declares it superseded.

## ADR-010 Domain Model Declaration

| | Transport Strategy | `exporter-registry` |
|---|---|---|
| **Canonical Owner** | `telemetry-adapter-contracts` (new `transport` module) | N/A — superseded, not owned going forward |
| **Canonical Model** | `Transport` trait (protocol-agnostic send), `DeliveryMode`, `TransportResult<T>`, `TransportError` — an optional building block for adapters needing wire-level delivery semantics, not a required part of the `Adapter` supertrait | N/A |
| **Consumers** | Future network-facing adapters (most naturally `otlp-exporter`); NOT required by `console-exporter`/`file-exporter`, which need no wire-level transport abstraction | N/A |
| **Existing Competing Models** | `telemetry_transport_contract::{transport::{Transport, DeliveryMode, TransportResult}, error::TransportError}` — the orphaned originals, now properly migrated (this phase finally executes Phase 7's deferred handoff) | `output-adapter-contracts` (change 014) — the dispatch-to-all model ADR-008 §6 actually committed to; `exporter-registry`'s single-selection-by-name model is declared superseded, not merged or reconciled |

## Scope

### In Scope

- Delete the eight orphaned modules whose behavior was already absorbed in Phases 3–5: `formatter.rs`, `output.rs`, `sampling.rs`, `redaction.rs`, `rotation.rs`, `buffering.rs`, `provider.rs`, `logger.rs`.
- Move `transport.rs`'s `Transport`/`DeliveryMode`/`TransportResult` and `error.rs`'s remaining `TransportError` (post change-017: `Timeout`, `Unavailable`, `Backpressure(telemetry_types::BackpressureSignal)`, `PayloadTooLarge`, `UnsupportedTransport`) into a new `transport` module of `telemetry-adapter-contracts`, unchanged in shape.
- Formally declare `exporter-registry` superseded by `output-adapter-contracts` — a `## REMOVED Requirements` delta marking every one of its requirements, with reason, not a silent deletion of the historical spec file.
- Delete `crates/telemetry-transport-contract/` entirely: source, `Cargo.toml`, and its workspace member entry.
- Delete `crates/telemetry-transport-contract/tests/*` (the remaining `integration_tests.rs`/`transport_test.rs`, if not already removed by change 017's own task for `batch_test.rs`/`payload_test.rs`).

### Out of Scope

- Any change to `output-adapter-contracts`, `console-exporter`, `file-exporter`, `kitlogger-redaction`, `kitlogger-sampling`, `kitlogger-buffering`, `kitlogger-format-selection`, or `kitlogger-formatter` — all already-frozen, unmodified.
- Designing how a future `otlp-exporter` actually uses the relocated `Transport` trait — only that it becomes available where an OTel-flavored adapter would look for it.
- `context-propagation`'s fate — already zero-dependent since change 017 (Phase 7), addressed by ADR-009/Phase 9, not this change.

## Capabilities

### New Capabilities

- None. The relocated `Transport`/`DeliveryMode`/`TransportResult`/`TransportError` are additions to the existing `telemetry-adapter-contracts` capability, not a new one.

### Modified Capabilities

- `telemetry-adapter-contracts`: gains the `transport` module (no existing openspec-format spec exists for this capability to write a delta against — its original spec lives in the older, pre-openspec `specs/archive/2026-06-19-002-core-telemetry-domain-model-as-03-telemetry-adapter-contracts/` convention; this change's spec is written fresh, scoped only to what's being added here).
- `exporter-registry`: every requirement removed, superseded by `output-adapter-contracts`.

## Approach

The eight orphaned-module deletions are mechanical — each was already fully absorbed elsewhere, verified during their own phases (changes 013–015's "Confirm ... subsumed, not silently dropped" tasks). The `Transport` relocation is the one place this phase makes a real decision: `Transport`/`DeliveryMode`/`TransportResult`/`TransportError` become an *optional* toolkit inside `telemetry-adapter-contracts`, not a mandatory part of the `Adapter` supertrait — `console-exporter`/`file-exporter` never needed wire-level transport semantics and still don't; only a future network-facing adapter (`otlp-exporter`, most naturally) would reach for it. `TransportError` stays a distinct type from the existing `AdapterError` — they cover different failure domains (wire-level vs. registry/lifecycle-level) and nothing found during this migration warrants merging them.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/telemetry-transport-contract/` | Deleted entirely | Source, `Cargo.toml`, tests |
| `Cargo.toml` (workspace) | Modified | Remove `telemetry-transport-contract` from `members` |
| `crates/telemetry-adapter-contracts/src/transport.rs` | New | `Transport`, `DeliveryMode`, `TransportResult`, `TransportError` (relocated, unchanged in shape) |
| `crates/telemetry-adapter-contracts/src/lib.rs` | Modified | Add `pub mod transport;` and its re-exports |
| `openspec/specs/exporter-registry/spec.md` | Superseded (via this change's delta) | Declared replaced by `output-adapter-contracts`, not deleted as a historical record |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| A future contributor rediscovers `exporter-registry`'s spec and assumes it's still live, active work | Low | Explicit `## REMOVED Requirements` delta with reasons, matching the pattern already used for `telemetry-config-semantics` FR-007/008/011 in change 012 |
| `TransportError`/`AdapterError` having two separate error types in one crate reads as duplication at a glance | Low | Explicitly justified in `design.md`: different failure domains, not a competing model of the same concept |
| Deleting `telemetry-transport-contract` removes `kit-config`'s and `fastrand`'s only-remaining-there dependency edge, but neither becomes orphaned workspace-wide | None | `kitlogger` already depends on `kit-config` (Phase 2); `kitlogger-sampling` already depends on `fastrand` (Phase 3) |
| This is the last of 8 phases — a missed cleanup item here has no later phase to catch it | Medium | Explicit grep-based verification tasks for zero remaining references to every deleted/relocated type |

## Rollback Plan

This is the migration's final, irreversible step by design — once `telemetry-transport-contract` is deleted and its useful content has landed in `telemetry-adapter-contracts`, there is no partial state to roll back to that makes sense; a revert would restore the orphaned crate wholesale (via version control), not selectively.

## Dependencies

- ADR-008 (Migration Plan Phase 8, the final step), ADR-010.
- Changes 013, 014, 015, 016, 017 — all their absorbed modules and deferred handoff are inputs consumed here.
- `openspec/specs/exporter-registry/spec.md` and its originating verify-report (`openspec/changes/archive/2026-06-19-005-console-exporter/verify-report.md`) — read and confirmed before this proposal was written.

## Success Criteria

- [ ] `crates/telemetry-transport-contract/` no longer exists.
- [ ] The workspace root `Cargo.toml` no longer lists it as a member.
- [ ] `telemetry-adapter-contracts` has a `transport` module containing `Transport`, `DeliveryMode`, `TransportResult`, `TransportError`, unchanged in shape from the orphaned originals.
- [ ] `exporter-registry`'s spec is marked superseded, with every requirement's removal reason pointing to `output-adapter-contracts`.
- [ ] No crate in the workspace references `telemetry_transport_contract::` anywhere.
- [ ] `kit-config` and `fastrand` remain workspace dependencies (via `kitlogger` and `kitlogger-sampling` respectively) — their removal from this crate does not orphan them.
- [ ] No two canonical models remain for the same concept within the scope of this change (ADR-010) — this closes the entire `telemetry-transport-contract` consolidation initiated by ADR-008.
