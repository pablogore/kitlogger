# Implementation Plan: Telemetry Configuration Semantics

**Branch**: `develop` | **Date**: 2026-06-17 | **Spec**: [Telemetry Configuration Semantics](spec.md)

**Input**: Feature specification from `specs/002-core-telemetry-domain-model-as-04-telemetry-configuration-semantics/spec.md`

**Technology gate**: Every technology named in this plan and its generated
artifacts MUST be declared in this specification's `tech-stack.yaml`. Missing
or undeclared technology is blocking; do not infer a replacement.

## Summary

Define the canonical telemetry configuration model with six semantic entities (TelemetryConfig, SamplingPolicy, ExporterConfig, ResourceConfig, VerbosityPolicy, SchemaVersion) plus cross-cutting inline validation constraints. AS-04 owns telemetry configuration semantics only; Kit Config owns configuration infrastructure. SamplingPolicy uses a closed OTel-aligned set (AlwaysOn, AlwaysOff, TraceIdRatio, ParentBased, ConsistentProbability) with an Extension variant (SchemaVersion bump required). ExporterConfig is a generic entity with typed settings map. VerbosityPolicy uses a fixed shared level set (OFF, ERROR, WARN, INFO, DEBUG, TRACE). ValidationRule is not a standalone entity — constraints are embedded as inline field metadata. SchemaVersion versions the entire telemetry configuration independently from Kit Config's pipeline version.

## Technical Context

**Language/Version**: Rust (edition 2021)

**Primary Dependencies**: serde (derive), AS-03 adapter contracts (exporter type identifiers)

**Storage**: N/A (configuration schema definitions only)

**Testing**: cargo test

**Target Platform**: Cross-platform (Rust library crate)

**Project Type**: Rust library crate providing telemetry configuration types and validation contracts

**Performance Goals**: N/A — configuration schema definitions only; validation execution performance is owned by Kit Config

**Constraints**:
- No configuration infrastructure (loading, sources, parsing, precedence, reload, secrets)
- ConfigurationSchema is an implementation artifact derived from semantic entities; AS-04 owns the entities, not the schema format
- ValidationRule is not a standalone entity — constraints are inline field metadata
- SamplingPolicy: closed canonical set + Extension variant; extension requires SchemaVersion bump
- ExporterConfig: generic entity with closed type set; new types require SchemaVersion bump
- VerbosityPolicy: fixed level set shared across traces, metrics, logs — NOT extensible
- SchemaVersion versions the entire telemetry configuration; Kit Config pipeline version is independent
- ResourceConfig requires service.name, service.version, deployment.environment defaults

**Scale/Scope**: Library crate defining six configuration entities, their field definitions, defaults, and inline validation constraints. Depends on AS-03 for exporter type identifiers. Pure data types — no runtime behavior, no async interfaces.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

1. **Atomic Specifications** - PASS: Single independently testable feature (telemetry configuration semantics)
2. **Clear Boundaries** - PASS: Scope/non-scope well-defined; Kit Config boundary explicitly declared; ValidationRule reclassified from entity to inline constraints
3. **Dependency Management** - PASS: Depends on AS-03 (adapter contracts — exporter type identifiers) and Kit Config (external); no circular dependencies
4. **Testability** - PASS: Four user stories with acceptance scenarios; SC-001 through SC-009 provide measurable outcomes
5. **Extensibility** - PASS: Extension variant for sampling policies, SchemaVersion for schema evolution, closed exporter type set with versioned addition

**Pre-Design Verdict**: ALL GATES PASS — proceed to Phase 0

**Post-Design Verdict**: ALL GATES PASS — design artifacts consistent with constitution

## Project Structure

### Documentation (this feature)

```text
specs/002-core-telemetry-domain-model-as-04-telemetry-configuration-semantics/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
crates/telemetry-config-semantics/
├── Cargo.toml
├── src/
│   ├── lib.rs               # Crate root with re-exports
│   ├── telemetry_config.rs  # TelemetryConfig entity
│   ├── sampling_policy.rs   # SamplingPolicy entity with closed type set
│   ├── exporter_config.rs   # ExporterConfig generic entity
│   ├── resource_config.rs   # ResourceConfig entity
│   ├── verbosity_policy.rs  # VerbosityPolicy entity with fixed level set
│   └── schema_version.rs    # SchemaVersion entity
└── tests/
    └── config_test.rs       # Configuration entity unit tests
```

**Structure Decision**: Single Rust library crate with one module per semantic entity. Each module contains the entity struct, its fields with serde attributes, defaults implementation, and inline validation constraint declarations. Tests validate entity construction, default application, and constraint correctness.

## Complexity Tracking

*No constitution violations — not applicable.*
