# Feature Specification: Telemetry Configuration Semantics

**SPEC_ID**: `002-core-telemetry-domain-model-as-04-telemetry-configuration-semantics`

**PARENT_SPEC_ID**: `002-core-telemetry-domain-model`

**PARENT_SPEC_NAME**: `core-telemetry-domain-model`

**CAPABILITY_ID**: `002`

**CAPABILITY_NAME**: `core-telemetry-domain-model`

**EXPAND_ID**: AS-04

**Created**: 2026-06-14

**Status**: Draft

## Scope

Define the canonical telemetry configuration model with explicit semantic entities: TelemetryConfig, SamplingPolicy, ExporterConfig, ResourceConfig, VerbosityPolicy, and SchemaVersion; plus cross-cutting inline validation constraints on all field definitions. This specification owns telemetry configuration semantics only. Configuration infrastructure (loading, sources, parsing, environment integration, secrets, precedence, runtime reload, lifecycle) belongs to Kit Config. ConfigurationSchema is an implementation artifact derived from these semantic entities.

## Non-Scope

- Configuration loading, providers, or sources
- Environment variable handling or configuration file formats
- Configuration parsing, precedence, or merging
- Runtime reload infrastructure or secrets management
- Configuration storage or discovery

## Responsibility

Define telemetry configuration schema, defaults, constraints, and validation rules.

## Dependencies

- `002-core-telemetry-domain-model-as-03-telemetry-adapter-contracts` (AS-03)
- Kit Config (external capability)

## User Scenarios & Testing

### User Story 1 - Enable/Disable Telemetry
A KitLogger administrator must be able to enable or disable telemetry output through configuration.

**Acceptance Scenarios**:
1. Given the telemetry configuration schema, When read by Kit Config, Then it includes an enabled/disabled flag
2. Given telemetry is disabled, When the application runs, Then no telemetry is emitted

### User Story 2 - Configure Sampling Policy
A KitLogger administrator must be able to configure sampling policies to control telemetry volume.

**Acceptance Scenarios**:
1. Given the sampling configuration schema, When read by Kit Config, Then it includes sampling rate and policy type
2. Given a sampling rate of 0.1, When telemetry is generated, Then approximately 10% of spans are sampled

### User Story 3 - Select and Configure Exporters
A KitLogger administrator must be able to select which exporters are active and configure their behavior.

**Acceptance Scenarios**:
1. Given the exporter configuration schema, When read by Kit Config, Then it includes exporter type and endpoint settings
2. Given exporter configuration with missing required field, When validated, Then the configuration is rejected with a clear validation error

### User Story 4 - Configure Resource Attributes
A KitLogger administrator must be able to configure resource attributes that identify the telemetry source.

**Acceptance Scenarios**:
1. Given the resource configuration schema, When read by Kit Config, Then it includes service.name, service.version, and deployment.environment defaults
2. Given telemetry is emitted, When no resource attributes are configured, Then sensible defaults are applied

## Requirements

### Functional Requirements

- **FR-001**: System MUST define a TelemetryConfig entity with enabled/disabled and top-level telemetry settings
- **FR-002**: System MUST define a SamplingPolicy entity with sampling rate and policy type from a closed canonical set: AlwaysOn, AlwaysOff, TraceIdRatio, ParentBased, ConsistentProbability. An Extension variant accepting a provider-defined identifier enables future extensibility; extension opt-in requires a SchemaVersion bump.
- **FR-003**: System MUST define a generic ExporterConfig entity with fields: exporter_type (closed set), endpoint, compression, headers, timeout. Exporter-specific settings via a structured settings map with per-type validation. New exporter types are added to the closed set; addition requires a SchemaVersion bump.
- **FR-004**: System MUST define a ResourceConfig entity with resource attribute defaults and overrides
- **FR-005**: System MUST define a VerbosityPolicy entity with a fixed canonical level set shared across all signals (traces, metrics, logs): OFF, ERROR, WARN, INFO, DEBUG, TRACE. Per-signal threshold may be set independently to any level in the set.
- **FR-006**: System MUST define inline validation constraints on all configurable fields using declarative rule metadata (e.g., range, required, uri, pattern). Constraints are embedded in field definitions; there is no standalone ValidationRule entity with its own identity or lifecycle.
- **FR-007**: System MUST define a SchemaVersion entity that versions the entire telemetry configuration: semantic model entities, their defaults, and all telemetry settings. Kit Config configuration pipeline version is independent and owned by Kit Config.
- **FR-008**: System MUST define configuration defaults for all required settings
- **FR-009**: System MUST NOT define configuration loading, sources, parsing, or infrastructure

### Key Entities

- **TelemetryConfig**: Top-level configuration entity controlling telemetry enabled/disabled and global settings
- **SamplingPolicy**: Configuration entity defining sampling rate and policy type. Canonical policy types: AlwaysOn, AlwaysOff, TraceIdRatio, ParentBased, ConsistentProbability. Extension variant for provider-defined identifiers. Extension opt-in requires SchemaVersion bump.
- **ExporterConfig**: Generic configuration entity with fields: exporter_type (closed set), endpoint, compression, headers, timeout. Exporter-specific settings via structured settings map with per-type validation. New exporter types require SchemaVersion bump.
- **ResourceConfig**: Configuration entity for resource attribute defaults and overrides (service.name, service.version, deployment.environment)
- **VerbosityPolicy**: Configuration entity defining a fixed canonical level set shared across traces, metrics, and logs: OFF, ERROR, WARN, INFO, DEBUG, TRACE. Per-signal threshold independently configurable. Set is fixed and not extensible.
- **ValidationRule**: Cross-cutting validation constraints embedded as declarative metadata on each entity's field definitions (e.g., range, required, uri, pattern). Not a standalone entity with identity or lifecycle. AS-04 defines what to validate; Kit Config owns how to execute validation.
- **SchemaVersion**: Version identifier for the entire telemetry configuration (semantic model entities, defaults, and telemetry settings). Kit Config pipeline version is independent.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All six semantic entities (TelemetryConfig, SamplingPolicy, ExporterConfig, ResourceConfig, VerbosityPolicy, SchemaVersion) plus cross-cutting inline validation constraints are defined and documented
- **SC-002**: TelemetryConfig includes enabled/disabled toggle with documented defaults
- **SC-003**: SamplingPolicy defines rate and closed canonical policy type set (AlwaysOn, AlwaysOff, TraceIdRatio, ParentBased, ConsistentProbability) with validation constraints and Extension variant
- **SC-004**: ExporterConfig defines generic fields (exporter_type, endpoint, compression, headers, timeout) with per-type validation for exporter-specific settings; new exporter types require SchemaVersion bump
- **SC-005**: ResourceConfig defines service.name, service.version, deployment.environment with defaults
- **SC-006**: VerbosityPolicy defines fixed canonical level set (OFF, ERROR, WARN, INFO, DEBUG, TRACE) shared across traces, metrics, and logs; per-signal threshold independently configurable; set is fixed and not extensible
- **SC-007**: All configuration fields carry inline declarative validation constraints (range, required, uri, pattern); validation contract is defined by AS-04, validation execution is owned by Kit Config
- **SC-008**: SchemaVersion is present, versions the entire telemetry configuration (semantic model + defaults + settings), and enables future schema evolution independently from Kit Config pipeline version
- **SC-009**: No configuration infrastructure (loading, parsing, sources) is defined in this specification

## Ownership Boundary

This specification owns:

- TelemetryConfig entity with enable/disable and global settings
- SamplingPolicy entity with rate and closed canonical policy type set (AlwaysOn, AlwaysOff, TraceIdRatio, ParentBased, ConsistentProbability) plus Extension variant
- ExporterConfig entity with generic fields (exporter_type, endpoint, compression, headers, timeout) and per-type validated settings map; exporter type is a closed set; new types require SchemaVersion bump
- ResourceConfig entity with resource attribute defaults
- VerbosityPolicy entity with fixed canonical level set (OFF, ERROR, WARN, INFO, DEBUG, TRACE) shared across all signals; per-signal threshold independently configurable
- Inline validation constraints (range, required, uri, pattern) on all configuration field definitions
- SchemaVersion entity versioning the entire telemetry configuration (semantic model + defaults + settings); Kit Config pipeline version is independent

This specification does not own:

- Configuration loading, providers, or sources
- Environment variable handling or configuration file formats
- Configuration parsing, precedence, or merging
- Runtime reload infrastructure or secrets management
- Configuration storage or discovery

## Clarifications

### Session 2026-06-14

- Q: Canonical Configuration Model → A: Explicit semantic entities: TelemetryConfig, SamplingPolicy, ExporterConfig, ResourceConfig, VerbosityPolicy, ValidationRule, SchemaVersion (B)

### Session 2026-06-17

- Q: SamplingPolicy — supported policy types, OTel alignment, and extensibility model → A: Closed set + versioned extensibility (C). Canonical policy types: AlwaysOn, AlwaysOff, TraceIdRatio, ParentBased (delegates to head decision), ConsistentProbability (trace-id consistent, OTel standard). An Extension variant accepts a provider-defined policy identifier. The canonical set is closed and versioned; extension opt-in requires explicit SchemaVersion bump.
- Q: ExporterConfig — generic entity with type discriminator vs per-exporter-type entities → A: Generic entity with typed settings (B). Single ExporterConfig entity with fields: exporter_type (closed set), endpoint, compression, headers, timeout. Exporter-specific settings via a structured settings map with per-type validation. New exporters add entries to the closed type set (SchemaVersion bump required).
- Q: ValidationRule — canonical entity, derived artifact, or validation contract → A: Validation contract embedded in field definitions (B). Constraints are inline field metadata (e.g., `sampling_rate: f64 [range=0.0..1.0]`), not a standalone entity with identity or lifecycle. AS-04 expresses what to validate; Kit Config owns how to execute.
- Q: SchemaVersion — what exactly does it version → A: SchemaVersion versions the entire telemetry configuration (B): semantic model, defaults, and all telemetry settings. Kit Config's configuration pipeline version is independent and owned by Kit Config.
- Q: VerbosityPolicy — canonical levels, cross-signal sharing, and extensibility → A: Simple fixed set shared across all signals (C). Canonical levels: OFF, ERROR, WARN, INFO, DEBUG, TRACE. Same level set applies to traces, metrics, and logs. Set is fixed and not extensible.

## Assumptions

- Kit Config provides configuration loading, environment integration, parsing, and lifecycle management
- Adapter contracts (AS-03) define which exporters are available for configuration
- Parent capability defines the canonical telemetry model entities that configuration controls
- ConfigurationSchema is an implementation artifact derived from the semantic entities; AS-04 owns the entities, not the schema format
