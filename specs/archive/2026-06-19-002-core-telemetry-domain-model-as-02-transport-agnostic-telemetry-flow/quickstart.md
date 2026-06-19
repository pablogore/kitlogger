# Quickstart: Transport-Agnostic Telemetry Flow

## Prerequisites

- Rust 2021 edition
- AS-01 (context propagation) types available in workspace
- No concrete transport implementations required (AS-02 is abstract contracts only)

## Validation Scenarios

### Scenario 1 — TelemetryBatch Rejects All-Empty Batches

**Purpose**: Verify FR-010 — TelemetryBatch constructor rejects empty batches.

```rust
use telemetry_types::TelemetryBatch;

let batch = TelemetryBatch::new(resource, vec![], vec![], vec![]);
assert!(batch.is_err(), "Empty batch must be rejected");
```

**Expected**: `TelemetryBatch::new()` returns `Err` when traces, metrics, and logs are all empty.

### Scenario 2 — DeliveryMode Enum Roundtrip

**Purpose**: Verify DeliveryMode variants are serializable and comparable.

```rust
use as_02::DeliveryMode;

let modes = vec![
    DeliveryMode::FireAndForget,
    DeliveryMode::RequestResponse,
    DeliveryMode::Batch,
    DeliveryMode::Streaming,
];

for mode in &modes {
    let serialized = serde_json::to_string(mode)?;
    let deserialized: DeliveryMode = serde_json::from_str(&serialized)?;
    assert_eq!(*mode, deserialized);
}
```

**Expected**: All four DeliveryMode variants serialize and deserialize without data loss.

### Scenario 3 — PayloadEnvelope Serde Roundtrip

**Purpose**: Verify PayloadEnvelope serialization/deserialization using MapCarrier from AS-01.

```rust
use telemetry_types::{PayloadEnvelope, TelemetryBatch, TransportMetadata};
use as_01::MapCarrier;

let envelope = PayloadEnvelope {
    transport_metadata: TransportMetadata::now(),
    propagation_metadata: PropagationMetadata::default(),
    payload: TelemetryBatch::new(resource, vec![span], vec![], vec![])?,
};

let json = serde_json::to_string(&envelope)?;
let deserialized: PayloadEnvelope = serde_json::from_str(&json)?;
assert_eq!(envelope.payload.traces.len(), deserialized.payload.traces.len());
```

**Expected**: PayloadEnvelope roundtrips through JSON (or any serde format) identically.

### Scenario 4 — TransportError Is Non-Exhaustive

**Purpose**: Verify callers can handle known variants and a wildcard.

```rust
use as_02::{TransportError, TransportResult};

fn handle(result: TransportResult<DeliveryMode>) {
    match result {
        Ok(mode) => println!("Delivered as {:?}", mode),
        Err(TransportError::Timeout) => eprintln!("timed out"),
        Err(TransportError::Backpressure(signal)) => {
            if let Some(duration) = signal.retry_after {
                // apply backoff
            }
        }
        Err(_) => eprintln!("other transport error"),
    }
}
```

**Expected**: Match compiles with wildcard arm; new variants added by child transport specs do not break existing callers.

### Scenario 5 — Transport Trait Is Runtime-Independent

**Purpose**: Verify Transport trait compiles with only std::future::Future (no Tokio dependency).

```rust
use as_02::{Transport, TransportResult, DeliveryMode};
use telemetry_types::PayloadEnvelope;
use async_trait::async_trait;

struct MockTransport;

#[async_trait]
impl Transport for MockTransport {
    async fn send(&self, envelope: PayloadEnvelope) -> TransportResult<DeliveryMode> {
        Ok(DeliveryMode::FireAndForget)
    }
}
```

**Expected**: Compiles without any async runtime dependency in Cargo.toml.

### Scenario 6 — Mock-Based Transport Contract Validation

**Purpose**: Verify abstract contract behavior via mocks (no concrete transports).

```rust
use as_02::{Transport, TransportResult, DeliveryMode};
use telemetry_types::{PayloadEnvelope, TransportMetadata};
use as_01::MapCarrier;

#[tokio::test]
async fn mock_transport_roundtrip() {
    let transport = MockTransport;
    let envelope = PayloadEnvelope {
        transport_metadata: TransportMetadata::now(),
        propagation_metadata: PropagationMetadata::default(),
        payload: valid_batch(),
    };
    let result = transport.send(envelope).await;
    assert!(result.is_ok());
}
```

**Expected**: Test passes using a mock transport; no HTTP, gRPC, or other concrete transport required.

## Running Tests

```bash
# Run all AS-02 contract validation tests
cargo test --lib -p as-02

# Run all shared types tests
cargo test -p telemetry-types

# Run with specific test name
cargo test --lib -p as-02 -- test_mock_transport_contract

# Run all workspace tests
cargo test --workspace
```

## References

- [Data Model](../data-model.md) — Full entity definitions and validation rules
- [Transport Contract API](../contracts/transport-api.md) — Trait signatures and type definitions
- [Tasks](../tasks.md) — Implementation tasks
- [Spec](../spec.md) — Feature specification with requirements and success criteria
