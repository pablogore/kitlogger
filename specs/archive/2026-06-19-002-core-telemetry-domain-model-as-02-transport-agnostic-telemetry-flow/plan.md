# Implementation Plan: Transport-Agnostic Telemetry Flow

**Branch**: `main` | **Date**: 2026-06-15 | **Spec**: [Transport-Agnostic Telemetry Flow](spec.md)

**Input**: Feature specification from `specs/002-core-telemetry-domain-model-as-02-transport-agnostic-telemetry-flow/spec.md`

**Technology gate**: Every technology named in this plan and its generated
artifacts MUST be declared in this specification's `tech-stack.yaml`. Missing
or undeclared technology is blocking; do not infer a replacement.

## Summary

Define the canonical transport abstraction for telemetry data flow across execution boundaries. AS-02 owns the Transport contract (trait, TransportResult/TransportError), DeliveryMode enum, and backpressure semantics as TransportError::Backpressure. AS-02 depends on `telemetry-types` for shared canonical payload types (PayloadEnvelope, TelemetryBatch, TransportMetadata, BackpressureSignal) and on AS-01 for context propagation types. No concrete transport implementations or concrete carriers are owned here; all transports (HTTP, gRPC, CLI, Background Jobs, Kafka, NATS, RabbitMQ, SQS, EventBridge) are separate binding specifications that implement these contracts.

## Technical Context

**Language/Version**: Rust (edition 2021)

**Primary Dependencies**: serde (derive), uuid (via AS-01 for PropagationMetadata types)

**Storage**: N/A (contract definitions only)

**Testing**: cargo test

**Target Platform**: Cross-platform (Rust library crate)

**Project Type**: Rust library crate providing transport contract types and traits

**Performance Goals**: N/A — contract definitions only; concrete binding specs own performance targets

**Constraints**: Must use std::future::Future only (no async runtime dependency). Must not depend on concrete transport implementations. Must not define concrete carrier implementations (HttpHeaderCarrier, GrpcMetadataCarrier belong to child specs). DeliveryMode returned as enum, not associated type. TelemetryBatch constructor must reject empty batches.

**Scale/Scope**: Library crate providing Transport trait, TransportResult/TransportError, and DeliveryMode. Depends on `telemetry-types` for PayloadEnvelope, TelemetryBatch, TransportMetadata, and BackpressureSignal. Tests validate contracts via mocks (MapCarrier from AS-01).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

1. **Atomic Specifications** - PASS: Single independently testable feature (transport-agnostic telemetry flow contracts)
2. **Clear Boundaries** - PASS: Scope/non-scope well-defined; concrete transports and concrete carriers explicitly excluded as separate binding specs
3. **Dependency Management** - PASS: Depends only on AS-01 (context propagation) for carrier traits and propagation metadata types; no circular dependencies
4. **Testability** - PASS: Success criteria defined with testable scenarios using mocks instead of concrete transports
5. **Extensibility** - PASS: DeliveryMode enum is non-exhaustive; TransportError is non-exhaustive; new bindings are always separate specs

**Pre-Design Verdict**: ALL GATES PASS - proceed to Phase 0

**Post-Design Verdict**: ALL GATES PASS — design artifacts consistent with constitution

## Project Structure

### Documentation (this feature)

```text
specs/002-core-telemetry-domain-model-as-02-transport-agnostic-telemetry-flow/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
src/
├── lib.rs               # Crate root with re-exports
├── transport.rs         # Transport trait, DeliveryMode, BackpressureSignal (from telemetry-types)
└── error.rs             # TransportResult, TransportError

tests/
└── transport_test.rs    # Transport contract validation tests (via mocks)

# Shared canonical types (telemetry-types crate)
crates/telemetry-types/
├── Cargo.toml           # Depends on context-propagation, serde
├── src/
│   ├── lib.rs           # Re-exports all public types
│   ├── payload.rs       # PayloadEnvelope, TransportMetadata
│   ├── batch.rs         # TelemetryBatch, TelemetryBatchError
│   └── signal.rs        # BackpressureSignal
└── tests/
    ├── payload_test.rs  # PayloadEnvelope serde tests
    └── batch_test.rs    # TelemetryBatch validation tests
```

No carrier_ext.rs. AS-02 uses MapCarrier from AS-01 for mock-based testing. Concrete carriers (HttpHeaderCarrier, GrpcMetadataCarrier) belong to child transport binding specs.

**Structure Decision**: Single Rust library crate with flat module organization. Transport contracts, payload types, batch model, and error types each in dedicated modules. Tests validate abstract contracts via mocks (MapCarrier from AS-01). No concrete transport or carrier implementations.

## Complexity Tracking

*No constitution violations - not applicable.*
