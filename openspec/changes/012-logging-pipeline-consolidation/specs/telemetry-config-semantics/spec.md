# Delta: Telemetry Configuration Semantics

## REMOVED Requirements

### Requirement: FR-007 Fallback State Evaluation (Validation Failure)

**Reason**: This state existed solely to signal that `sampling_rate` validation had failed (FR-008, removed below). `SamplingPolicy`/`sampling_rate` is removed in this change because it conceptually duplicates `kit_config::LoggingConfig.sampling` — a different concept (trace-ratio vs. log-event sampling) sharing the same word, which is exactly the drift ADR-010 exists to prevent. With that validation gone, no code path can produce `Fallback`; removing it here avoids leaving an unreachable enum variant.

### Requirement: FR-008 Sampling Validation

**Reason**: `sampling_rate` modeled trace-ratio sampling, distinct from `kit_config::LoggingConfig.sampling` (log-event sampling: probabilistic/every-Nth/rate-limit). Log-event sampling has exactly one canonical model going forward: `kit_config::LoggingConfig.sampling` (ADR-010). A future trace-sampling concept, if introduced for the Plugin layer (Migration Plan Phase 10), MUST be named and scoped distinctly from "sampling."

### Requirement: FR-011 KITLogger Config Acceptance

**Reason**: `KITLogger` now consumes `kit_config::LoggingConfig` directly (see new capability `kitlogger-config-integration` in this change). The `TelemetryConfig`-accepting constructor triggered no behavior beyond evaluating `effective_state()`, which no longer factors into `KITLogger`'s construction per ADR-008 §4 (`LoggingConfig` is the single configuration model, owned by the Logging domain and materialized/validated by kit-config).

## MODIFIED Requirements

### Requirement: FR-001 Capability Flag Model

`TelemetryConfig` MUST expose four boolean capability flags: `telemetry_enabled`, `tracing_enabled`, `metrics_enabled`, and `propagation_enabled`. Each flag MUST default to `true` via serde `default` so configs that omit them deserialize with all capabilities active. `telemetry_enabled` represents whether the plugin layer's telemetry subsystem (tracing, metrics, and propagation collectively) is enabled — it MUST NOT be read as, or used to derive, "is logging enabled"; that concept is owned exclusively by the Logging domain's `LoggingConfig.enabled` (materialized and validated by `kit-config`).

#### Scenario: All flags present in config

- GIVEN a serialized config with all four flags explicitly set to `false`
- WHEN the config is deserialized
- THEN each flag holds the value from the serialized form

#### Scenario: Config omits all capability flags

- GIVEN a serialized config with no capability flag keys
- WHEN the config is deserialized
- THEN all four flags default to `true`

### Requirement: FR-003 EffectiveTelemetryState Enum

The system MUST provide an `EffectiveTelemetryState` enum with exactly three variants: `Enabled`, `Disabled`, and `Partial`, deriving `Clone`, `Debug`, `PartialEq`, `Eq`, `Serialize`, and `Deserialize`. The enum MUST NOT include a `Fallback` variant.

#### Scenario: Serde round-trip for EffectiveTelemetryState

- GIVEN each of the three `EffectiveTelemetryState` variants serialized individually
- WHEN each serialized form is deserialized back
- THEN each result matches the original variant

### Requirement: FR-004 Disabled State Evaluation

`TelemetryConfig::effective_state(&self)` MUST return `EffectiveTelemetryState::Disabled` when `telemetry_enabled` is `false`, regardless of any other flag values.

#### Scenario: Disabled overrides all capability flags

- GIVEN a config where `telemetry_enabled = false` and all other capability flags are `true`
- WHEN `effective_state()` is called
- THEN the result is `EffectiveTelemetryState::Disabled`

### Requirement: FR-005 Enabled State Evaluation

`effective_state()` MUST return `EffectiveTelemetryState::Enabled` when `telemetry_enabled` is `true` AND all three remaining capability flags (`tracing_enabled`, `metrics_enabled`, `propagation_enabled`) are `true`.

#### Scenario: All flags true

- GIVEN a config where all four flags are `true`
- WHEN `effective_state()` is called
- THEN the result is `EffectiveTelemetryState::Enabled`

### Requirement: FR-006 Partial State Evaluation

`effective_state()` MUST return `EffectiveTelemetryState::Partial` when `telemetry_enabled` is `true` AND at least one of the three remaining capability flags (`tracing_enabled`, `metrics_enabled`, `propagation_enabled`) is `false`.

#### Scenario: One capability disabled

- GIVEN a config where `telemetry_enabled = true`, `tracing_enabled = false`, and the other two capability flags are `true`
- WHEN `effective_state()` is called
- THEN the result is `EffectiveTelemetryState::Partial`

#### Scenario: Multiple capabilities disabled

- GIVEN a config where `telemetry_enabled = true`, `metrics_enabled = false`, `propagation_enabled = false`, and `tracing_enabled` is `true`
- WHEN `effective_state()` is called
- THEN the result is `EffectiveTelemetryState::Partial`

### Requirement: FR-012 Serde Round-Trip for All Eight Types

All eight public types MUST serialize and deserialize without data loss. The eight types are: `TelemetryConfig`, `ExporterConfig`, `CompressionType`, `ResourceConfig`, `VerbosityPolicy`, `SchemaVersion`, `CapabilityState`, and `EffectiveTelemetryState`.

#### Scenario: Round-trip for TelemetryConfig with capability flags

- GIVEN a `TelemetryConfig` with mixed capability flag values serialized to JSON
- WHEN the JSON is deserialized back into `TelemetryConfig`
- THEN the deserialized value equals the original

#### Scenario: Round-trip for each of the remaining seven types

- GIVEN a populated instance of each type
- WHEN each is serialized then deserialized
- THEN each result equals the original instance

#### Scenario: Backward-compatible deserialization

- GIVEN a serialized config payload that contains no capability flag keys (legacy format)
- WHEN it is deserialized into `TelemetryConfig`
- THEN `telemetry_enabled`, `tracing_enabled`, `metrics_enabled`, and `propagation_enabled` all default to `true`
- AND `effective_state()` returns `EffectiveTelemetryState::Enabled`
