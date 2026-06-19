# Research: Telemetry Configuration Semantics

## 1. Configuration Entity Model

**Decision**: Six semantic entities (TelemetryConfig, SamplingPolicy, ExporterConfig, ResourceConfig, VerbosityPolicy, SchemaVersion) plus cross-cutting inline validation constraints.
**Rationale**: Each entity maps to a distinct concern in telemetry configuration. ValidationRule was reclassified from entity to inline constraints to avoid entity bloat and maintain clear ownership boundaries with Kit Config.
**Alternatives considered**: Single monolithic config (rejected — violates single-responsibility); ValidationRule as standalone entity (rejected — no independent identity/lifecycle).

## 2. SamplingPolicy Type Set

**Decision**: Closed canonical set with an Extension variant. Types: AlwaysOn, AlwaysOff, TraceIdRatio, ParentBased, ConsistentProbability.
**Rationale**: OTel alignment ensures semantic compatibility. Extension variant allows provider-defined policies without expanding the canonical set.
**Alternatives considered**: Minimal set (AlwaysOn, AlwaysOff, TraceIdRatio) — lacks ParentBased and ConsistentProbability which are standard OTel types; Fully extensible (string-based) — no validation at config time.

## 3. ExporterConfig Modeling

**Decision**: Generic entity with closed type discriminator + typed settings map.
**Rationale**: Avoids entity explosion for each exporter while retaining type safety via per-type validation. New exporter types are added to the closed set (SchemaVersion bump).
**Alternatives considered**: Per-exporter-type entities — excessive entity count; Fully opaque — no compile-time validation.

## 4. VerbosityPolicy Level Set

**Decision**: Fixed shared level set: OFF, ERROR, WARN, INFO, DEBUG, TRACE. Applied uniformly to traces, metrics, and logs. Per-signal threshold independently configurable. Not extensible.
**Rationale**: Simple, predictable, covers all needed verbosity bins. Fixed set ensures configuration portability across signals and implementations.
**Alternatives considered**: OTel severity numbers (1-24) — more granular but over-engineered for config semantics; Signal-specific level sets — unnecessary complexity.

## 5. SchemaVersion Scope

**Decision**: SchemaVersion versions the entire telemetry configuration (semantic model + defaults + settings). Kit Config pipeline version is independent.
**Rationale**: Keeps versioning authority within AS-04's ownership boundary. Independently versioned from Kit Config avoids cross-capability coordination for schema bumps.
**Alternatives considered**: Semantic model only — wouldn't capture default/setting changes; Entire KitLogger config — requires cross-capability coordination.

## 6. ResourceConfig Mandatory Attributes and Defaults

**Decision**:
- Mandatory: `service.name` (no default — must be explicitly set by the deployer)
- Optional with defaults: `service.version` (default: "unknown"), `deployment.environment` (default: "development")
- Arbitrary additional resource attributes may be added as key-value pairs
**Rationale**: service.name is the only OTel-required resource attribute. Other attributes have sensible defaults for development. Arbitrary attributes enable environment-specific metadata.
**Alternatives considered**: All attributes mandatory — too strict for development; No arbitrary attributes — too restrictive.

## 7. TelemetryConfig Global Settings

**Decision**: TelemetryConfig contains only composition (nested entity references) plus the enabled/disabled flag. Global behavior settings (batch interval, queue size, export timeout) are modeled as fields within the relevant nested entities, not at the TelemetryConfig top level.
**Rationale**: Avoids TelemetryConfig becoming a catch-all. Each nested entity owns its behavior settings.
**Alternatives considered**: TelemetryConfig as global settings bag — violates single-responsibility.

## 8. Validation Constraints per Field

**Decision**:
- `sampling_rate`: f64, range [0.0, 1.0], default 1.0
- `exporter_type`: closed set enum, required
- `endpoint`: string, uri format validation
- `compression`: string, one of "none", "gzip" (with future extensibility via SchemaVersion)
- `headers`: map of string to string
- `timeout`: duration (or integer seconds), default 30s
- `verbosity_level`: enum from fixed set (OFF, ERROR, WARN, INFO, DEBUG, TRACE), required
- `service.name`: string, required, non-empty
- `service.version`: string, default "unknown"
- `deployment.environment`: string, default "development"
- `enabled`: boolean, default true
**Rationale**: All constraints are declarative inline metadata — no standalone ValidationRule entity. Kit Config owns constraint execution.
**Alternatives considered**: Programmatic validation in AS-04 — crosses into Kit Config territory.

## 9. Dependency Resolution

**Decision**: AS-04 depends on AS-03 for exporter type identifiers only. No code dependency on AS-04's adapter traits or lifecycle interfaces.
**Rationale**: Configuration needs to reference which adapters exist but does not interact with adapters directly.
**Alternatives considered**: AS-04 independently defines exporter types — would diverge from AS-03 registry.

## 10. Configuration Defaults Strategy

**Decision**: All defaults are defined as part of the semantic entity definition. Kit Config applies defaults before delivering validated configuration to consumers.
**Rationale**: AS-04 owns what the defaults are; Kit Config owns when/how to apply them.
**Alternatives considered**: Defaults in Kit Config — violates ownership boundary.
