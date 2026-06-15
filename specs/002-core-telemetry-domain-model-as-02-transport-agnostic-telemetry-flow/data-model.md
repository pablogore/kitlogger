# Data Model: Transport-Agnostic Telemetry Flow

## Entities

### DeliveryMode

Abstract delivery mode returned by the Transport trait as a value.

| Field | Type | Description |
|-------|------|-------------|
| variant | enum | FireAndForget, RequestResponse, Batch, Streaming |

- Non-exhaustive enum
- Returned as a value from `Transport::send()`, not an associated type

### BackpressureSignal

Signal sent back via TransportError::Backpressure to indicate flow control.

| Field | Type | Description |
|-------|------|-------------|
| retry_after | Option\<Duration\> | Recommended wait time before retrying |

### TelemetryBatch

The canonical batch model carrying traces, metrics, and logs.

| Field | Type | Constraints |
|-------|------|-------------|
| resource | Resource | Mandatory — origin resource/entity identifier |
| traces | Vec\<Span\> | May be empty |
| metrics | Vec\<Metric\> | May be empty |
| logs | Vec\<LogRecord\> | May be empty |

- **Validation**: At least one of traces, metrics, or logs must be non-empty
- Constructor returns a `Result` and rejects all-empty batches

### PayloadEnvelope

Canonical wrapper for transporting a TelemetryBatch across execution boundaries.

| Field | Type | Description |
|-------|------|-------------|
| transport_metadata | TransportMetadata | Timestamp, content-type, encoding hints |
| propagation_metadata | PropagationMetadata | Context/correlation metadata from AS-01 |
| payload | TelemetryBatch | The telemetry data |

- Serde Serialize/Deserialize derives
- Carries but does not create propagation metadata

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
| Backpressure | BackpressureSignal | Flow control signal with optional retry-after hint |
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

- Uses `std::future::Future` only — no async runtime dependency
- DeliveryMode returned as enum value (not associated type)
- Propagates carrier metadata via PayloadEnvelope.propagation_metadata

## Relationships

- `Transport::send()` takes a `PayloadEnvelope` and returns `TransportResult<DeliveryMode>`
- `PayloadEnvelope` contains one `TelemetryBatch`
- `TelemetryBatch` contains lists of `Span`, `Metric`, and `LogRecord` (types from child specs)
- `TransportResult::Err(TransportError::Backpressure(BackpressureSignal))` carries flow control signal
- Carrier abstraction traits (Injector, Extractor) are referenced from AS-01
- MapCarrier (from AS-01) is the test-only carrier implementation; concrete carriers belong to child specs

## Validation Rules

1. TelemetryBatch must have at least one signal type non-empty
2. PayloadEnvelope must always carry a TelemetryBatch (never empty)
3. DeliveryMode variants are fixed for the contract lifetime; child specs do not add variants here
4. TransportError variants are non-exhaustive; child specs contribute new variants via architecture finding

## State Transitions

N/A — AS-02 defines contracts only, no stateful entities or lifecycle.
