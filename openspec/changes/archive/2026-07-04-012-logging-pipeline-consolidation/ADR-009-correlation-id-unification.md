# ADR-009: Correlation ID Unification

## Status

Accepted, amended during Phase 9 scoping (change 019) — see "Amendment" below. The original Decision (shared extraction crate) is superseded; Context is retained for history but its "comparably minimal" claim is corrected.

## Amendment (Phase 9 scoping)

The original analysis below compared `kitlogger_log_domain::{CorrelationId, TraceId, SpanId}` only against `context_propagation::models`' type aliases (`TraceId = [u8; 16]`, `SpanId = [u8; 8]`, `CorrelationId = String`) and concluded both sides were "comparably minimal." Reading the rest of `context-propagation` (`correlation.rs`, `trace_context.rs`) before actually scoping Phase 9 found this incomplete: `context-propagation` also owns a full W3C Trace Context implementation — `TraceId`/`SpanId` newtypes with hex `Display`/`FromStr`, `TraceFlags`, `TraceState` (vendor entries), a `TraceContext` type validating the complete `traceparent` format, and a UUID-v7-based `CorrelationIdentifier` with creation-timestamp extraction. None of this has any counterpart in `kitlogger-log-domain`'s simple, unvalidated `String` newtypes.

These are not the same concept at two levels of polish — they serve different purposes at different layers: `kitlogger-log-domain`'s identifiers exist to *tag a log line* for human/log-search correlation, opaque and format-free by design; `context-propagation`'s identifiers exist to *interoperate over the wire* per the W3C spec, where the byte-exact format is the entire point. Forcing a single shared type would either strip `context-propagation`'s spec compliance to fit logging's simplicity, or force logging to carry byte-array/hex-parsing complexity it has no use for.

**Corrected Decision** (replaces the original numbered list below): no shared extraction crate. `context-propagation` remains the sole owner of the rich, W3C-compliant `TraceId`/`SpanId`/`CorrelationIdentifier`. `kitlogger-log-domain` keeps its simple `String`-based `CorrelationId`/`TraceId`/`SpanId` newtypes exactly as they are today — unchanged, not merged, not deprecated. Neither crate depends on the other, matching the original goal of not inverting either domain's dependency direction, achieved here by there being no shared type to require a dependency for in the first place. Converting a `context-propagation` identifier into the display string a `kitlogger-log-domain::LogContext` carries is a future tracing-plugin's boundary responsibility, not something either crate does internally.

The original Decision, Consequences, and their reasoning are retained below for historical record; they no longer reflect this ADR's operative decision.

## Context (original, superseded by the Amendment above)

Two independent identifier models for correlation/trace/span identity already exist in the workspace:

- `kitlogger_log_domain::{CorrelationId, TraceId, SpanId}` (`crates/kitlogger-log-domain/src/{correlation_id,trace_id,span_id}.rs`) — newtypes held by `LogContext`, part of the canonical, infra-agnostic logging domain that every live crate (`kitlogger`, `kitlogger-formatter`, `console-exporter`, `security-jwt`) actually consumes.
- `context_propagation::models` (`crates/context-propagation/src/models.rs`) — its own identity/context types, consumed today only by `telemetry-transport-contract`'s orphaned transport half (`batch.rs`, `payload.rs`).

Both crates are comparably minimal in their own dependencies (`kitlogger-log-domain`: serde, serde_json, chrono; `context-propagation`: serde, uuid) — neither is architecturally "more pure" than the other, so purity alone does not decide ownership.

This is not a hypothetical risk. ADR-008 documents that `telemetry-transport-contract` already reimplemented `PayloadEnvelope`/`TelemetryBatch`/`BackpressureSignal` independently of the canonical versions in `telemetry-types` (ADR-007), purely because it was never forced to depend on the canonical crate. The same failure mode is guaranteed to recur with correlation IDs the moment a tracing/OTel plugin is built: a `LogRecord` needs a `CorrelationId` to tag log lines, and a `TraceContext` needs the *same* `CorrelationId` to propagate causally-related work — if these remain two independent types, whichever plugin tries to correlate them will either silently pick one arbitrarily or need a conversion layer that itself becomes a third, ad hoc definition of identity.

Correlation IDs are usable by base logging (e.g. tagging a log line with a request ID) even when no distributed tracing plugin is present at all — so the identifier concept is more primitive than either "logging" or "tracing/propagation" individually. Neither domain should be forced to depend on the other just to share an identifier type: `context-propagation` depending on the logging domain would invert "observability is built on top of logging" (propagation is a cross-cutting mechanism usable by non-logging plugins too, e.g. metrics), and `kitlogger-log-domain` depending on `context-propagation` would compromise the audit's own finding that `kitlogger-log-domain` is "the correct innermost layer, don't touch."

## Decision (original, superseded by the Amendment above)

1. There will be exactly one `CorrelationId`, one `TraceId`, and one `SpanId` type in the workspace.
2. These three identifier types are extracted to a minimal, dependency-free shared location that both `kitlogger-log-domain` and `context-propagation` depend on — not on each other. This preserves `kitlogger-log-domain` as the innermost, infra-agnostic layer, and keeps `context-propagation` free of a logging-domain dependency it has no reason to carry.
3. `kitlogger_log_domain::LogContext` and `context_propagation::models` are both updated to reference the shared identifier types instead of each independently defining their own.
4. This unification must land **before** any tracing-correlation Logger Plugin is built (Migration Plan Phase 10). Building such a plugin against the current split would create a third, ad hoc identity mapping instead of removing the duplication.

## Consequences (original, superseded by the Amendment above)

### Positive

- One identity concept, shared by the logging domain and the propagation domain, with neither depending on the other.
- Removes a duplication that would otherwise resurface — with higher cost, since it would be discovered mid-implementation of a tracing plugin rather than during an architecture review.
- Establishes a template for future identifier-shaped primitives that multiple bounded contexts need without implying a dependency direction between them.

### Negative

- Introduces a new, very small shared crate (or equivalent extraction point) purely to hold three newtypes — an additional compilation unit for a small amount of code.
- Both `kitlogger-log-domain` and `context-propagation` require a (mechanical, non-behavioral) migration to reference the shared types before this is resolved.

## Related

- ADR-007: Shared Canonical Types — establishes the precedent that a "canonical, shared-dependency" crate is the correct pattern when multiple contexts need the same type (still valid in general; this ADR's Amendment found it doesn't apply here, since the two sides aren't the same type at two levels of polish).
- ADR-008: Logging Pipeline Consolidation — this ADR is a named prerequisite of ADR-008's migration step 9 (building Logger Plugins) — still true under the Amendment: a future tracing plugin is exactly where the conversion boundary between the two identifier models belongs.
- ADR-010: Canonical Domain Models — the general rule this ADR is a concrete instance of; the Amendment applies the same rule more precisely by recognizing two adjacent-but-distinct concepts instead of forcing one.
- Change 019 (Phase 9 closure) — where this Amendment was made, during scoping, before any code was written against the original Decision.
- engram `architecture/telemetry-transport-contract-fate` — originating finding.
