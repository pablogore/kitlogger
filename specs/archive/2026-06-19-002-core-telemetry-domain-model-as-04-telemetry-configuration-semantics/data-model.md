# Data Model: Telemetry Configuration Semantics

## Entities

### 1. TelemetryConfig

Top-level configuration entity. Contains only composition references plus the enabled/disabled flag.

| Field | Type | Required | Default | Constraints |
|-------|------|----------|---------|-------------|
| `enabled` | boolean | no | `true` | — |
| `sampling` | SamplingPolicy | no | default sampling policy | — |
| `exporters` | Vec\<ExporterConfig\> | no | default exporter (console) | at least one if enabled |
| `resources` | ResourceConfig | no | default resource config | — |
| `verbosity` | VerbosityPolicy | no | default verbosity (INFO all signals) | — |
| `schema_version` | SchemaVersion | yes | — | must be supported version |

**Relationships**:
- TelemetryConfig **composes** → SamplingPolicy (0..1)
- TelemetryConfig **composes** → ExporterConfig (0..*)
- TelemetryConfig **composes** → ResourceConfig (0..1)
- TelemetryConfig **composes** → VerbosityPolicy (0..1)
- TelemetryConfig **references** → SchemaVersion (exactly 1)

---

### 2. SamplingPolicy

Controls telemetry volume via sampling rate and policy type.

| Field | Type | Required | Default | Constraints |
|-------|------|----------|---------|-------------|
| `policy_type` | SamplingPolicyType | yes | `AlwaysOn` | closed enum set |
| `sampling_rate` | f64 | no | `1.0` | range [0.0, 1.0]; required if type is TraceIdRatio or ConsistentProbability |

**SamplingPolicyType enum**:
- `AlwaysOn` — sample all telemetry
- `AlwaysOff` — sample no telemetry
- `TraceIdRatio` — probabilistic sampling based on trace ID hash
- `ParentBased` — delegate sampling decision to parent span
- `ConsistentProbability` — trace-id consistent probability sampling (OTel standard)
- `Extension(String)` — provider-defined policy identifier; requires SchemaVersion bump

**Relationships**:
- SamplingPolicy is **owned by** → TelemetryConfig (0..1 composition)

---

### 3. ExporterConfig

Generic per-exporter configuration entity.

| Field | Type | Required | Default | Constraints |
|-------|------|----------|---------|-------------|
| `exporter_type` | ExporterType | yes | — | closed set; validated against AS-03 adapter registry identifiers |
| `endpoint` | string | no | — | uri format if present |
| `compression` | CompressionType | no | `None` | enum: None, Gzip |
| `headers` | Map\<String, String\> | no | empty map | — |
| `timeout_secs` | u64 | no | `30` | range [1, 300] |
| `settings` | Map\<String, String\> | no | empty map | per-type validated; keys and values defined by exporter implementation |

**ExporterType**: String identifier matching AS-03 adapter registry keys. Closed set defined by AS-03; addition requires SchemaVersion bump.

**CompressionType enum**: None, Gzip (extensible via SchemaVersion bump).

**Relationships**:
- ExporterConfig is **owned by** → TelemetryConfig (0..* composition)
- ExporterConfig **references** → AS-03 adapter registry (exporter_type must match registered adapter)

---

### 4. ResourceConfig

Identifies the telemetry source entity.

| Field | Type | Required | Default | Constraints |
|-------|------|----------|---------|-------------|
| `service_name` | string | yes | — | non-empty |
| `service_version` | string | no | `"unknown"` | — |
| `deployment_environment` | string | no | `"development"` | — |
| `attributes` | Map\<String, String\> | no | empty map | arbitrary additional resource attributes |

**Relationships**:
- ResourceConfig is **owned by** → TelemetryConfig (0..1 composition)

---

### 5. VerbosityPolicy

Controls per-signal verbosity thresholds using a fixed shared level set.

| Field | Type | Required | Default | Constraints |
|-------|------|----------|---------|-------------|
| `trace_level` | VerbosityLevel | no | `INFO` | from fixed enum set |
| `metric_level` | VerbosityLevel | no | `INFO` | from fixed enum set |
| `log_level` | VerbosityLevel | no | `INFO` | from fixed enum set |

**VerbosityLevel enum** (fixed, not extensible):
- `Off` — signal suppressed
- `Error` — errors only
- `Warn` — warnings and above
- `Info` — informational and above
- `Debug` — debug and above
- `Trace` — trace and above (most verbose)

Semantic ordering: Off < Error < Warn < Info < Debug < Trace.

**Relationships**:
- VerbosityPolicy is **owned by** → TelemetryConfig (0..1 composition)

---

### 6. SchemaVersion

Identifies the version of the telemetry configuration schema.

| Field | Type | Required | Default | Constraints |
|-------|------|----------|---------|-------------|
| `version` | string | yes | — | semver format (MAJOR.MINOR.PATCH) |
| `description` | string | no | — | human-readable change description |

**Versioning rules**:
- Versions the entire telemetry configuration (semantic model + defaults + settings)
- MAJOR bump: breaking field changes (removal, type change, constraint tightening)
- MINOR bump: additive changes (new fields, new enum variants, relaxed constraints)
- PATCH bump: documentation, non-semantic changes
- Kit Config pipeline version is independent — no coupling between schema versions

**Relationships**:
- SchemaVersion is **referenced by** → TelemetryConfig (exactly 1)

---

## Entity Relationship Diagram

```text
TelemetryConfig (1)
  ├── SchemaVersion (1)       [reference — required]
  ├── SamplingPolicy (0..1)   [composition — optional]
  ├── ExporterConfig (0..*)   [composition — optional, multiple]
  ├── ResourceConfig (0..1)   [composition — optional]
  └── VerbosityPolicy (0..1)  [composition — optional]
```

All relationships are ownership-based (TelemetryConfig is the root aggregate). No entity has an independent lifecycle outside of TelemetryConfig.

---

## Cross-Cutting Validation Contracts

Validation constraints are embedded as inline metadata on each field definition (see per-field Constraints column above). AS-04 defines what to validate; Kit Config owns how to execute validation.

Constraint types used:
- `required` — field must have a non-null/non-empty value
- `range [min, max]` — numeric value must be within inclusive bounds
- `uri` — string must conform to URI format
- `enum { variants }` — value must be one of the listed variants (closed set)
- `non-empty` — string must not be empty

---

## Default Configuration

When TelemetryConfig is not provided, the following defaults apply:

```text
TelemetryConfig {
  enabled: true,
  sampling: SamplingPolicy { policy_type: AlwaysOn, sampling_rate: 1.0 },
  exporters: [ExporterConfig { exporter_type: "console", ...defaults }],
  resources: ResourceConfig { service_name: <required from deployer>, service_version: "unknown", deployment_environment: "development" },
  verbosity: VerbosityPolicy { trace_level: INFO, metric_level: INFO, log_level: INFO },
  schema_version: SchemaVersion { version: "1.0.0" }
}
```

Note: `service_name` has no default — the deployer MUST provide it explicitly.
