# Archive Report: Transport/Envelope Cleanup (Change 017)

## Status

Shipped. All 13 tasks in `tasks.md` complete (`[x]`). Merged into `develop` via PR #52 (implementation) and PR #51 (proposal/design correction). No canonical spec created or updated — this change introduced/modified no capability, pure consolidation.

## What shipped

Deleted `telemetry-transport-contract`'s local `TelemetryBatch`/`TelemetryBatchError` (`batch.rs`), `PayloadEnvelope`/`TransportMetadata` (`payload.rs`), and local `BackpressureSignal` (`transport.rs`). Repointed `Transport::send()` and `TransportError::Backpressure` to `telemetry_types`'s canonical versions of `PayloadEnvelope`/`BackpressureSignal`. Removed the `context-propagation` dependency (only used by the deleted modules and `lib.rs`'s re-exports); added `telemetry-types`. `TransportError` lost its `PartialEq`/`Eq` derive as a necessary consequence — `telemetry_types::BackpressureSignal` doesn't implement either.

## Premise correction, found and resolved before implementation

The original proposal claimed the four local types were "exact duplicates" of `telemetry_types`'s canonical versions — field-for-field identical, safe to delete-and-repoint with "no data lost." Verification before implementation found that specific claim false: none of the four types matched field-for-field (`TransportMetadata`'s fields didn't overlap at all; `TelemetryBatch` was missing a `resource` field and used richer `context_propagation` domain types; `TelemetryBatchError` didn't exist in `telemetry_types` at all, despite `ADR-007`'s "Implementation" section claiming it did; `BackpressureSignal`'s `retry_after` was `Duration` locally vs. `u64` canonically, with a missing `attributes` field).

This did not change the underlying architectural conclusion. An architecture decision session distinguished two separate questions:
- **Canonical ownership** (which crate owns the concept) — unaffected by the field mismatch. `ADR-010` already named this exact pair as a confirmed duplication-via-drift instance, and its own decision text rejects "different fields" as evidence of "different concept." Applying `ADR-009`'s own "different purpose vs. different maturity" test (the one that correctly kept `CorrelationId`'s two representations split) found this case fails it: `Transport::send()` and `TelemetryDelivery::deliver()` serve the identical purpose, unlike `CorrelationId`'s genuinely distinct wire-format vs. log-tag use cases.
- **Representation** (the specific field layout) — this is where the real divergence was. Resolved by discarding the local representation rather than migrating it field-for-field, since `telemetry-transport-contract` had (and has) no in-workspace dependents and nothing live needed the extra richness.

`proposal.md` and a newly-added `design.md` (this change originally had none, on the premise that no design decision was needed) were corrected to state this precisely — "same concept, discarded representation" — instead of the original, factually wrong "exact duplicate" framing. `ADR-007`'s "Implementation" section was also corrected (plain correction, not a formal Amendment — no decision changed, only a factual claim about already-built work that turned out false).

Issue #50 tracked this finding through to resolution and is closed.

## Review findings applied before merge

Two rounds of code review on PR #52 found and fixed:

1. **Breaking-change documentation gap**: the type deletions are a public API break for `telemetry-transport-contract`, but `proposal.md`'s Risks table didn't call this out explicitly, unlike change 012's precedent for the same situation. Added a Risks table row matching that precedent (not a new "Breaking Changes" section — this repo's proposals document breaking changes as Risks rows, not dedicated headings).
2. **Test coverage gap**: deleting `tests/{batch_test,payload_test,integration_tests,transport_test}.rs` (needed since they constructed literals in the discarded local shape) also discarded real coverage of `Transport`/`TransportError` — types that survive this change, not just of the deleted `TelemetryBatch`/`PayloadEnvelope`/`TransportMetadata`. Added back two targeted regression tests covering exactly the surface this change touched (`error.rs`: `TransportError::Backpressure` construction/serde/Display with the canonical `BackpressureSignal`; `transport.rs`: a mock `Transport` impl proving `send()` still works with the canonical `PayloadEnvelope`) — not a restoration of the ~1000 deleted lines, which correctly aren't migrated since they tested the now-gone types.

A third, smaller review round found two LOW-severity wording issues, both fixed: "Status: unblocked" wasn't explicitly tied to issue #50's closure (fixed to reference it), and "orphaned crate" was used interchangeably for two different claims — ADR-008's already-decided permanent fate for this crate, and this change's own verified-today "zero in-workspace dependents" finding. Reworded the argument-bearing occurrences (in `design.md`'s Option A/B reasoning and `proposal.md`'s Revision note) to say what was actually verified, leaving "orphaned"/"still-orphaned" only where citing ADR-008's established terminology (Out of Scope, Success Criteria).

## Verification

- `cargo check -p telemetry-transport-contract` / `cargo check --workspace` — clean, no compatibility stub.
- `cargo test --workspace` — all green, zero regressions.
- `cargo test -p telemetry-transport-contract` — 3/3 passing (the surviving `DeliveryMode` test plus the two added regression tests).
- `cargo clippy -p telemetry-transport-contract --all-targets -- -D warnings` — clean.
- `cargo fmt --package telemetry-transport-contract -- --check` — clean for every file this change touched; pre-existing, unrelated drift in `redaction.rs`/`sampling.rs` (confirmed untouched) left alone.
- Confirmed `context-propagation` has zero workspace dependents and `TelemetryBatch`/`PayloadEnvelope`/`TransportMetadata`/`BackpressureSignal` each have exactly one definition site (`telemetry_types`).

## Out of scope, unaffected

`Transport` trait, `DeliveryMode`, `TransportResult`, and `TransportError`'s non-`Backpressure` variants remain in `telemetry-transport-contract`, unmodified beyond the necessary repoint — per ADR-008, these are input to `telemetry-adapter-contracts`'s own future roadmap, not decided here. `redaction.rs`, `rotation.rs`, `sampling.rs` remain untouched, for Phase 8 to handle once every module in this crate is empty.
