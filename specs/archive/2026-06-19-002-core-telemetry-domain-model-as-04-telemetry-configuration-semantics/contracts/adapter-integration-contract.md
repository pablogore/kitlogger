# Adapter Integration Contract: AS-04 ↔ AS-03

## Purpose

Defines the contract between AS-04 (Telemetry Configuration Semantics) and AS-03 (Telemetry Adapter Contracts). AS-04 references AS-03 adapter registry keys to identify exporters in ExporterConfig. AS-03 does not depend on AS-04; the dependency is unidirectional (AS-04 → AS-03).

## Contract Points

### 1. Exporter Type Identifier

AS-04's `ExporterConfig.exporter_type` field uses string identifiers that MUST match the adapter registry keys defined by AS-03's `AdapterRegistry`.

- AS-03 defines which adapter identifiers are valid
- AS-04 consumes these identifiers as a closed set
- Adding a new exporter type to AS-03 requires a SchemaVersion bump in AS-04

### 2. No Trait or Type Dependency

AS-04 MUST NOT depend on AS-03's adapter traits (CommonAdapterBase, LifecycleAdapter, TelemetryDelivery, ProviderAdapter, ExporterAdapter) or AS-03's lifecycle types (AdapterLifecycle, AdapterHealth, HealthReport, AdapterError).

AS-04 depends on AS-03 for:
- Exporter type string identifiers (adapter registry keys)
- Validation that configured exporter types match registered adapters

### 3. Validation Flow

```text
Kit Config loads configuration
  → AS-04 schema validates structural constraints
  → AS-04 exporter_type validated against AS-03 adapter registry keys
  → Kit Config delivers validated configuration to consumers
  → Consumer instantiates adapters from AS-03 registry using configured exporter_type
```

### 4. SchemaVersion Coordination

When AS-03 adds new adapter types:
- AS-04 SchemaVersion MUST be bumped (MINOR)
- New exporter_type values are added to the closed set
- Existing configurations remain valid without changes
