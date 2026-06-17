# Feature Specification: Transport-Agnostic Telemetry Flow

**SPEC_ID**: `002-core-telemetry-domain-model-as-02-transport-agnostic-telemetry-flow`

**PARENT_SPEC_ID**: `002-core-telemetry-domain-model`

**PARENT_SPEC_NAME**: `core-telemetry-domain-model`

**CAPABILITY_ID**: `002`

**CAPABILITY_NAME**: `core-telemetry-domain-model`

**EXPAND_ID**: AS-02

**Created**: 2026-06-14

**Status**: Draft

## Scope

Define the canonical transport abstraction for telemetry data flow across execution boundaries. This specification owns the Transport contract, PayloadEnvelope, TelemetryBatch, TransportResult/TransportError model, abstract delivery modes, and backpressure semantics. No concrete transport implementations are owned here; all transports (HTTP, gRPC, CLI, Background Jobs, Kafka, NATS, RabbitMQ, Cron Jobs, Event Systems) are implemented as separate specifications that implement these contracts.

## Non-Scope

- Domain model entities (Resource, Trace, Span, Metric, Log Record)
- Context propagation semantics (covered by AS-01); AS-02 carries propagation metadata only
- Adapter contracts or exporter interfaces
- Configuration infrastructure
- Concrete transport implementations (HTTP, gRPC, CLI, Background Jobs, Kafka, NATS, RabbitMQ, Cron Jobs, Event Systems are separate specs)
- Concrete carrier implementations (HttpHeaderCarrier, GrpcMetadataCarrier belong to child transport specs)
- Async runtime coupling; AS-02 is runtime-independent (std::future::Future only)

## Responsibility

Define abstract transport contracts for telemetry data flow across execution boundaries. AS-02 owns only contract abstractions (Transport trait, payload types, error model, delivery modes). Examples such as HTTP, gRPC, CLI, and Background Jobs are informative only; they belong to separate binding specifications.

## Dependencies

- `002-core-telemetry-domain-model-as-01-context-propagation-and-correlation` (AS-01)

## User Scenarios & Testing

### User Story 1 - Request/Response Delivery Contract
The transport contract must support request/response semantics where the caller awaits a delivery result.

**Acceptance Scenarios**:
1. Given a mock transport, When send() returns TransportResult::Ok(DeliveryMode::RequestResponse), Then the caller can identify request/response delivery
2. Given a transport operation, When it succeeds, Then TransportResult::Ok carries the DeliveryMode; when it fails, TransportResult::Err carries a TransportError

### User Story 2 - Streaming Delivery Contract
The transport contract must support streaming delivery semantics for continuous data flow.

**Acceptance Scenarios**:
1. Given a mock transport, When send() returns DeliveryMode::Streaming, Then streaming delivery is distinguishable from other modes
2. Given a transport destination that is unreachable, When TransportError::Unavailable is returned, Then the error is distinguishable from other error types

### User Story 3 - Batch Delivery Contract
The transport contract must support batch delivery semantics where telemetry is delivered as discrete, complete batches.

**Acceptance Scenarios**:
1. Given a mock transport, When send() returns DeliveryMode::Batch, Then batch delivery is distinguishable from other modes
2. Given a transport under backpressure, When TransportError::Backpressure is returned with a BackpressureSignal, Then the caller can retrieve the retry-after hint for flow control

### User Story 4 - Extensible Transport Contract
The transport contract must remain stable when new delivery modes, error variants, or transport implementations are added.

**Acceptance Scenarios**:
1. Given a new transport implementation, When it implements the Transport trait, Then it integrates without modifying any AS-02 types
2. Given the abstract contract, When DeliveryMode and TransportError are pattern-matched with wildcard arms, Then existing callers continue to compile unchanged

## Requirements

### Functional Requirements

- **FR-001**: System MUST define a canonical Transport contract consisting of Transport trait, PayloadEnvelope, and TransportResult/TransportError
- **FR-002**: System MUST define a canonical TelemetryBatch model containing traces, metrics, and logs as the sole payload type
- **FR-003**: System MUST define a PayloadEnvelope carrying transport metadata, propagation metadata (from AS-01), and a TelemetryBatch
- **FR-004**: System MUST define a TransportResult and TransportError model covering timeout, unavailable, backpressure, payload-too-large, and unsupported transport
- **FR-005**: System MUST support abstract delivery modes as an enum return value on Transport trait: fire-and-forget, request/response, batch, and streaming
- **FR-006**: System MUST define backpressure semantics via TransportError::Backpressure variant
- **FR-007**: System MUST carry propagation metadata provided by AS-01 but MUST NOT create or own context
- **FR-008**: AS-02 defines contracts only; concrete transport implementations (HTTP, gRPC, CLI, Background Jobs) are separate specifications
- **FR-009**: The Transport contract MUST remain stable when new transport implementations (Kafka, NATS, RabbitMQ, SQS, EventBridge) are added; only new binding specifications are created
- **FR-010**: TelemetryBatch constructor MUST reject batches where traces, metrics, and logs are all empty
- **FR-011**: Transport trait MUST be runtime-independent using only std::future::Future; no async runtime dependency

### Key Entities

- **Transport Contract**: Canonical abstraction owned by AS-02, consisting of Transport trait, PayloadEnvelope, TelemetryBatch, and TransportResult/TransportError
- **Transport Binding**: Concrete implementation of the transport contract for a specific protocol (owned by separate specs)
- **PayloadEnvelope**: The canonical wrapper carrying transport metadata, propagation metadata (from AS-01), and a TelemetryBatch; owned by AS-02
- **TelemetryBatch**: The canonical batch model containing traces, metrics, and logs; the sole payload type carried inside PayloadEnvelope
- **TransportResult / TransportError**: Result type covering success, timeout, unavailable, backpressure, payload-too-large, and unsupported transport
- **DeliveryMode**: Abstract delivery mode enum supporting fire-and-forget, request/response, batch, and streaming
- **Execution Boundary**: Informative concept representing the scope of a single execution unit; AS-02 does not model concrete boundary types. Examples (HTTP request, gRPC stream, CLI command, job run) are for illustration only and belong to child transport specs.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Transport contract (trait + PayloadEnvelope + TelemetryBatch + TransportResult/TransportError) is defined and documented
- **SC-002**: TelemetryBatch is the sole payload type inside PayloadEnvelope, carrying traces, metrics, and logs
- **SC-003**: TransportResult covers timeout, unavailable, backpressure, payload-too-large, and unsupported transport errors
- **SC-004**: All four delivery modes (fire-and-forget, request/response, batch, streaming) are representable as abstract modes
- **SC-005**: Backpressure signals are propagated through the transport contract
- **SC-006**: A new transport protocol can be implemented as a separate spec without modifying AS-02 contracts

## Ownership Boundary

This specification owns:

- Transport contract (trait, PayloadEnvelope, TelemetryBatch, TransportResult/TransportError)
- TelemetryBatch model containing traces, metrics, and logs
- PayloadEnvelope format carrying transport metadata, propagation metadata, and TelemetryBatch
- Abstract delivery modes (fire-and-forget, request/response, batch, streaming)
- Backpressure semantics as TransportError::Backpressure variant
- Carrier abstraction traits (Injector, Extractor — from AS-01, referenced as contract dependencies)

This specification does not own:

- Domain model entities (Resource, Trace, Span, Metric, Log Record)
- Context propagation semantics or creation (AS-01); AS-02 carries propagation metadata only
- Adapter contracts or exporter interfaces
- Configuration infrastructure
- Concrete transport implementations (HTTP, gRPC, CLI, Background Jobs are separate specs)
- Concrete carrier implementations (HttpHeaderCarrier, GrpcMetadataCarrier belong to child transport specs)
- Async runtime coupling (AS-02 is runtime-independent)
- Concrete execution boundary types (examples in this spec are informative only)

## Clarifications

### Session 2026-06-14

- Q: Transport Contract Shape → A: Transport + Payload Envelope + TransportResult/TransportError (C)
- Q: Transport Implementation Scope → A: AS-02 defines contracts only; concrete transports are separate specs (B)
- Q: Payload Envelope Ownership → A: AS-02 owns PayloadEnvelope and transport metadata (A)
- Q: Context Propagation Integration → A: AS-02 carries propagation metadata provided by AS-01 but does not create or own context (B)
- Q: Transport Error Model → A: Transport-level errors including timeout, unavailable, backpressure, payload-too-large, unsupported transport (A)
- Q: Delivery Semantics → A: Fire-and-forget, request/response, batch, and streaming as abstract modes (D)
- Q: Backpressure Ownership → A: Backpressure semantics as transport contract behavior (A)
- Q: Future Transport Extensibility → A: Only new implementations are added; AS-02 contracts remain unchanged (A)
- Q: Telemetry Payload Ownership → A: PayloadEnvelope carries a canonical TelemetryBatch model containing traces, metrics, and logs (C)

### Session 2026-06-15

- Q: Ownership of Transport Carriers → A: AS-02 owns only carrier traits (Injector, Extractor); concrete carriers (HttpHeaderCarrier, GrpcMetadataCarrier) belong to child transport specs (B)
- Q: Transport Trait Delivery Mode Shape → A: DeliveryMode enum return value, not associated type (B)
- Q: Async Runtime Dependency → A: std::future::Future only; no runtime dependency; AS-02 is a pure contract (C)
- Q: Backpressure Ownership → A: Backpressure belongs to TransportError::Backpressure, not DeliveryMode (A)
- Q: Transport Validation Scope → A: AS-02 tests only abstract contracts via mocks; no concrete protocol testing (B)
- Q: TelemetryBatch Validation Ownership → A: Constructor of TelemetryBatch validates at least one signal type non-empty (A)
- Q: Future Transport Extensibility Strategy → A: New binding specification per protocol; AS-02 remains unchanged (B)
- Q: Scope of Execution Boundaries → A: Examples (HTTP, gRPC, CLI, Jobs) are informative only; AS-02 does not model concrete types (A)
- Q: User Story Ownership Boundary → A: Rewritten as transport-agnostic contract capabilities: Request/Response Delivery, Streaming Delivery, Batch Delivery, Extensible Transport. Protocol-specific acceptance scenarios replaced with abstract contract validations (B)

## Assumptions

- Context propagation (AS-01) provides the correlation metadata carried by transport payloads; AS-02 carries but does not create context
- Adapter contracts (AS-03) define how telemetry enters the transport layer
- Transport implementations handle wire-level encoding and protocol negotiation
- Concrete transport specs implement the Transport contract defined here without modifying AS-02
- AS-02 has no runtime dependency; Transport trait uses std::future::Future only
