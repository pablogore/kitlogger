# ADR-007: Shared Canonical Types Layer

## Status

Accepted

## Context

The telemetry system requires a shared canonical types layer to avoid duplication and ensure consistency across different capabilities. This layer should contain the core data structures that are used throughout the system.

## Decision

We will implement a `telemetry-types` crate that contains the following canonical types:

1. `PayloadEnvelope` - The canonical payload envelope that wraps telemetry data
2. `TelemetryBatch` - The canonical telemetry batch containing traces, metrics, and logs
3. `TransportMetadata` - Transport metadata for telemetry data
4. `BackpressureSignal` - Backpressure signal for flow control

These types will be owned by the `telemetry-types` crate and imported by other capabilities as needed.

## Consequences

### Positive

- Eliminates duplication of core types across capabilities
- Ensures consistency of data structures throughout the system
- Enables better interoperation between different capabilities
- Simplifies maintenance of shared data structures

### Negative

- Adds an additional dependency to other crates
- Requires coordination when changing shared types

## Implementation

The `telemetry-types` crate has been implemented with the following structure:

- `PayloadEnvelope` - wraps transport metadata, propagation metadata, and telemetry batch
- `TransportMetadata` - transport protocol, endpoint, and additional attributes
- `PropagationMetadata` - context propagation headers
- `TelemetryBatch` - contains traces, metrics, and logs
- `BackpressureSignal` - flow control signal with optional retry-after hint

All types are serializable using serde and are designed to be used across different capabilities in the telemetry system.