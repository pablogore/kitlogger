# Telemetry Configuration Semantics Specification

## Purpose

Define the behavioral contract for telemetry configuration: the capability-flag model that will drive a future Plugin layer (Migration Plan Phase 10). This spec covers what the system MUST express — not how it is implemented. It does not model, and MUST NOT be used to derive, "is logging enabled" or "how should logging sample/redact/buffer" — those concepts are owned exclusively by `kit_config::LoggingConfig`, materialized and validated by `kit-config`.

## Requirements

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

---

### Requirement: FR-002 CapabilityState Enum

The system MUST provide a `CapabilityState` enum with variants `Enabled` and `Disabled`, deriving `Clone`, `Debug`, `PartialEq`, `Eq`, `Serialize`, and `Deserialize`.

#### Scenario: Serde round-trip for CapabilityState

- GIVEN a `CapabilityState::Enabled` value serialized to its canonical representation
- WHEN the serialized value is deserialized back
- THEN the result is `CapabilityState::Enabled`

---

### Requirement: FR-012 Serde Round-Trip for All Seven Types

All seven public types MUST serialize and deserialize without data loss. The seven types are: `TelemetryConfig`, `ExporterConfig`, `CompressionType`, `ResourceConfig`, `VerbosityPolicy`, `SchemaVersion`, and `CapabilityState`.

#### Scenario: Round-trip for TelemetryConfig with capability flags

- GIVEN a `TelemetryConfig` with mixed capability flag values serialized to JSON
- WHEN the JSON is deserialized back into `TelemetryConfig`
- THEN the deserialized value equals the original

#### Scenario: Round-trip for each of the remaining six types

- GIVEN a populated instance of each type
- WHEN each is serialized then deserialized
- THEN each result equals the original instance

#### Scenario: Backward-compatible deserialization

- GIVEN a serialized config payload that contains no capability flag keys (legacy format)
- WHEN it is deserialized into `TelemetryConfig`
- THEN `telemetry_enabled`, `tracing_enabled`, `metrics_enabled`, and `propagation_enabled` all default to `true`

---

### Requirement: Technology Agnosticism

All types MUST NOT reference, import, or depend on any telemetry vendor, protocol, or SDK (including but not limited to OpenTelemetry, OTLP, Jaeger, Zipkin, Prometheus). Capability semantics represent user intent, not transport or pipeline configuration.

#### Scenario: No vendor dependency in crate graph

- GIVEN the compiled dependency graph of `telemetry-config-semantics`
- WHEN all transitive dependencies are enumerated
- THEN no vendor-specific telemetry library appears in the graph
