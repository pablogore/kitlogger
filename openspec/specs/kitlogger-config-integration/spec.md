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

### Requirement: FR-003 Behavioral Integration via the Emission Pipeline

`LoggingConfig`'s behavioral fields (`.enabled`, `.level`, `.sampling`, `.redact`, `.buffering`, `.format`, `.output`) drive real `KITLogger::log`/`log_record` behavior, exactly as specified by the `kitlogger-emission-pipeline` capability (change 015). This supersedes this capability's original restriction ("fields other than those needed for construction-time validity MUST NOT be consulted by any runtime code path") — that restriction existed only as a scope boundary until this phase landed, per its own original wording ("gating any runtime behavior... is folded into Phase 5").

#### Scenario: LoggingConfig fields now gate real behavior

- GIVEN a `KITLogger` constructed from a `LoggingConfig`
- WHEN a log call is made
- THEN `LoggingConfig.enabled`, `.level`, `.sampling`, `.redact`, `.buffering`, and `.format` each observably affect the outcome, per `kitlogger-emission-pipeline`'s requirements

#### Scenario: Construction-time validation is unaffected

- GIVEN an invalid `LoggingConfig` (failing `kit_config`'s `Validation` trait)
- WHEN `KITLogger` construction is attempted
- THEN construction still fails at construction time, exactly as this capability originally specified (FR-002, unchanged by this delta)

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
