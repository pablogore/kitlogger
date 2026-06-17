# Tasks: Transport-Agnostic Telemetry Flow

**Input**: Design documents from `specs/002-core-telemetry-domain-model-as-02-transport-agnostic-telemetry-flow/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Technology gate**: Use only technologies and commands declared in
`tech-stack.yaml`. A missing or undeclared language, runtime, framework,
database, transport, test tool, package manager, SDK, cloud provider, or
deployment target blocks task generation.

**Tests**: Validation tests are included per spec.md acceptance criteria.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Verify project initialization and validate dependencies per tech-stack.yaml

- [ ] T001 Verify crate configuration in Cargo.toml: confirm serde (derive), uuid features present and `cargo build` succeeds

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core transport contract types that all user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T002 Implement DeliveryMode non-exhaustive enum (FireAndForget, RequestResponse, Batch, Streaming) and BackpressureSignal struct (retry_after: Option\<Duration\>) in `src/transport.rs` per data-model.md
- [ ] T003 [P] Implement TransportError non-exhaustive enum (Timeout, Unavailable, Backpressure(BackpressureSignal), PayloadTooLarge, UnsupportedTransport) with manual Display and Error impls in `src/error.rs` per data-model.md
- [ ] T004 [P] Implement TelemetryBatch struct (resource, traces, metrics, logs) with serde derives and constructor that returns Err when all signal types are empty in `src/batch.rs` per data-model.md
- [ ] T005 [P] Implement PayloadEnvelope struct (transport_metadata, propagation_metadata, payload) with serde derives in `src/payload.rs` per data-model.md
- [ ] T006 Implement Transport trait with `fn send(&self, envelope: PayloadEnvelope) -> impl Future<Output = TransportResult<DeliveryMode>> + Send` in `src/transport.rs` per contracts/transport-api.md; uses std::future::Future only, no async runtime dependency
- [ ] T007 Wire up all modules (transport, error, batch, payload) and re-export public types in `src/lib.rs`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 — Request/Response Delivery Contract (Priority: P1) 🎯 MVP

**Goal**: Verify the transport contract supports request/response delivery patterns using mock-based tests. Acceptance: transport returns success/failure status and reports DeliveryMode::RequestResponse.

**Independent Test**: `cargo test test_request_response_delivery` validates a mock transport returns DeliveryMode::RequestResponse; `cargo test test_telemetry_batch_rejects_empty` validates TelemetryBatch constructor

- [ ] T008 [P] [US1] Test that a mock transport returns DeliveryMode::RequestResponse in `tests/transport_test.rs`; validates success result passes DeliveryMode
- [ ] T009 [P] [US1] Test TelemetryBatch rejects all-empty constructor and accepts non-empty batches in `tests/payload_test.rs` per FR-010
- [ ] T010 [US1] Test PayloadEnvelope serde roundtrip using MapCarrier from AS-01 in `tests/payload_test.rs` per SC-002

**Checkpoint**: US1 complete — request/response contract verified via mocks

---

## Phase 4: User Story 2 — Streaming Delivery Contract (Priority: P2)

**Goal**: Verify the transport contract supports streaming delivery patterns using mock-based tests. Acceptance: streaming mode is representable; connection failures report TransportError::Unavailable.

**Independent Test**: `cargo test test_streaming_delivery` validates a mock transport returns DeliveryMode::Streaming

- [ ] T011 [P] [US2] Test that a mock transport returns DeliveryMode::Streaming in `tests/transport_test.rs` per SC-004
- [ ] T012 [US2] Test TransportError::Unavailable can be returned from mock transport in `tests/transport_test.rs`; validates error path for connection-failure equivalence per US2 acceptance

**Checkpoint**: US2 complete — streaming contract verified via mocks

---

## Phase 5: User Story 3 — Batch Delivery Contract (Priority: P3)

**Goal**: Verify the transport contract supports batch delivery and backpressure semantics. Acceptance: batch mode is representable; backpressure signals propagate through TransportError::Backpressure.

**Independent Test**: `cargo test test_batch_delivery` validates DeliveryMode::Batch; `cargo test test_backpressure_signal` validates TransportError::Backpressure with BackpressureSignal

- [ ] T013 [P] [US3] Test that a mock transport returns DeliveryMode::Batch in `tests/transport_test.rs` per SC-004
- [ ] T014 [US3] Test TransportError::Backpressure with BackpressureSignal (retry_after) from mock transport in `tests/transport_test.rs` per SC-005; validates backpressure propagation per FR-006

**Checkpoint**: US3 complete — batch delivery and backpressure contract verified

---

## Phase 6: User Story 4 — Extensible Transport Contract (Priority: P4)

**Goal**: Verify the transport contracts remain stable when new transport protocols are added per FR-009. Acceptance: non-exhaustive matching compiles; mock transport implements trait without domain model changes.

**Independent Test**: `cargo test test_non_exhaustive_matching` validates DeliveryMode/TransportError match with wildcard; `cargo test test_mock_transport_implements_trait` validates trait implementation

- [ ] T015 [P] [US4] Test non-exhaustive matching on DeliveryMode and TransportError with wildcard arm in `tests/transport_test.rs` per SC-006
- [ ] T016 [US4] Test a full MockTransport implementing the Transport trait without modifying any AS-02 types in `tests/transport_test.rs` per SC-006; validates FR-009 extensibility

**Checkpoint**: US4 complete — extensibility contract verified

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and cleanup

- [ ] T017 [P] Run full test suite: `cargo test` — all tests pass
- [ ] T018 Validate all scenarios documented in quickstart.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup — core types BLOCK all user stories
- **US1 (Phase 3, P1)**: Depends on Phase 2
- **US2 (Phase 4, P2)**: Depends on Phase 2 — independent of US1
- **US3 (Phase 5, P3)**: Depends on Phase 2 — independent of US1, US2
- **US4 (Phase 6, P4)**: Depends on Phase 2 — independent of US1, US2, US3
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: No story dependencies — block for all subsequent
- **User Story 2 (P2)**: No story dependencies — can start in parallel with US1
- **User Story 3 (P3)**: No story dependencies — can start in parallel with US1, US2
- **User Story 4 (P4)**: No story dependencies — can start in parallel with US1, US2, US3

### Within Each User Story

- Tests validate the existing contract types from Phase 2 — no new implementation needed
- Each story phase produces independently runnable tests
- Story complete before moving to next priority

### Parallel Opportunities

- T003, T004, T005 within Phase 2 can run in parallel (different files)
- T008, T009 within US1 can run in parallel (different test files)
- T011, T013, T015 can all run in parallel with each other and any user story
- US1, US2, US3, US4 can all be implemented in parallel after Phase 2 completes

---

## Parallel Example: User Story 1

```bash
# Launch US1 test tasks together (no shared files):
Task: "Test mock transport DeliveryMode::RequestResponse in tests/transport_test.rs (T008)"
Task: "Test TelemetryBatch validation in tests/payload_test.rs (T009)"

# Then validate:
cargo test test_request_response_delivery
cargo test test_telemetry_batch_rejects_empty
```

## Parallel Example: User Story 2

```bash
# Launch US2 test tasks together:
Task: "Test mock transport DeliveryMode::Streaming in tests/transport_test.rs (T011)"
Task: "Test TransportError::Unavailable in tests/transport_test.rs (T012)"

# Then validate:
cargo test test_streaming_delivery
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (T002-T007) — build all contract types
3. Complete Phase 3: User Story 1 (T008-T010) — validate request/response contract
4. **STOP and VALIDATE**: `cargo test test_request_response_delivery && cargo test test_telemetry_batch_rejects_empty && cargo test test_payload_envelope_roundtrip`
5. MVP is ready — core transport contract verified

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add US1 (T008-T010) → Validate → MVP Ready!
3. Add US2 (T011-T012) → Validate → Streaming contract verified
4. Add US3 (T013-T014) → Validate → Batch + backpressure verified
5. Add US4 (T015-T016) → Validate → Extensibility verified
6. Each story adds test coverage without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Complete Phase 1 + Phase 2 together
2. Once Foundational is done:
   - Developer A: User Story 1 (request/response contract tests)
   - Developer B: User Story 2 (streaming contract tests)
   - Developer C: User Story 3 (batch + backpressure tests)
   - Developer D: User Story 4 (extensibility tests)
3. All stories are independent — no cross-story conflicts

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- No concrete carrier implementations (HttpHeaderCarrier, GrpcMetadataCarrier) in AS-02 — use MapCarrier from AS-01 for mock testing
- No async runtime dependency — Transport trait uses std::future::Future only
- DeliveryMode is returned as an enum value, not an associated type
- Backpressure belongs to TransportError::Backpressure, not DeliveryMode
- Tests validate only abstract contracts via mocks — no concrete protocol testing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
