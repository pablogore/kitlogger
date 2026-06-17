# ADR-007: Shared Canonical Types Layer

**Status**: Approved
**Date**: 2026-06-17
**Author**: Architecture Governance

## Problem Statement

`PayloadEnvelope` is defined in AS-02 (telemetry-transport-contract) but required by AS-03 (telemetry-adapter-contracts) in the `TelemetryDelivery::deliver()` method signature. AS-03 cannot legally depend on AS-02 — the architecture documents both as independent peers with no cross-dependency. The implementation substituted `Vec<u8>` as a workaround, causing a contract compliance failure (6 code sites in AS-03 use `Vec<u8>` where `PayloadEnvelope` is contractually required).

## Alternatives Considered

### Alternative A: AS-03 depends on AS-02

Create a dependency edge AS-03 → AS-02.

**Rejected** because:
- Dependency direction is wrong: adapters conceptually feed INTO transport, not the reverse
- AS-03 would depend on all of AS-02 (Transport trait, DeliveryMode, TransportError) when it only needs `PayloadEnvelope`
- Violates the documented DAG (architecture.md, decomposition.md, feature-index.md)
- Future transport changes could cascade into adapter contracts
- Creates latent circular dependency risk if AS-02 ever needs adapter concepts

### Alternative B: Move `PayloadEnvelope` to AS-01 (context-propagation)

**Rejected** because:
- `PayloadEnvelope` contains `TransportMetadata` (timestamp, content-type, encoding) — a transport concern with no semantic relationship to context propagation
- AS-01's responsibility is "Context Propagation and Correlation"; `PayloadEnvelope` is a transport DTO
- Would pollute the context-propagation crate with transport-level concerns
- Adding transport-level fields (compression, encryption, routing) to `PayloadEnvelope` in the future would require modifying a context-propagation crate

### Alternative C: Shared Canonical Types Layer (`telemetry-types`)

**Selected**.

### Alternative D: Keep `Vec<u8>` workaround

**Rejected** because:
- Loses `transport_metadata` (content-type, encoding hints)
- Loses `propagation_metadata` (trace context, correlation IDs, baggage)
- Loses `TelemetryBatch` typing (traces, metrics, logs separation)
- Contract compliance failure — `adapter-api.md` specifies `PayloadEnvelope` but code uses `Vec<u8>`
- Future extensibility destroyed

## Decision

Create a new shared crate `telemetry-types` that owns all cross-capability canonical pipeline types.

### Ownership Model

| Type | Previous Owner | New Owner | Rationale |
|------|---------------|-----------|-----------|
| `PayloadEnvelope` | AS-02 | `telemetry-types` | Consumed by AS-02 and AS-03 |
| `TelemetryBatch` | AS-02 | `telemetry-types` | Carried inside PayloadEnvelope; cross-cutting |
| `TelemetryBatchError` | AS-02 | `telemetry-types` | Owned by TelemetryBatch |
| `TransportMetadata` | AS-02 | `telemetry-types` | Field of PayloadEnvelope |
| `BackpressureSignal` | AS-02 | `telemetry-types` | Referenced by TransportError; semantically a flow-control concept usable by both transport and adapters |

### Unchanged Ownership

| Type | Owner | Rationale |
|------|-------|-----------|
| `Transport` trait | AS-02 | Only AS-02 defines the send contract |
| `TransportError` | AS-02 | Transport-specific error model |
| `TransportResult` | AS-02 | Transport-specific result type |
| `DeliveryMode` | AS-02 | Transport-specific delivery semantics |
| `Adapter` supertrait | AS-03 | Adapter-specific abstraction |
| `LifecycleAdapter` | AS-03 | Adapter lifecycle |
| `ProviderAdapter` | AS-03 | Provider-side operations |
| `ExporterAdapter` | AS-03 | Exporter-side operations |
| `AdapterRegistry` | AS-03 | Registry management |
| `AdapterError` | AS-03 | Adapter-specific error model |
| `HealthReport` | AS-03 | Health reporting |
| `LifecycleState` | AS-03 | Lifecycle state machine |
| Mapping contracts | AS-03 | OTel entity mapping |

### Ownership Rule

A type belongs in `telemetry-types` when:

1. It is consumed by multiple peer capabilities.
2. It represents canonical telemetry data-in-transit.
3. It has no single natural owner.
4. Moving it eliminates peer-to-peer dependencies.

All other types remain owned by their originating capability.

## Updated Dependency Graph

```
context-propagation (AS-01)
          ↑
          │
    telemetry-types (new)
       ↑       ↑
       │       │
    AS-02    AS-03
       ↑
       │
    AS-04
```

**Verification**:
- No cycles: DAG structure preserved
- No peer dependencies: AS-02 and AS-03 remain independent
- Stable dependency direction: all edges flow from specific to general

## Consequences

### Positive

1. AS-02 and AS-03 remain independent peers (no cross-dependency)
2. Single authoritative source for `PayloadEnvelope`, `TelemetryBatch`, `TransportMetadata`, `BackpressureSignal`
3. Future specs (concrete adapters, transport bindings) can depend on `telemetry-types` for canonical data types
4. All architecture principles satisfied: DIP, SDP, Clean Architecture, Hexagonal
5. Contract compliance restored — `TelemetryDelivery::deliver()` uses `PayloadEnvelope`

### Negative

1. One additional crate in the workspace
2. Migration effort: move code from AS-02 to `telemetry-types`, update imports in AS-02 and AS-03
3. Ownership boundary documentation must be updated across all affected specifications
4. AS-02 loses ownership of types that were previously part of its identity

## Compliance

- `spec.md` files: Updated ownership statements
- `data-model.md`: `PayloadEnvelope`/`TelemetryBatch`/`TransportMetadata`/`BackpressureSignal` defined exactly once in `telemetry-types`
- `contracts/adapter-api.md`: References `PayloadEnvelope` from `telemetry-types`
- `contracts/transport-api.md`: References `PayloadEnvelope` from `telemetry-types`
- No `Vec<u8>` as replacement for `PayloadEnvelope` in any contract
