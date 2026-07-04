# Proposal: Logging Pipeline Consolidation — Phase 1+2 (Configuration Ownership Reconciliation & Facade Wiring)

## Intent

An architecture audit found that `telemetry-transport-contract` is an orphaned crate (zero workspace dependents) reimplementing a second, parallel logging pipeline directly on `kit_config::LoggingConfig`, while the production facade `kitlogger::KITLogger` has never consumed `kit_config` at all. ADR-008 (Logging Pipeline Consolidation), ADR-009 (Correlation ID Unification), and ADR-010 (Canonical Domain Models) — included in this change — record the accepted decisions resolving this. This proposal executes the first two phases of that consolidation's migration plan:

- **Phase 1 — Configuration Ownership Reconciliation**: eliminate the conceptual duplication between `telemetry-config-semantics::TelemetryConfig` and `kit_config::LoggingConfig` (both currently model "is telemetry/logging enabled" and "sampling" with zero shared types) and redefine `TelemetryConfig`'s scope accordingly.
- **Phase 2 — Facade Config Wiring**: move the `kit-config` dependency edge so it originates at `kitlogger` (the facade a host imports) instead of the doomed `telemetry-transport-contract`, and give `KITLogger` a construction path driven by `kit_config::LoggingConfig`.

These two phases are bundled into one change because Phase 3 (redaction + sampling absorption) cannot start until both land, and because reviewing them together is the only way to see that Phase 1's `TelemetryConfig` scope-down and Phase 2's `KITLogger` rewiring are two sides of the same "one config domain for logging" decision (ADR-008 §4).

Phases 3–9 of the full migration (redaction, sampling, buffering, output, formatter reconciliation, orchestration fold, `LogEvent` retirement, transport/envelope cleanup, crate removal, correlation-id unification) are tracked in `design.md` for context but are **not** part of this change — they will land as their own, later changes under this same consolidation initiative.

## Scope

### In Scope

- Scoping down `telemetry-config-semantics::TelemetryConfig`: remove `correlation_enabled` and the trace-ratio `SamplingPolicy`/`sampling_rate`, both of which conceptually compete with `kit_config::LoggingConfig`. Retain `telemetry_enabled` (redefined), `tracing_enabled`, `metrics_enabled`, `propagation_enabled` as plugin-enablement flags scoped to a future Plugin layer (Migration Plan Phase 10), not to logging behavior.
- Removing `EffectiveTelemetryState::Fallback` (its only trigger, `sampling_rate` validation failure, no longer exists once `SamplingPolicy` is removed — kept as a dead, unreachable variant otherwise).
- Retiring `KITLogger::with_config(TelemetryConfig)` entirely, including its call to `effective_state()`.
- Adding a new capability, `kitlogger-config-integration`: `KITLogger` becomes constructible from `kit_config::LoggingConfig`, failing fast on an invalid config via `kit_config`'s own `Validation` trait.
- Moving the `kit-config` Cargo dependency edge so it originates at `kitlogger`.

### Out of Scope

- Any pipeline behavior: level filtering, sampling, redaction, buffering, formatting, dispatch (Migration Plan Phases 3–5). `LoggingConfig` fields other than bare construction-time validity are not consulted by any runtime code path in this change.
- Gating any `KITLogger` runtime behavior on `LoggingConfig.enabled` — folded into Phase 5 alongside level filtering, so that gate is designed once.
- Removing `telemetry-transport-contract`'s own `kit-config` dependency or the crate itself (Phase 8). A transitional state where both `kitlogger` and `telemetry-transport-contract` depend on `kit-config` is expected.
- `CorrelationId`/`TraceId`/`SpanId` unification (ADR-009, a separate future change).
- Any change to `kit_config` itself (sibling repo) — read-only reference throughout.
- How a future Plugin layer (Phase 10) reads `tracing_enabled`/`metrics_enabled`/`propagation_enabled` — only that the fields survive with that intended purpose.

## Capabilities

### New Capabilities

- `kitlogger-config-integration`: `KITLogger`'s construction contract against `kit_config::LoggingConfig`, including construction-time validation failure behavior.

### Modified Capabilities

- `telemetry-config-semantics`: `TelemetryConfig`'s capability-flag model, `EffectiveTelemetryState` (loses `Fallback`), and the removal of `KITLogger` config-acceptance (`FR-011`) and sampling validation (`FR-008`) from this capability's contract.

## Approach

`telemetry-config-semantics` keeps `TelemetryConfig` as the source of plugin-enablement flags for a Plugin layer that does not exist yet (Phase 10) — it does not disappear, and it does not merge into `kit_config`. What it loses is everything that duplicated a concept the Logging domain's `LoggingConfig` model already owns: `correlation_enabled` (owned via `LoggingConfig.correlation.enabled`, which `kit-config` materializes and validates) and the trace-ratio `SamplingPolicy` (owned via `LoggingConfig.sampling`, and keeping a second, differently-scoped "sampling" behind the same word is exactly the drift ADR-010 exists to prevent).

Removing `SamplingPolicy` has a second-order consequence worth calling out explicitly: `EffectiveTelemetryState::Fallback` existed solely to signal that `sampling_rate` validation had failed. With that validation gone, `Fallback` would become an unreachable enum variant — so it is removed in the same change, not left as dead code.

`KITLogger` gains a construction path from `kit_config::LoggingConfig` — the first time this facade has ever consumed `kit_config` directly. This is deliberately the *only* thing that changes in `KITLogger` here: no pipeline stage reads `LoggingConfig`'s behavioral fields yet. That restraint is what keeps Phase 5 (orchestration fold) from having to un-do or reconcile a partially-built gate.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/telemetry-config-semantics/src/telemetry_config.rs` | Modified | Remove `correlation_enabled`, `sampling` (`SamplingPolicy`); redefine `telemetry_enabled` |
| `crates/telemetry-config-semantics/src/*` (`sampling_policy.rs` and `EffectiveTelemetryState`) | Modified/Removed | `SamplingPolicy`/`SamplingPolicyType` removed; `EffectiveTelemetryState::Fallback` removed |
| `crates/telemetry-config-semantics/tests/*` | Modified | Coverage updated for removed fields/variant; retained-field coverage preserved |
| `crates/kitlogger/Cargo.toml` | Modified | Add direct path dependency on `kit-config` (sibling repo) |
| `crates/kitlogger/src/lib.rs` | Modified | Remove `with_config(TelemetryConfig)`; add construction from `kit_config::LoggingConfig` |
| `crates/kitlogger/tests/with_config_test.rs` | Modified | Update or replace — currently asserts the retired `with_config`/`effective_state()` path |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `FR-011` (`KITLogger` config acceptance) is an already-accepted requirement in `openspec/specs/telemetry-config-semantics/spec.md`; removing it is a breaking change to an accepted contract | High (certain) | Explicit `## REMOVED Requirements` entry in this change's spec delta, with rationale; not a silent drop |
| Field-parity of `kitlogger/tests/with_config_test.rs` after `with_config` removal | Medium | Task explicitly covers updating this test in the same change, not leaving it red |
| Starting Phase 3 before this change lands would recreate a two-config-source problem | Low (procedural, not code) | Migration Plan's own phase gate; Phase 3 is not scheduled until this change is accepted |
| `EffectiveTelemetryState::Fallback` removal is a public API break for any external consumer | Low | Confirmed via audit: only `kitlogger` and `telemetry-config-semantics`'s own tests reference this type in the workspace |

## Rollback Plan

Both capabilities affected here (`telemetry-config-semantics`, `kitlogger`) are internal to this workspace with no external published consumers. Reverting this change's commit(s) restores `TelemetryConfig`'s five-flag shape, `EffectiveTelemetryState::Fallback`, and `KITLogger::with_config(TelemetryConfig)` exactly as they were — no data migration or external coordination is involved.

## Dependencies

- ADR-008, ADR-009, ADR-010 (this change) — immutable architectural decisions this proposal executes against.
- `openspec/specs/telemetry-config-semantics/spec.md` — the accepted spec this change modifies.
- `kit_config::LoggingConfig` and its `Validation` trait (external, sibling repo, read-only reference).

## Success Criteria

- [ ] `TelemetryConfig` no longer has `correlation_enabled` or `SamplingPolicy`/`sampling_rate`.
- [ ] `TelemetryConfig` retains `telemetry_enabled` (redefined), `tracing_enabled`, `metrics_enabled`, `propagation_enabled`.
- [ ] `EffectiveTelemetryState` has exactly three variants: `Enabled`, `Disabled`, `Partial`.
- [ ] `kitlogger`'s Cargo.toml lists `kit-config` as a direct dependency.
- [ ] `KITLogger` is constructible from `kit_config::LoggingConfig`; an invalid config is rejected at construction time.
- [ ] `KITLogger::with_config(TelemetryConfig)` no longer exists.
- [ ] No source or test file in the workspace references `EffectiveTelemetryState::Fallback` or `with_config(TelemetryConfig)`.
- [ ] `telemetry-config-semantics` and `kitlogger` test suites pass with coverage matching the updated contracts.
- [ ] No two canonical models remain for the same concept within the scope of this change (ADR-010).
