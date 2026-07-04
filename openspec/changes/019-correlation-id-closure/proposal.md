# Proposal: Correlation ID Closure (Migration Plan Phase 9)

## Intent

ADR-009 originally decided `kitlogger_log_domain::{CorrelationId, TraceId, SpanId}` and `context_propagation`'s identifier types should be unified into one shared, extracted type, based on an analysis that compared `kitlogger-log-domain`'s newtypes only against `context_propagation::models`' bare type aliases (`TraceId = [u8; 16]`, `SpanId = [u8; 8]`, `CorrelationId = String`) and found them "comparably minimal."

Scoping this phase required reading the rest of `context-propagation` first (`correlation.rs`, `trace_context.rs`) — something the original ADR-009 analysis never did. That reading found a full W3C Trace Context implementation: byte-array `TraceId`/`SpanId` with hex `Display`/`FromStr`, `TraceFlags`, vendor `TraceState`, complete `traceparent`-string validation, and a UUID-v7-based `CorrelationIdentifier` with timestamp extraction — none of which has any counterpart in `kitlogger-log-domain`'s simple, unvalidated `String` newtypes.

**ADR-009 is amended** (see the ADR itself, `openspec/changes/012-logging-pipeline-consolidation/ADR-009-correlation-id-unification.md`, "Amendment" section) rather than executed as originally written. This proposal is the closure of that amended decision: confirming no code changes are required, and formally closing Migration Plan Phase 9.

## Corrected Understanding

These are not one concept duplicated at two levels of polish — they serve different purposes at different layers:

- `kitlogger-log-domain`'s identifiers exist to *tag a log line* for correlation — opaque, format-free, exactly as simple as logging needs.
- `context-propagation`'s identifiers exist to *interoperate over the wire* per the W3C Trace Context spec — the byte-exact, validated format is the entire point.

Unifying them into one shared type would force a choice between stripping `context-propagation`'s spec compliance or burdening logging with byte-array/hex-parsing complexity it never needs. Per the amended ADR-009: no shared type, no shared crate, no dependency either direction.

## Scope

### In Scope

- Formally close Migration Plan Phase 9 with **zero code changes** — both `kitlogger_log_domain::{CorrelationId, TraceId, SpanId}` and `context-propagation`'s identifier types remain exactly as they are today.
- Add a one-line doc-comment cross-reference in each of `kitlogger-log-domain`'s three identifier files, pointing a future reader to `context-propagation`'s richer types for wire-level interop — so this question isn't silently rediscovered later.
- Confirm, for the record, that no crate today attempts to convert between the two (so there is no existing behavior this amendment could regress).

### Out of Scope

- Any new shared crate (the original ADR-009 plan) — explicitly not built.
- Designing the actual conversion boundary a future tracing-correlation Logger Plugin (Migration Plan Phase 10) would use to map a `context-propagation` identifier onto a `kitlogger-log-domain::LogContext` — that plugin's own future proposal decides this, informed by this ADR's amendment, not decided here.
- Any change to `context-propagation`'s own internal inconsistency, noted but not addressed here: `context_propagation::models::TraceId` (`[u8; 16]` type alias) and `context_propagation::trace_context::TraceId` (a newtype struct wrapping the same byte layout) are two different representations of the same concept *within that one crate* — out of scope for this migration, which only concerns the boundary between `kitlogger-log-domain` and `context-propagation`, not `context-propagation`'s own internals.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- None.

## Approach

This is a documentation and record-closure change, not a design or implementation change — the corrected decision is "do nothing to unify," and the only artifact this produces is the cross-reference comment plus the ADR-009 amendment itself (already written).

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/kitlogger-log-domain/src/correlation_id.rs` | Modified | One-line doc comment cross-reference added, no behavior change |
| `crates/kitlogger-log-domain/src/trace_id.rs` | Modified | Same |
| `crates/kitlogger-log-domain/src/span_id.rs` | Modified | Same |
| `openspec/changes/012-logging-pipeline-consolidation/ADR-009-correlation-id-unification.md` | Amended | Already done — see the ADR's own "Amendment" section |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| A future tracing plugin reintroduces the original "just extract a shared type" idea without reading this amendment | Low | The amendment and this closure proposal are both explicit, cross-referenced records; the doc-comment cross-reference in `kitlogger-log-domain`'s source is the first thing a future implementer would see |
| `context-propagation`'s own internal `TraceId` duplication (`models` vs. `trace_context`) is mistaken for something this migration already resolved | Low | Explicitly called out as out of scope above, not silently left ambiguous |

## Rollback Plan

Trivial — this change touches three doc comments and one ADR document. Reverting restores the pre-amendment wording with no functional impact either way.

## Dependencies

- ADR-009 (as amended), ADR-010.
- `context-propagation`'s actual source (`correlation.rs`, `trace_context.rs`, `models/mod.rs`) — read in full before this proposal was written.

## Success Criteria

- [ ] ADR-009's Amendment section accurately reflects the corrected finding (already done).
- [ ] No shared crate exists or was created for correlation/trace/span identifiers.
- [ ] `kitlogger-log-domain`'s three identifier files carry a cross-reference to `context-propagation`'s richer types.
- [ ] Neither `kitlogger-log-domain` nor `context-propagation` depends on the other.
- [ ] This is recorded as the closure of Migration Plan Phase 9 — the last open item from ADR-008/ADR-009's original migration plan (excluding Phase 10, explicitly out of scope for the entire initiative).
