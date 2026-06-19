# Data Model: Transport-Agnostic Telemetry Flow

## Shared Canonical Types

The following types are owned by the `telemetry-types` crate and are documented in its data model. They are referenced here for completeness but their authoritative definition lives in `telemetry-types`.

- **PayloadEnvelope**: Canonical wrapper for transporting a TelemetryBatch across execution boundaries. Contains transport_metadata, propagation_metadata (from AS-01), and payload (TelemetryBatch).
- **TelemetryBatch**: Canonical batch model carrying traces, metrics, and logs. Validation: at least one signal type must be non-empty.
- **TransportMetadata**: Timestamp, content-type, encoding hints.
- **BackpressureSignal**: Flow control signal with optional retry-after hint.
- **TelemetryBatchError**: Validation error for TelemetryBatch empty rejection.

See `crates/telemetry-types/` for authoritative definitions.

## Entities (Owned by AS-02)

### DeliveryMode

Abstract delivery mode returned by the Transport trait as a value.

| Field | Type | Description |
|-------|------|-------------|
| variant | enum | FireAndForget, RequestResponse, Batch, Streaming |

- Non-exhaustive enum
- Returned as a value from `Transport::send()`, not an associated type

### TransportResult\<DeliveryMode\>

Result type returned by Transport trait operations.

- `type TransportResult<T> = Result<T, TransportError>`
- Success variant carries DeliveryMode for the completed operation

### TransportError

Error type for transport operations.

| Variant | Payload | Description |
|---------|---------|-------------|
| Timeout | (none) | The transport operation timed out |
| Unavailable | (none) | The destination is unreachable |
| Backpressure | BackpressureSignal | Flow control signal with optional retry-after hint (type from telemetry-types) |
| PayloadTooLarge | (none) | Payload exceeds transport limits |
| UnsupportedTransport | (none) | The requested transport is not available |

- Non-exhaustive enum
- Manual Display and Error impls (no thiserror dependency)
- No new variants may be added here without an architecture finding; child specs extend via their own error types or contribute variants back

### Transport Trait

Contract abstraction for sending telemetry across execution boundaries.

```rust
#[async_trait]
pub trait Transport {
    async fn send(&self, envelope: PayloadEnvelope) -> TransportResult<DeliveryMode>;
}
```

- PayloadEnvelope is from `telemetry-types`
- Uses `std::future::Future` only — no async runtime dependency
- DeliveryMode returned as enum value (not associated type)
- Propagates carrier metadata via PayloadEnvelope.propagation_metadata

## Relationships

- `Transport::send()` takes a `PayloadEnvelope` (from telemetry-types) and returns `TransportResult<DeliveryMode>`
- `PayloadEnvelope` contains one `TelemetryBatch` (both from telemetry-types)
- `TelemetryBatch` contains lists of `Span`, `Metric`, and `LogRecord` (types from AS-01)
- `TransportResult::Err(TransportError::Backpressure(BackpressureSignal))` carries flow control signal (BackpressureSignal from telemetry-types)
- Carrier abstraction traits (Injector, Extractor) are referenced from AS-01
- MapCarrier (from AS-01) is the test-only carrier implementation; concrete carriers belong to child specs

## Validation Rules

1. TelemetryBatch must have at least one signal type non-empty (defined in telemetry-types)
2. PayloadEnvelope must always carry a TelemetryBatch (never empty) (defined in telemetry-types)
3. DeliveryMode variants are fixed for the contract lifetime; child specs do not add variants here
4. TransportError variants are non-exhaustive; child specs contribute new variants via architecture finding

## State Transitions

N/A — AS-02 defines contracts only, no stateful entities or lifecycle.
