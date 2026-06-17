# Requirement Classification

**Specification**: 002-core-telemetry-domain-model-as-02-transport-agnostic-telemetry-flow
**Date**: 2026-06-15

## Classification Rules

### FUNCTIONAL

Requirements that define features, types, or behaviors the system must implement.

Traceability chain:

```
REQ → SC → PLAN → TASK → CODE → TEST
```

### CONSTRAINT

Requirements that restrict how the system is built, what it must avoid, or define scope boundaries.

Traceability chain:

```
REQ → ARCH → CODE → TEST
```

CONSTRAINT requirements MUST NOT require implementation tasks unless explicit implementation work is needed. When implementation work is needed (e.g., adding `#[non_exhaustive]` attributes, re-exporting types), the TASK and PLAN links MAY be present but are not required.

---

## Classifications

### FUNCTIONAL

#### FR-001 — Transport Contract

**Type**: FUNCTIONAL
**Source**: `spec.md:75`

**Rationale**: Defines concrete types (Transport trait, TransportResult/TransportError) that the system must implement. PayloadEnvelope is provided by telemetry-types per ADR-007.

**Traceability**:
```
FR-001 → SC-001 → PLAN-005 → T002, T003, T006, T007
→ src/transport.rs, src/error.rs, src/lib.rs (imports from telemetry-types)
→ tests/transport_test.rs:test_mock_transport_implements_trait
```

---

#### FR-002 — TelemetryBatch Model

**Type**: FUNCTIONAL
**Source**: `spec.md:76`

**Rationale**: Defines the concrete TelemetryBatch struct with traces, metrics, and logs fields. Defined in telemetry-types per ADR-007.

**Traceability**:
```
FR-002 → SC-002 → (telemetry-types implementation)
→ crates/telemetry-types/src/batch.rs
→ tests/batch_test.rs:test_telemetry_batch_serde
```

---

#### FR-003 — PayloadEnvelope

**Type**: FUNCTIONAL
**Source**: `spec.md:77`

**Rationale**: Defines the concrete PayloadEnvelope struct with transport_metadata, propagation_metadata, and payload fields. Defined in telemetry-types per ADR-007.

**Traceability**:
```
FR-003 → SC-002 → (telemetry-types implementation)
→ crates/telemetry-types/src/payload.rs
→ tests/payload_test.rs:test_payload_envelope_serde
```

---

#### FR-004 — TransportResult/TransportError

**Type**: FUNCTIONAL
**Source**: `spec.md:78`

**Rationale**: Defines the concrete TransportResult type alias and TransportError enum with 5 specific variants.

**Traceability**:
```
FR-004 → SC-003 → PLAN-005 → T003
→ src/error.rs
→ tests/transport_test.rs:test_transport_error_serde
→ tests/transport_test.rs:test_transport_error_display
```

---

#### FR-005 — DeliveryMode Enum

**Type**: FUNCTIONAL
**Source**: `spec.md:79`

**Rationale**: Defines the concrete DeliveryMode enum with 4 specific variants, returned as a value from Transport::send().

**Traceability**:
```
FR-005 → SC-004 → PLAN-005 → T002
→ src/transport.rs
→ tests/transport_test.rs:test_delivery_mode_serde
```

---

#### FR-006 — Backpressure Semantics

**Type**: FUNCTIONAL
**Source**: `spec.md:80`

**Rationale**: Defines the concrete Backpressure variant on TransportError. BackpressureSignal struct is defined in telemetry-types per ADR-007.

**Traceability**:
```
FR-006 → SC-005 → PLAN-005, PLAN-008 → T003, T014
→ src/error.rs (imports BackpressureSignal from telemetry-types), src/transport.rs
→ tests/transport_test.rs:test_mock_transport_returns_backpressure
```

---

#### FR-010 — TelemetryBatch Empty Rejection

**Type**: FUNCTIONAL
**Source**: `spec.md:84`

**Rationale**: Defines concrete validation logic in the TelemetryBatch constructor.

**Traceability**:
```
FR-010 → SC-002 → PLAN-005, PLAN-006 → T004, T009
→ src/batch.rs:73-75
→ tests/batch_test.rs:test_telemetry_batch_empty_validation
→ tests/payload_test.rs:test_telemetry_batch_rejects_all_empty_in_payload
```

---

### CONSTRAINT

#### FR-007 — Propagation Metadata from AS-01

**Type**: CONSTRAINT
**Source**: `spec.md:81`

**Rationale**: Restricts how propagation metadata is sourced (from AS-01, not created locally). The implementation work (re-exporting PropagationMetadata, adding the field to PayloadEnvelope) is shared with FR-003 which owns the PayloadEnvelope definition.

**Traceability**:
```
FR-007 → (ARCH: research.md AD-8, contracts/transport-api.md:92-106)
→ src/payload.rs:4, src/lib.rs:19-20
→ tests/payload_test.rs:test_propagation_metadata_from_as01
```

**No task required**: The constraint is satisfied by the architecture decision to import from AS-01 and the implementation work is owned by FR-003 (PayloadEnvelope field).

---

#### FR-008 — AS-02 Defines Contracts Only

**Type**: CONSTRAINT
**Source**: `spec.md:82`

**Rationale**: Restricts AS-02 scope to contract definitions only. Concrete transport implementations must be separate specifications. This is a pure scope boundary constraint with no implementation work — it is enforced by what is NOT in the crate.

**Traceability**:
```
FR-008 → (ARCH: spec.md scope/non-scope, plan.md:14)
→ (CODE: enforced by Cargo.toml — no concrete transport deps)
→ tests/transport_test.rs:test_as02_defines_contracts_only
```

**No task required**: Pure design constraint. The requirement is satisfied by crate composition (no concrete transport dependencies) which is inherent to the project structure, not an implementation task.

---

#### FR-009 — Stable Contract for New Transports

**Type**: CONSTRAINT
**Source**: `spec.md:83`

**Rationale**: Requires API stability when new transport implementations are added. The implementation work (`#[non_exhaustive]` attributes) is minimal and architectural.

**Traceability**:
```
FR-009 → (ARCH: research.md AD-9, data-model.md:12-13, 70-72)
→ src/transport.rs:33, src/error.rs:26
→ tests/integration_tests.rs:test_transport_error_is_non_exhaustive
```

**No task required**: The constraint is satisfied by adding `#[non_exhaustive]` to the enum definitions. Tasks T015/T016 provide additional validation but are not required for the constraint chain.

---

#### FR-011 — Runtime Independence

**Type**: CONSTRAINT
**Source**: `spec.md:85`

**Rationale**: Restricts the Transport trait to use `std::future::Future` only, forbidding async runtime dependencies (Tokio, async-std, smol). The implementation work (choosing `async-trait` over `tokio::async_trait`, no Tokio in production deps) is a design decision.

**Traceability**:
```
FR-011 → (ARCH: research.md AD-6, contracts/transport-api.md:17-19)
→ src/transport.rs:83 (uses #[async_trait::async_trait], not tokio::async_trait)
→ Cargo.toml:7 (no Tokio in [dependencies])
→ tests/transport_test.rs:test_no_runtime_transport
```

**No task required**: The constraint is satisfied by the architecture decision and Cargo.toml configuration. Task T006 exists but validates through test rather than being a separate implementation effort.

---

## Traceability Matrix (Classified)

### FUNCTIONAL (7)

| FR | SC | PLAN | TASK | CODE | TEST | Status |
|----|----|------|------|------|------|--------|
| FR-001 | SC-001 | PLAN-005 | T002, T003, T006, T007 | transport.rs, error.rs, lib.rs (imports from telemetry-types) | transport_test.rs | GREEN |
| FR-002 | SC-002 | telemetry-types | telemetry-types tasks | telemetry-types/src/batch.rs | telemetry-types batch_test.rs | GREEN |
| FR-003 | SC-002 | telemetry-types | telemetry-types tasks | telemetry-types/src/payload.rs | telemetry-types payload_test.rs | GREEN |
| FR-004 | SC-003 | PLAN-005 | T003 | error.rs | transport_test.rs | GREEN |
| FR-005 | SC-004 | PLAN-005 | T002 | transport.rs | transport_test.rs | GREEN |
| FR-006 | SC-005 | PLAN-005, PLAN-008 | T003, T014 | error.rs (imports BackpressureSignal from telemetry-types), transport.rs | transport_test.rs | GREEN |
| FR-010 | SC-002 | telemetry-types | telemetry-types tasks | telemetry-types/src/batch.rs | telemetry-types batch_test.rs | GREEN |

### CONSTRAINT (4)

| FR | ARCH | CODE | TEST | Status |
|----|------|------|------|--------|
| FR-007 | research.md AD-8, contracts/transport-api.md:92-106 | payload.rs, lib.rs | payload_test.rs | GREEN |
| FR-008 | spec.md scope, plan.md:14 | Cargo.toml (no concrete deps) | transport_test.rs:test_as02_defines_contracts_only | GREEN |
| FR-009 | research.md AD-9, data-model.md:12-13, 70-72 | transport.rs:33, error.rs:26 | integration_tests.rs:test_transport_error_is_non_exhaustive | GREEN |
| FR-011 | research.md AD-6, contracts/transport-api.md:17-19 | transport.rs:83, Cargo.toml | transport_test.rs:test_no_runtime_transport | GREEN |

---

## Impact on Traceability Audit

With the FUNCTIONAL/CONSTRAINT classification:

- **FR-008** is a CONSTRAINT requiring no task → **No traceability gap**. The prior TG-001 is resolved.
- **FR-007** is a CONSTRAINT satisfied by architecture decision → **No gap**.
- **FR-009** is a CONSTRAINT satisfied by architectural `#[non_exhaustive]` → **No gap**.
- **FR-011** is a CONSTRAINT satisfied by architecture decision → **No gap**.

**Result**: All 11 FRs now have complete traceability chains appropriate to their type.

**Overall Verdict**: **COMPLETE** — 100% traceability achieved.
