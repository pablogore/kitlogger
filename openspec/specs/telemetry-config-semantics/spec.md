# Telemetry Configuration Semantics Specification

## Purpose

Define the behavioral contract for telemetry configuration: capability flags, deterministic effective-state evaluation, sampling validation, and the `KITLogger` config-acceptance constructor. This spec covers what the system MUST express — not how evaluation is implemented.

## Requirements

### Requirement: FR-001 Capability Flag Model

`TelemetryConfig` MUST expose five boolean capability flags: `telemetry_enabled`, `tracing_enabled`, `metrics_enabled`, `correlation_enabled`, and `propagation_enabled`. Each flag MUST default to `true` via serde `default` so configs that omit them deserialize with all capabilities active.

#### Scenario: All flags present in config

- GIVEN a serialized config with all five flags explicitly set to `false`
- WHEN the config is deserialized
- THEN each flag holds the value from the serialized form

#### Scenario: Config omits all capability flags

- GIVEN a serialized config with no capability flag keys
- WHEN the config is deserialized
- THEN all five flags default to `true`

---

### Requirement: FR-002 CapabilityState Enum

The system MUST provide a `CapabilityState` enum with variants `Enabled` and `Disabled`, deriving `Clone`, `Debug`, `PartialEq`, `Eq`, `Serialize`, and `Deserialize`.

#### Scenario: Serde round-trip for CapabilityState

- GIVEN a `CapabilityState::Enabled` value serialized to its canonical representation
- WHEN the serialized value is deserialized back
- THEN the result is `CapabilityState::Enabled`

---

### Requirement: FR-003 EffectiveTelemetryState Enum

The system MUST provide an `EffectiveTelemetryState` enum with variants `Enabled`, `Disabled`, `Partial`, and `Fallback`, deriving `Clone`, `Debug`, `PartialEq`, `Eq`, `Serialize`, and `Deserialize`.

#### Scenario: Serde round-trip for EffectiveTelemetryState

- GIVEN each of the four `EffectiveTelemetryState` variants serialized individually
- WHEN each serialized form is deserialized back
- THEN each result matches the original variant

---

### Requirement: FR-004 Disabled State Evaluation

`TelemetryConfig::effective_state(&self)` MUST return `EffectiveTelemetryState::Disabled` when `telemetry_enabled` is `false`, regardless of any other flag values.

#### Scenario: Disabled overrides all capability flags

- GIVEN a config where `telemetry_enabled = false` and all other capability flags are `true`
- WHEN `effective_state()` is called
- THEN the result is `EffectiveTelemetryState::Disabled`

#### Scenario: Disabled with sampling_rate out of range

- GIVEN a config where `telemetry_enabled = false` and `sampling_rate = -0.5`
- WHEN `effective_state()` is called
- THEN the result is `EffectiveTelemetryState::Fallback` (Fallback checked first)

---

### Requirement: FR-005 Enabled State Evaluation

`effective_state()` MUST return `EffectiveTelemetryState::Enabled` when `telemetry_enabled` is `true` AND all four capability flags (`tracing_enabled`, `metrics_enabled`, `correlation_enabled`, `propagation_enabled`) are `true`.

#### Scenario: All flags true, valid sampling

- GIVEN a config where all five flags are `true` and `sampling_rate` is within `[0.0, 1.0]`
- WHEN `effective_state()` is called
- THEN the result is `EffectiveTelemetryState::Enabled`

---

### Requirement: FR-006 Partial State Evaluation

`effective_state()` MUST return `EffectiveTelemetryState::Partial` when `telemetry_enabled` is `true` AND at least one of the four capability flags is `false`, and validation passes.

#### Scenario: One capability disabled

- GIVEN a config where `telemetry_enabled = true`, `tracing_enabled = false`, and the other three capability flags are `true`, with a valid `sampling_rate`
- WHEN `effective_state()` is called
- THEN the result is `EffectiveTelemetryState::Partial`

#### Scenario: Multiple capabilities disabled

- GIVEN a config where `telemetry_enabled = true`, `metrics_enabled = false`, `correlation_enabled = false`, and the remaining flags are `true`, with a valid `sampling_rate`
- WHEN `effective_state()` is called
- THEN the result is `EffectiveTelemetryState::Partial`

---

### Requirement: FR-007 Fallback State Evaluation (Validation Failure)

`effective_state()` MUST return `EffectiveTelemetryState::Fallback` when `validate()` fails. Fallback MUST be checked before enabled/partial logic, meaning an invalid config always returns `Fallback` regardless of flag values.

#### Scenario: sampling_rate below range

- GIVEN a config with `telemetry_enabled = true`, all capability flags `true`, and `sampling_rate = -0.1`
- WHEN `effective_state()` is called
- THEN the result is `EffectiveTelemetryState::Fallback`

#### Scenario: sampling_rate above range

- GIVEN a config with `telemetry_enabled = true`, all capability flags `true`, and `sampling_rate = 1.1`
- WHEN `effective_state()` is called
- THEN the result is `EffectiveTelemetryState::Fallback`

#### Scenario: Fallback supersedes Partial

- GIVEN a config with `telemetry_enabled = true`, `tracing_enabled = false`, and `sampling_rate = 2.0`
- WHEN `effective_state()` is called
- THEN the result is `EffectiveTelemetryState::Fallback` (not `Partial`)

---

### Requirement: FR-008 Sampling Validation

`TelemetryConfig::validate(&self) -> Result<(), ConfigError>` MUST return `Err(ConfigError)` when `sampling_rate` is outside `[0.0, 1.0]`. It MUST return `Ok(())` when `sampling_rate` is within the inclusive range.

#### Scenario: Valid boundary values

- GIVEN a config with `sampling_rate = 0.0`
- WHEN `validate()` is called
- THEN the result is `Ok(())`
- AND a config with `sampling_rate = 1.0` also returns `Ok(())`

#### Scenario: Invalid sampling_rate returns ConfigError

- GIVEN a config with `sampling_rate = 1.5`
- WHEN `validate()` is called
- THEN the result is `Err(ConfigError)` describing the invalid range

---

### Requirement: FR-009 Deterministic Evaluation

`effective_state()` MUST be pure and side-effect free. Calling it multiple times with identical inputs MUST return the same result each time.

#### Scenario: Repeated calls on same config

- GIVEN a `TelemetryConfig` value that does not change between calls
- WHEN `effective_state()` is called three times consecutively
- THEN all three results are identical

---

### Requirement: FR-010 Default Config Effective State

A `TelemetryConfig::default()` MUST produce a config whose `effective_state()` is `EffectiveTelemetryState::Enabled`.

#### Scenario: Default config evaluates to Enabled

- GIVEN `TelemetryConfig::default()`
- WHEN `effective_state()` is called
- THEN the result is `EffectiveTelemetryState::Enabled`

---

### Requirement: FR-011 KITLogger Config Acceptance

`KITLogger` MUST expose a `with_config(config: TelemetryConfig) -> KITLogger` constructor. This constructor MUST evaluate `effective_state()` at construction time. It MUST NOT alter the runtime logging behavior introduced by `new()` or `with_format()`.

#### Scenario: with_config constructs successfully

- GIVEN a valid `TelemetryConfig`
- WHEN `KITLogger::with_config(config)` is called
- THEN a `KITLogger` instance is returned without error

#### Scenario: Existing constructors unaffected

- GIVEN a `KITLogger` constructed via `KITLogger::new()`
- WHEN any logging operation is performed
- THEN behavior is identical to before this change was applied

---

### Requirement: FR-012 Serde Round-Trip for All Eight Types

All eight public types MUST serialize and deserialize without data loss. The eight types are: `TelemetryConfig`, `SamplingPolicy`, `SamplingPolicyType`, `ExporterConfig`, `CompressionType`, `ResourceConfig`, `VerbosityPolicy`, `SchemaVersion`, plus the two new types `CapabilityState` and `EffectiveTelemetryState`.

#### Scenario: Round-trip for TelemetryConfig with capability flags

- GIVEN a `TelemetryConfig` with mixed capability flag values serialized to JSON
- WHEN the JSON is deserialized back into `TelemetryConfig`
- THEN the deserialized value equals the original

#### Scenario: Round-trip for each of the eight existing types

- GIVEN a populated instance of each existing type
- WHEN each is serialized then deserialized
- THEN each result equals the original instance

#### Scenario: Backward-compatible deserialization

- GIVEN a serialized config payload that contains no capability flag keys (legacy format)
- WHEN it is deserialized into `TelemetryConfig`
- THEN `telemetry_enabled`, `tracing_enabled`, `metrics_enabled`, `correlation_enabled`, and `propagation_enabled` all default to `true`
- AND `effective_state()` returns `EffectiveTelemetryState::Enabled` (assuming valid sampling)

---

### Requirement: Technology Agnosticism

All types and evaluation logic MUST NOT reference, import, or depend on any telemetry vendor, protocol, or SDK (including but not limited to OpenTelemetry, OTLP, Jaeger, Zipkin, Prometheus). Capability semantics represent user intent, not transport or pipeline configuration.

#### Scenario: No vendor dependency in crate graph

- GIVEN the compiled dependency graph of `telemetry-config-semantics`
- WHEN all transitive dependencies are enumerated
- THEN no vendor-specific telemetry library appears in the graph
