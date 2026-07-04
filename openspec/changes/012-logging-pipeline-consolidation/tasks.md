# Tasks: Logging Pipeline Consolidation — Phase 1+2

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 250–380 |
| 400-line budget risk | Medium |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | single-pr |
| Chain strategy | size-exception |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: size-exception
400-line budget risk: Medium

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | `telemetry-config-semantics` scope-down (Phase 1) | PR 1 | Must land before Unit 2's tests can exercise a stable config shape |
| 2 | `kitlogger` kit-config wiring (Phase 2) | PR 1 | Same PR — the two phases are reviewed together per `proposal.md`'s bundling rationale |

---

## Phase 1: `telemetry-config-semantics` Scope-Down

- [ ] 1.1 **RED** — Update the existing capability-flag test(s) to assert `TelemetryConfig` has exactly four flags (`telemetry_enabled`, `tracing_enabled`, `metrics_enabled`, `propagation_enabled`) with no `correlation_enabled`. Run `cargo test -p telemetry-config-semantics` — expect compile errors referencing the field being removed. Satisfies FR-001.
- [ ] 1.2 **RED** — Update/remove tests referencing `SamplingPolicy`/`SamplingPolicyType`/`sampling_rate`, asserting these types no longer exist on `TelemetryConfig`. Satisfies the FR-008 removal.
- [ ] 1.3 **GREEN** — Remove `correlation_enabled` and `sampling: Option<SamplingPolicy>` from `TelemetryConfig` in `telemetry_config.rs`; delete `SamplingPolicy`/`SamplingPolicyType` (`sampling_policy.rs`). Update `telemetry_enabled`'s doc comment to describe it as the plugin layer's master switch, not "is logging enabled." Run `cargo test -p telemetry-config-semantics` — expect remaining compile errors in `effective_state()`/`validate()`.
- [ ] 1.4 **GREEN** — Update `effective_state()`: FR-004/FR-005/FR-006 evaluation logic drops `correlation_enabled` from the flag set it checks. Remove `EffectiveTelemetryState::Fallback` and its match arms; `validate()` no longer has a `sampling_rate` range to check (resolve whatever remains of `validate()`'s body accordingly — if nothing is left to validate, document that explicitly rather than leaving a vestigial always-`Ok` check with no explanation). Run `cargo test -p telemetry-config-semantics` — compiles.
- [ ] 1.5 **RED** — Replace old scenario-based tests for FR-001, FR-003–FR-007 with the updated four-flag / three-variant scenarios from `specs/telemetry-config-semantics/spec.md` in this change. Run — red until 1.6.
- [ ] 1.6 **GREEN** — Implementation from 1.3–1.4 makes all updated tests pass. Run `cargo test -p telemetry-config-semantics`.
- [ ] 1.7 Update the FR-012 serde round-trip test to the corrected eight-type list (`TelemetryConfig`, `ExporterConfig`, `CompressionType`, `ResourceConfig`, `VerbosityPolicy`, `SchemaVersion`, `CapabilityState`, `EffectiveTelemetryState` — `SamplingPolicy`/`SamplingPolicyType` dropped). Run — passes.

---

## Phase 2: `kitlogger` Facade Wiring

- [ ] 2.1 Add `kit-config` as a direct path dependency in `crates/kitlogger/Cargo.toml`, mirroring `telemetry-transport-contract`'s existing pattern (`{ path = "../../../kit-config/crates/kit-config", default-features = false, features = ["logging"] }`). Satisfies FR-005.
- [ ] 2.2 **RED** — Write a failing test `constructs_from_valid_logging_config` asserting `KITLogger` can be constructed from a valid `kit_config::LoggingConfig` value. Run `cargo test -p kitlogger` — expect a compile error (no such constructor yet). Satisfies FR-001.
- [ ] 2.3 **RED** — Write a failing test `rejects_invalid_logging_config` using a `LoggingConfig` with an out-of-range `Probabilistic` sampling rate, asserting construction fails and surfaces the validation error. Satisfies FR-002.
- [ ] 2.4 **GREEN** — Implement the `LoggingConfig`-accepting construction path on `KITLogger`, invoking `kit_config`'s `Validation::validate()` and surfacing failure to the caller. Run `cargo test -p kitlogger` — 2.2 and 2.3 pass.
- [ ] 2.5 **RED** — Write a test `enabled_false_does_not_change_emission_yet` asserting `KITLogger::log`/`log_record` output is identical regardless of `LoggingConfig.enabled`. This documents the explicit scope boundary (FR-003), not new behavior to build.
- [ ] 2.6 **GREEN** — Confirm 2.5 passes with no gating code added — if it fails, something outside this change's scope has been built; back it out.
- [ ] 2.7 Remove `KITLogger::with_config(TelemetryConfig)` and its `effective_state()` call from `crates/kitlogger/src/lib.rs`. Satisfies FR-004.
- [ ] 2.8 Update `crates/kitlogger/tests/with_config_test.rs`: remove assertions against the retired `with_config(TelemetryConfig)` path. Replace with assertions covering FR-001/FR-002 if not already covered by 2.2/2.3, or delete the file if fully superseded.
- [ ] 2.9 Run `cargo test -p kitlogger` — all tests pass; zero references to `EffectiveTelemetryState` remain anywhere in `kitlogger`.

---

## Phase 3: Verification

- [ ] 3.1 Run `cargo clippy --workspace -- -D warnings` — zero warnings.
- [ ] 3.2 Run `cargo fmt --workspace -- --check` — passes.
- [ ] 3.3 Run `cargo test --workspace` — all tests pass; no regressions in crates outside `telemetry-config-semantics`/`kitlogger`.
- [ ] 3.4 Grep the workspace for `EffectiveTelemetryState::Fallback`, `SamplingPolicy`, `SamplingPolicyType`, `correlation_enabled` (in the `telemetry-config-semantics` context), and `with_config(TelemetryConfig` — zero remaining references.
