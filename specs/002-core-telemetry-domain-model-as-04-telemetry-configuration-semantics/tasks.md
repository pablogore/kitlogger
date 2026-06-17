---
description: "Task list for Telemetry Configuration Semantics"
---

# Tasks: Telemetry Configuration Semantics

**Input**: Design documents from `specs/002-core-telemetry-domain-model-as-04-telemetry-configuration-semantics/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Technology gate**: Use only technologies and commands declared in `tech-stack.yaml`: Rust, Tokio, cargo test, HTTP, gRPC, Kafka, RabbitMQ, serde, OpenTelemetry. Missing or undeclared technology blocks task generation.

**Tests**: Unit tests are included per user story as specified in the feature specification.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `crates/telemetry-config-semantics/src/`, `crates/telemetry-config-semantics/tests/` at repository root

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Create `crates/telemetry-config-semantics/Cargo.toml` with Rust edition 2021, serde (derive) dependency, and library crate declaration

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core entity types that MUST be complete before user stories can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T002 [P] Define SchemaVersion struct with semver `version: String` and optional `description: String` fields in `crates/telemetry-config-semantics/src/schema_version.rs`; implement Default returning "1.0.0"
- [ ] T003 [P] Define VerbosityLevel enum (Off, Error, Warn, Info, Debug, Trace) and VerbosityPolicy struct with per-signal fields (trace_level, metric_level, log_level) in `crates/telemetry-config-semantics/src/verbosity_policy.rs`; implement Default returning Info for all signals
- [ ] T004 Create `crates/telemetry-config-semantics/src/lib.rs` with `pub mod` declarations for all six entity modules (schema_version, verbosity_policy, telemetry_config, sampling_policy, exporter_config, resource_config) and re-export all public structs and enums

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Enable/Disable Telemetry (Priority: P1) 🎯 MVP

**Goal**: KitLogger administrators can enable or disable telemetry output through configuration.

**Independent Test**: Create a TelemetryConfig with `enabled: false` and verify that all default composition fields are populated while the disabled flag is respected.

- [ ] T005 [US1] Define TelemetryConfig struct with `enabled: bool`, optional composition fields (sampling, exporters, resources, verbosity), and required `schema_version: SchemaVersion` in `crates/telemetry-config-semantics/src/telemetry_config.rs`; implement Default with enabled=true and all sub-entity defaults
- [ ] T006 [US1] Create unit tests in `crates/telemetry-config-semantics/tests/config_test.rs` for: TelemetryConfig default construction (enabled=true), disabled flag parsing, schema_version default ("1.0.0"), and Quickstart scenarios 1-2

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently — TelemetryConfig with SchemaVersion and VerbosityPolicy defaults

---

## Phase 4: User Story 2 - Configure Sampling Policy (Priority: P2)

**Goal**: KitLogger administrators can configure sampling policies to control telemetry volume.

**Independent Test**: Create a SamplingPolicy with type TraceIdRatio and rate 0.1; verify the type and rate are stored correctly; verify rate 1.5 is rejected by the type system or constructor.

- [ ] T007 [US2] Define SamplingPolicyType enum (AlwaysOn, AlwaysOff, TraceIdRatio, ParentBased, ConsistentProbability, Extension(String)) in `crates/telemetry-config-semantics/src/sampling_policy.rs`
- [ ] T008 [US2] Define SamplingPolicy struct with `policy_type: SamplingPolicyType` and `sampling_rate: f64` fields in `crates/telemetry-config-semantics/src/sampling_policy.rs`; implement Default (AlwaysOn, rate 1.0); add constructor validation for rate range [0.0, 1.0]
- [ ] T009 [US2] Integrate SamplingPolicy as optional composition field (`sampling: Option<SamplingPolicy>`) in TelemetryConfig; update Default to include SamplingPolicy default
- [ ] T010 [US2] Add unit tests in `crates/telemetry-config-semantics/tests/config_test.rs` for: SamplingPolicyType enum variants, rate validation (0.0, 0.5, 1.0 valid; -0.1, 1.5 invalid), Default construction, Extension variant, and Quickstart scenario 3

**Checkpoint**: User Stories 1 AND 2 should both work independently — TelemetryConfig with SamplingPolicy

---

## Phase 5: User Story 3 - Select and Configure Exporters (Priority: P3)

**Goal**: KitLogger administrators can select which exporters are active and configure their behavior.

**Independent Test**: Create an ExporterConfig with type "otlp" and endpoint "http://localhost:4317"; verify serialization round-trip; verify invalid timeout_secs is rejected.

- [ ] T011 [P] [US3] Define CompressionType enum (None, Gzip) in `crates/telemetry-config-semantics/src/exporter_config.rs`
- [ ] T012 [US3] Define ExporterConfig struct with fields: `exporter_type: String`, `endpoint: Option<String>`, `compression: CompressionType`, `headers: HashMap<String, String>`, `timeout_secs: u64`, `settings: HashMap<String, String>` in `crates/telemetry-config-semantics/src/exporter_config.rs`; implement Default ("console", no endpoint, no compression, empty headers, 30s timeout, empty settings)
- [ ] T013 [US3] Integrate ExporterConfig as optional vec field (`exporters: Option<Vec<ExporterConfig>>`) in TelemetryConfig; update Default to include single default console ExporterConfig
- [ ] T014 [US3] Add unit tests in `crates/telemetry-config-semantics/tests/config_test.rs` for: ExporterConfig default construction, exporter_type as string, CompressionType enum, timeout_secs bounds, settings map, and Quickstart scenario 4

**Checkpoint**: User Stories 1, 2, AND 3 should all work independently — TelemetryConfig with SamplingPolicy and ExporterConfig

---

## Phase 6: User Story 4 - Configure Resource Attributes (Priority: P4)

**Goal**: KitLogger administrators can configure resource attributes that identify the telemetry source.

**Independent Test**: Create a ResourceConfig with service_name "my-service", service_version "2.0.0", deployment_environment "production"; verify all fields are stored correctly.

- [ ] T015 [P] [US4] Define ResourceConfig struct with fields: `service_name: String`, `service_version: String`, `deployment_environment: String`, `attributes: HashMap<String, String>` in `crates/telemetry-config-semantics/src/resource_config.rs`; implement Default ("", "unknown", "development", empty map)
- [ ] T016 [US4] Integrate ResourceConfig as optional composition field (`resources: Option<ResourceConfig>`) in TelemetryConfig; update Default to include default ResourceConfig
- [ ] T017 [US4] Add unit tests in `crates/telemetry-config-semantics/tests/config_test.rs` for: ResourceConfig default values (service_version="unknown", deployment_environment="development"), empty service_name handling, arbitrary attributes map, and Quickstart scenarios 5-7 (verbosity + schema_version)

**Checkpoint**: All four user stories should now work independently — complete TelemetryConfig aggregate

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T018 [P] Add serde Serialize/Deserialize derives to all entity structs and enums; add serde rename attributes matching config-schema-contract.md field naming conventions
- [ ] T019 [P] Audit all contract files against implementation: verify config-schema-contract.md field names match serde attributes, verify adapter-integration-contract.md constraints match ExporterConfig.exporter_type handling
- [ ] T020 Run `cargo test` in `crates/telemetry-config-semantics/` and fix any compilation or test failures
- [ ] T021 Run quickstart.md full validation suite against the implemented crate

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - User stories can proceed in priority order (P1 → P2 → P3 → P4)
  - Each story adds a new entity to TelemetryConfig without breaking previous stories
- **Polish (Phase 7)**: Depends on all desired user stories being complete

### User Story Dependencies

- **US1 (P1)**: Can start after Foundational (Phase 2) — No dependencies on other stories
- **US2 (P2)**: Can start after Foundational — Adds SamplingPolicy; no US1 dependency
- **US3 (P3)**: Can start after Foundational — Adds ExporterConfig; no US1/US2 dependency
- **US4 (P4)**: Can start after Foundational — Adds ResourceConfig; no US1/US2/US3 dependency

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Entity struct before TelemetryConfig integration
- TelemetryConfig integration before tests

### Parallel Opportunities

- Foundational tasks (T002, T003) can run in parallel
- All user story phases can start simultaneously after Foundational (different source files per story)
- Polish tasks (T018, T019) can run in parallel

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1 — TelemetryConfig + SchemaVersion + VerbosityPolicy
4. **STOP and VALIDATE**: Test User Story 1 independently
5. TelemetryConfig with enabled/disable + defaults is complete

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add US1 (Enable/Disable) → Test independently → **MVP complete**
3. Add US2 (Sampling) → Test independently → Deploy/Demo
4. Add US3 (Exporters) → Test independently → Deploy/Demo
5. Add US4 (Resources) → Test independently → Deploy/Demo
6. Each story adds value without breaking previous stories

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- The TelemetryConfig crate is pure data types — no runtime behavior, no async interfaces, no I/O
- All entity Default implementations align with data-model.md Default Configuration section
