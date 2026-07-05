# Proposal: Transport/Envelope Cleanup (Migration Plan Phase 7)

## BLOCKED — premise verification failed (do not implement as written)

Implementation was attempted and stopped before any code was touched. This proposal's core claim — that `TelemetryBatch`, `PayloadEnvelope`, `TransportMetadata`, and `BackpressureSignal` are "exact duplicates" of `telemetry_types`'s canonical versions, safe to delete-and-repoint with "no data lost by switching" — does not hold. Verified against actual source:

| Type | Local (`telemetry-transport-contract`) | Canonical (`telemetry_types`) | Divergence |
|---|---|---|---|
| `TransportMetadata` | `{timestamp: SystemTime, content_type: String, encoding: String}` | `{protocol: String, endpoint: String, attributes: HashMap<String,String>}` | Entirely different fields — not the same concept despite the shared name |
| `TelemetryBatch` | `{resource: Resource, traces: Vec<Span>, metrics: Vec<Metric>, logs: Vec<LogRecord>}` (rich `context_propagation` domain types) + rejects an all-empty batch | `{traces: Vec<TraceData>, metrics: Vec<MetricData>, logs: Vec<LogData>}` (string/f64/u64-only placeholder types) | Missing `resource`; element types are placeholders, not the real domain types; no validation |
| `TelemetryBatchError` | `enum { EmptyBatch }` | Does not exist in `telemetry_types` at all | ADR-007's own "Implementation" section claims it was implemented there — it wasn't |
| `PayloadEnvelope.propagation_metadata` | `context_propagation::propagation_metadata::PropagationMetadata` | `telemetry_types`'s own internal `PropagationMetadata` (`{headers: HashMap<String,String>}`) | Different type, same field name |
| `BackpressureSignal` | `{retry_after: Option<Duration>}`, derives `Eq` | `{retry_after: Option<u64>, attributes: HashMap<String,String>}`, no `Eq` | Different `retry_after` representation, missing field, different derives |

None of the four types are field-for-field identical. `ADR-007` itself has drifted from the actual `telemetry_types` implementation (it documents a `TelemetryBatchError` that was never written). Deleting the local versions and repointing call sites to the "canonical" ones would silently narrow/change the data shape — theoretical rather than operational risk today, since `telemetry-transport-contract` has zero external workspace dependents, but the proposal's premise as written is factually wrong and must not be implemented until re-scoped.

**Status: paused, not implemented.** See issue tracking this finding for the resolution options considered. Do not run `sdd-apply` on this proposal until it is corrected.

## Intent

`telemetry-transport-contract`'s transport/envelope half (`batch.rs`, `payload.rs`, and part of `transport.rs`/`error.rs`) duplicates types already canonical in `telemetry_types` per ADR-007 — `TelemetryBatch`, `PayloadEnvelope`, `TransportMetadata`, and `BackpressureSignal` all exist twice. ADR-008 scheduled this cleanup as Phase 7: independent of Phases 1–6, no dependency either direction, safe to execute in parallel with any of them. This proposal deletes the literal duplicates and hands off the genuinely non-duplicate remainder (`Transport` trait, `DeliveryMode`, `TransportResult`, and `TransportError`'s non-`Backpressure` variants) to `telemetry-adapter-contracts`'s own future roadmap, per ADR-008's explicit distinction between "delete" and "transfer as input, not executed here."

## Scope

### In Scope

- Delete `batch.rs` (`TelemetryBatch`, `TelemetryBatchError`) — exact duplicate of `telemetry_types::{TelemetryBatch, TelemetryBatchError}`.
- Delete `payload.rs` (`PayloadEnvelope`, `TransportMetadata`) — exact duplicate of `telemetry_types::{PayloadEnvelope, TransportMetadata}`.
- Delete `transport.rs`'s `BackpressureSignal` struct — exact duplicate of `telemetry_types::BackpressureSignal`.
- **Necessary consequence, not scope expansion**: `transport.rs`'s `Transport::send()` currently takes `crate::payload::PayloadEnvelope` (about to be deleted) — it MUST be repointed to `telemetry_types::PayloadEnvelope` to keep compiling. `error.rs`'s `TransportError::Backpressure(BackpressureSignal)` MUST be repointed to `telemetry_types::BackpressureSignal` for the same reason. Neither repoint changes either type's public shape or behavior — both already have field-for-field identical canonical counterparts in `telemetry_types`.
- **Discovered consequence**: once repointed, `telemetry-transport-contract` no longer needs `context-propagation` as a dependency at all — it was only ever used by `batch.rs`/`payload.rs` (both deleted) and `lib.rs`'s crate-root re-export of `context_propagation::models::*`/`propagation_metadata::PropagationMetadata` (removed as part of this cleanup). Remove the dependency and the re-export.
- Remove `batch`/`payload` module declarations and `pub use` re-exports from `lib.rs`.

### Out of Scope (explicit handoff, not executed here)

- `transport.rs`'s `Transport` trait, `DeliveryMode` enum, `TransportResult` type alias.
- `error.rs`'s `TransportError` variants other than `Backpressure` (`Timeout`, `Unavailable`, `PayloadTooLarge`, `UnsupportedTransport`).

These remain in the orphaned crate, unmodified beyond the necessary `BackpressureSignal`/`PayloadEnvelope` repoint above. ADR-008 designates them as input to `telemetry-adapter-contracts`'s own future roadmap — this proposal does not decide whether or how they get adopted there; that decision belongs to whoever specs that crate's next change.

- Deleting any other module of `telemetry-transport-contract` (`formatter`, `output`, `sampling`, `redaction`, `rotation`, `buffering`, `provider`, `logger` — all still hold logic already absorbed elsewhere in Phases 3–4, but their physical deletion is Phase 8's job, gated on every module being empty).
- Any change to `telemetry_types` or `telemetry-adapter-contracts` themselves — read-only references.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- None. `telemetry_types` is consumed, not modified.

## Approach

This phase is a duplication removal, not a design exercise — every deleted type already has an accepted, canonical counterpart (ADR-007). The two repoints (`PayloadEnvelope`, `BackpressureSignal`) are mechanical consequences of deletion, not new decisions. The `context-propagation` dependency removal was not anticipated in ADR-008's original phrasing but follows directly from tracing what actually still uses it after the deletions above — verified, not assumed.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/telemetry-transport-contract/src/batch.rs` | Deleted | Duplicate of `telemetry_types::TelemetryBatch` |
| `crates/telemetry-transport-contract/src/payload.rs` | Deleted | Duplicate of `telemetry_types::PayloadEnvelope`/`TransportMetadata` |
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

## Rollback Plan

All changes are confined to `telemetry-transport-contract`, a crate with zero workspace dependents. Reverting restores `batch.rs`, `payload.rs`, the local `BackpressureSignal`, and the `context-propagation` dependency exactly as they were — no other crate is affected either way.

## Dependencies

- ADR-007 (canonical types already established in `telemetry_types`), ADR-008 (Migration Plan Phase 7), ADR-010.

## Success Criteria

- [ ] `batch.rs` and `payload.rs` no longer exist.
- [ ] `transport.rs` has no local `BackpressureSignal` definition; `Transport::send()` uses `telemetry_types::PayloadEnvelope`.
- [ ] `error.rs`'s `Backpressure` variant uses `telemetry_types::BackpressureSignal`.
- [ ] `telemetry-transport-contract` no longer depends on `context-propagation`.
- [ ] `telemetry-transport-contract` compiles with only its remaining, still-orphaned modules (`transport`, `error`, `formatter`, `output`, `sampling`, `redaction`, `rotation`, `buffering`, `provider`, `logger`).
- [ ] No two canonical models remain for the same concept within the scope of this change (ADR-010) — `TelemetryBatch`, `PayloadEnvelope`, `TransportMetadata`, `BackpressureSignal` each exist exactly once workspace-wide, in `telemetry_types`.
