# Research: Telemetry Adapter Contracts

**Spec**: [Telemetry Adapter Contracts](spec.md)
**Date**: 2026-06-17

## Research Decisions

### AD-1: Async Trait Pattern

**Decision**: Use `#[async_trait::async_trait]` macro for all adapter traits (CommonAdapterBase, LifecycleAdapter, TelemetryDelivery, ProviderAdapter, ExporterAdapter) (consistent with AS-02). All traits MUST be object-safe for `dyn Trait` usage. All adapter methods use `&self` receiver (no `&mut self`).

**Rationale**: The canonical domain model (AS-01 and AS-02) already uses `async-trait` crate for async trait definitions. Using the same pattern ensures consistency across the capability. `async-trait` is compatible with the declared Tokio runtime and does not introduce a conflicting async runtime dependency. Object safety is required because the registry stores `Arc<dyn Adapter>` and must be able to call methods through trait objects. Using `&self` on all methods ensures all lifecycle operations are callable through `Arc<dyn Adapter>`, eliminating the `&mut self` ambiguity with `Arc` shared ownership.

**Alternatives considered**:
- N/A — following established AS-02 convention

---

### AD-2: Thread-Safe Registry Pattern

**Decision**: Implement AdapterRegistry using `std::sync::RwLock<HashMap<AdapterId, Arc<dyn Adapter>>>`. Registration phase enforces freeze semantics via a `frozen` bool guard; lookup phase allows concurrent reads through RwLock. `Arc` provides shared ownership so multiple callers can reference the same adapter post-freeze.

**Rationale**: The spec requires thread-safe lookup after freeze and registration only during bootstrap. `RwLock` provides optimal read concurrency for the lookup-heavy frozen phase. Wrapping in a dedicated struct enforces the freeze contract at the type level (expose `&Self` for reads, `&mut Self` for writes during bootstrap). `Arc` is required for shared ownership across threads after freeze — `Box` would require exclusive access.

**Alternatives considered**:
- `Mutex<HashMap<...>>`: Lower read concurrency; acceptable but RwLock is more appropriate for read-heavy workloads
- `parking_lot::RwLock`: Better performance but adds undeclared dependency; stick with std::sync
- `Box<dyn Adapter>`: Exclusive ownership only; cannot share adapters across threads post-freeze

---

### AD-3: Error Handling Approach

**Decision**: Manual `std::error::Error` and `Display` implementations for AdapterError with a typed lifecycle error hierarchy.

**Rationale**: Consistent with AS-02's approach to avoid undeclared dependencies (thiserror is not in tech-stack.yaml). Manual impls provide full control over error formatting and are straightforward for a finite set of error variants.

**Alternatives considered**:
- `thiserror` derive: Not declared in tech-stack.yaml; would require technology clarification
- `anyhow`: Not appropriate for library contract types; designed for application-level error handling

---

### AD-4: Health Reporting Mechanism

**Decision**: Add `fn health(&self) -> HealthReport` as a required method on CommonAdapterBase trait. HealthReport is a struct containing `AdapterHealth + String reason + SystemTime timestamp`. Health is a polling-based check; the adapter implementation determines the health status synchronously.

**Rationale**: Polling-based health checks are the most common pattern in Rust telemetry systems and avoid the complexity of push-based health reporting (subscription management, heartbeat timeouts). Required method ensures every adapter must implement health reporting. The struct bundling status + reason + timestamp provides richer diagnostic information than a bare enum.

**Alternatives considered**:
- `fn health(&self) -> AdapterHealth`: Insufficient diagnostic information; caller cannot distinguish "just checked" from "stale"
- Push-based health with subscription: Over-engineered for a contract layer; more suitable for concrete implementations
- Optional health (default implementation returning Unknown): Weaker contract; explicit implementation ensures conscious health design

---

### AD-5: Lifecycle Transition Matrix

**Decision**: Define an explicit `Transition` type that validates allowed transitions and rejects invalid ones with a typed `TransitionError`. The matrix is encoded as match arms. Allowed transitions include `Registered→Shutdown` and `Initialized→Shutdown` for startup failure scenarios. Semantics: `Stopped` retains resources, `Shutdown` releases resources and is terminal.

**Rationale**: An explicit transition matrix ensures invalid transitions are caught at runtime with clear error types. Encoding in match arms provides compiler-enforced exhaustiveness checking when new states are added. Startup failure transitions (Registered→Shutdown, Initialized→Shutdown) are needed because an adapter may fail before reaching Started. The Stopped/Shutdown distinction allows operational stop with retained resources vs. full terminal shutdown.

**Alternatives considered**:
- `From<State> for State` partial impls: Less explicit; harder to test exhaustively
- State machine library (e.g., `rust-machine`): Undeclared dependency; not worth external dependency for a simple linear machine

---

### AD-6: Mapping Contract Pattern

**Decision**: Define entity-specific mapping traits with bidirectional conversion methods (e.g., `fn to_otel(&self) -> OtelSpan`, `fn from_otel(otel: OtelSpan) -> Self`).

**Rationale**: Trait-based mapping contracts allow concrete adapter implementations to provide their own mapping logic while the contract ensures the bidirectional contract is enforced at the type level.

**Alternatives considered**:
- Single generic `Mapper<T, U>` trait: Loses entity-specific semantics
- Free functions: No type-level contract enforcement

---

### AD-7: Multiplexing Failure Model

**Decision**: Adapter multiplexing iterates through all matching adapters, collects per-adapter `AdapterResult`, and returns an aggregated `AdapterError::PartialDelivery` containing individual failures.

**Rationale**: Best-effort semantics (don't stop on first failure) with aggregate error reporting allows callers to inspect which adapters failed while still delivering to all remaining adapters. This matches the "best-effort + aggregate failures" clarification.

**Alternatives considered**:
- Fail-fast (first error aborts all): Violates spec requirement
- Ignore failures: No observability into delivery problems
- Callback-based reporting: Overly complex for the contract layer

---

### AD-8: Duplicate Registration Behavior

**Decision**: `register()` returns `Err(AdapterError::AlreadyRegistered(AdapterId))` when the AdapterId already exists in the registry.

**Rationale**: Typed error return allows callers to handle duplicates specifically (e.g., logging, recovery) rather than silently ignoring or replacing. Replace semantics are available via an explicit `replace()` method if needed.

**Alternatives considered**:
- Silent replace: Risks accidental adapter overwrite with no observable signal
- No-op / ignore: Silent failure; caller has no visibility into registration issues

---

### AD-9: AdapterId Type

**Decision**: `AdapterId` is a newtype wrapper over `String` (internally stored) with `Display`, `Deref<Target=str>`, and `FromStr` implementations.

**Rationale**: Human-readable identifiers are essential for configuration and debugging. String-based identity allows configuration files (AS-04) to reference adapters by name. Newtype wrapper prevents mixing adapter IDs with arbitrary strings at the type level.

**Alternatives considered**:
- `Uuid`: Stronger identity guarantees but less human-friendly for configuration
- Integer ID: Opaque and configuration-unfriendly

---

### AD-10: Shutdown Flush Semantics

**Decision**: `shutdown()` method is placed on the `LifecycleAdapter` trait (separate from `CommonAdapterBase`). No default `shutdown()` implementation is provided because shutdown semantics depend on the concrete adapter's own state management. Concrete adapters SHOULD call `flush()` then transition to `Stopped` as part of their shutdown sequence.

**Rationale**: Separating lifecycle operations into their own trait keeps `CommonAdapterBase` focused on identity and health. A default implementation is infeasible because shutdown requires adapter-specific state management (interior mutability, lifecycle state transitions), and an async fn default would break object safety. The "SHOULD" qualifier allows custom shutdown sequences while the behavioral contract in spec.md SC-004 requires shutdown to implicitly invoke flush.

**Alternatives considered**:
- `shutdown()` on CommonAdapterBase: Blurry responsibility boundary; identity/health mixed with lifecycle
- Default shutdown impl: Infeasible with object safety; async trait default methods cannot be object-safe
- Required `flush()` call before `shutdown()`: Weaker contract; caller must remember

---

### AD-11: LifecycleAdapter Trait Separation

**Decision**: Lifecycle operations (flush, shutdown) are placed on a separate `LifecycleAdapter` trait, distinct from `CommonAdapterBase` (identity, health). Both `ProviderAdapter` and `ExporterAdapter` inherit from both `CommonAdapterBase` and `LifecycleAdapter`.

**Rationale**: Keeps the trait hierarchy clean — `CommonAdapterBase` is for identity/health, `LifecycleAdapter` is for lifecycle operations. Concrete adapters compose both. This also makes testing easier (mock lifecycle without implementing full identity).

**Alternatives considered**:
- Single `CommonAdapterBase` with all methods: Blurred responsibilities; harder to mock
- Marker traits only: Less explicit about what each adapter provides

---

### AD-12: TelemetryDelivery Trait

**Decision**: Telemetry delivery operations are placed on a dedicated `TelemetryDelivery` trait with a `deliver(&self, envelope: PayloadEnvelope) -> AdapterResult<()>` method. Uses `&self` for `Arc` compatibility. `PayloadEnvelope` is imported from `telemetry-types` crate per ADR-007.

**Rationale**: Separating delivery from lifecycle and identity allows the multiplexing contract to operate solely through the `TelemetryDelivery` interface without coupling to lifecycle state. The `&self` signature ensures object safety and compatibility with `Arc<dyn Adapter>` in the registry. PayloadEnvelope is a shared canonical type owned by telemetry-types to avoid peer dependency between AS-02 and AS-03.

**Alternatives considered**:
- `deliver()` on `CommonAdapterBase`: Blurry responsibility; delivery is orthogonal to identity
- `deliver()` with `&mut self`: Incompatible with `Arc` shared ownership; forces exclusive access

---

### AD-13: Arc-Based Registry Storage

**Decision**: `AdapterRegistry` stores `Arc<dyn Adapter>` (where `Adapter` is a supertrait of `CommonAdapterBase + LifecycleAdapter + TelemetryDelivery + Send + Sync`). `get()` returns `Option<Arc<dyn Adapter>>` for shared ownership.

**Rationale**: After freeze, the registry is read-only and multiple callers may need concurrent access to the same adapter. `Arc` provides shared ownership without cloning the adapter itself. Both `ProviderAdapter` and `ExporterAdapter` can be stored through the common `Adapter` supertrait.

**Alternatives considered**:
- `Box<dyn Adapter>`: Exclusive ownership; cannot share across threads post-freeze
- `Rc<dyn Adapter>`: Not thread-safe; incompatible with `Send + Sync` bound
- Bare references (`&'a dyn Adapter`): Lifetime constraints make registry hard to manage

---

### AD-14: Common Adapter Supertrait for Registry

**Decision**: A unified `Adapter` supertrait (`CommonAdapterBase + LifecycleAdapter + TelemetryDelivery + Send + Sync`) serves as the common storage type in `AdapterRegistry`. Both `ProviderAdapter` and `ExporterAdapter` satisfy `Adapter`.

**Rationale**: The registry needs a single homogeneous storage type. A supertrait combining all base concerns provides this without losing access to any base trait methods after downcasting through `Arc`. Both provider and exporter adapters implement the same interfaces at the base level.

**Alternatives considered**:
- `Box<dyn Any>`: Loses type safety; requires manual downcasting
- Separate registries for ProviderAdapter and ExporterAdapter: Heterogeneous lookup requires two registries
- `enum RegistryEntry { Provider(Box<dyn ProviderAdapter>), Exporter(Box<dyn ExporterAdapter>) }`: Explicit but not extensible; adding new adapter types requires enum variants

---

### AD-15: Lifecycle Startup Failure Transitions

**Decision**: `Registered→Shutdown` and `Initialized→Shutdown` are valid transitions, enabling startup failure handling. Shutdown is the terminal state; once in Shutdown, no further transitions are possible.

**Rationale**: An adapter may fail during `initialize()` (before reaching `Initialized`) or during `start()` (before reaching `Started`). In both cases, the adapter should be moved to `Shutdown` rather than left in an intermediate state. This distinction supports startup failure scenarios without requiring the adapter to go through the full Registered→Initialized→Started→Stopping→Stopped→Shutdown chain.

**Alternatives considered**:
- Must go through full chain: Requires unnecessary initialize→start for failed adapters
- `Stopped` as terminal: Loses resource cleanup semantics

---

### AD-16: Stopped vs Shutdown Semantics

**Decision**: `Stopped` retains resources (operationally stopped, can inspect state); `Shutdown` releases all resources (terminal state, no further operations possible).

**Rationale**: The distinction allows graceful temporary suspension (stop + resumable inspection) vs. permanent teardown. Concrete adapter implementations can decide what "retains resources" means (e.g., keep connections open vs. close everything).

**Alternatives considered**:
- Single terminal state: No way to distinguish "ready to resume" from "cleaned up"
- Three states (Stopped, ShuttingDown, Shutdown): Over-engineered for current scope; can be added later

---

### AD-17: HealthReport Struct

**Decision**: `HealthReport` bundles `AdapterHealth status + String reason + SystemTime timestamp`. Returned by `CommonAdapterBase::health()`.

**Rationale**: A bare `AdapterHealth` enum provides insufficient diagnostic information. Adding `reason` (why the adapter is degraded) and `timestamp` (when the status was determined) enables better observability for health polling consumers.

**Alternatives considered**:
- `fn health(&self) -> AdapterHealth`: Missing diagnostic context
- `fn health(&self) -> (AdapterHealth, String, SystemTime)`: Tuple lacks semantic clarity
- `fn health_status(&self) -> AdapterHealth + fn health_reason(&self) -> String`: Two separate calls risk inconsistency between status and reason

---

### AD-18: Object Safety for All Adapter Traits

**Decision**: All public adapter traits (CommonAdapterBase, LifecycleAdapter, TelemetryDelivery, ProviderAdapter, ExporterAdapter) MUST be object-safe. This restricts: no `Self: Sized` bounds on methods, no type parameters on methods (unless defaulted), no `Self` return types, and `TelemetryDelivery::deliver()` uses `&self` not `&mut self`.

**Rationale**: The registry stores `Arc<dyn Adapter>` and must be able to call methods through trait objects. Object safety is the standard Rust mechanism for dynamic dispatch. All public traits in the contracts crate should support `dyn Trait` usage for maximum flexibility.

**Alternatives considered**:
- Generic-based approach with `impl Adapter`: Monomorphization preferred at higher layers; contracts crate must support dynamic dispatch
- `ObjectSafe` marker traits: Additional complexity with no benefit; object safety is a trait-level concern

---

### AD-19: Adapter Registry Supports Both Provider and Exporter

**Decision**: `AdapterRegistry.register()` accepts `Arc<dyn Adapter>` — a common supertrait that both `ProviderAdapter` and `ExporterAdapter` satisfy. The registry does not distinguish between provider and exporter at storage time.

**Rationale**: Runtime type distinction between provider and exporter is unnecessary at the registry level; both share the same base interfaces. Concrete transport binding specs (AS-04, AS-05, etc.) can enforce type safety at their own level. This simplifies the registry to a single storage map.

**Alternatives considered**:
- `enum AdapterKind { Provider, Exporter }` in RegistryEntry: Type-tag adds complexity without clear benefit
- Generic `Registry<T>`: Different concrete types cannot coexist in a single collection

---

### AD-20: Multiplexing via TelemetryDelivery Trait

**Decision**: Multiplexing iterates over target adapters retrieved from the registry, calls `TelemetryDelivery::deliver()` on each, collects failures, and returns either `Ok(())`, `AdapterError::PartialDelivery`, or `AdapterError::DeliveryFailed`.

**Rationale**: Routing multiplexing through `TelemetryDelivery` keeps delivery logic decoupled from lifecycle and identity. The `&self` signature ensures compatibility with `Arc`-shared adapters from the registry. The three-result model (all succeeded, some failed, all failed) provides granular error reporting.

**Alternatives considered**:
- Direct trait method on adapters: Couples delivery to full adapter interface
- Separate multiplexer struct: Additional abstraction without clear benefit for the contract layer

---

### AD-21: LifecycleAdapter Receiver Type (&self)

**Decision**: LifecycleAdapter methods `flush()` and `shutdown()` use `&self` receiver (not `&mut self`). Concrete adapters manage internal state behind synchronization primitives.

**Rationale**: Since adapters are stored as `Arc<dyn Adapter>` in the registry, `&mut self` methods cannot be called through the Arc without exclusive ownership. Using `&self` allows lifecycle operations (flush, shutdown) to be invoked directly through `Arc<dyn Adapter>` returned by `registry.get()`. Concrete adapters own their synchronization — typically wrapping mutable state in `Mutex` or `RwLock`.

**Alternatives considered**:
- `&mut self`: Incompatible with `Arc<dyn Adapter>`; requires caller to obtain exclusive ownership
- Separate lifecycle manager (not on adapter trait): Overhead of indirection; unnecessary since interior mutability solves it

---

### AD-22: Adapter Concurrency Ownership

**Decision**: Concrete adapters own synchronization for their mutable state. The registry does not manage adapter-level concurrency.

**Rationale**: The registry's responsibility is storage and lookup. Concurrency strategy is implementation-specific per adapter (some may use `Mutex`, others `RwLock`, `atomics`, or lock-free structures). Centralizing synchronization in the registry would force all adapters into a single strategy, preventing optimization for specific adapter needs.

**Alternatives considered**:
- `AdapterRegistry` owns synchronization: Forces uniform concurrency strategy; violates separation of concerns
- Mixed (registry for storage, adapter for state): Clean separation; each layer owns what it knows best

---

### AD-23: Registry Storage Model Canonical Form

**Decision**: The canonical storage model is `RwLock<HashMap<AdapterId, Arc<dyn Adapter>>>`.

**Rationale**: `RwLock` allows concurrent reads during the frozen phase. `Arc` enables shared ownership of adapters across threads. `HashMap` provides O(1) lookup by `AdapterId`. This combination balances read concurrency, shared ownership, and lookup performance.

**Alternatives considered**:
- `RwLock<HashMap<AdapterId, Box<dyn Adapter>>>`: Box does not support shared ownership; cannot clone without re-registration
- Implementation-defined: Would create inconsistency across specs; canonical form is required for interoperability

---

### AD-24: Object Safety Enforcement for LifecycleAdapter

**Decision**: LifecycleAdapter MUST be object-safe (all methods usable through `dyn LifecycleAdapter`).

**Rationale**: The registry stores `Arc<dyn Adapter>` where Adapter includes LifecycleAdapter. For lifecycle operations to be callable through registry lookups (AD-25), LifecycleAdapter must be object-safe. Using `&self` on all methods satisfies object safety. This is consistent with the requirement that ALL public adapter traits be object-safe.

**Alternatives considered**:
- No object safety: Cannot store different adapter types in the same registry; requires type erasure workarounds

---

### AD-25: Lifecycle Invocation Through Registry

**Decision**: All lifecycle operations (initialize, start, stop, flush, shutdown) MUST be callable directly through `Arc<dyn Adapter>` returned by `registry.get()`.

**Rationale**: Callers should be able to obtain an adapter from the registry and invoke any lifecycle method without acquiring exclusive ownership or going through a separate lifecycle manager. This requires `&self` receivers on all lifecycle methods (enforced by AD-21 and AD-24) and interior mutability in concrete adapters. This simplifies the caller API: get adapter, call method.

**Alternatives considered**:
- Separate lifecycle manager: Adds indirection; every lifecycle call goes through a manager that holds exclusive access
- `Arc::get_mut()` pattern: Requires unique ownership (no other Arc references), which cannot be guaranteed post-freeze when multiple callers may hold clones
