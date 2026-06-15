# Data Model: Telemetry Adapter Contracts

**Spec**: [Telemetry Adapter Contracts](spec.md)
**Date**: 2026-06-17

## Entities

### CommonAdapterBase

Shared base trait inherited by both ProviderAdapter and ExporterAdapter for identity and health.

| Method | Signature | Description |
|--------|-----------|-------------|
| `id()` | `&AdapterId` | Returns the adapter's identity |
| `health()` | `HealthReport` | Returns the adapter's current health report |

**Validation**: All adapters must implement `health()` to return a meaningful HealthReport.

---

### LifecycleAdapter

Separate base trait for lifecycle operations (flush, shutdown).

| Method | Signature | Description |
|--------|-----------|-------------|
| `flush()` | `async fn(&self) -> AdapterResult<()>` | Flushes buffered telemetry to the transport layer |
| `shutdown()` | `async fn(&self) -> AdapterResult<()>` | Implicitly calls `flush()` then transitions to Stopped; MAY be overridden |

**Validation**: Uses `&self` for Arc compatibility; concrete adapters own synchronization via interior mutability. `shutdown()` MUST call `flush()` before transitioning to `Stopped`.

---

### TelemetryDelivery

Dedicated trait for telemetry delivery operations; used during multiplexing.

| Method | Signature | Description |
|--------|-----------|-------------|
| `deliver()` | `async fn(&self, PayloadEnvelope) -> AdapterResult<()>` | Delivers a payload to this adapter |

**Validation**: Uses `&self` (consistent with all adapter traits) for `Arc` compatibility.

---

### ProviderAdapter

Trait for provider-side telemetry operations.

| Method | Signature | Description |
|--------|-----------|-------------|
| `initialize()` | `async fn(&self) -> AdapterResult<()>` | Prepares the adapter for operation |
| `start()` | `async fn(&self) -> AdapterResult<()>` | Starts the adapter |
| `stop()` | `async fn(&self) -> AdapterResult<()>` | Stops the adapter gracefully |

**Validation**: All methods use `&self` for Arc compatibility; concrete adapters own via interior mutability.

**Relationships**: Inherits from `CommonAdapterBase + LifecycleAdapter + TelemetryDelivery`.

---

### ExporterAdapter

Trait for exporter-side telemetry operations.

| Method | Signature | Description |
|--------|-----------|-------------|
| `initialize()` | `async fn(&self) -> AdapterResult<()>` | Prepares the adapter for operation |
| `start()` | `async fn(&self) -> AdapterResult<()>` | Starts the adapter |
| `stop()` | `async fn(&self) -> AdapterResult<()>` | Stops the adapter gracefully |

**Validation**: All methods use `&self` for Arc compatibility; concrete adapters own via interior mutability.

**Relationships**: Inherits from `CommonAdapterBase + LifecycleAdapter + TelemetryDelivery`.

---

### Adapter (common supertrait)

Convenience supertrait combining all base concerns for registry storage.

```rust
pub trait Adapter: CommonAdapterBase + LifecycleAdapter + TelemetryDelivery + Send + Sync {}
```

---

### AdapterRegistry

Manages adapter registration, lookup, and lifecycle orchestration.

| Method | Signature | Description |
|--------|-----------|-------------|
| `register()` | `fn(AdapterId, Arc<dyn Adapter>) -> AdapterResult<()>` | Registers an adapter; rejects duplicates |
| `get()` | `fn(&AdapterId) -> Option<Arc<dyn Adapter>>` | Returns a registered adapter by ID (shared ownership) |
| `contains()` | `fn(&AdapterId) -> bool` | Checks if an adapter ID is registered |
| `list()` | `fn() -> Vec<AdapterId>` | Lists all registered adapter IDs |
| `freeze()` | `fn()` | Freezes the registry; no further mutations allowed |

**Validation**:
- Mutable until `freeze()` is called; post-freeze mutations return `AdapterError::Frozen`
- Registration of duplicate `AdapterId` returns `AdapterError::AlreadyRegistered`
- Thread-safe lookup after freeze via `RwLock`; registration only during single-threaded bootstrap phase
- Storage uses `Arc<dyn Adapter + Send + Sync>` for shared ownership

---

### AdapterLifecycle

State machine with canonical states and explicit transition matrix.

| State | Description |
|-------|-------------|
| `Registered` | Adapter has been registered but not initialized |
| `Initialized` | Adapter has been initialized and is ready to start |
| `Started` | Adapter is actively processing telemetry |
| `Stopping` | Adapter is in the process of stopping |
| `Stopped` | Adapter has stopped operationally; resources retained |
| `Shutdown` | Adapter has fully shut down; resources released; terminal state |

**Transition Matrix**:

| From \ To | Registered | Initialized | Started | Stopping | Stopped | Shutdown |
|-----------|------------|-------------|---------|----------|---------|----------|
| Registered | - | initialize() | - | - | - | ✔ (startup failure) |
| Initialized | - | - | start() | - | - | ✔ (startup failure) |
| Started | - | - | - | stop() | - | - |
| Stopping | - | - | - | - | ✔ (after flush) | ✔ (flush failed) |
| Stopped | - | - | - | - | - | ✔ (final) |
| Shutdown | - | - | - | - | - | - |

**Validation**: Invalid transitions return `AdapterError::InvalidTransition { from, to }`.

---

### AdapterId

Strongly typed value object for adapter registration identity.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new()` | `fn(String) -> Self` | Creates a new AdapterId from a string |
| `as_str()` | `fn(&str)` | Returns the underlying string reference |

**Validation**:
- No empty-string IDs (minimum 1 character)
- Display and FromStr implementations for serialization
- Duplicate registration returns `AdapterError::AlreadyRegistered`

---

### HealthReport

Structured health information returned by adapters.

| Field | Type | Description |
|-------|------|-------------|
| `status` | `AdapterHealth` | Current health status |
| `reason` | `String` | Human-readable explanation of health status |
| `timestamp` | `SystemTime` | When the health status was determined |

**Validation**: Default reason is `""`; timestamp defaults to `SystemTime::UNIX_EPOCH`.

---

### AdapterHealth

Typed status model for adapter health reporting.

| Variant | Description |
|---------|-------------|
| `Healthy` | Adapter is operating normally |
| `Degraded` | Adapter is operating but with reduced functionality |
| `Unhealthy` | Adapter is not operating; requires intervention |
| `Unknown` | Adapter health status cannot be determined |

**Validation**: Default variant is `Unknown`; adapters MUST return a meaningful status.

---

### AdapterResult / AdapterError

Canonical result type for adapter operations.

```rust
type AdapterResult<T> = Result<T, AdapterError>;
```

**AdapterError Variants**:

| Variant | Description |
|---------|-------------|
| `InvalidTransition { from: State, to: State }` | Invalid lifecycle state transition |
| `AlreadyRegistered(AdapterId)` | Duplicate adapter registration |
| `Frozen` | Registry mutation attempted after freeze |
| `InitializationFailed(String)` | Adapter initialization failure |
| `FlushFailed(String)` | Adapter flush failure |
| `ShutdownFailed(String)` | Adapter shutdown failure |
| `DeliveryFailed(Vec<(AdapterId, String)>)` | All adapters failed during multiplexed delivery |
| `PartialDelivery(Vec<(AdapterId, String)>)` | Some adapters failed; others succeeded |

---

### Mapping Contracts

Bidirectional entity-specific mapping contracts.

**TraceMappingContract**:
| Method | Signature | Description |
|--------|-----------|-------------|
| `to_otel()` | `fn(&Trace) -> OtelTrace` | Maps canonical Trace to OpenTelemetry trace |
| `from_otel()` | `fn(OtelTrace) -> Trace` | Maps OpenTelemetry trace to canonical Trace |

**SpanMappingContract**:
| Method | Signature | Description |
|--------|-----------|-------------|
| `to_otel()` | `fn(&Span) -> OtelSpan` | Maps canonical Span to OpenTelemetry span |
| `from_otel()` | `fn(OtelSpan) -> Span` | Maps OpenTelemetry span to canonical Span |

**MetricMappingContract**:
| Method | Signature | Description |
|--------|-----------|-------------|
| `to_otel()` | `fn(&Metric) -> OtelMetric` | Maps canonical Metric to OpenTelemetry metric |
| `from_otel()` | `fn(OtelMetric) -> Metric` | Maps OpenTelemetry metric to canonical Metric |

**LogRecordMappingContract**:
| Method | Signature | Description |
|--------|-----------|-------------|
| `to_otel()` | `fn(&LogRecord) -> OtelLogRecord` | Maps canonical LogRecord to OpenTelemetry log record |
| `from_otel()` | `fn(OtelLogRecord) -> LogRecord` | Maps OpenTelemetry log record to canonical LogRecord |

**ResourceMappingContract**:
| Method | Signature | Description |
|--------|-----------|-------------|
| `to_otel()` | `fn(&Resource) -> OtelResource` | Maps canonical Resource to OpenTelemetry resource |
| `from_otel()` | `fn(OtelResource) -> Resource` | Maps OpenTelemetry resource to canonical Resource |

## Concurrency Model

- All adapter traits use `&self` receivers — concrete adapters own synchronization via interior mutability
- AdapterRegistry storage: `RwLock<HashMap<AdapterId, Arc<dyn Adapter>>>` — thread-safe read after freeze
- Concrete adapters are responsible for their own internal state synchronization (Mutex, RwLock, atomics, etc.)
- All lifecycle operations (initialize, start, stop, flush, shutdown) are callable through `Arc<dyn Adapter>`
- Registry does not own or manage adapter-level synchronization

## Relationships

```text
CommonAdapterBase (id, health)    LifecycleAdapter (flush, shutdown)    TelemetryDelivery (deliver)
         \                              |                                    /
          \                             |                                   /
           \                            |                                  /
             ProviderAdapter ─────────────────────────────────────────────
             ExporterAdapter ─────────────────────────────────────────────
                          (both inherit all three bases)

All adapter traits: &self receivers, object-safe, Send + Sync

AdapterRegistry
    ├── stores: HashMap<AdapterId, Arc<dyn Adapter>>
    ├── state: Mutable | Frozen
    └── concurrency: RwLock<HashMap<...>> for reads; Arc for shared ownership

Concrete Adapter
    ├── implements: ProviderAdapter or ExporterAdapter
    ├── synchronization: interior mutability (Mutex, RwLock, atomics)
    ├── lifecycle: internal AdapterLifecycle behind lock
    └── storage: stored as Arc<dyn Adapter> in registry

AdapterLifecycle
    ├── states: Registered → Initialized → Started → Stopping → Stopped → Shutdown
    ├── transitions: explicit matrix, Registered→Shutdown and Initialized→Shutdown allowed
    └── Stopped = resources retained, Shutdown = terminal

AdapterResult<T> = Result<T, AdapterError>

AdapterHealth = Healthy | Degraded | Unhealthy | Unknown
HealthReport = { status: AdapterHealth, reason: String, timestamp: SystemTime }

TelemetryDelivery.deliver()  ← used by multiplexing contract

TraceMappingContract → Trace ↔ OpenTelemetry
SpanMappingContract → Span ↔ OpenTelemetry
MetricMappingContract → Metric ↔ OpenTelemetry
LogRecordMappingContract → LogRecord ↔ OpenTelemetry
ResourceMappingContract → Resource ↔ OpenTelemetry
```
