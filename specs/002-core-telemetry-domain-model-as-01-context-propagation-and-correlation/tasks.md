# Tasks: Context Propagation and Correlation

**Input**: Design documents from `specs/002-core-telemetry-domain-model-as-01-context-propagation-and-correlation/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Technology gate**: Use only technologies and commands declared in
`tech-stack.yaml`. A missing or undeclared language, runtime, framework,
database, transport, test tool, package manager, SDK, cloud provider, or
deployment target blocks task generation.

**Tests**: Test tasks are included per the spec.md acceptance criteria.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Verify project initialization and validate dependencies

- [ ] T001 Verify crate configuration in Cargo.toml: confirm uuid v7, serde derive features are present and `cargo build` succeeds

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Fix underlying issues all user stories depend on

- [ ] T002 [P] Implement Propagation Metadata model for transport-specific context carriage in src/propagation_metadata.rs per data-model.md
- [ ] T003 [P] Fix CorrelationIdentifier UUID generation to use `Uuid::now_v7()` instead of hardcoded constant in src/correlation.rs
- [ ] T004 Register Propagation Metadata module and expose from src/lib.rs

**Checkpoint**: Foundation ready - user story implementation can begin

---

## Phase 3: User Story 1 - Propagate Trace Context Across Boundaries (Priority: P1) MVP

**Goal**: W3C Trace Context propagates correctly across 5+ simulated service hops

**Independent Test**: `cargo test test_trace_context_roundtrip` verifies inject/extract roundtrip; `cargo test test_multi_hop_propagation` verifies 5+ hop chain

### Implementation for User Story 1

- [ ] T005 [P] [US1] Add tracestate serialization to TraceContextPropagator::inject in src/propagation.rs
- [ ] T006 [P] [US1] Preserve parent_span_id during extract and propagate in TraceContextPropagator in src/propagation.rs
- [ ] T007 [US1] Implement multi-hop (5+) trace context propagation test in tests/trace_context_test.rs per SC-001
- [ ] T008 [US1] Add malformed context handling test (SC-004) to verify graceful parsing failure in tests/trace_context_test.rs

**Checkpoint**: User Story 1 complete - Trace Context propagates and can be independently verified

---

## Phase 4: User Story 2 - Correlate Across Telemetry Signals (Priority: P2)

**Goal**: A single correlation identifier retrieves related Trace, Metric, and Log Record

**Independent Test**: `cargo test test_correlation_roundtrip` and `cargo test test_cross_signal_correlation` verify correlation across signal types

### Implementation for User Story 2

- [ ] T009 [P] [US2] Fix CorrelationIdentifier::from_uuid to preserve original created_at instead of regenerating in src/correlation.rs
- [ ] T010 [P] [US2] Add serde Serialize/Deserialize derives to CorrelationIdentifier in src/correlation.rs
- [ ] T011 [US2] Implement cross-signal correlation test verifying same correlation-id links Trace, Metric, LogRecord in tests/correlation_test.rs per SC-002

**Checkpoint**: User Story 2 complete - Correlation Identifier links telemetry across signals

---

## Phase 5: User Story 3 - Propagate Baggage Across Execution Boundaries (Priority: P3)

**Goal**: Baggage entries survive 3+ service hops without data loss

**Independent Test**: `cargo test test_baggage_roundtrip` and `cargo test test_baggage_multi_hop` verify baggage propagation chain

### Implementation for User Story 3

- [ ] T012 [P] [US3] Implement Baggage serialization to W3C Baggage header format in src/baggage.rs
- [ ] T013 [P] [US3] Implement Baggage deserialization from W3C Baggage header format in src/baggage.rs
- [ ] T014 [US3] Implement BaggagePropagator::inject and BaggagePropagator::extract using serialization in src/propagation.rs
- [ ] T015 [US3] Add 3-hop baggage propagation test in tests/baggage_test.rs per SC-003

**Checkpoint**: User Story 3 complete - Baggage propagates across multiple hops

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and cleanup

- [ ] T016 [P] Run full test suite: `cargo test` - all tests pass
- [ ] T017 Validate all scenarios documented in quickstart.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - fixes US2 broken UUID generation and adds Propagation Metadata
- **User Stories (Phase 3-5)**: All depend on Phase 2 completion
  - US1 (P1) and US3 (P3) are fully independent of each other
  - US2 (P2) foundational fix (T003) must complete before US2 story tasks
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: No story dependencies - can start after Phase 2
- **User Story 2 (P2)**: No story dependencies - can start after Phase 2
- **User Story 3 (P3)**: No story dependencies - can start after Phase 2

### Within Each User Story

- Models before services (where applicable)
- Core implementation before integration tests
- Story complete before moving to next priority

### Parallel Opportunities

- T002, T003 within Phase 2 can run in parallel
- T005, T006 within US1 can run in parallel
- T009, T010 within US2 can run in parallel
- T012, T013 within US3 can run in parallel
- US1, US2, US3 can be implemented in parallel by different developers after Phase 2

---

## Parallel Example: User Story 1

```bash
# Launch all US1 implementation tasks together:
Task: "Add tracestate serialization in src/propagation.rs (T005)"
Task: "Preserve parent_span_id during extract in src/propagation.rs (T006)"

# Then, run tests to validate:
cargo test test_trace_context_roundtrip
cargo test test_multi_hop_propagation
cargo test test_trace_context_from_str_invalid
```

## Parallel Example: User Story 2

```bash
# Launch all US2 implementation tasks together:
Task: "Fix from_uuid to preserve timestamp in src/correlation.rs (T009)"
Task: "Add serde derives to CorrelationIdentifier in src/correlation.rs (T010)"

# Then validate:
cargo test test_correlation_roundtrip
cargo test test_cross_signal_correlation
```

## Parallel Example: User Story 3

```bash
# Launch all US3 implementation tasks together:
Task: "Implement Baggage serialization in src/baggage.rs (T012)"
Task: "Implement Baggage deserialization in src/baggage.rs (T013)"

# Then:
cargo test test_baggage_add_entry
cargo test test_baggage_get_entry
cargo test test_baggage_multi_hop
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (T002-T004)
3. Complete Phase 3: User Story 1 (T005-T008)
4. **STOP and VALIDATE**: `cargo test test_trace_context_roundtrip && cargo test test_multi_hop_propagation`
5. MVP is ready - Trace Context propagation across 5+ hops

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 (T005-T008) → Test independently → MVP Ready!
3. Add User Story 2 (T009-T011) → Test independently → Cross-signal correlation ready
4. Add User Story 3 (T012-T015) → Test independently → Full context propagation ready
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Complete Phase 1 + Phase 2 together
2. Once Foundational is done:
   - Developer A: User Story 1 (Trace Context)
   - Developer B: User Story 2 (Correlation)
   - Developer C: User Story 3 (Baggage)
3. All three stories are independent - no integration conflicts

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Tests exist at `tests/` for each component; update and verify
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
