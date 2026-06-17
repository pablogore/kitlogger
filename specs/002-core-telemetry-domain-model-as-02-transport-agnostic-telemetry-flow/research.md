# Research: Transport-Agnostic Telemetry Flow

## 1. Transport Trait Shape

- **Decision**: Single trait with `async fn send(&self, envelope: PayloadEnvelope) -> TransportResult<DeliveryMode>` using `std::future::Future` only; PayloadEnvelope is from `telemetry-types`
- **Rationale**: Contract-only trait returns DeliveryMode as an enum value (not associated type). Uses std::future::Future for runtime independence — no Tokio coupling. Concrete transports (HTTP, gRPC) implement this trait without modifying AS-02. PayloadEnvelope owned by `telemetry-types` per ADR-007.
- **Alternatives considered**: Associated type for delivery mode (couples trait to mode), synchronous trait (incompatible with streaming)

## 2. DeliveryMode Enum Return Value

- **Decision**: `DeliveryMode` is a non-exhaustive enum returned as a value from `Transport::send()`, not an associated type on the trait
- **Rationale**: Return value allows the transport to report dynamically which delivery mode was used, enabling runtime selection without generic parameter proliferation on the trait
- **Alternatives considered**: Associated type on Transport trait (requires generic param everywhere, harder to compose), generic parameter (complicates usage of trait objects)

## 3. PayloadEnvelope Serialization

- **Decision**: Serde Serialize/Deserialize derives on PayloadEnvelope and TelemetryBatch (defined in telemetry-types)
- **Rationale**: Serde is the standard Rust serialization framework, already declared in tech-stack. Derives provide zero-cost abstract serialization without coupling to a specific wire format. Types are owned by telemetry-types per ADR-007.
- **Alternatives considered**: Manual serialization (error-prone, no serde ecosystem), custom trait (reinventing serde)

## 4. TransportError Model

- **Decision**: Non-exhaustive enum with manual Display and Error impls (no thiserror — undeclared dependency)
- **Rationale**: Non-exhaustive allows transport implementations to add error variants without breaking changes. Manual impls avoid adding thiserror dependency (not declared in tech-stack).
- **Alternatives considered**: Box-dyn error (loses typed variants), thiserror derive (undeclared dependency)

## 5. Backpressure Semantics

- **Decision**: Explicit `Backpressure` variant in `TransportError` with a `BackpressureSignal` value. `BackpressureSignal` is owned by `telemetry-types`.
- **Rationale**: Q4 clarifies backpressure belongs to TransportError::Backpressure, not DeliveryMode. Backpressure signal provides retry-after hint for flow control. BackpressureSignal is shared between AS-02 (TransportError::Backpressure) and AS-03 (flush/semantics) per ADR-007.
- **Alternatives considered**: Backpressure in DeliveryMode (per Q4 rejected), separate backpressure callback (hard to compose), DeliveryMode-level flag (would require every mode variant to carry it)

## 6. Async Runtime Independence

- **Decision**: Transport trait uses `std::future::Future` only; no Tokio, async-std, or smol dependency
- **Rationale**: AS-02 is a pure contract specification. Runtime coupling would force runtime choice on all downstream implementations. Concrete transport binding specs may choose their own runtime.
- **Alternatives considered**: Tokio::async_trait (couples to Tokio), generic runtime trait (YAGNI, no usage in contracts)

## 7. TelemetryBatch Non-Empty Validation

- **Decision**: `TelemetryBatch::new()` returns a `Result` that rejects batches where traces, metrics, and logs are all empty
- **Rationale**: FR-010 mandates validation. Carrying empty batches is a no-op that wastes serialization and transport resources. Callers must provide at least one signal type.
- **Alternatives considered**: Allow empty batches (violates FR-010), warn-only (weak enforcement), type-state pattern (over-engineered for initial contract)

## 8. Carrier Ownership in AS-02

- **Decision**: AS-02 owns only the carrier abstraction traits (Injector, Extractor) as contract dependencies from AS-01. Concrete carriers (HttpHeaderCarrier, GrpcMetadataCarrier) belong to child transport binding specs.
- **Rationale**: AS-01 owns Injector/Extractor/Propagator traits. Transport-specific carrier implementations (HTTP headers, gRPC metadata) are part of concrete transport bindings, not the abstract contract. AS-02 uses MapCarrier from AS-01 for mock-based testing.
- **Alternatives considered**: Keep concrete carriers in AS-02 (blurs ownership boundary, OCP violation), re-home carriers from AS-01 (ownership confusion)

## 9. Test Strategy

- **Decision**: AS-02 tests validate only abstract contracts via mocks using MapCarrier from AS-01. No concrete protocol testing (HTTP, gRPC) in AS-02.
- **Rationale**: AS-02 is a pure contract specification. Concrete protocol validation belongs to child transport binding specs. Mock-based tests keep AS-02 lightweight and runtime-independent.
- **Alternatives considered**: Integration tests with HTTP/gRPC (mixed ownership, violates scope), no tests at all (unacceptable)

## 10. Execution Boundary Scope

- **Decision**: Execution boundaries are informative examples only (HTTP, gRPC, CLI, Jobs). AS-02 does not model concrete execution boundary types.
- **Rationale**: Q8 confirms execution boundaries are for illustration. Concrete boundary types belong to child transport binding specs.
- **Alternatives considered**: Model ExecutionBoundary as enum in AS-02 (scope creep, violates non-scope)
