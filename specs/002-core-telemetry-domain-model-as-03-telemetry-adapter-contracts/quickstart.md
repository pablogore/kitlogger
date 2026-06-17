# Quickstart: Telemetry Adapter Contracts

**Spec**: [Telemetry Adapter Contracts](spec.md)
**Date**: 2026-06-17

## Prerequisites

- Rust (edition 2021) with cargo test
- Parent crate providing canonical domain types (Span, Metric, LogRecord, Resource)
- `telemetry-types` crate for PayloadEnvelope, TelemetryBatch, TransportMetadata, BackpressureSignal

## Validation Scenarios

### Scenario 1: Mock Adapter Implements CommonAdapterBase and LifecycleAdapter

Define a mock adapter that implements CommonAdapterBase and LifecycleAdapter, and verify health() returns HealthReport.

```rust
struct MockAdapter {
    id: AdapterId,
    health: AdapterHealth,
}

#[async_trait]
impl CommonAdapterBase for MockAdapter {
    fn id(&self) -> &AdapterId { &self.id }
    fn health(&self) -> HealthReport {
        HealthReport {
            status: self.health.clone(),
            reason: String::new(),
            timestamp: SystemTime::now(),
        }
    }
}

#[async_trait]
impl LifecycleAdapter for MockAdapter {
    async fn flush(&self) -> AdapterResult<()> { Ok(()) }
    async fn shutdown(&self) -> AdapterResult<()> { Ok(()) }
}
```

**Expected**: MockAdapter compiles and implements CommonAdapterBase + LifecycleAdapter. All methods use `&self`, enabling direct invocation through `Arc<dyn Adapter>`. `health()` returns a HealthReport with the configured status.

### Scenario 2: Lifecycle Transition Validation

Create an AdapterLifecycle and verify valid transitions succeed (including startup failure paths) and invalid transitions return `AdapterError::InvalidTransition`.

```rust
let mut lifecycle = AdapterLifecycle::new();
assert_eq!(lifecycle.state(), LifecycleState::Registered);

// Valid: Registered -> Initialized
lifecycle.transition(LifecycleState::Initialized).unwrap();
assert_eq!(lifecycle.state(), LifecycleState::Initialized);

// Valid: Initialized -> Shutdown (startup failure)
lifecycle.transition(LifecycleState::Shutdown).unwrap();
assert_eq!(lifecycle.state(), LifecycleState::Shutdown);

// Invalid: Shutdown -> anything (terminal)
lifecycle.transition(LifecycleState::Started)
    .expect_err("Should reject transition from terminal state");
```

**Expected**: Invalid transitions return typed error; Registered→Shutdown and Initialized→Shutdown are valid.

### Scenario 3: Registry Freeze Behavior

Create an AdapterRegistry, register adapters, freeze, then verify post-freeze registration is rejected.

```rust
let mut registry = AdapterRegistry::new();
let id = AdapterId::new("test-adapter").unwrap();
let adapter: Arc<dyn Adapter> = Arc::new(MockAdapter::new(id.clone()));
registry.register(id.clone(), adapter).unwrap();
registry.freeze();
let another_id = AdapterId::new("another").unwrap();
let another: Arc<dyn Adapter> = Arc::new(MockAdapter::new(another_id.clone()));
let result = registry.register(another_id, another);
assert!(matches!(result, Err(AdapterError::Frozen)));
```

**Expected**: Post-freeze registration returns `AdapterError::Frozen`.

### Scenario 4: Duplicate Registration Rejection

Register the same AdapterId twice and verify the second registration is rejected.

```rust
let mut registry = AdapterRegistry::new();
let id = AdapterId::new("dup-adapter").unwrap();
let adapter: Arc<dyn Adapter> = Arc::new(MockAdapter::new(id.clone()));
registry.register(id.clone(), adapter).unwrap();
let duplicate: Arc<dyn Adapter> = Arc::new(MockAdapter::new(id.clone()));
let result = registry.register(id.clone(), duplicate);
assert!(matches!(result, Err(AdapterError::AlreadyRegistered(_))));
```

**Expected**: Duplicate registration returns `AdapterError::AlreadyRegistered`.

### Scenario 5: Adapter Multiplexing with Partial Failure

Deliver telemetry to multiple adapters; one fails, others succeed. Verify aggregate error.

```rust
use telemetry_types::PayloadEnvelope;

let mut registry = AdapterRegistry::new();
let id1 = AdapterId::new("adapter-1").unwrap();
let id2 = AdapterId::new("adapter-2").unwrap();
registry.register(id1.clone(), Arc::new(FailingAdapter)).unwrap();
registry.register(id2.clone(), Arc::new(OkAdapter)).unwrap();
let envelope = PayloadEnvelope { /* ... */ };
let result = deliver_to_all(&registry, &[id1, id2], envelope).await;
assert!(matches!(result, Err(AdapterError::PartialDelivery(_))));
```

**Expected**: `AdapterError::PartialDelivery` with failures from adapter-1 only.

### Scenario 6: Mapping Contract Bidirectionality

Implement TraceMappingContract and verify both directions.

```rust
struct MockTraceMapper;
impl TraceMappingContract for MockTraceMapper {
    fn to_otel(&self, trace: &Trace) -> OtelTrace {
        // Mock mapping
        OtelTrace { /* ... */ }
    }
    fn from_otel(&self, otel: OtelTrace) -> Trace {
        // Mock mapping
        Trace { /* ... */ }
    }
}
```

**Expected**: Both `to_otel()` and `from_otel()` are implemented; roundtrip preserves identity.

## Running Validation

```bash
# Run all workspace tests (includes telemetry-types and AS-03)
cargo test --workspace

# Run AS-03 adapter contract tests only
cargo test -p telemetry-adapter-contracts

# Run telemetry-types tests only
cargo test -p telemetry-types
```

All 6 scenarios are validated through unit tests in the crate's test suite.
