---
description: "Task list for KIT-001 Foundational Observability Abstractions"
---

# Tasks: KIT-001 Foundational Observability Abstractions

**Input**: Design documents from `/specs/001-kit-foundational/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: The examples below include test tasks. Tests are OPTIONAL - only include them if explicitly requested in the feature specification.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- Paths shown below assume single project - adjust based on plan.md structure

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Create project structure per implementation plan
- [ ] T002 Initialize Rust project with required dependencies
- [ ] T003 [P] Configure linting and formatting tools (clippy, rustfmt)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Setup core data models in src/models/
- [ ] T005 [P] Implement Context operations (create_root_context, create_child_context, with_correlation_id)
- [ ] T006 [P] Implement Resource operations (new, merge_with)
- [ ] T007 [P] Implement InstrumentationScope operations (new)
- [ ] T008 [P] Implement Span operations (new)
- [ ] T009 [P] Implement LogRecord operations (new)
- [ ] T010 [P] Implement Metric operations (new)
- [ ] T011 [P] Configure error handling and logging infrastructure
- [ ] T012 [P] Setup environment configuration management

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Instrument Application Code With Structured Telemetry (Priority: P1) 🎯 MVP

**Goal**: Enable developers to create and emit traces, logs, and metrics using a unified, domain-agnostic API

**Independent Test**: A standalone test creates a span, attaches a log record to it, records a metric, and verifies that all three telemetry signals carry the expected context and arbitrary attributes — all without configuring any exporter or backend.

### Tests for User Story 1 (OPTIONAL - only if tests requested) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T013 [P] [US1] Contract test for Context creation in tests/contract/test_context.py
- [ ] T014 [P] [US1] Integration test for end-to-end telemetry flow in tests/integration/test_telemetry_flow.py

### Implementation for User Story 1

- [ ] T015 [P] [US1] Create Context model in src/models/context.rs
- [ ] T016 [P] [US1] Create Resource model in src/models/resource.rs
- [ ] T017 [P] [US1] Create InstrumentationScope model in src/models/instrumentation_scope.rs
- [ ] T018 [P] [US1] Create Span model in src/models/span.rs
- [ ] T019 [P] [US1] Create LogRecord model in src/models/log_record.rs
- [ ] T020 [P] [US1] Create Metric model in src/models/metric.rs
- [ ] T021 [US1] Implement Logger trait in src/traits/logger.rs
- [ ] T022 [US1] Implement Tracer trait in src/traits/tracer.rs
- [ ] T023 [US1] Implement Meter trait in src/traits/meter.rs
- [ ] T024 [US1] Add validation and error handling for all core entities
- [ ] T025 [US1] Add logging for user story 1 operations

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Attach Resource Metadata to Telemetry (Priority: P1)

**Goal**: Enable every telemetry record to carry metadata about the running service instance

**Independent Test**: A test creates a Resource with custom attributes, creates a span and a log record associated with that Resource, and verifies that both carry the Resource attributes — without any infrastructure provider being hardcoded.

### Tests for User Story 2 (OPTIONAL - only if tests requested) ⚠️

- [ ] T026 [P] [US2] Contract test for Resource creation in tests/contract/test_resource.py
- [ ] T027 [P] [US2] Integration test for Resource association in tests/integration/test_resource_association.py

### Implementation for User Story 2

- [ ] T028 [P] [US2] Create Resource model in src/models/resource.rs
- [ ] T029 [US2] Implement Resource operations in src/models/resource.rs
- [ ] T030 [US2] Integrate Resource with Span, LogRecord, and Metric models
- [ ] T031 [US2] Add validation and error handling for Resource attributes

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Correlate Telemetry Across Systems via Correlation ID (Priority: P2)

**Goal**: Enable correlation identifiers to be used independently of tracing

**Independent Test**: A test creates a log record and a metric datapoint with a correlation_id but no trace context, and verifies that both carry the correlation_id correctly.

### Tests for User Story 3 (OPTIONAL - only if tests requested) ⚠️

- [ ] T032 [P] [US3] Contract test for correlation_id in tests/contract/test_correlation.py
- [ ] T033 [P] [US3] Integration test for correlation_id without trace context in tests/integration/test_correlation_no_trace.py

### Implementation for User Story 3

- [ ] T034 [P] [US3] Add correlation_id support to Context model in src/models/context.rs
- [ ] T035 [P] [US3] Add correlation_id support to LogRecord model in src/models/log_record.rs
- [ ] T036 [P] [US3] Add correlation_id support to Metric model in src/models/metric.rs
- [ ] T037 [US3] Implement correlation_id handling in Logger, Tracer, and Meter traits
- [ ] T038 [US3] Add validation and error handling for correlation_id

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: User Story 4 - Use All Four Metric Instrument Types (Priority: P2)

**Goal**: Enable developers to use Counter, Gauge, Histogram, and UpDownCounter instruments

**Independent Test**: A test creates one instrument of each type, records values on each, and verifies that each instrument preserves the correct semantic type and recorded data points.

### Tests for User Story 4 (OPTIONAL - only if tests requested) ⚠️

- [ ] T039 [P] [US4] Contract test for all metric instruments in tests/contract/test_metrics.py
- [ ] T040 [P] [US4] Integration test for metric instrument usage in tests/integration/test_metric_instruments.py

### Implementation for User Story 4

- [ ] T041 [P] [US4] Create Counter trait in src/traits/counter.rs
- [ ] T042 [P] [US4] Create Gauge trait in src/traits/gauge.rs
- [ ] T043 [P] [US4] Create Histogram trait in src/traits/histogram.rs
- [ ] T044 [P] [US4] Create UpDownCounter trait in src/traits/up_down_counter.rs
- [ ] T045 [US4] Implement Counter, Gauge, Histogram, and UpDownCounter in src/models/metric.rs
- [ ] T046 [US4] Integrate metric instruments with Meter trait
- [ ] T047 [US4] Add validation and error handling for metric instruments

---

## Phase 7: User Story 5 - Attribute Telemetry to an Instrumentation Scope (Priority: P3)

**Goal**: Enable library authors to tag telemetry with instrumentation scope

**Independent Test**: A test creates two instrumentation scopes, emits a span under each, and verifies that each span carries the correct scope name.

### Tests for User Story 5 (OPTIONAL - only if tests requested) ⚠️

- [ ] T048 [P] [US5] Contract test for InstrumentationScope in tests/contract/test_instrumentation_scope.py
- [ ] T049 [P] [US5] Integration test for scope attribution in tests/integration/test_scope_attribution.py

### Implementation for User Story 5

- [ ] T050 [P] [US5] Create InstrumentationScope model in src/models/instrumentation_scope.rs
- [ ] T051 [US5] Implement InstrumentationScope operations in src/models/instrumentation_scope.rs
- [ ] T052 [US5] Integrate InstrumentationScope with Span, LogRecord, and Metric models
- [ ] T053 [US5] Add validation and error handling for InstrumentationScope

---

## Phase 8: User Story 6 - Macro-Based Instrumentation (Priority: P3)

**Goal**: Ensure the core design is compatible with future macro-based instrumentation

**Independent Test**: A test exercises the underlying core API in the same pattern that each macro would expand to, and verifies the resulting telemetry is structurally identical to what a macro would produce.

### Tests for User Story 6 (OPTIONAL - only if tests requested) ⚠️

- [ ] T054 [P] [US6] Contract test for macro compatibility in tests/contract/test_macro_compatibility.py
- [ ] T055 [P] [US6] Integration test for macro expansion patterns in tests/integration/test_macro_patterns.py

### Implementation for User Story 6

- [ ] T056 [P] [US6] Document macro expansion patterns in src/docs/macro_patterns.md
- [ ] T057 [US6] Ensure core API supports macro expansion without modification
- [ ] T058 [US6] Add documentation for macro compatibility

---

## Phase 9: User Story 7 - Async Compatibility for Concurrent Runtimes (Priority: P3)

**Goal**: Ensure observability works correctly in asynchronous and concurrent execution environments

**Independent Test**: A test spawns multiple concurrent async tasks, creates a parent trace context, propagates it into each task, and verifies that child spans within each task correctly reference the parent trace and maintain independent span_ids.

### Tests for User Story 7 (OPTIONAL - only if tests requested) ⚠️

- [ ] T059 [P] [US7] Contract test for async context propagation in tests/contract/test_async_context.py
- [ ] T060 [P] [US7] Integration test for concurrent async tasks in tests/integration/test_concurrent_async.py

### Implementation for User Story 7

- [ ] T061 [P] [US7] Implement async context propagation utilities in src/utils/async_context.rs
- [ ] T062 [US7] Add async compatibility to Context operations
- [ ] T063 [US7] Add async compatibility to all core traits and models
- [ ] T064 [US7] Add documentation for async usage patterns

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T065 [P] Documentation updates in docs/
- [ ] T066 Code cleanup and refactoring
- [ ] T067 [P] Additional unit tests in tests/unit/
- [ ] T068 Security hardening
- [ ] T069 Run quickstart.md validation
- [ ] T070 [P] Implement NoOpLogger, NoOpTracer, NoOpMeter implementations
- [ ] T071 [P] Implement default global instances for NoOp implementations

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P1)**: Can start after Foundational (Phase 2) - May integrate with US1 but should be independently testable
- **User Story 3 (P2)**: Can start after Foundational (Phase 2) - May integrate with US1/US2 but should be independently testable
- **User Story 4 (P2)**: Can start after Foundational (Phase 2) - May integrate with US1/US2/US3 but should be independently testable
- **User Story 5 (P3)**: Can start after Foundational (Phase 2) - May integrate with US1/US2/US3/US4 but should be independently testable
- **User Story 6 (P3)**: Can start after Foundational (Phase 2) - Should be independently testable
- **User Story 7 (P3)**: Can start after Foundational (Phase 2) - Should be independently testable

### Within Each User Story

- Tests (if included) MUST be written and FAIL before implementation
- Models before services
- Services before endpoints
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Models within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1
   - Developer B: User Story 2
   - Developer C: User Story 3
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence