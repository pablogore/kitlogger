# Proposal: Transport/Envelope Cleanup (Migration Plan Phase 7)

## Revision note (premise corrected before implementation)

Implementation was attempted and stopped before any code was touched. This proposal originally claimed `TelemetryBatch`, `PayloadEnvelope`, `TransportMetadata`, and `BackpressureSignal` were "exact duplicates" of `telemetry_types`'s canonical versions — field-for-field identical, safe to delete-and-repoint with "no data lost by switching." Verification found that specific claim false: none of the four types match field-for-field (see the comparison table preserved in issue tracking this finding for the full evidence).

That does not change the underlying architectural conclusion. The two questions are separate:

- **Canonical ownership** — which crate owns this concept. Unaffected by the field mismatch: ADR-010 already names this exact pair (`telemetry_types` vs. `telemetry-transport-contract`'s copies) as a confirmed instance of the duplication pattern it exists to prevent, and its own decision text is explicit that "a type with different fields than the canonical model, but the same reason to exist, is still a violation of this rule — divergent shape is not evidence of a different concept, it is usually evidence that the duplicate evolved independently and drifted." Both `PayloadEnvelope`s exist to do the identical job — envelope a telemetry batch plus metadata for handoff to a transport/delivery boundary (`Transport::send()` here, `TelemetryDelivery::deliver()` in the live `telemetry-adapter-contracts`/`kitlogger` path) — which is the same purpose, not a different one at a different layer (contrast ADR-009's Amendment, where `CorrelationId`'s two versions serve genuinely different purposes at different layers and were correctly kept separate).
- **Representation** — the specific field layout. This is where the divergence is real: `telemetry_types`'s versions are simpler than the local ones (no `resource`, simplified `TraceData`/`MetricData`/`LogData` instead of the richer `context_propagation` domain types, no `TelemetryBatchError`, a `u64` instead of a `Duration`, etc.).

The local models represent the same architectural concept as `telemetry_types`'s, but evolved independently and accumulated additional fields nothing live ever needed. `telemetry_types` remains the canonical owner of that concept — confirmed by it being the type two real crates (`kitlogger`, `telemetry-adapter-contracts`) actually depend on today, in a live trait signature (`TelemetryDelivery::deliver`), while `context_propagation`'s richer models have exactly one consumer in the whole workspace: the orphaned crate this proposal is cleaning up. The local representation is intentionally discarded, not migrated field-for-field, because the crate that holds it is orphaned and no live consumer depends on its additional richness. **This is a semantic consolidation onto the existing canonical owner, not a field-preserving replacement** — ownership is preserved, representation is not, and that distinction is deliberate, not an oversight.

`ADR-007` also has an unrelated documentation error, corrected in this revision: its "Implementation" section claims `telemetry_types` implements `TelemetryBatchError`. It never did — this is a factual mistake in the ADR text, not a decision that changed, so it's fixed as a plain correction rather than a formal amendment (see this change's own notes for the reasoning). `telemetry_types` gains no error type as part of this proposal — nothing live needs one, and adding one speculatively would repeat the same unrequested-capability mistake change 016 already flagged for `LogEvent`'s unused fields.

**Status: unblocked.** Issue #50 (which tracked this finding and the resolution options) is closed, recording this same decision. The scope below (unchanged from the original) may now proceed.

## Intent

`telemetry-transport-contract`'s transport/envelope half (`batch.rs`, `payload.rs`, and part of `transport.rs`/`error.rs`) represents the same telemetry-envelope concept `telemetry_types` already canonically owns per ADR-007/ADR-010 — `TelemetryBatch`, `PayloadEnvelope`, `TransportMetadata`, and `BackpressureSignal` each exist in both crates, evolved independently, and no longer match in shape. ADR-008 scheduled this cleanup as Phase 7: independent of Phases 1–6, no dependency either direction, safe to execute in parallel with any of them. This proposal deletes the local, drifted representation — discarding its extra fields rather than migrating them, since nothing live depends on them — and hands off the genuinely distinct remainder (`Transport` trait, `DeliveryMode`, `TransportResult`, and `TransportError`'s non-`Backpressure` variants) to `telemetry-adapter-contracts`'s own future roadmap, per ADR-008's explicit distinction between "delete" and "transfer as input, not executed here."

## Scope

### In Scope

- Delete `batch.rs` (`TelemetryBatch`, `TelemetryBatchError`) — same concept as `telemetry_types::TelemetryBatch` (ADR-010), evolved independently into a richer, non-matching shape (`resource` field, `context_propagation` domain types for `traces`/`metrics`/`logs`, an `EmptyBatch` validation rule). None of that extra shape migrates; `telemetry_types::TelemetryBatch` has no corresponding fields and gains none here. `telemetry_types` has no `TelemetryBatchError` counterpart at all — it is discarded, not repointed.
- Delete `payload.rs` (`PayloadEnvelope`, `TransportMetadata`) — same concept as `telemetry_types::{PayloadEnvelope, TransportMetadata}` (ADR-010), likewise diverged in representation (`TransportMetadata`'s fields don't overlap at all; `PayloadEnvelope.propagation_metadata` references a different, `context_propagation`-owned type). The local shape is discarded, not migrated.
- Delete `transport.rs`'s `BackpressureSignal` struct — same concept as `telemetry_types::BackpressureSignal` (ADR-010), diverged in representation (`Duration` vs. `u64` `retry_after`; local lacks the canonical's `attributes` field). Discarded, not migrated.
- **Necessary consequence, not scope expansion**: `transport.rs`'s `Transport::send()` currently takes `crate::payload::PayloadEnvelope` (about to be deleted) — it MUST be repointed to `telemetry_types::PayloadEnvelope` to keep compiling. `error.rs`'s `TransportError::Backpressure(BackpressureSignal)` MUST be repointed to `telemetry_types::BackpressureSignal` for the same reason. Both repoints move the reference onto the canonical owner of the same concept; neither preserves the local type's exact shape, since that shape is what's being discarded.
- **Discovered consequence**: once repointed, `telemetry-transport-contract` no longer needs `context-propagation` as a dependency at all — it was only ever used by `batch.rs`/`payload.rs` (both deleted) and `lib.rs`'s crate-root re-export of `context_propagation::models::*`/`propagation_metadata::PropagationMetadata` (removed as part of this cleanup). Remove the dependency and the re-export.
- Remove `batch`/`payload` module declarations and `pub use` re-exports from `lib.rs`.

### Out of Scope (explicit handoff, not executed here)

- `transport.rs`'s `Transport` trait, `DeliveryMode` enum, `TransportResult` type alias.
- `error.rs`'s `TransportError` variants other than `Backpressure` (`Timeout`, `Unavailable`, `PayloadTooLarge`, `UnsupportedTransport`).

These remain in the orphaned crate, unmodified beyond the necessary `BackpressureSignal`/`PayloadEnvelope` repoint above. ADR-008 designates them as input to `telemetry-adapter-contracts`'s own future roadmap — this proposal does not decide whether or how they get adopted there; that decision belongs to whoever specs that crate's next change.

- Deleting any other module of `telemetry-transport-contract`. `formatter`, `output`, `buffering`, `provider`, and `logger` no longer exist — change 016 already deleted them (this bullet originally listed them as still-present and deferred to Phase 8; corrected here, since that's a factual staleness, not an architectural claim). What remains out of scope for this change is `redaction`, `rotation`, and `sampling` — their physical deletion is Phase 8's job, gated on every module being empty.
- Any change to `telemetry_types` or `telemetry-adapter-contracts` themselves — read-only references.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- None. `telemetry_types` is consumed, not modified.

## Approach

This phase is a concept consolidation, not a design exercise — every deleted type already has an accepted, canonical owner for the same concept (ADR-007/ADR-010), even though the two sides' representations no longer match. The two repoints (`PayloadEnvelope`, `BackpressureSignal`) consolidate onto that canonical owner; they are not shape-preserving, and are not claimed to be. The `context-propagation` dependency removal was not anticipated in ADR-008's original phrasing but follows directly from tracing what actually still uses it after the deletions above — verified, not assumed.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/telemetry-transport-contract/src/batch.rs` | Deleted | Same concept as `telemetry_types::TelemetryBatch` (ADR-010); local representation discarded, not migrated |
| `crates/telemetry-transport-contract/src/payload.rs` | Deleted | Same concept as `telemetry_types::PayloadEnvelope`/`TransportMetadata` (ADR-010); local representation discarded, not migrated |
| `crates/telemetry-transport-contract/src/transport.rs` | Modified | `BackpressureSignal` struct deleted; `Transport::send()` repointed to `telemetry_types::PayloadEnvelope` |
| `crates/telemetry-transport-contract/src/error.rs` | Modified | `Backpressure` variant repointed to `telemetry_types::BackpressureSignal` |
| `crates/telemetry-transport-contract/src/lib.rs` | Modified | Remove `batch`/`payload` modules and re-exports; remove `context_propagation::*` re-exports |
| `crates/telemetry-transport-contract/Cargo.toml` | Modified | Remove `context-propagation` dependency; add `telemetry-types` dependency |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `context-propagation` becomes a zero-dependent crate in the workspace once this phase lands | High (certain) | This was previously flagged (change 012's migration risk list) as a Phase-8 concern — it actually materializes here, one phase earlier than originally estimated. ADR-009 (Correlation ID Unification, Phase 9) is expected to give it a renewed, intentional role (shared `CorrelationId`/`TraceId`/`SpanId` consumer) — this is a documented, temporary gap between Phase 7 and Phase 9, not a silent orphan |
| `telemetry-transport-contract`'s own existing tests (`tests/{batch_test,payload_test}.rs`) reference the deleted types | High (certain) | Not migrated or preserved — consistent with change 016's precedent (the crate's test suite is being dismantled module by module, not maintained mid-dismantling) |
| A future reader assumes `Transport`/`TransportError`'s remaining variants are still "live" architecture rather than an unresolved handoff | Low | Explicitly labeled "Out of Scope (explicit handoff, not executed here)" above, matching ADR-008's own wording |
| Deleting `TelemetryBatch`, `PayloadEnvelope`, `TransportMetadata`, and the local `BackpressureSignal` is a public API break for any external consumer of `telemetry-transport-contract` | Low | Confirmed via audit: zero crates in the workspace depend on `telemetry-transport-contract` (`grep` across every `Cargo.toml`, no matches); the crate is not published externally (no `publish` field in its `Cargo.toml`) |

## Rollback Plan

All changes are confined to `telemetry-transport-contract`, a crate with zero workspace dependents. Reverting restores `batch.rs`, `payload.rs`, the local `BackpressureSignal`, and the `context-propagation` dependency exactly as they were — no other crate is affected either way.

## Dependencies

- ADR-007 (canonical types already established in `telemetry_types`), ADR-008 (Migration Plan Phase 7), ADR-010.

## Success Criteria

- [ ] `batch.rs` and `payload.rs` no longer exist.
- [ ] `transport.rs` has no local `BackpressureSignal` definition; `Transport::send()` uses `telemetry_types::PayloadEnvelope`.
- [ ] `error.rs`'s `Backpressure` variant uses `telemetry_types::BackpressureSignal`.
- [ ] `telemetry-transport-contract` no longer depends on `context-propagation`.
- [ ] `telemetry-transport-contract` compiles with only its remaining, still-orphaned modules (`transport`, `error`, `redaction`, `rotation`, `sampling` — `formatter`, `output`, `buffering`, `provider`, `logger` no longer exist, deleted by change 016).
- [ ] No two representations of the same canonical concept remain within the scope of this change (ADR-010) — `TelemetryBatch`, `PayloadEnvelope`, `TransportMetadata`, `BackpressureSignal` each have exactly one owning definition workspace-wide, in `telemetry_types`; the local versions are removed, not reconciled to match.
