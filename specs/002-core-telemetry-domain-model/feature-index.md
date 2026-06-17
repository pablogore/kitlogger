# Core Telemetry Domain Model - Feature Index

## Atomic Specification Candidates

| Key | Name | Responsibility | Dependencies | Ownership Boundary | Specification ID |
|-----|------|----------------|--------------|--------------------|------------------|
| AS-01 | Context Propagation and Correlation | Define context propagation (Trace Context, Correlation ID, Baggage) and cross-signal correlation across Traces, Metrics, and Logs | None (parent capability) | Context, Correlation, Propagation Metadata; Domain model entities (Resource, Span, Metric, LogRecord) | `002-core-telemetry-domain-model-as-01-context-propagation-and-correlation` |
| — | Shared Canonical Types Layer | Own cross-capability canonical pipeline types: PayloadEnvelope, TelemetryBatch, TransportMetadata, BackpressureSignal | AS-01 | Canonical data-in-transit types | `telemetry-types` |
| AS-02 | Transport-Agnostic Telemetry Flow | Define abstract transport contracts for telemetry data flow across HTTP, gRPC, CLI, Background Jobs, and future transports | AS-01, telemetry-types | Transport abstraction, Protocol contracts, Execution boundary mapping, TransportError, DeliveryMode | `002-core-telemetry-domain-model-as-02-transport-agnostic-telemetry-flow` |
| AS-03 | Telemetry Adapter Contracts | Define OpenTelemetry adapter contract, adapter registry, and adapter lifecycle | telemetry-types | OpenTelemetry adapter contract, Adapter registry, Adapter lifecycle, HealthReport, mapping contracts | `002-core-telemetry-domain-model-as-03-telemetry-adapter-contracts` |
| AS-04 | Telemetry Configuration Semantics | Define telemetry configuration schema, defaults, constraints, and validation rules | AS-03, Kit Config (external) | Configuration semantics, Validation rules, Defaults | `002-core-telemetry-domain-model-as-04-telemetry-configuration-semantics` |
