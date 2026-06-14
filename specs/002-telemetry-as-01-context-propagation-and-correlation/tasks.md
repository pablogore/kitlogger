# Tasks: Context Propagation and Correlation

**Input**: Design documents from `specs/002-telemetry-as-01-context-propagation-and-correlation/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/carrier.md, contracts/propagation.md, quickstart.md

**Technology gate**: Rust, Tokio, cargo test, OpenTelemetry, serde

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Paths use `<package-root>/` as a placeholder. Actual package root, crate placement, and workspace structure are determined by CORE-000 Release Engineering.

- Source: `<package-root>/src/`
- Tests: `<package-root>/tests/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare the implementation environment per CORE-000 Release Engineering package structure

- [x] T001 Create `Cargo.toml` (dependencies: `uuid` with `v7,serde` features, `serde` with `derive`) and `src/lib.rs` at the package root determined by CORE-000

---

## Phase 2: Foundational - Carrier Abstraction (Blocks All User Stories)

**Purpose**: Core Carrier traits that all propagators depend on

- [x] T002 [P] Define `Injector` trait with `fn set(&mut self, key: &str, value: &str)` in `<package-root>/src/carrier.rs`
- [x] T003 [P] Define `Extractor` trait with `fn get(&self, key: &str) -> Option<&str>` and `fn get_all(&self, key: &str) -> Vec<&str>` in `<package-root>/src/carrier.rs`
- [x] T004 Define `Propagator` trait with associated type `Context`, methods `inject`, `extract`, `fields` in `<package-root>/src/carrier.rs`
- [x] T005 Implement `MapCarrier` (HashMap-based) as both `Injector` and `Extractor` in `<package-root>/src/carrier.rs`
- [x] T006 Wire up `lib.rs` with `pub mod carrier; pub mod trace_context; pub mod correlation; pub mod baggage; pub mod propagation;` and re-exports in `<package-root>/src/lib.rs`

**Checkpoint**: Carrier abstraction ready -- all user stories can now be implemented independently

---

## Phase 3: User Story 1 - Propagate Trace Context Across Boundaries (Priority: P1) MVP

**Goal**: Implement W3C Trace Context model and propagation so a distributed trace can cross service boundaries

**Independent Test**: `cargo test -- test_trace_context_roundtrip` -- inject a TraceContext into a carrier, extract from the same carrier, and verify trace_id, span_id, and trace_flags match

### Implementation for User Story 1

- [x] T007 [P] [US1] Create `TraceId([u8; 16])`, `SpanId([u8; 8])` newtypes with hex formatting and parsing in `<package-root>/src/trace_context.rs`
- [x] T008 [P] [US1] Create `TraceFlags(u8)` newtype with sampled/random-trace-id bitmask methods in `<package-root>/src/trace_context.rs`
- [x] T009 [P] [US1] Create `TraceState` struct with max 32 vendor entries in `<package-root>/src/trace_context.rs`
- [x] T010 [P] [US1] Create `TraceContext` struct with `trace_id`, `span_id`, `parent_span_id`, `trace_flags`, `trace_state` fields and validation (no all-zero IDs, no version ff) in `<package-root>/src/trace_context.rs`
- [x] T011 [US1] Implement `traceparent` header parsing (`FromStr` for `TraceContext`) with fixed-length 55-char format in `<package-root>/src/trace_context.rs`
- [x] T012 [US1] Implement `traceparent` header serialization (`Display` for `TraceContext`) in `<package-root>/src/trace_context.rs`
- [ ] T013 [US1] Implement `tracestate` header parsing and serialization in `<package-root>/src/trace_context.rs`
- [ ] T014 [P] [US1] Implement `TraceContextPropagator` (inject: sets `traceparent`/`tracestate`, extract: parses headers, fields: `["traceparent", "tracestate"]`) in `<package-root>/src/propagation.rs`
- [x] T015 [P] [US1] Write round-trip test -- inject then extract yields original context -- in `<package-root>/tests/trace_context_test.rs`
- [ ] T016 [P] [US1] Write multi-hop test -- 5 simulated hops, same trace_id, unique span_ids, correct parent chain -- in `<package-root>/tests/trace_context_test.rs`
- [ ] T017 [P] [US1] Write malformed context test -- all-zeros, bad format, wrong length produce empty context without panic -- in `<package-root>/tests/trace_context_test.rs`

**Checkpoint**: Trace Context propagation is fully functional and independently testable

---

## Phase 4: User Story 2 - Correlate Across Telemetry Signals (Priority: P2)

**Goal**: Generate UUID v7 correlation identifiers and propagate them so Traces, Metrics, and Logs can be correlated

**Independent Test**: `cargo test -- test_correlation_generation` -- generate a UUID v7, verify it is valid, time-sortable, and non-zero

### Implementation for User Story 2

- [ ] T018 [P] [US2] Create `CorrelationIdentifier` struct with `id: Uuid` and `created_at: i64`, implement `new()` using `uuid::Uuid::new_v7()` in `<package-root>/src/correlation.rs`
- [x] T019 [US2] Implement `CorrelationPropagator` (inject: sets `correlation-id` header, extract: parses UUID v7 or generates new one, fields: `["correlation-id"]`) in `<package-root>/src/propagation.rs`
- [ ] T020 [P] [US2] Write correlation ID generation test -- valid UUID v7, time-sortable, non-zero -- in `<package-root>/tests/correlation_test.rs`
- [x] T021 [P] [US2] Write correlation round-trip test -- inject then extract returns same UUID -- in `<package-root>/tests/correlation_test.rs`

**Checkpoint**: Cross-signal correlation is functional and independently testable

---

## Phase 5: User Story 3 - Propagate Baggage Across Execution Boundaries (Priority: P3)

**Goal**: Implement W3C Baggage model and propagation so application context survives multi-hop service chains

**Independent Test**: `cargo test -- test_baggage_propagation` -- set 3 baggage entries, inject into carrier, extract from carrier, verify all entries present

### Implementation for User Story 3

- [x] T022 [P] [US3] Create `BaggageProperty` enum (key-value or flag) and `BaggageEntry` struct with `key`, `value`, `properties` in `<package-root>/src/baggage.rs`
- [x] T023 [P] [US3] Create `Baggage` struct with `entries: Vec<BaggageEntry>`, max 180 entries, max 64KB total size in `<package-root>/src/baggage.rs`
- [ ] T024 [US3] Implement `baggage` header parsing -- comma-separated `key=value` with URL-percent-encoded values -- in `<package-root>/src/baggage.rs`
- [ ] T025 [US3] Implement `baggage` header serialization in `<package-root>/src/baggage.rs`
- [ ] T026 [P] [US3] Implement `BaggagePropagator` (inject: sets `baggage` header, extract: parses header, fields: `["baggage"]`) in `<package-root>/src/propagation.rs`
- [ ] T027 [P] [US3] Write baggage propagation test -- 3 entries round-trip -- in `<package-root>/tests/baggage_test.rs`
- [ ] T028 [P] [US3] Write multi-hop baggage test -- 3 hops, all entries survive -- in `<package-root>/tests/baggage_test.rs`
- [ ] T029 [P] [US3] Write baggage edge case test -- empty baggage, max entries, invalid entries skipped -- in `<package-root>/tests/baggage_test.rs`

**Checkpoint**: Baggage propagation is functional and independently testable

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: CompositePropagator integration and final validation

- [ ] T030 Implement `CompositePropagator` with `Vec<Box<dyn Propagator>>` that delegates to all registered propagators in `<package-root>/src/propagation.rs`
- [ ] T031 [P] Write `CompositePropagator` test -- all propagators inject their fields and extract their context correctly -- in `<package-root>/tests/propagation_test.rs`
- [ ] T032 Add serde `Serialize`/`Deserialize` derives to `TraceContext`, `CorrelationIdentifier`, `Baggage` for downstream use
- [ ] T033 Run full test suite and verify all 5 quickstart scenarios pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies -- can start immediately
- **Foundational (Phase 2)**: Depends on Setup -- BLOCKS all user stories
- **US1 / US2 / US3 (Phases 3-5)**: Depend on Foundational phase -- independent of each other
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: No dependency on other user stories -- can start after Phase 2
- **US2 (P2)**: No dependency on other user stories -- can start after Phase 2
- **US3 (P3)**: No dependency on other user stories -- can start after Phase 2

### Within Each User Story

- Models before propagators
- Propagators before tests
- Core implementation before integration

### Parallel Opportunities

- T002, T003 can run in parallel (Phase 2)
- All [P]-marked tasks within a phase can run in parallel
- US1, US2, US3 can be implemented in parallel once Phase 2 completes
- All tests within a user story (marked [P]) can run in parallel

---

## Parallel Example: User Story 1

```bash
cargo test -- test_trace_context_roundtrip
cargo test -- test_multi_hop_propagation
cargo test -- test_malformed_context
```

## Parallel Example: User Story 2

```bash
cargo test -- test_correlation_generation
cargo test -- test_correlation_roundtrip
```

## Parallel Example: User Story 3

```bash
cargo test -- test_baggage_propagation
cargo test -- test_baggage_multi_hop
cargo test -- test_baggage_edge_cases
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (Carrier abstraction)
3. Complete Phase 3: User Story 1 (Trace Context)
4. **STOP and VALIDATE**: `cargo test -- test_trace_context_roundtrip`
5. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational -> Foundation ready
2. Add User Story 1 -> Test independently -> Deploy/Demo (MVP!)
3. Add User Story 2 -> Test independently -> Deploy/Demo
4. Add User Story 3 -> Test independently -> Deploy/Demo
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:
1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (Trace Context)
   - Developer B: User Story 2 (Correlation)
   - Developer C: User Story 3 (Baggage)
3. Stories complete and integrate independently
