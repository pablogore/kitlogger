# Archive Report: Logging Pipeline Consolidation — Phase 1+2

**Date**: 2026-07-04
**Change ID**: 012
**Change Name**: 012-logging-pipeline-consolidation
**Status**: ARCHIVED
**Implementation PR**: [#30](https://github.com/pablogore/kitlogger/pull/30) — `feat(kitlogger): add LoggingConfig-based construction path`, merged to `develop`

## Executive Summary

Implements Migration Plan Phases 1 (Configuration Ownership Reconciliation) and 2 (Facade Config Wiring) of ADR-008. `KITLogger` gains `from_logging_config(kit_config::LoggingConfig) -> Result<Self, ValidationReport>`; the retired `with_config(TelemetryConfig)` path and its dependency on `telemetry-config-semantics::EffectiveTelemetryState` are removed. `telemetry-config-semantics::TelemetryConfig` is scoped down to plugin-enablement flags only, per ADR-008 §4 and ADR-010.

## Capabilities Merged Into Canonical Specs

- `openspec/specs/telemetry-config-semantics/spec.md` — capability-flag model reduced to four flags (`telemetry_enabled`, `tracing_enabled`, `metrics_enabled`, `propagation_enabled`); `EffectiveTelemetryState`, `effective_state()`, `Fallback`, and the sampling-validation contract removed.
- `openspec/specs/kitlogger-config-integration/spec.md` — new capability, `KITLogger`'s `LoggingConfig`-based construction contract.

## Divergence From the Original Spec Delta (Recorded for Traceability)

The spec delta at `specs/telemetry-config-semantics/spec.md` (this folder) originally called for **shrinking** `EffectiveTelemetryState` to three variants (`Enabled`/`Disabled`/`Partial`, dropping `Fallback`) — not removing the type. During the post-apply code review, three independent review passes found `EffectiveTelemetryState`/`effective_state()` had zero remaining production call sites anywhere in the workspace once `KITLogger::with_config` was removed, and its sole justification (design.md: "may still serve a future `AdapterRegistry::health()`-style concern") named no concrete, scheduled consumer in the accepted migration roadmap. Per explicit direction, the type was removed entirely rather than kept at three variants — applying the same "no speculative provisioning" standard the migration itself established (e.g. change 014's crate-boundary principle). `ConfigError::InvalidSamplingRate` — left unconstructable once `FR-008` was removed, and never addressed by the original delta — was removed the same way; `ConfigError` is now an empty enum, `TelemetryConfig::validate()` remains a documented no-op.

**The canonical specs merged above reflect the code that actually shipped** (post-review), not the original delta's narrower "reduce to three variants" text. This report is the record of that divergence and its justification.

## Requirements Delivered

`telemetry-config-semantics`: FR-001 (MODIFIED, four flags), FR-002 (unchanged), FR-012 (MODIFIED, seven types). FR-003 through FR-011 (as originally numbered) are fully retired — some per the original delta (FR-007 Fallback, FR-008 Sampling Validation, FR-011 KITLogger Config Acceptance), the rest (FR-003/004/005/006/009/010, all `EffectiveTelemetryState`/`effective_state()`-dependent) as a consequence of the divergence above.

`kitlogger-config-integration`: FR-001 through FR-005, all delivered as specified — construction from `LoggingConfig`, fail-fast validation, no behavioral leakage beyond validity, retirement of the `TelemetryConfig`-based path, direct `kit-config` dependency.

## Verification Performed

- `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --check` — all clean on the merge commit.
- Runtime verification at the package boundary (throwaway example, not committed): valid config constructs; out-of-range `Probabilistic` rate rejected; boundary rates `0.0`/`1.0` accepted; `enabled = false` does not bypass sub-validation; multiple simultaneous validation failures aggregate into one `ValidationReport`.
- 8-angle code review (correctness ×3, reuse, simplification, efficiency, altitude, conventions) found zero correctness bugs; 4 of 6 cleanup candidates confirmed and fixed (see Divergence above and PR #30's follow-up commit), 2 refuted against this change's own accepted spec (a naive suggestion to wire sampling/redaction/buffering now was refuted by FR-003 and ADR-008's migration sequencing).

## Not Delivered By This Change (Explicitly Deferred)

Per this change's own `proposal.md` Out of Scope: sampling, redaction, buffering, formatter selection, output/dispatch, file output, and `telemetry-transport-contract` removal are not part of this change. They are tracked by later, separate changes (013 onward) already frozen in `openspec/changes/`.

## Follow-Up Items

None blocking. `telemetry-config-semantics::TelemetryConfig`'s remaining plugin-enablement flags (`tracing_enabled`, `metrics_enabled`, `propagation_enabled`, `telemetry_enabled`) still have no concrete consumer — this is expected and intentional (Migration Plan Phase 10, not yet scoped), unlike `EffectiveTelemetryState`, which had no scheduled consumer at all and was removed for that reason.
