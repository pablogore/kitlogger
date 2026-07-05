# Design: Transport/Envelope Cleanup (Migration Plan Phase 7)

## Why this document exists

`tasks.md` originally stated no `design.md` was needed — "every deletion and repoint follows mechanically from ADR-007's already-accepted canonical types." That held only if the local and canonical types were field-for-field equivalent. Verification before implementation found they aren't (see `proposal.md`'s revision note for the full comparison). This document exists to record the distinction that resolves the apparent contradiction — the types differ in representation, but not in the concept they belong to — so the deletion proceeds on a stated, correct basis rather than a false "identical shapes" one.

## Canonical Concept vs. Concrete Representation

Two separate questions were conflated in the original proposal:

1. **Canonical Concept** — which crate owns the *idea* of "an envelope wrapping a telemetry batch plus metadata, for handoff across a transport/delivery boundary"? ADR-007 and ADR-010 already answer this: `telemetry_types`. This is a question about identity and purpose, not about field layout.
2. **Concrete Representation** — what fields does *this specific struct* have today? This is where `telemetry-transport-contract`'s versions and `telemetry_types`'s versions diverge — evolved independently, by different authors, at different times, into non-matching shapes.

ADR-010's own decision text anticipates exactly this conflation and rules on it directly: "A type with different fields than the canonical model, but the same reason to exist, is still a violation of this rule — divergent shape is not evidence of a different concept, it is usually evidence that the duplicate evolved independently and drifted." That is precisely what happened here. Representation drift is not evidence against shared ownership; it's the expected symptom of the exact failure ADR-010 exists to catch.

## Why this isn't the same situation as ADR-009's `CorrelationId` Amendment

ADR-009 originally assumed `kitlogger_log_domain`'s and `context_propagation`'s correlation identifiers were "the same concept at two levels of polish," then found on closer reading that they serve genuinely different purposes at different layers — one exists to *tag a log line* (opaque, format-free), the other to *interoperate over the wire* per W3C Trace Context (byte-exact format is the entire point). That's a real test for telling "different maturity of the same concept" apart from "different concept entirely": ask what purpose each side actually serves, not just what fields it has.

Applying the same test here: `telemetry-transport-contract::Transport::send(envelope: PayloadEnvelope) -> DeliveryMode` and `telemetry-adapter-contracts::TelemetryDelivery::deliver(envelope: PayloadEnvelope) -> ()` serve the *identical* purpose — handing a telemetry envelope to something that delivers it across a boundary. Neither is more "wire-format-exact" or more "log-tag-opaque" than the other; they are the same seam, implemented twice. This case fails the test that let `CorrelationId` stay split. It confirms `PayloadEnvelope`/`TelemetryBatch`/`TransportMetadata`/`BackpressureSignal` are one concept with two accidental representations, not two concepts.

## Why the richer representation is discarded, not migrated

`telemetry_types::PayloadEnvelope` is not a placeholder nobody uses — it is the type in `telemetry-adapter-contracts::TelemetryDelivery::deliver()`'s live trait signature, which `KITLogger` itself implements today (`crates/kitlogger/src/lib.rs`, currently a stub body but a real, compiled trait implementation). Two live crates (`kitlogger`, `telemetry-adapter-contracts`) depend on `telemetry_types` for this. `context_propagation`'s richer domain models (`Resource`, `Span`, `Metric`, `LogRecord`, the W3C-aligned `PropagationMetadata`) have exactly one consumer in the entire workspace: `telemetry-transport-contract` — the orphaned crate this change is cleaning up, and whose remaining modules (`transport`, `error`, plus `redaction`/`rotation`/`sampling`, out of scope here) are themselves slated for full removal in Phase 8.

Migrating the local representation's extra fields into `telemetry_types` (enriching it to match) was considered and rejected: it would mean the *canonical* crate's shape gets dictated by the *orphaned* crate's needs, at the cost of pulling `context_propagation` in as a real dependency of code two live crates rely on — the same "never forced to depend on the canonical crate" failure mode ADR-009's Context section names as the root cause of this entire family of duplication. Nothing today needs that richness on the `telemetry_types` side; adding it speculatively repeats the mistake change 016 already flagged and declined to make for `LogEvent`'s unused `target`/`module`/`file`/`line` fields.

## What actually happens, concretely

- `telemetry_types` remains exactly as it is — untouched, read-only, per the original proposal's scope.
- `batch.rs`, `payload.rs`, and `transport.rs`'s local `BackpressureSignal` are deleted outright, not migrated field-by-field.
- `Transport::send()` and `TransportError::Backpressure` are repointed to reference `telemetry_types`'s versions of `PayloadEnvelope`/`BackpressureSignal` — this is a reference change, not a data-preserving conversion, because there is no data flowing through this orphaned, dead-code path to preserve.
- No compatibility shim, adapter, or enrichment of `telemetry_types` is introduced. Ownership consolidates; representation does not carry over.

## ADR-007: correction, not amendment

ADR-007's "Implementation" section states `telemetry_types` implements `TelemetryBatchError` ("error type for batch validation"). It does not — confirmed by reading the crate's full source (`crates/telemetry-types/src/lib.rs`, 154 lines, zero occurrences of `Error`). This is a factual mistake about what was built, not a decision that has since changed. ADR-009's Amendment is the right precedent for *when* to amend: it recorded a changed conclusion, reached from new evidence, that reversed an operative decision (shared extraction crate → no shared crate). Here, no decision is reversed — ADR-007's decision ("`telemetry_types` owns these five types") still holds; only its description of what got implemented was wrong. A plain correction to ADR-007's "Implementation" section is the proportionate fix. Filing this as a formal Amendment would overstate what changed: nothing about the *decision* is being revisited, only a claim about *already-completed work* that turns out not to have happened.

## Related

- ADR-007: Shared Canonical Types — decision is affirmed; implementation description corrected.
- ADR-008: Logging Pipeline Consolidation — Phase 7 scope, unchanged.
- ADR-009: Correlation ID Unification (Amendment) — supplies the "different purpose at different layer" test applied above, and this case fails it (unlike `CorrelationId`, which passed it).
- ADR-010: Canonical Domain Models — the general rule this change is an instance of; its own text already anticipates and resolves the "divergent shape" objection.
