# Configuration Schema Contract: AS-04 → Kit Config

## Purpose

Defines the contract between AS-04 (Telemetry Configuration Semantics) and Kit Config (configuration infrastructure). AS-04 provides the canonical entity definitions, field types, defaults, and inline validation constraints. Kit Config consumes these definitions to load, validate, and deliver configuration to telemetry consumers.

## Contract Boundary

| Responsibility | Owner |
|----------------|-------|
| Define configuration entities and field types | AS-04 |
| Define field defaults | AS-04 |
| Define inline validation constraints (range, required, uri, enum, non-empty) | AS-04 |
| Define SchemaVersion semantics and versioning rules | AS-04 |
| Configuration loading (sources, parsing, environment integration) | Kit Config |
| Configuration validation execution | Kit Config |
| Configuration delivery to consumers | Kit Config |
| Configuration lifecycle and reload | Kit Config |

## Configuration Shape

Kit Config MUST provide the following top-level telemetry configuration structure:

```text
telemetry: {                   # TelemetryConfig
  enabled: bool,               # default: true
  sampling: {                  # SamplingPolicy (optional)
    policy_type: string,       # required: AlwaysOn|AlwaysOff|TraceIdRatio|ParentBased|ConsistentProbability|Extension(id)
    sampling_rate: f64         # optional, default: 1.0, range: [0.0, 1.0]
  },
  exporters: [                 # ExporterConfig[] (optional, default: console)
    {
      exporter_type: string,   # required, must match AS-03 registry
      endpoint: string,        # optional, uri format
      compression: string,     # optional, default: "none", enum: ["none", "gzip"]
      headers: {string: string}, # optional, default: {}
      timeout_secs: u64,       # optional, default: 30, range: [1, 300]
      settings: {string: string} # optional, per-type validated
    }
  ],
  resources: {                 # ResourceConfig (optional)
    service_name: string,      # required, non-empty, NO default
    service_version: string,   # optional, default: "unknown"
    deployment_environment: string, # optional, default: "development"
    attributes: {string: string} # optional, arbitrary key-value pairs
  },
  verbosity: {                 # VerbosityPolicy (optional)
    trace_level: string,       # optional, default: "info"
    metric_level: string,      # optional, default: "info"
    log_level: string          # optional, default: "info"
  },
  schema_version: {            # SchemaVersion (required)
    version: string,           # required, semver MAJOR.MINOR.PATCH
    description: string        # optional
  }
}
```

## Validation Rules (Inline Constraints)

Kit Config MUST enforce the following validation rules:

| Path | Constraint | Violation Behavior |
|------|-----------|--------------------|
| `telemetry.enabled` | must be boolean | reject configuration |
| `telemetry.sampling.policy_type` | must be one of the enum variants | reject configuration |
| `telemetry.sampling.sampling_rate` | must be f64 in [0.0, 1.0] | reject configuration |
| `telemetry.exporters[].exporter_type` | must match known adapter type | reject configuration |
| `telemetry.exporters[].endpoint` | if present, must be valid URI | reject configuration |
| `telemetry.exporters[].timeout_secs` | if present, must be u64 in [1, 300] | reject configuration |
| `telemetry.resources.service_name` | must be non-empty string | reject configuration |
| `telemetry.verbosity.*_level` | must be one of: off, error, warn, info, debug, trace | reject configuration |
| `telemetry.schema_version.version` | must be valid semver | reject configuration |

## Default Application

Kit Config MUST apply defaults AFTER loading and BEFORE delivering to consumers. Defaults are defined in `data-model.md`.

## Schema Versioning

- Kit Config MUST accept configurations with any supported SchemaVersion
- Kit Config MUST reject configurations with unsupported SchemaVersion
- SchemaVersion is independent from Kit Config's pipeline version
- SchemaVersion bump rules are owned by AS-04 (see data-model.md)
