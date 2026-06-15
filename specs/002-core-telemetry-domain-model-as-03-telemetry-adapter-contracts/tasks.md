# Tasks: Telemetry Adapter Contracts

**Input**: Design documents from `specs/002-core-telemetry-domain-model-as-03-telemetry-adapter-contracts/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Technology gate**: Use only technologies and commands declared in
`tech-stack.yaml`. A missing or undeclared language, runtime, framework,
database, transport, test tool, package manager, SDK, cloud provider, or
deployment target blocks task generation.

**Tests**: Test tasks are included per user story for contract validation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Crate**: `crates/telemetry-adapter-contracts/`
- **Source**: `crates/telemetry-adapter-contracts/src/`
- **Tests**: `crates/telemetry-adapter-contracts/tests/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the Rust crate structure and workspace membership

- [x] T001 Create `crates/telemetry-adapter-contracts/` directory structure with `src/`, `tests/`
- [x] T002 [P] Create `crates/telemetry-adapter-contracts/Cargo.toml` with dependencies: serde (derive), async-trait; dev-deps: serde_json, tokio (rt, macros), async-trait
- [x] T003 Add `crates/telemetry-adapter-contracts` to workspace `members` in root `Cargo.toml`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Create `crates/telemetry-adapter-contracts/src/id.rs` with `AdapterId` newtype over `String`, `Display`, `FromStr`, plus validation (non-empty, min 1 char)
- [x] T005 [P] Create `crates/telemetry-adapter-contracts/src/health.rs` with `AdapterHealth` enum (Healthy, Degraded, Unhealthy, Unknown) with `Default` (Unknown), and `HealthReport` struct (`status: AdapterHealth`, `reason: String`, `timestamp: SystemTime`)
- [x] T006 [P] Create `crates/telemetry-adapter-contracts/src/error.rs` with `AdapterResult<T>`, `AdapterError` enum (InvalidTransition, AlreadyRegistered, Frozen, InitializationFailed, FlushFailed, ShutdownFailed, DeliveryFailed, PartialDelivery), manual `Display` and `Error` impls
- [x] T007 Create `crates/telemetry-adapter-contracts/src/lib.rs` with public module declarations and re-exports for all public types

**Checkpoint**: Foundation ready - `cargo build` passes with core types

---

## Phase 3: User Story 1 - OpenTelemetry Adapter Contract (Priority: P1) 🎯 MVP

**Goal**: Define ProviderAdapter and ExporterAdapter traits sharing a common base, plus bidirectional mapping contracts for Canonical ↔ OpenTelemetry. Includes LifecycleAdapter (flush/shutdown) and TelemetryDelivery (deliver) as separate base traits. All traits MUST be object-safe. All methods use `&self` receiver for Arc compatibility; concrete adapters own synchronization via interior mutability.

**Independent Test**: Mock adapter implementing all base traits compiles and returns health/identity

### Tests for User Story 1

- [x] T008 [P] [US1] Contract test for adapter traits in `crates/telemetry-adapter-contracts/tests/adapter_test.rs` — mock adapter implementing CommonAdapterBase, LifecycleAdapter, TelemetryDelivery, ProviderAdapter, and ExporterAdapter
- [x] T009 [P] [US1] Contract test for mapping contracts in `crates/telemetry-adapter-contracts/tests/adapter_test.rs` — mock mapper implementing all five mapping contracts with roundtrip verification

### Implementation for User Story 1

- [x] T010 [US1] Create `crates/telemetry-adapter-contracts/src/adapter.rs` with:
  - `CommonAdapterBase` trait (id, health)
  - `LifecycleAdapter` trait (flush(&self), shutdown(&self)) — separate from identity
  - `TelemetryDelivery` trait (deliver(&self)) — uses `&self` for `Arc` compatibility
  - `Adapter` supertrait (`CommonAdapterBase + LifecycleAdapter + TelemetryDelivery + Send + Sync`)
  - `ProviderAdapter` trait (`CommonAdapterBase + LifecycleAdapter + TelemetryDelivery + initialize(&self), start(&self), stop(&self)`)
  - `ExporterAdapter` trait (`CommonAdapterBase + LifecycleAdapter + TelemetryDelivery + initialize(&self), start(&self), stop(&self)`)
  - All using `#[async_trait]`, `&self` receivers, and object-safe signatures
- [x] T011 [P] [US1] Create `crates/telemetry-adapter-contracts/src/mapping.rs` with five traits: `TraceMappingContract`, `SpanMappingContract`, `MetricMappingContract`, `LogRecordMappingContract`, `ResourceMappingContract` — each with `to_otel` and `from_otel` methods

**Checkpoint**: `cargo test` — adapter_test.rs passes; mock adapter compiles and is functional

---

## Phase 4: User Story 2 - Extensible Adapter Contract (Priority: P2)

**Goal**: AdapterRegistry with register, get, contains, list operations; frozen-after-init semantics; duplicate rejection. Registry stores `Arc<dyn Adapter>` for shared ownership.

**Independent Test**: Registry accepts registration, then freezes; post-freeze registration is rejected; duplicate registration returns error

### Tests for User Story 2

- [x] T012 [P] [US2] Contract test for registry in `crates/telemetry-adapter-contracts/tests/registry_test.rs` — register, get, contains, list operations
- [x] T013 [P] [US2] Contract test for freeze behavior in `crates/telemetry-adapter-contracts/tests/registry_test.rs` — registration before/after freeze, duplicate rejection

### Implementation for User Story 2

- [x] T014 [US2] Create `crates/telemetry-adapter-contracts/src/registry.rs` with `AdapterRegistry` struct using `RwLock<HashMap<AdapterId, Arc<dyn Adapter>>>`, methods: `register`, `get` (returns `Option<Arc<dyn Adapter>>`), `contains`, `list`, `freeze`
- [x] T015 [US2] Implement freeze validation in registry — reject post-freeze registrations with `AdapterError::Frozen`
- [x] T016 [US2] Implement duplicate registration rejection — return `AdapterError::AlreadyRegistered` on duplicate AdapterId

**Checkpoint**: `cargo test` — registry_test.rs passes; registry enforces freeze and duplicate rules

---

## Phase 5: User Story 3 - Adapter Lifecycle Management (Priority: P3)

**Goal**: AdapterLifecycle state machine with explicit transition matrix, including startup failure transitions (Registered→Shutdown, Initialized→Shutdown) and Stopped vs. Shutdown semantics.

**Independent Test**: Lifecycle transitions follow the matrix; invalid transitions return typed error; shutdown implicitly flushes

### Tests for User Story 3

- [x] T017 [P] [US3] Contract test for lifecycle transitions in `crates/telemetry-adapter-contracts/tests/lifecycle_test.rs` — valid and invalid transitions per matrix (including Registered→Shutdown, Initialized→Shutdown)
- [x] T018 [P] [US3] Contract test for shutdown-flush semantics in `crates/telemetry-adapter-contracts/tests/lifecycle_test.rs` — LifecycleAdapter::shutdown() invokes flush() before Stopped
- [x] T019 [P] [US3] Integration test in `crates/telemetry-adapter-contracts/tests/integration_tests.rs` — full lifecycle (Registered → Initialized → Started → Stopping → Stopped → Shutdown) with multiplexing over multiple adapters via TelemetryDelivery

### Implementation for User Story 3

- [x] T020 [US3] Create `crates/telemetry-adapter-contracts/src/lifecycle.rs` with `LifecycleState` enum (Registered, Initialized, Started, Stopping, Stopped, Shutdown), `AdapterLifecycle` struct, transition matrix encoded as match arms (reject invalid with `AdapterError::InvalidTransition`; allow Registered→Shutdown and Initialized→Shutdown)
- [x] T021 [US3] Implement `shutdown(&self)` default impl on LifecycleAdapter that calls `flush()` then transitions to Stopped; concrete adapters use interior mutability for lifecycle state

**Checkpoint**: `cargo test` — lifecycle_test.rs passes; all lifecycle scenarios validated

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Integration, documentation, validation

- [x] T022 [P] Add `deliver_to_all` function in `crates/telemetry-adapter-contracts/src/registry.rs` — iterates adapters from registry via `TelemetryDelivery::deliver()`, collects failures, returns `Ok(())`, `AdapterError::PartialDelivery`, or `AdapterError::DeliveryFailed`
- [x] T023 [P] Add docs and doc-tests for all public types (id, health, error, adapter, registry, lifecycle, mapping)
- [x] T024 Run `cargo test --workspace` and verify all tests pass (adapter_test, registry_test, lifecycle_test, integration_tests)
- [x] T025 Run quickstart.md validation scenarios

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories
- **US1 - OTel Adapter (Phase 3)**: Depends on Foundational — No dependencies on other stories
- **US2 - Registry (Phase 4)**: Depends on Foundational + US1 (needs Adapter supertrait + all base traits for `Arc<dyn Adapter>`)
- **US3 - Lifecycle (Phase 5)**: Depends on Foundational + US1 (LifecycleAdapter trait) — independently testable with mocks
- **Polish (Phase 6)**: Depends on all stories

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational — No dependencies on other stories
- **User Story 2 (P2)**: Can start after US1 — Needs CommonAdapterBase + LifecycleAdapter + TelemetryDelivery + Adapter supertrait
- **User Story 3 (P3)**: Can start after Foundational + US1 — Depends on LifecycleAdapter trait but testable with mocks

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Core types before services/lifecycle
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks (T001-T003) can run in parallel
- All Foundational tasks (T004-T007) can run in parallel
- Once Foundational completes: US1 can start
- Within a story: all tests marked [P] can run in parallel; all impl marked [P] can run in parallel

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1 (adapter traits + mapping contracts)
4. **STOP and VALIDATE**: `cargo test` — adapter_test passes
5. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1 (adapter contracts) → Test → MVP!
3. Add US2 (registry) → Test → Extensible platform
4. Add US3 (lifecycle) → Test → Full lifecycle management
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done: Developer A starts US1 (adapter traits + mapping)
3. After US1: Developer B starts US2 (registry); Developer C starts US3 (lifecycle)
4. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
