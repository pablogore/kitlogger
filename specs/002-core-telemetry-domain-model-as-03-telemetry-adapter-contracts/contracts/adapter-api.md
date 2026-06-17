# Adapter Contract API

**Spec**: [Telemetry Adapter Contracts](../spec.md)
**Date**: 2026-06-16

> Shared canonical types (PayloadEnvelope, TelemetryBatch, TransportMetadata, BackpressureSignal) are owned by the `telemetry-types` crate per ADR-007. AS-03 imports them from `telemetry-types`.

## CommonAdapterBase

Identity and health; no lifecycle concerns. All traits in AS-03 MUST be object-safe.

```rust
#[async_trait]
pub trait CommonAdapterBase: Send + Sync {
    fn id(&self) -> &AdapterId;
    fn health(&self) -> HealthReport;
}
```

## LifecycleAdapter

Lifecycle operations (flush, shutdown) separated from identity/health. Uses `&self` for compatibility with `Arc<dyn Adapter>` in the registry. Concrete adapters own synchronization via interior mutability.

```rust
#[async_trait]
pub trait LifecycleAdapter: Send + Sync {
    async fn flush(&self) -> AdapterResult<()>;
    async fn shutdown(&self) -> AdapterResult<()>;
}
```

No default `shutdown()` implementation is provided because shutdown semantics depend on the concrete adapter's own state management. Concrete adapters SHOULD call `flush()` then transition to `Stopped` as part of their shutdown sequence. The `&self` receiver means concrete adapters must use interior mutability (e.g., `Mutex<AdapterLifecycle>`) for lifecycle state transitions.

## TelemetryDelivery

Dedicated trait for delivery operations. Defines the operation executed during multiplexing. Uses `PayloadEnvelope` from the shared `telemetry-types` crate.

```rust
use telemetry_types::PayloadEnvelope;

#[async_trait]
pub trait TelemetryDelivery: Send + Sync {
    async fn deliver(&self, envelope: PayloadEnvelope) -> AdapterResult<()>;
}
```

Uses `&self` (consistent with all adapter traits) for compatibility with `Arc`-based shared ownership in the registry.

## ProviderAdapter

Provider-side operations; inherits all three base traits.

```rust
#[async_trait]
pub trait ProviderAdapter: CommonAdapterBase + LifecycleAdapter + TelemetryDelivery {
    async fn initialize(&self) -> AdapterResult<()>;
    async fn start(&self) -> AdapterResult<()>;
    async fn stop(&self) -> AdapterResult<()>;
}
```

## ExporterAdapter

Exporter-side operations; inherits all three base traits.

```rust
#[async_trait]
pub trait ExporterAdapter: CommonAdapterBase + LifecycleAdapter + TelemetryDelivery {
    async fn initialize(&self) -> AdapterResult<()>;
    async fn start(&self) -> AdapterResult<()>;
    async fn stop(&self) -> AdapterResult<()>;
}
```

## Common Adapter Abstraction

For registry storage, a common supertrait combining all base concerns:

```rust
pub trait Adapter: CommonAdapterBase + LifecycleAdapter + TelemetryDelivery + Send + Sync {}
```

## AdapterLifecycle

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    Registered,
    Initialized,
    Started,
    Stopping,
    Stopped,
    Shutdown,
}

pub struct AdapterLifecycle {
    state: LifecycleState,
}

impl AdapterLifecycle {
    pub fn new() -> Self;
    pub fn state(&self) -> LifecycleState;
    pub fn transition(&mut self, to: LifecycleState) -> AdapterResult<()>;
}
```

Valid transitions (returns `AdapterError::InvalidTransition` for illegal transitions):

| From | To | Method | Notes |
|------|----|--------|-------|
| Registered | Initialized | `transition(Initialized)` | Normal startup |
| Registered | Shutdown | `transition(Shutdown)` | Startup failure before init |
| Initialized | Started | `transition(Started)` | Normal startup |
| Initialized | Shutdown | `transition(Shutdown)` | Startup failure after init |
| Started | Stopping | `transition(Stopping)` | Graceful stop |
| Stopping | Stopped | `transition(Stopped)` | After flush completes |
| Stopping | Shutdown | `transition(Shutdown)` | Flush failed during stop |
| Stopped | Shutdown | `transition(Shutdown)` | Final transition |

Semantic distinction: **Stopped** = operationally stopped, resources retained (can inspect state).
**Shutdown** = resources released, terminal state (no further operations possible).

## AdapterRegistry

Thread-safe registry with Arc-based storage. Stores a common `Arc<dyn Adapter>` supporting both ProviderAdapter and ExporterAdapter. Returns `Arc<dyn Adapter>` for shared ownership.

```rust
pub struct AdapterRegistry {
    adapters: RwLock<HashMap<AdapterId, Arc<dyn Adapter>>>,
    frozen: bool,
}

impl AdapterRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, id: AdapterId, adapter: Arc<dyn Adapter>) -> AdapterResult<()>;
    pub fn get(&self, id: &AdapterId) -> Option<Arc<dyn Adapter>>;
    pub fn contains(&self, id: &AdapterId) -> bool;
    pub fn list(&self) -> Vec<AdapterId>;
    pub fn freeze(&mut self);
}
```

## AdapterId

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AdapterId(String);

impl AdapterId {
    pub fn new(id: impl Into<String>) -> AdapterResult<Self>;
    pub fn as_str(&self) -> &str;
}

impl Display for AdapterId;
impl FromStr for AdapterId;
```

## HealthReport

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthReport {
    pub status: AdapterHealth,
    pub reason: String,
    pub timestamp: SystemTime,
}
```

## AdapterHealth

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl Default for AdapterHealth;  // Defaults to Unknown
```

## AdapterResult / AdapterError

```rust
pub type AdapterResult<T> = Result<T, AdapterError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    InvalidTransition { from: LifecycleState, to: LifecycleState },
    AlreadyRegistered(AdapterId),
    Frozen,
    InitializationFailed(String),
    FlushFailed(String),
    ShutdownFailed(String),
    DeliveryFailed(Vec<(AdapterId, String)>),
    PartialDelivery(Vec<(AdapterId, String)>),
}

impl Display for AdapterError;
impl std::error::Error for AdapterError;
```

## Mapping Contracts

### TraceMappingContract

```rust
pub trait TraceMappingContract {
    fn to_otel(&self, trace: &Trace) -> OtelTrace;
    fn from_otel(&self, otel: OtelTrace) -> Trace;
}
```

### SpanMappingContract

```rust
pub trait SpanMappingContract {
    fn to_otel(&self, span: &Span) -> OtelSpan;
    fn from_otel(&self, otel: OtelSpan) -> Span;
}
```

### MetricMappingContract

```rust
pub trait MetricMappingContract {
    fn to_otel(&self, metric: &Metric) -> OtelMetric;
    fn from_otel(&self, otel: OtelMetric) -> Metric;
}
```

### LogRecordMappingContract

```rust
pub trait LogRecordMappingContract {
    fn to_otel(&self, log: &LogRecord) -> OtelLogRecord;
    fn from_otel(&self, otel: OtelLogRecord) -> LogRecord;
}
```

### ResourceMappingContract

```rust
pub trait ResourceMappingContract {
    fn to_otel(&self, resource: &Resource) -> OtelResource;
    fn from_otel(&self, otel: OtelResource) -> Resource;
}
```

## Multiplexing Contract

Delivers telemetry to multiple adapters via TelemetryDelivery trait. Uses `Arc`-based access from registry. PayloadEnvelope is from `telemetry-types`.

```rust
use telemetry_types::PayloadEnvelope;

pub async fn deliver_to_all(
    registry: &AdapterRegistry,
    ids: &[AdapterId],
    envelope: PayloadEnvelope,
) -> AdapterResult<()> {
    let mut failures = Vec::new();
    for id in ids {
        if let Some(adapter) = registry.get(id) {
            match adapter.deliver(envelope.clone()).await {
                Ok(()) => continue,
                Err(e) => failures.push((id.clone(), e.to_string())),
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else if failures.len() < ids.len() {
        Err(AdapterError::PartialDelivery(failures))
    } else {
        Err(AdapterError::DeliveryFailed(failures))
    }
}
```
