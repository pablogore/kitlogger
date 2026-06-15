# Feature Specification: Telemetry Adapter Contracts

**SPEC_ID**: `002-core-telemetry-domain-model-as-03-telemetry-adapter-contracts`

**PARENT_SPEC_ID**: `002-core-telemetry-domain-model`

**PARENT_SPEC_NAME**: `core-telemetry-domain-model`

**CAPABILITY_ID**: `002`

**CAPABILITY_NAME**: `core-telemetry-domain-model`

**EXPAND_ID**: AS-03

**Created**: 2026-06-14

**Status**: Draft

## Key Entities

- **CommonAdapterBase**: Base trait with `id()` and `health()` methods; shared by all adapter types
- **LifecycleAdapter**: Base trait with `flush(&self)` and `shutdown(&self)` methods using `&self` for Arc compatibility; inherited by ProviderAdapter and ExporterAdapter; concrete adapters own synchronization via interior mutability
- **TelemetryDelivery**: Dedicated trait for telemetry delivery operations; inherited by delivery-capable adapters; defines the operation executed during multiplexing
- **ProviderAdapter**: Trait for provider-side telemetry operations (initialize, start, stop); inherits from CommonAdapterBase, LifecycleAdapter, and TelemetryDelivery
- **ExporterAdapter**: Trait for exporter-side telemetry operations (initialize, start, stop); inherits from CommonAdapterBase, LifecycleAdapter, and TelemetryDelivery
- **AdapterRegistry**: Manages adapter registration (register(), get(), contains(), list()) using `Arc<dyn Adapter + Send + Sync>` storage; mutable until initialization, then frozen with thread-safe lookup; supports both ProviderAdapter and ExporterAdapter
- **AdapterLifecycle**: State machine with canonical states (registered → initialized → started → stopping → stopped → shutdown) and explicit transition matrix; invalid transitions rejected with typed lifecycle errors; Registered→Shutdown and Initialized→Shutdown allowed for startup failure; Stopped retains resources, Shutdown is terminal
- **AdapterId**: Strongly typed value object for adapter registration identity; duplicate registration rejected with AdapterError
- **HealthReport**: Struct containing AdapterHealth status (Healthy, Degraded, Unhealthy, Unknown), reason string, and timestamp
- **AdapterResult / AdapterError**: Canonical result type for adapter operations (success or typed error); typed lifecycle error hierarchy for invalid state transitions
- **TraceMappingContract**: Bidirectional entity-specific mapping contract for Trace ↔ OTel trace structure
- **SpanMappingContract**: Bidirectional entity-specific mapping contract for Span ↔ OTel span structure
- **MetricMappingContract**: Bidirectional entity-specific mapping contract for Metric ↔ OTel metric structure
- **LogRecordMappingContract**: Bidirectional entity-specific mapping contract for LogRecord ↔ OTel log structure
- **ResourceMappingContract**: Bidirectional entity-specific mapping contract for Resource ↔ OTel resource structure

## Success Criteria

### Measurable Outcomes

- **SC-001**: ProviderAdapter and ExporterAdapter with CommonAdapterBase + LifecycleAdapter base traits are defined and documented; all public adapter traits are object-safe
- **SC-002**: Registry accepts adapters during initialization, then freezes; post-init registrations are rejected with typed error; registry provides get(), contains(), list() operations for thread-safe lookup; registry stores and returns `Arc<dyn Adapter>`
- **SC-003**: All lifecycle states (registered → initialized → started → stopping → stopped → shutdown) are enumerable with explicit transition matrix; invalid transitions are rejected with typed lifecycle errors; Registered→Shutdown and Initialized→Shutdown transitions are valid; Stopped and Shutdown are semantically distinct
- **SC-004**: flush() hands buffered telemetry to the transport layer via LifecycleAdapter using `&self` receiver; best-effort-only is not accepted; shutdown(&self) implicitly invokes flush() before transitioning to stopped; all lifecycle operations are callable through `Arc<dyn Adapter>`
- **SC-005**: Adapters are identified by strongly typed AdapterId; no string-typed lookups exist; duplicate registration is rejected
- **SC-006**: Multiple registered adapters all receive the same telemetry data via TelemetryDelivery trait (multiplexing verified); individual adapter failures do not prevent delivery to remaining adapters
- **SC-007**: AdapterResult/AdapterError covers adapter operation failures with typed lifecycle error hierarchy; boolean-only is not accepted
- **SC-008**: Only OpenTelemetry-compatible contracts are defined in AS-03; no concrete implementation is included
- **SC-009**: Five bidirectional entity-specific mapping contracts (Trace, Span, Metric, LogRecord, Resource) are defined and documented separately
- **SC-010**: HealthReport struct with AdapterHealth, reason, and timestamp is defined and testable
- **SC-011**: All public adapter traits (CommonAdapterBase, LifecycleAdapter, TelemetryDelivery, ProviderAdapter, ExporterAdapter) compile with `dyn Trait` usage

## Ownership Boundary

This specification owns:

- ProviderAdapter and ExporterAdapter traits with CommonAdapterBase + LifecycleAdapter + TelemetryDelivery base traits
- LifecycleAdapter trait (flush, shutdown) separating lifecycle concerns from identity/health
- TelemetryDelivery trait defining the operation executed during multiplexing
- AdapterRegistry with frozen-after-init semantics; register(), get(), contains(), list() operations; Arc-based thread-safe storage; supports both ProviderAdapter and ExporterAdapter
- AdapterLifecycle state machine with explicit transition matrix and typed transition errors; Registered→Shutdown and Initialized→Shutdown allowed; Stopped vs Shutdown semantic distinction
- AdapterId strongly typed identity; duplicate registration rejection
- HealthReport struct with AdapterHealth, reason, and timestamp
- AdapterResult and AdapterError canonical failure model with typed lifecycle error hierarchy
- Bidirectional entity-specific mapping contracts: TraceMappingContract, SpanMappingContract, MetricMappingContract, LogRecordMappingContract, ResourceMappingContract
- flush() semantics guarantee (handoff to transport layer) via LifecycleAdapter with `&self` receiver; shutdown() implicitly flushes; delivery guarantees owned by transport after handoff
- Adapter multiplexing via TelemetryDelivery trait with best-effort and aggregate failures
- Object safety for all public adapter traits

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

### Session 2026-06-16

- Q: Adapter Trait Hierarchy → A: LifecycleAdapter trait containing flush() and shutdown(); ProviderAdapter and ExporterAdapter inherit from it (C)
- Q: Registry Adapter Type → A: Registry supports both ProviderAdapter and ExporterAdapter through a common registry abstraction (C)
- Q: Registry Storage Model → A: Arc<dyn Adapter + Send + Sync> for thread-safe lookup after freeze (B)
- Q: Registry Lookup API → A: Shared ownership via Arc<dyn Adapter> (B)
- Q: Telemetry Delivery Contract → A: Separate TelemetryDelivery trait inherited by delivery-capable adapters (C)
- Q: Multiplexing Contract → A: Operation defined by a dedicated TelemetryDelivery trait (C)
- Q: Lifecycle Startup Failure Handling → A: Registered → Shutdown and Initialized → Shutdown are allowed (B)
- Q: Lifecycle Terminal State → A: Stopped = operationally stopped, resources retained; Shutdown = resources released and terminal state (B)
- Q: Adapter Health Detail Model → A: HealthReport containing AdapterHealth plus reason and timestamp (B)
- Q: Object Safety Requirement → A: Yes, all adapter traits MUST be object-safe (B)

### Session 2026-06-17

- Q: LifecycleAdapter Receiver Type → A: Use `&self` for flush() and shutdown(); adapters manage internal mutability behind synchronization primitives (B)
- Q: Adapter Concurrency Contract → A: Concrete adapters own synchronization and internal mutability (B)
- Q: Registry Storage Contract → A: RwLock<HashMap<AdapterId, Arc<dyn Adapter>>> (B)
- Q: Object Safety Enforcement → A: Yes, LifecycleAdapter must remain object-safe when used through Arc<dyn Adapter> (B)
- Q: Lifecycle Invocation Model → A: Yes, all lifecycle operations must be callable through Arc<dyn Adapter> (A)

## Assumptions

- Parent capability defines the canonical domain model entities
- Transport layer (AS-02) handles delivery and owns delivery guarantees after flush() handoff; adapters handle provider mapping
- Configuration semantics (AS-04) define adapter selection and behavior; registry does not select adapters
- Concrete adapter implementations (OpenTelemetry, Console Export) are separate specs consuming AS-03 contracts
- AdapterRegistry concurrency: lookup is thread-safe after freeze; registration only during single-threaded bootstrap phase; Arc-based storage enables shared ownership
- Mapping contracts are bidirectional; each canonical entity maps to and from its OpenTelemetry equivalent
- LifecycleAdapter separates lifecycle concerns (flush, shutdown) from identity/health (CommonAdapterBase)
- TelemetryDelivery trait defines the multiplexing operation; senders call deliver() rather than directly interacting with adapter lifecycle
- All adapter traits are object-safe; dyn Trait usage is required for registry storage
- LifecycleAdapter uses `&self` receiver for flush() and shutdown(); concrete adapters own internal synchronization
- AdapterRegistry storage is `RwLock<HashMap<AdapterId, Arc<dyn Adapter>>>`
- Concrete adapters are responsible for interior mutability of their own state
- All lifecycle operations are callable directly through Arc<dyn Adapter> from registry lookups
