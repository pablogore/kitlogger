# Tasks: Structured Log Domain Model

**Input**: Design documents from `specs/003-structured-logging-core-as-01-structured-log-domain-model/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Technology gate**: Use only technologies and commands declared in
`tech-stack.yaml`. A missing or undeclared language, runtime, framework,
database, transport, test tool, package manager, SDK, cloud provider, or
deployment target blocks task generation.

**Tests**: This specification explicitly defines testing criteria. Test tasks are included per the spec's Testing section.

**Organization**: Tasks are grouped by implementation phase — this is a single atomic specification with one feature scope (no independent user stories).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact file paths in descriptions

## Path Conventions

- **Crate**: `crates/kitlogger-log-domain/` (workspace member)
- **Source**: `crates/kitlogger-log-domain/src/`
- **Tests**: `crates/kitlogger-log-domain/tests/`

---

## Phase 1: Setup

**Purpose**: Create the workspace crate structure

- [ ] T001 Create `crates/kitlogger-log-domain/` directory structure with `src/` and `tests/` subdirectories
- [ ] T002 Create `crates/kitlogger-log-domain/Cargo.toml` with package name `kitlogger-log-domain`, edition 2021, version 0.1.0, and add to workspace members in root `Cargo.toml`

---

## Phase 2: Foundational Domain Types

**Purpose**: Implement the primitive value types that all other entities depend on

- [ ] T003 [P] Implement `Severity` enum in `crates/kitlogger-log-domain/src/severity.rs` with six variants (Trace, Debug, Info, Warn, Error, Fatal), `PartialOrd` derive, `Display`, and `FromStr`
- [ ] T004 [P] Implement `LogAttributeValue` enum in `crates/kitlogger-log-domain/src/log_attribute_value.rs` with variants: String, Integer(i64), Float(f64), Boolean(bool), Timestamp(SystemTime), Array(Vec<LogAttributeValue>); enforce homogeneous array in Array constructor
- [ ] T005 [P] Implement `ValidationError` enum in `crates/kitlogger-log-domain/src/validation.rs` with variants: EmptyMessage, InvalidSeverity, InvalidAttributeName(String), InvalidAttributeValue(String); implement `Display` and `std::error::Error` manually

---

## Phase 3: Core Domain Model

**Purpose**: Implement LogAttribute, LogRecord, identifier types, and validation rules

- [ ] T006 [P] Implement attribute naming validation function `validate_attribute_name(name: &str) -> Result<(), ValidationError>` in `crates/kitlogger-log-domain/src/validation.rs` that checks pattern `^[a-z][a-z0-9._]{0,63}$`, max 64 chars, and reserved field name exclusion (timestamp, severity, message, attributes)
- [ ] T007 [P] Implement `LogAttribute` struct in `crates/kitlogger-log-domain/src/log_attribute.rs` with `new(name, value) -> Result<Self, ValidationError>`, `name() -> &str`, and `value() -> &LogAttributeValue` accessors
- [ ] T008 [P] Implement `CorrelationId` newtype in `crates/kitlogger-log-domain/src/correlation_id.rs` with `new(id: String) -> Self`, `as_str() -> &str`, `Display`, and `From<String>`
- [ ] T009 [P] Implement `TraceId` newtype in `crates/kitlogger-log-domain/src/trace_id.rs` with same interface as CorrelationId
- [ ] T010 [P] Implement `SpanId` newtype in `crates/kitlogger-log-domain/src/span_id.rs` with same interface as CorrelationId
- [ ] T011 Implement `LogRecord` struct in `crates/kitlogger-log-domain/src/log_record.rs` with `new(timestamp, severity, message, attributes) -> Result<Self, ValidationError>`, accessor methods for each field, and private fields ensuring immutability
- [ ] T012 Create `crates/kitlogger-log-domain/src/lib.rs` that re-exports all public types (LogRecord, Severity, LogAttribute, LogAttributeValue, CorrelationId, TraceId, SpanId, ValidationError) and declares modules

---

## Phase 4: Testing

**Purpose**: Verify all success criteria are met per spec.md

- [ ] T013 Create `crates/kitlogger-log-domain/tests/log_record_tests.rs` with tests for valid LogRecord construction, accessor methods, empty message rejection, invalid severity rejection, and immutability (compile-time check via private fields)
- [ ] T014 Create `crates/kitlogger-log-domain/tests/attribute_tests.rs` with tests for LogAttribute construction with valid/invalid names, LogAttributeValue type variants, homogeneous array enforcement, and naming pattern validation
- [ ] T015 Create `crates/kitlogger-log-domain/tests/severity_tests.rs` with tests for severity ordering (Trace < Debug < Info < Warn < Error < Fatal), Display formatting, and FromStr round-trip
- [ ] T016 Create `crates/kitlogger-log-domain/tests/identifier_tests.rs` with tests for CorrelationId, TraceId, SpanId construction, as_str(), Display, and From<String>

---

## Phase 5: Polish

**Purpose**: Final quality assurance

- [ ] T017 Run `cargo clippy -p kitlogger-log-domain` and fix any warnings
- [ ] T018 Run `cargo test -p kitlogger-log-domain` and verify all 16 test scenarios pass
- [ ] T019 Run `scripts/validate-tech-stack.sh` to confirm no undeclared technology violations

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational Types (Phase 2)**: Depends on Setup completion
- **Core Domain Model (Phase 3)**: Depends on Foundational Types completion
- **Testing (Phase 4)**: Depends on Core Domain Model completion
- **Polish (Phase 5)**: Depends on all prior phases

### Task Dependencies Within Phases

**Phase 2 (Foundational Types)**:
- T003, T004, T005 are independent (parallel)

**Phase 3 (Core Domain Model)**:
- T006, T007, T008, T009, T010 are independent (parallel)
- T011 depends on T006, T007 (uses LogAttribute, validation)
- T012 depends on all prior tasks in Phase 2 and Phase 3

**Phase 4 (Testing)**:
- T013, T014, T015, T016 are independent (parallel)

### Parallel Opportunities

- All Phase 2 tasks (T003-T005) can run in parallel
- T006-T010 in Phase 3 can run in parallel
- All Phase 4 test tasks (T013-T016) can run in parallel

---

## Parallel Execution Example

```bash
# Phase 2 — launch all foundational types together:
Task: "Implement Severity enum" (T003)
Task: "Implement LogAttributeValue enum" (T004)
Task: "Implement ValidationError enum" (T005)

# Phase 3 — launch independent implementations together:
Task: "Implement attribute naming validation" (T006)
Task: "Implement LogAttribute" (T007)
Task: "Implement CorrelationId" (T008)
Task: "Implement TraceId" (T009)
Task: "Implement SpanId" (T010)

# Phase 4 — launch all test files together:
Task: "Test LogRecord construction and validation" (T013)
Task: "Test attribute types and naming" (T014)
Task: "Test severity ordering and display" (T015)
Task: "Test identifier types" (T016)
```

---

## Implementation Strategy

### Complete Delivery (single atomic spec)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational Types
3. Complete Phase 3: Core Domain Model
4. Complete Phase 4: Testing — all 16 test scenarios pass
5. Complete Phase 5: Polish — clippy clean, all tests green

### Parallel Team Strategy

1. Developer A: Phase 1 Setup → Phase 5 Polish
2. Phase 2 tasks can be parallelized across multiple developers
3. Phase 3 tasks T006-T010 can be parallelized
4. T011 and T012 must be sequential after T006-T010
5. Phase 4 tests can be written in parallel after core model stabilizes

---

## Notes

- No external dependencies required (pure std library)
- Crate must compile on stable Rust with no warnings
- No unsafe code permitted
- All construction paths return `Result` types for validation
- Identifier types are opaque newtypes over String
- Tests are required per spec.md testing section
