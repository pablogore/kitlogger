# Proposal: Telemetry Configuration Semantics

## Intent

The `telemetry-config-semantics` crate exists as a pure data model, but its core declarations have no meaning. `TelemetryConfig.enabled` is dead code — nothing reads it. `SamplingPolicy.sampling_rate` accepts any `f64` with no range guard. No consumer (including `kitlogger`) ever evaluates a config into an effective runtime posture. This change gives the config behavioral meaning: a deterministic, technology-agnostic evaluation of what telemetry state a config actually expresses, plus validation and a config-accepting constructor on `KITLogger`.

## Scope

### In Scope
- New `CapabilityState` enum (`Enabled`, `Disabled`).
- New `EffectiveTelemetryState` enum (`Enabled`, `Disabled`, `Partial`, `Fallback`).
- `TelemetryConfig::effective_state(&self) -> EffectiveTelemetryState` deterministic evaluation.
- `sampling_rate` validation (`[0.0, 1.0]`) via `validate(&self) -> Result<(), ConfigError>`; invalid sampling resolves to `Fallback`.
- `KITLogger::with_config(config: TelemetryConfig) -> KITLogger` — evaluates effective state, no runtime behavior change.
- Serde round-trip tests for all 8 types (6 existing + 2 new).
- Unit tests covering all 4 effective states.

### Out of Scope
- All telemetry vendors and protocols: OpenTelemetry, OTLP, Jaeger, Zipkin, Prometheus.
- Exporters, transports, runtime pipelines, background workers, network I/O.
- Config loading/parsing from TOML/YAML/env vars (no `kit-config` owner exists).
- Per-exporter `enabled` flag, env-var override semantics.
- Changing any existing public API behavior.

## Capabilities

### New Capabilities
- `telemetry-config-semantics`: the data model, deterministic effective-state evaluation rules, sampling validation, and the `KITLogger` config-acceptance contract.

### Modified Capabilities
- None.

## Approach

Approach B from exploration. Add two value enums and a pure evaluation method to `TelemetryConfig`. Evaluation is deterministic — same input always yields the same `EffectiveTelemetryState`:
- `enabled == false` → `Disabled`
- `enabled == true` + all capabilities enabled → `Enabled`
- `enabled == true` + some capabilities disabled → `Partial`
- validation failure (e.g. invalid `sampling_rate`) → `Fallback`

`CapabilityState` derives the per-capability posture used to distinguish `Enabled` from `Partial`. `KITLogger::with_config` evaluates the state at construction but does not alter logging behavior — it is an additive constructor alongside `new()` and `with_format()`. All new types derive `Clone, Debug, PartialEq, Eq, Serialize, Deserialize`.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/telemetry-config-semantics/src/` | Modified | New enums, `effective_state`, `validate`, `ConfigError` |
| `crates/telemetry-config-semantics/tests/` | Modified | Serde round-trip + 4-state tests |
| `crates/kitlogger/src/lib.rs` | Modified | Add `with_config` constructor |
| `crates/kitlogger/Cargo.toml` | Modified | Add `telemetry-config-semantics` dependency |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| New coupling `kitlogger` → `telemetry-config-semantics` | Med | Additive constructor only; `new()`/`with_format()` untouched; semver-compatible |
| Capability enum design too narrow/broad | Med | Keep `CapabilityState` binary (Enabled/Disabled); defer per-signal granularity |
| `Eq` on config containing `f64` | Low | `Eq` only on new enums, not on `f64`-bearing types |
| Unverified serialization contract | Low | Add round-trip tests for all 8 types |

## Rollback Plan

Revert the commit. The two new enums, `effective_state`, `validate`, and `with_config` are purely additive; removing them restores the prior data-only crate and the `kitlogger` dependency line. No data migration or persisted state is involved.

## Dependencies

- None external. Internal: `kitlogger` gains a workspace path dependency on `telemetry-config-semantics`.

## Success Criteria

- [ ] `CapabilityState` and `EffectiveTelemetryState` exist with the required derives.
- [ ] `effective_state()` returns correct value for all 4 states, deterministically.
- [ ] `sampling_rate` outside `[0.0, 1.0]` fails `validate()` and yields `Fallback`.
- [ ] `KITLogger::with_config` compiles and existing constructors remain unchanged.
- [ ] Serde round-trip tests pass for all 8 types.
- [ ] No OpenTelemetry/vendor/transport dependency is introduced.
