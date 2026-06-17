# Transport Contract API

> **Ownership**: AS-02 owns Transport trait, TransportResult/TransportError, and DeliveryMode. PayloadEnvelope, TelemetryBatch, TransportMetadata, and BackpressureSignal are owned by the shared `telemetry-types` crate (see ADR-007). Carrier abstraction traits (Injector/Extractor) are from AS-01. Concrete transports (HTTP, gRPC, CLI, Kafka, etc.) and concrete carriers (HttpHeaderCarrier, GrpcMetadataCarrier) are separate binding specifications.

## Transport Trait

```rust
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a telemetry payload across an execution boundary.
    ///
    /// Returns the delivery mode used for the operation.
    async fn send(&self, envelope: PayloadEnvelope) -> TransportResult<DeliveryMode>;
}
```

- `#[async_trait]` is NOT from tokio; it requires `async-trait` crate or manual `Pin<Box<dyn Future>>` equivalent
- Uses `std::future::Future` only — no async runtime dependency
- `Send + Sync` bounds allow sharing across threads

## PayloadEnvelope, TelemetryBatch, TransportMetadata, BackpressureSignal

These types are defined in the shared `telemetry-types` crate. See `crates/telemetry-types/` for authoritative definitions.

```rust
// telemetry-types provides:
use telemetry_types::{PayloadEnvelope, TelemetryBatch, TransportMetadata, BackpressureSignal};
```

- `PayloadEnvelope`: Wraps transport_metadata, propagation_metadata, and a TelemetryBatch
- `TelemetryBatch`: Contains resource, traces, metrics, logs; constructor rejects all-empty batches
- `TransportMetadata`: Timestamp, content-type, encoding hints
- `BackpressureSignal`: Flow control signal with retry-after hint

## DeliveryMode

```rust
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryMode {
    FireAndForget,
    RequestResponse,
    Batch,
    Streaming,
}
```

- Non-exhaustive; concrete transports do not add variants here (contract boundary)
- Returned as a value from `Transport::send()` — not an associated type

## TransportResult / TransportError

```rust
pub type TransportResult<T> = Result<T, TransportError>;

#[non_exhaustive]
pub enum TransportError {
    Timeout,
    Unavailable,
    Backpressure(BackpressureSignal),
    PayloadTooLarge,
    UnsupportedTransport,
}
```

- `TransportError` implements `std::error::Error` and `std::fmt::Display` manually

## BackpressureSignal

Defined in `telemetry-types`. Used by `TransportError::Backpressure`.

```rust
use telemetry_types::BackpressureSignal;

pub struct BackpressureSignal {
    pub retry_after: Option<Duration>,
}
```

## Carrier Abstraction (from AS-01, re-exported by AS-02)

```rust
pub trait Injector {
    fn set(&mut self, key: &str, value: &str);
}

pub trait Extractor {
    fn get(&self, key: &str) -> Option<&str>;
}
```

- AS-02 re-exports these traits from AS-01 as contract dependencies
- MapCarrier (from AS-01) is used for mock-based testing in AS-02
- HttpHeaderCarrier and GrpcMetadataCarrier are NOT defined in AS-02 — they belong to child transport binding specs

## Mock-Based Testing Contract

AS-02 tests validate abstract contracts using `MapCarrier` from AS-01 and types from `telemetry-types`:

```rust
// Example test structure (not real code):
#[cfg(test)]
mod tests {
    use super::*;
    use as_01::MapCarrier;
    use telemetry_types::{PayloadEnvelope, TelemetryBatch};

    #[test]
    fn telemetry_batch_rejects_all_empty() {
        let batch = TelemetryBatch::new(
            resource.clone(),
            vec![],  // no traces
            vec![],  // no metrics
            vec![],  // no logs
        );
        assert!(batch.is_err());
    }

    #[test]
    fn transport_contract_mock_roundtrip() {
        let mock = MockTransport::new();
        let envelope = PayloadEnvelope { ... };
        let result = mock.send(envelope).await;
        assert!(result.is_ok());
    }
}
```

## Carrier Implementation Ownership

| Carrier | Owner | Location |
|---------|-------|----------|
| Injector trait | AS-01 | Context propagation spec |
| Extractor trait | AS-01 | Context propagation spec |
| MapCarrier (test-only) | AS-01 | Context propagation spec |
| HttpHeaderCarrier | Child transport binding spec | e.g., AS-02-HTTP |
| GrpcMetadataCarrier | Child transport binding spec | e.g., AS-02-gRPC |
