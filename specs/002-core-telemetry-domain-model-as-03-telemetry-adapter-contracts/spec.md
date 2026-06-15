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
- **FR-003**: System MUST support adapter registration through a standard registry interface providing register(), get(), contains(), and list() operations; registry MUST be mutable until initialization completes, then frozen; lookup MUST be thread-safe after freeze; registration MUST be rejected with typed AdapterError if the AdapterId is already registered
- **FR-004**: System MUST define adapter lifecycle using canonical states (registered → initialized → started → stopping → stopped → shutdown) with an explicit transition matrix; invalid transitions MUST be rejected with typed lifecycle errors
- **FR-005**: System MUST define flush() semantics guaranteeing all buffered telemetry is handed to the transport layer; shutdown() MUST implicitly invoke flush() before transitioning to stopped; after handoff, delivery guarantees are owned by the transport layer (AS-02)
- **FR-006**: System MUST define a strongly typed AdapterId for adapter registration identity
- **FR-007**: System MUST own bidirectional entity-specific mapping contracts (Canonical ↔ OpenTelemetry): TraceMappingContract, SpanMappingContract, MetricMappingContract, LogRecordMappingContract, ResourceMappingContract
- **FR-008**: System MUST define AdapterResult and AdapterError as the canonical failure model with a typed lifecycle error hierarchy for invalid state transitions
- **FR-009**: System MUST deliver telemetry to all matching registered adapters (multiplexing); if one adapter fails, remaining adapters MUST still receive the data and all failures MUST be aggregated
- **FR-010**: Adapter selection (which adapters are active) is owned by configuration semantics (AS-04), not the registry
- **FR-011**: System MUST define a typed AdapterHealth status model with states: Healthy, Degraded, Unhealthy, Unknown

### Key Entities

- **ProviderAdapter**: Trait for provider-side telemetry operations, sharing a common base with ExporterAdapter
- **ExporterAdapter**: Trait for exporter-side telemetry operations, sharing a common base with ProviderAdapter
- **CommonAdapterBase**: Shared base trait inherited by both ProviderAdapter and ExporterAdapter
- **AdapterRegistry**: Manages adapter registration (register(), get(), contains(), list()), lookup, and lifecycle orchestration; mutable until initialization, then frozen with thread-safe lookup
- **AdapterLifecycle**: State machine with canonical states (registered → initialized → started → stopping → stopped → shutdown) and explicit transition matrix; invalid transitions rejected with typed lifecycle errors
- **AdapterId**: Strongly typed value object for adapter registration identity; duplicate registration rejected with AdapterError
- **AdapterHealth**: Typed status model (Healthy, Degraded, Unhealthy, Unknown) for adapter health reporting
- **AdapterResult / AdapterError**: Canonical result type for adapter operations (success or typed error); typed lifecycle error hierarchy for invalid state transitions
- **TraceMappingContract**: Bidirectional entity-specific mapping contract for Trace ↔ OTel trace structure
- **SpanMappingContract**: Bidirectional entity-specific mapping contract for Span ↔ OTel span structure
- **MetricMappingContract**: Bidirectional entity-specific mapping contract for Metric ↔ OTel metric structure
- **LogRecordMappingContract**: Bidirectional entity-specific mapping contract for LogRecord ↔ OTel log structure
- **ResourceMappingContract**: Bidirectional entity-specific mapping contract for Resource ↔ OTel resource structure

## Success Criteria

### Measurable Outcomes

- **SC-001**: ProviderAdapter and ExporterAdapter with a common base trait are defined and documented
- **SC-002**: Registry accepts adapters during initialization, then freezes; post-init registrations are rejected with typed error; registry provides get(), contains(), list() operations for thread-safe lookup
- **SC-003**: All lifecycle states (registered → initialized → started → stopping → stopped → shutdown) are enumerable with explicit transition matrix; invalid transitions are rejected with typed lifecycle errors
- **SC-004**: flush() hands buffered telemetry to the transport layer; best-effort-only is not accepted; shutdown() implicitly invokes flush() before transitioning to stopped
- **SC-005**: Adapters are identified by strongly typed AdapterId; no string-typed lookups exist; duplicate registration is rejected
- **SC-006**: Multiple registered adapters all receive the same telemetry data (multiplexing verified); individual adapter failures do not prevent delivery to remaining adapters
- **SC-007**: AdapterResult/AdapterError covers adapter operation failures with typed lifecycle error hierarchy; boolean-only is not accepted
- **SC-008**: Only OpenTelemetry-compatible contracts are defined in AS-03; no concrete implementation is included
- **SC-009**: Five bidirectional entity-specific mapping contracts (Trace, Span, Metric, LogRecord, Resource) are defined and documented separately
- **SC-010**: AdapterHealth status model (Healthy, Degraded, Unhealthy, Unknown) is defined and testable

## Ownership Boundary

This specification owns:

- ProviderAdapter and ExporterAdapter traits with common base
- AdapterRegistry with frozen-after-init semantics; register(), get(), contains(), list() operations; thread-safe lookup after freeze
- AdapterLifecycle state machine with explicit transition matrix and typed transition errors
- AdapterId strongly typed identity; duplicate registration rejection
- AdapterHealth typed status model (Healthy, Degraded, Unhealthy, Unknown)
- AdapterResult and AdapterError canonical failure model with typed lifecycle error hierarchy
- Bidirectional entity-specific mapping contracts: TraceMappingContract, SpanMappingContract, MetricMappingContract, LogRecordMappingContract, ResourceMappingContract
- flush() semantics guarantee (handoff to transport layer); shutdown() implicitly flushes; delivery guarantees owned by transport after handoff
- Adapter multiplexing with best-effort and aggregate failures

This specification does not own:

- Domain model entities (Trace, Span, Metric, Log Record, Resource)
- Transport bindings or execution boundary infrastructure
- Transport delivery guarantees (owned by AS-02 after flush() handoff)
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

### Session 2026-06-15

- Q: Lifecycle Transition Rules → A: Explicit transition matrix plus typed transition errors (C)
- Q: Adapter Health Contract → A: Typed AdapterHealth status model (Healthy, Degraded, Unhealthy, Unknown) (C)
- Q: Shutdown Flush Semantics → A: shutdown() MUST implicitly invoke flush() (B)
- Q: Registry Lookup Contract → A: Registration + get() + contains() + list() (C)
- Q: Duplicate Adapter Registration → A: Reject with typed AdapterError (C)
- Q: Mapping Direction Ownership → A: Bidirectional mapping contracts (C)
- Q: Registry Concurrency Model → A: Thread-safe lookup after initialization; registration only during bootstrap phase (C)
- Q: Transport Boundary Clarification → A: Transport layer owns delivery guarantees after handoff (B)
- Q: Adapter Lifecycle Failure Handling → A: AdapterResult + typed lifecycle error hierarchy (C)
- Q: Adapter Multiplexing Failure Policy → A: Best-effort delivery to remaining adapters and aggregate failures (B)

## Assumptions

- Parent capability defines the canonical domain model entities
- Transport layer (AS-02) handles delivery and owns delivery guarantees after flush() handoff; adapters handle provider mapping
- Configuration semantics (AS-04) define adapter selection and behavior; registry does not select adapters
- Concrete adapter implementations (OpenTelemetry, Console Export) are separate specs consuming AS-03 contracts
- AdapterRegistry concurrency: lookup is thread-safe after freeze; registration only during single-threaded bootstrap phase
- Mapping contracts are bidirectional; each canonical entity maps to and from its OpenTelemetry equivalent
