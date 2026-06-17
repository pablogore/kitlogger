# Quickstart: Telemetry Configuration Semantics

## Prerequisites

- Rust toolchain (per `tech-stack.yaml`)
- Kit Config implementation (external dependency)
- AS-03 adapter contracts crate (exporter type identifiers)

## Validation Scenarios

### Scenario 1: Default Configuration

**Goal**: Verify that all defaults are applied when no telemetry configuration is provided.

**Setup**: Kit Config receives no telemetry configuration section.

**Expected outcome**: Kit Config applies defaults:
- `telemetry.enabled` = `true`
- `telemetry.sampling.policy_type` = `AlwaysOn`
- `telemetry.sampling.sampling_rate` = `1.0`
- `telemetry.exporters` = `[{exporter_type: "console"}]`
- `telemetry.resources.service_version` = `"unknown"`
- `telemetry.resources.deployment_environment` = `"development"`
- `telemetry.verbosity.{trace,metric,log}_level` = `"info"`
- `telemetry.schema_version.version` = `"1.0.0"`

**Validation**: See `data-model.md` Default Configuration section for full default values.

---

### Scenario 2: Enable/Disable Telemetry

**Goal**: Verify that telemetry can be enabled or disabled through configuration.

**Setup**: Kit Config loads configuration with `telemetry.enabled: false`.

**Expected outcome**: Configuration is valid; telemetry subsystem does not emit any telemetry data.

**Validation**: `telemetry.enabled` field type: boolean, default: true (see data-model.md).

---

### Scenario 3: Configure Sampling Policy

**Goal**: Verify all supported sampling policy types are valid.

**Setup**: Kit Config loads configuration with various sampling policies:
1. `telemetry.sampling.policy_type: "AlwaysOn"`
2. `telemetry.sampling.policy_type: "TraceIdRatio"`, `telemetry.sampling.sampling_rate: 0.1`
3. `telemetry.sampling.policy_type: "Extension"`, `telemetry.sampling.extension_id: "my-custom-policy"`

**Expected outcome**: All three configurations are valid.

**Expected rejection**: `telemetry.sampling.sampling_rate: 1.5` — violates range [0.0, 1.0].

**Validation**: See SamplingPolicy in data-model.md and validation rules in contracts/config-schema-contract.md.

---

### Scenario 4: Configure Exporters

**Goal**: Verify exporter configuration validation.

**Setup**: Kit Config loads configuration with:
1. Valid: `telemetry.exporters: [{exporter_type: "otlp", endpoint: "http://localhost:4317", compression: "gzip"}]`
2. Invalid: `telemetry.exporters: [{exporter_type: "otlp", timeout_secs: 600}]` — exceeds range [1, 300]
3. Invalid: `telemetry.exporters: [{exporter_type: "otlp", endpoint: "not-a-uri"}]` — fails URI validation
4. Invalid: `telemetry.exporters: [{exporter_type: "unknown-exporter"}]` — unknown exporter type

**Expected outcome**: #1 valid; #2, #3, #4 rejected with clear validation errors.

**Validation**: See ExporterConfig in data-model.md and validation rules in contracts/config-schema-contract.md. For exporter type validation, see contracts/adapter-integration-contract.md.

---

### Scenario 5: Resource Attributes

**Goal**: Verify resource configuration validation.

**Setup**: Kit Config loads configuration with:
1. Valid: `telemetry.resources: {service_name: "my-service"}`
2. Invalid: `telemetry.resources: {}` — missing required service_name
3. Valid: `telemetry.resources: {service_name: "my-service", service_version: "2.0.0", deployment_environment: "production", attributes: {custom_key: "custom_value"}}`

**Expected outcome**: #1 valid with defaults applied for missing fields; #2 rejected; #3 valid with all explicit values.

**Validation**: See ResourceConfig in data-model.md.

---

### Scenario 6: Verbosity Configuration

**Goal**: Verify verbosity validation.

**Setup**: Kit Config loads configuration with:
1. Valid: `telemetry.verbosity: {trace_level: "debug", metric_level: "info", log_level: "warn"}`
2. Invalid: `telemetry.verbosity: {trace_level: "verbose"}` — unknown level

**Expected outcome**: #1 valid; #2 rejected. Level set is fixed: off, error, warn, info, debug, trace.

**Validation**: See VerbosityPolicy in data-model.md.

---

### Scenario 7: SchemaVersion Validation

**Goal**: Verify schema version validation.

**Setup**: Kit Config loads configuration with:
1. Valid: `telemetry.schema_version: {version: "1.0.0"}`
2. Invalid: `telemetry.schema_version: {version: "not-a-version"}` — not semver
3. Missing: no `telemetry.schema_version` — schema_version is required

**Expected outcome**: #1 valid; #2 rejected; #3 rejected.

**Validation**: See SchemaVersion in data-model.md.
