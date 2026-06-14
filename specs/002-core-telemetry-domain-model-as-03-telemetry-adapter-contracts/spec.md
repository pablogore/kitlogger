# Feature Specification: Telemetry Adapter Contracts

**SPEC_ID**: `002-core-telemetry-domain-model-as-03-telemetry-adapter-contracts`

**PARENT_SPEC_ID**: `002-core-telemetry-domain-model`

**PARENT_SPEC_NAME**: `core-telemetry-domain-model`

**CAPABILITY_ID**: `002`

**CAPABILITY_NAME**: `core-telemetry-domain-model`

**EXPAND_ID**: AS-03

**Created**: 2026-06-14

**Status**: Draft

## Scope

Define the canonical adapter contract for telemetry provider abstraction, including ProviderAdapter and ExporterAdapter traits sharing a common base, adapter registry with frozen-after-init semantics, and adapter lifecycle management (registered → initialized → started → stopping → stopped → shutdown). This specification owns the abstract contract decoupling the telemetry domain model from providers and exporters. Only OpenTelemetry-compatible contracts are owned here; concrete implementations belong to later specs.

## Non-Scope

- Domain model entities (Trace, Span, Metric, Log Record, Resource)
- Transport bindings or execution boundary infrastructure
- Configuration infrastructure or management
- Console Export or other exporter implementations
- Custom exporter implementations
- Provider implementations
- Concrete OpenTelemetry adapter implementation (contracts only)
- Concrete mapping implementations from OTel ↔ Canonical model (AS-03 owns mapping contracts only)

## Responsibility

Define the OpenTelemetry adapter contract, including unified adapter interface, adapter registry, and adapter lifecycle.

## Dependencies

None (depends on parent capability canonical model).

## User Scenarios & Testing

### User Story 1 - OpenTelemetry Adapter Contract
Telemetry data must be exportable through OpenTelemetry-compatible adapters without tight coupling to the OpenTelemetry SDK.

**Acceptance Scenarios**:
1. Given a domain model span, When passed through the OpenTelemetry adapter, Then it maps to the correct OTel span structure
2. Given an OpenTelemetry metric, When received through the adapter, Then it maps to the correct domain metric entity

### User Story 2 - Extensible Adapter Contract
Third-party adapters must be pluggable through a standard contract without modifying core telemetry code.

**Acceptance Scenarios**:
1. Given a custom adapter implementing the contract, When registered, Then it receives telemetry data without core code changes
2. Given multiple registered adapters, When telemetry is emitted, Then all registered adapters receive the data

### User Story 3 - Adapter Lifecycle Management
Adapters must support lifecycle management including initialize, start, stop, flush, and shutdown.

**Acceptance Scenarios**:
1. Given a registered adapter, When the system starts, Then the adapter transitions through initialize to start
2. Given a running adapter, When the system shuts down, Then the adapter transitions through stop, flush, and shutdown

## Requirements

### Functional Requirements

- **FR-001**: System MUST define ProviderAdapter and ExporterAdapter traits sharing a common base trait as the canonical adapter abstraction
- **FR-002**: System MUST define OpenTelemetry-compatible contracts only; concrete OpenTelemetry adapter implementation is a separate spec
- **FR-003**: System MUST support adapter registration through a standard registry interface; registry MUST be mutable until initialization completes, then frozen
- **FR-004**: System MUST define adapter lifecycle using canonical states: registered → initialized → started → stopping → stopped → shutdown
- **FR-005**: System MUST define flush() semantics guaranteeing all buffered telemetry is handed to the transport layer
- **FR-006**: System MUST define a strongly typed AdapterId for adapter registration identity
- **FR-007**: System MUST own entity-specific mapping contracts: TraceMappingContract, SpanMappingContract, MetricMappingContract, LogRecordMappingContract, ResourceMappingContract
- **FR-008**: System MUST define AdapterResult and AdapterError as the canonical failure model
- **FR-009**: System MUST deliver telemetry to all matching registered adapters (multiplexing)
- **FR-010**: Adapter selection (which adapters are active) is owned by configuration semantics (AS-04), not the registry

### Key Entities

- **ProviderAdapter**: Trait for provider-side telemetry operations, sharing a common base with ExporterAdapter
- **ExporterAdapter**: Trait for exporter-side telemetry operations, sharing a common base with ProviderAdapter
- **CommonAdapterBase**: Shared base trait inherited by both ProviderAdapter and ExporterAdapter
- **AdapterRegistry**: Manages adapter registration, lookup, and lifecycle orchestration; mutable until initialization, then frozen
- **AdapterLifecycle**: State machine with canonical states: registered → initialized → started → stopping → stopped → shutdown
- **AdapterId**: Strongly typed value object for adapter registration identity
- **AdapterResult / AdapterError**: Canonical result type for adapter operations (success or typed error)
- **TraceMappingContract**: Entity-specific mapping contract for Trace ↔ OTel trace structure
- **SpanMappingContract**: Entity-specific mapping contract for Span ↔ OTel span structure
- **MetricMappingContract**: Entity-specific mapping contract for Metric ↔ OTel metric structure
- **LogRecordMappingContract**: Entity-specific mapping contract for LogRecord ↔ OTel log structure
- **ResourceMappingContract**: Entity-specific mapping contract for Resource ↔ OTel resource structure

## Success Criteria

### Measurable Outcomes

- **SC-001**: ProviderAdapter and ExporterAdapter with a common base trait are defined and documented
- **SC-002**: Registry accepts adapters during initialization, then freezes; post-init registrations are rejected
- **SC-003**: All lifecycle states (registered → initialized → started → stopping → stopped → shutdown) are enumerable and transitions are testable
- **SC-004**: flush() hands buffered telemetry to the transport layer; best-effort-only is not accepted
- **SC-005**: Adapters are identified by strongly typed AdapterId; no string-typed lookups exist
- **SC-006**: Multiple registered adapters all receive the same telemetry data (multiplexing verified)
- **SC-007**: AdapterResult/AdapterError covers adapter operation failures; boolean-only is not accepted
- **SC-008**: Only OpenTelemetry-compatible contracts are defined in AS-03; no concrete implementation is included
- **SC-009**: Five entity-specific mapping contracts (Trace, Span, Metric, LogRecord, Resource) are defined and documented separately

## Ownership Boundary

This specification owns:

- ProviderAdapter and ExporterAdapter traits with common base
- AdapterRegistry with frozen-after-init semantics
- AdapterLifecycle state machine (registered → initialized → started → stopping → stopped → shutdown)
- AdapterId strongly typed identity
- AdapterResult and AdapterError canonical failure model
- Entity-specific mapping contracts: TraceMappingContract, SpanMappingContract, MetricMappingContract, LogRecordMappingContract, ResourceMappingContract
- flush() semantics guarantee (handoff to transport layer)
- Adapter multiplexing (all matching adapters receive telemetry)

This specification does not own:

- Domain model entities (Trace, Span, Metric, Log Record, Resource)
- Transport bindings or execution boundary infrastructure
- Configuration infrastructure or management
- Console Export or other exporter implementations
- Custom exporter implementations
- Provider implementations
- Concrete OpenTelemetry adapter implementation
- Concrete mapping implementation (contracts only)
- Adapter selection — owned by configuration semantics (AS-04)

## Clarifications

### Session 2026-06-14

- Q: Adapter Contract Shape → A: ProviderAdapter and ExporterAdapter traits sharing a common base trait (B)
- Q: Registry Mutability → A: Mutable until initialization completes, then frozen (B)
- Q: Lifecycle State Machine → A: registered → initialized → started → stopping → stopped → shutdown (C)
- Q: Flush Semantics → A: flush() guarantees all buffered telemetry is handed to the transport layer (B)
- Q: Registration Identity → A: Strongly typed AdapterId (B)
- Q: Mapping Ownership → A: AS-03 owns all OpenTelemetry ↔ Canonical Model mapping contracts (A)
- Q: Adapter Failure Model → A: AdapterResult and AdapterError model (B)
- Q: Adapter Multiplexing → A: All matching adapters receive telemetry (C)
- Q: OpenTelemetry Dependency Boundary → A: Only OpenTelemetry-compatible contracts; implementation is separate (A)
- Q: Adapter Selection → A: Configuration semantics (AS-04) chooses active adapters (B)
- Q: Mapping Contract Granularity → A: Entity-specific mapping contracts per canonical entity (Trace, Span, Metric, LogRecord, Resource) (B)

## Assumptions

- Parent capability defines the canonical domain model entities
- Transport layer (AS-02) handles delivery; adapters handle provider mapping
- Configuration semantics (AS-04) define adapter selection and behavior; registry does not select adapters
- Concrete adapter implementations (OpenTelemetry, Console Export) are separate specs consuming AS-03 contracts
