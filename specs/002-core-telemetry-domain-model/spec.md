# Core Telemetry Domain Model

## Specification

This specification defines the core domain model for telemetry data within the KitLogger system. It establishes the fundamental concepts, relationships, and constraints that govern how telemetry information is captured, stored, and processed.

## Goals

- Define the essential entities and their relationships in the telemetry domain
- Establish a consistent data model for telemetry information
- Provide a foundation for telemetry-related capabilities and features
- Ensure data integrity and consistency across telemetry operations

## Non-Goals

- Implementation details of specific telemetry collection mechanisms
- Specific storage or processing technologies
- User interface designs for telemetry visualization
- Integration with external systems beyond the core domain model

## Stakeholders

- KitLogger system administrators
- Developers working on telemetry features
- Data analysts and scientists
- System architects

## Canonical Telemetry Model

The telemetry domain follows an OpenTelemetry-compatible model while remaining implementation-independent. The canonical entities are:

- **Resource** - First-class entity representing the entity producing telemetry (service.name, service.version, deployment.environment, host.name, process information)
- **Instrumentation Scope** - Identifies the instrumentation library that produced the telemetry
- **Trace** - A directed acyclic graph of spans representing a distributed operation
- **Span** - A single operation within a trace, with start/end timestamps, status, and relationships
- **Span Event** - A time-stamped event within a span's lifecycle
- **Span Link** - A causal relationship between spans across trace boundaries
- **Metric** - A measured value or aggregation over time
- **Log Record** - A structured log entry with timestamp, severity, and body
- **Context** - Trace Context, Correlation Identifier, Baggage, and Propagation Metadata
- **Correlation** - Cross-signal correlation across Traces, Metrics, and Logs
- **Attributes** - Unified key-value metadata model shared across all telemetry entities

## Architectural Constraints

- **Canonical Model**: OpenTelemetry-compatible, implementation-independent domain model
- **Resource**: First-class telemetry entity, not external metadata
- **Attributes**: Unified attribute model shared across all telemetry entities (Trace, Span, Metric, Log Record)
- **Context Model**: Full context including Trace Context, Correlation Identifier, Baggage, and Propagation Metadata
- **Correlation Scope**: Cross-signal correlation across Traces, Metrics, and Logs
- **Transport Scope**: Extensible transport model supporting HTTP, gRPC, CLI, Background Jobs initially; Kafka, NATS, RabbitMQ, Cron Jobs, Event Systems in future - without domain model changes
- **Adapter Model**: OpenTelemetry adapter contract with adapter registry and lifecycle; Console Export is a separate concern outside AS-03
- **Configuration Ownership**: Telemetry defines configuration semantics; Kit Config owns configuration infrastructure
- **Dependency Graph**: Atomic specifications form a DAG with no circular dependencies
- **Future Instrumentation**: Domain model remains stable while new instrumentation capabilities (HTTP Middleware, gRPC Middleware, CLI, Background Jobs, Kafka, NATS) are added
- **Multi-Tenancy**: Tenant-neutral core with future extension support
- **Instrumentation Scope**: Remains within the canonical domain model, not a separate atomic specification

## Clarifications

### Session 2026-06-13

- Q: Configuration Authority → A: Kit Config is the configuration authority. KIT-002 defines only telemetry configuration semantics, requirements, defaults, constraints, and validation rules. KIT-002 must not define configuration loading, configuration providers, configuration sources, parsing, precedence, reload mechanisms, or secret management. Telemetry consumes validated configuration provided by Kit Config. All future KitLogger capabilities must follow the same pattern and depend on Kit Config for configuration infrastructure.
- Q: Configuration Scope → A: KIT-002 defines configuration schema and configuration semantics only. KIT-002 may define: configuration concepts, configuration requirements, configuration defaults, configuration constraints, configuration validation requirements. KIT-002 must not define: configuration management, configuration loading, configuration providers, configuration sources, configuration parsing, environment variable handling, configuration precedence, runtime reload infrastructure, secret management, configuration storage. These responsibilities belong exclusively to Kit Config. KIT-002 is a configuration consumer and schema owner, not a configuration management capability.
- Q: Configuration Source Responsibility → A: All configuration source responsibilities are delegated to Kit Config. This includes: Environment variables, Configuration files, Remote configuration, Secrets integration, Configuration precedence, Configuration merging, Runtime reload, Configuration validation execution, Configuration discovery, Configuration storage. KIT-002 must consume validated configuration provided by Kit Config and remain agnostic to where configuration originates. Telemetry configuration semantics belong to KIT-002. Configuration infrastructure belongs exclusively to Kit Config. No telemetry-specific configuration loaders, parsers, file formats, environment variable handling, or reload mechanisms may be introduced.
- Q: Telemetry Configuration Domain → A: Optional Telemetry Configuration is limited to observability configuration semantics only. The specification may define: telemetry enabled / disabled, sampling policy, exporter selection, exporter behavior, propagation policy, correlation policy, instrumentation policy, telemetry verbosity, resource attributes, default observability behavior, configuration validation requirements. The specification must not define: configuration loading, configuration providers, configuration sources, environment variable handling, configuration file formats, configuration parsing, configuration precedence, configuration merging, configuration discovery, runtime reload, secrets management. These responsibilities belong exclusively to Kit Config. KIT-002 owns telemetry configuration semantics. Kit Config owns configuration infrastructure. Any future capability requiring configuration must follow the same pattern: the capability defines configuration semantics, while Kit Config provides configuration infrastructure.
- Q: Dependency Relationship → A: Optional Telemetry Configuration formally depends on Kit Config as an external capability. Kit Config: configuration loading, configuration sources, configuration parsing, environment integration, secrets integration, configuration precedence, configuration validation execution, runtime reload, configuration lifecycle management. KIT-002 Optional Telemetry Configuration: telemetry configuration semantics, telemetry configuration requirements, telemetry configuration defaults, telemetry configuration constraints, telemetry configuration validation rules. The telemetry capability consumes validated configuration supplied by Kit Config. No telemetry-specific configuration infrastructure may be introduced. This dependency establishes Kit Config as the single configuration authority for the entire Kit ecosystem. Architectural Constraint: Any future capability requiring configuration must declare an external dependency on Kit Config and must not implement an independent configuration framework.

### Session 2026-06-13 (Resource & Instrumentation Scope)

- Q: Instrumentation Scope Atomic Specification → A: Keep inside Core Telemetry Domain Model (A). Resource and Instrumentation Scope are foundational domain concepts and remain part of the canonical model rather than becoming separate atomic specifications.

### Session 2026-06-13 (AS-03 Atomicity)

- Q: AS-03 Adapter Responsibility Boundary → A: Unified adapter abstraction for both providers and exporters (C)
- Q: AS-03 Registry Ownership → A: Registry is part of adapter contracts (A)
- Q: AS-03 Lifecycle Ownership → A: Lifecycle is part of adapter contracts (A)
- Q: AS-03 Provider vs Exporter Separation → A: Unified adapter contract (A)
- Q: AS-03 OpenTelemetry Ownership → A: Only OpenTelemetry contract definitions (A)
- Q: AS-03 Console Export Ownership → A: Console Export becomes its own specification, removed from AS-03 scope (B)
- Q: AS-03 Future Growth Test → A: Existing contracts remain unchanged when exporters are added (A)
- Q: AS-03 Independent Implementation Test → A: Some can evolve independently (B)

Decision: AS-03 scope narrowed to OpenTelemetry adapter contract, adapter registry, and adapter lifecycle. Console Export is a separate concern outside AS-03 scope.

### Session 2026-06-13 (Architecture Clarifications)

- Q1: Canonical Telemetry Model → A: OpenTelemetry-compatible telemetry model (B). Telemetry is modeled around Resource, Instrumentation Scope, Trace, Span, Span Event, Span Link, Metric, Log Record, Context, and Correlation Identifier while remaining implementation-independent.
- Q2: Resource Ownership → A: Resource is a first-class telemetry entity (B). service.name, service.version, deployment.environment, host.name, process information are modeled as a first-class entity.
- Q3: Attributes Model → A: Unified attribute model (B) shared across Trace, Span, Metric, and Log Record entities, matching OpenTelemetry semantics and future extensibility.
- Q4: Context Model → A: Full context model (B) including Trace Context, Correlation Identifier, Baggage, and Propagation Metadata.
- Q5: Correlation Scope → A: Trace + Metrics + Logs correlation (B). Observability requires correlation across all telemetry signals.
- Q6: Transport Scope → A: Extensible transport model (D). Initial transports: HTTP, gRPC, CLI, Background Jobs. Future transports: Kafka, NATS, RabbitMQ, Cron Jobs, Event Systems - without changing the domain model.
- Q7: Adapter Model → A: OpenTelemetry-first but extensible (B). Adapters support OpenTelemetry, Console Export, Custom Exporters, Future Providers.
- Q8: Configuration Ownership → A: Kit Config owns configuration infrastructure (B). Telemetry owns configuration semantics. Kit Config owns configuration infrastructure.
- Q9: Dependency Graph → A: Yes, atomic specifications form a DAG. 001-001 Core Telemetry Domain Model → 001-002 Context Propagation and Correlation (depends on 001-001) → 001-003 Transport-Agnostic Telemetry Flow (depends on 001-001, 001-002) | 001-004 Telemetry Adapter Contracts (depends on 001-001) | 001-005 Telemetry Configuration Semantics (depends on 001-001, 001-004, external: Kit Config). No circular dependencies.
- Q10: Future Instrumentation Targets → A: Yes. Domain model must remain stable while new instrumentation capabilities (HTTP Middleware, gRPC Middleware, CLI Instrumentation, Background Job Instrumentation, Kafka Instrumentation, NATS Instrumentation) are added.
- Q11: Multi-Tenancy → A: Tenant-neutral core with future extension support (C). Keeps the telemetry domain simple while preserving future enterprise support.