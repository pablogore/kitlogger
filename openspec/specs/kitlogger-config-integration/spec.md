# KITLogger Config Integration Specification

## Purpose

Define the behavioral contract for `KITLogger`'s consumption of `kit_config::LoggingConfig`: how it is constructed from that config, what happens when the config is invalid, and what explicitly does NOT change as a result. This spec covers what the system MUST express — not how it is implemented. `LoggingConfig` is owned by the Logging domain; `kit-config`'s role here is limited to materializing and validating it, not authoring its shape or semantics.

## Requirements

### Requirement: FR-001 Construction from LoggingConfig

`KITLogger` MUST provide a construction path that accepts a `kit_config::LoggingConfig` value.

#### Scenario: Valid config constructs successfully

- GIVEN a `kit_config::LoggingConfig` value that passes `kit_config`'s `Validation` trait
- WHEN `KITLogger` is constructed from it
- THEN construction succeeds and returns a usable `KITLogger` instance

### Requirement: FR-002 Fail-Fast on Invalid Config

Constructing `KITLogger` from a `LoggingConfig` that fails `kit_config`'s `Validation` trait MUST surface that failure to the caller. `KITLogger` MUST NOT be constructed with undefined behavior from a config that fails validation.

#### Scenario: Invalid sampling configuration rejected

- GIVEN a `LoggingConfig` with `sampling.strategy = Probabilistic` and `sampling.rate` outside `[0.0, 1.0]`
- WHEN `KITLogger` construction is attempted with that config
- THEN construction fails and the validation error is surfaced to the caller

#### Scenario: Invalid rotation configuration rejected

- GIVEN a `LoggingConfig` with `rotation.max_size_mb = 0`
- WHEN `KITLogger` construction is attempted with that config
- THEN construction fails and the validation error is surfaced to the caller

### Requirement: FR-003 No Behavioral Change from LoggingConfig Fields

Fields of `LoggingConfig` other than those needed to determine construction-time validity MUST NOT be consulted by any runtime code path introduced by this capability. `LoggingConfig.enabled`, `.level`, `.sampling`, `.redact`, `.buffering`, `.rotation`, and `.output` MUST NOT gate or alter `KITLogger::log`/`log_record` behavior as a result of this capability alone.

#### Scenario: enabled = false does not change emission behavior yet

- GIVEN a `LoggingConfig` with `enabled = false` that otherwise passes validation
- WHEN `KITLogger` is constructed from it and a log call is made
- THEN the log call behaves identically to a `KITLogger` constructed from an `enabled = true` config
- AND this is expected: gating emission on `enabled` is a separate, future capability (Migration Plan Phase 5), not part of this one

### Requirement: FR-004 Removal of TelemetryConfig-Based Construction

`KITLogger::with_config(TelemetryConfig)` MUST NOT exist. `KITLogger` MUST NOT depend on `EffectiveTelemetryState` or call `effective_state()` as part of its construction.

#### Scenario: TelemetryConfig-based construction is absent

- GIVEN the public API surface of `KITLogger`
- WHEN its constructors are enumerated
- THEN no constructor accepts a `TelemetryConfig` value
- AND no constructor references `EffectiveTelemetryState` or `effective_state()`

### Requirement: FR-005 kit-config Dependency Origin

The `kitlogger` crate's Cargo manifest MUST declare a direct dependency on `kit-config`. This dependency MUST NOT be routed through any intermediary crate (e.g. `telemetry-transport-contract`).

#### Scenario: Direct dependency confirmed

- GIVEN the `kitlogger` crate's `Cargo.toml`
- WHEN its dependency list is inspected
- THEN `kit-config` appears as a direct entry
- AND no intermediary crate is required to reach `kit_config::LoggingConfig` types from `kitlogger`
