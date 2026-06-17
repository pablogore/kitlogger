<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan
at specs/002-core-telemetry-domain-model-as-04-telemetry-configuration-semantics/plan.md
<!-- SPECKIT END -->

<!-- ANCHORED SUMMARY -->
# Anchored Summary

## Session Context
- **ACTIVE_SPEC_ID**: `002-core-telemetry-domain-model-as-03-telemetry-adapter-contracts`
- **Branch**: `main`
- **Purpose**: Design phase for telemetry adapter contracts crate

## Status
- **Architecture**: Complete — Adapter contracts owned by AS-03; concrete adapters are separate specs. Shared canonical types layer (telemetry-types) introduced per ADR-007, owned by parent capability.
- **Plan**: Complete — research.md (25 decisions), data-model.md (15 entities), contracts/adapter-api.md, quickstart.md (6 scenarios), plan.md regenerated, tasks.md (25 tasks across 6 phases)
- **Implementation**: Complete — 24 unit tests = **24 passing**
- **Requirement Classification**: Pending
- **Traceability**: Pending
- **Governance**: Frozen artifacts immutable after approval; all Session 2026-06-16 and 2026-06-17 clarifications integrated. ADR-007 approved.
- **ADR-007**: Shared canonical types layer (telemetry-types) owns PayloadEnvelope, TelemetryBatch, TelemetryBatchError, TransportMetadata, BackpressureSignal. AS-02 and AS-03 depend on telemetry-types instead of each other.

## Implemented Entities (Design)
- `spec.md` — 3 clarification sessions (2026-06-14, 2026-06-15, 2026-06-16), 11 SCs, Key Entities, Ownership Boundary, Assumptions
- `contracts/adapter-api.md` — CommonAdapterBase, LifecycleAdapter, TelemetryDelivery, ProviderAdapter, ExporterAdapter, Adapter supertrait, AdapterRegistry (Arc-based), AdapterLifecycle transition matrix, HealthReport, AdapterResult/AdapterError, mapping contracts, multiplexing contract
- `data-model.md` — 15 entities with fields, relationships, transition matrix
- `research.md` — 25 architecture decisions (AD-1 through AD-25), including object safety, Arc registry, LifecycleAdapter, TelemetryDelivery, HealthReport, Stopped vs Shutdown semantics
- `tasks.md` — 25 tasks across 6 phases (Setup, Foundational, US1-MVP, US2-Registry, US3-Lifecycle, Polish)
- `tech-stack.yaml` — Added async-trait macro declaration

## Key Architecture Decisions (AS-03)
- LifecycleAdapter trait (flush, shutdown) separated from CommonAdapterBase (identity, health)
- TelemetryDelivery trait for multiplexing operations; uses `&self` for Arc compatibility
- All adapter traits MUST be object-safe for `dyn Trait` registry usage
- Registry stores `Arc<dyn Adapter + Send + Sync>`; `get()` returns `Arc<dyn Adapter>`
- Registered→Shutdown and Initialized→Shutdown allowed for startup failure scenarios
- Stopped retains resources; Shutdown releases resources and is terminal
- HealthReport struct (AdapterHealth + String reason + SystemTime timestamp)
- Manual Error/Display impls (no thiserror — undeclared dependency)
- async-trait for all async traits (consistent with AS-02)
- Registry supports both ProviderAdapter and ExporterAdapter through common Adapter supertrait
- All adapter methods use `&self` receiver; concrete adapters own synchronization via interior mutability
- Registry storage: `RwLock<HashMap<AdapterId, Arc<dyn Adapter>>>` (canonical form)
- LifecycleAdapter remains object-safe; all lifecycle operations callable through Arc<dyn Adapter>

## Generated Artifacts (AS-03)
- `research.md` — 25 research decisions resolved (4 sessions: 10 + 10 + 5 ADs)
- `data-model.md` — 15 entities defined with full relationship diagram
- `contracts/adapter-api.md` — 7 traits, Registry API, transition matrix, mapping contracts
- `quickstart.md` — 6 validation scenarios (Arc-based registry, startup failure transitions, HealthReport)
- `plan.md` — Technical Context, Constitution Check, project structure
- `tasks.md` — 25 tasks across 6 phases (Setup→Foundational→US1→US2→US3→Polish)
- `tech-stack.yaml` — async-trait added, all technologies validated<!-- /ANCHORED SUMMARY -->

# Architecture Governance (Mandatory)

## Frozen Artifacts (immutable after approval)
- spec.md, research.md, data-model.md, contracts/**
- plan.md, tasks.md, tech-stack.yaml

## Implementation Scope (may modify)
- src/**, tests/**, examples/**, benchmarks/**
- operational implementation documentation

## Architecture Conflict Procedure
1. STOP implementation
2. Create Architecture Finding (requirement, contract, blocker, remediation)
3. Request architecture remediation
4. Wait for approval
5. Regenerate downstream artifacts if approved
6. Resume implementation

Architecture always wins. Code never becomes source of truth.

## Traceability Chains

### FUNCTIONAL Requirements
```
REQ → SC → PLAN → TASK → CODE → TEST
```

### CONSTRAINT Requirements
```
REQ → ARCH → CODE → TEST
```

CONSTRAINT requirements MUST NOT require implementation tasks unless explicit implementation work is needed.

Missing any link → STOP → Traceability Gap Report → no implementation.

## Governance Failure Conditions
- Implementation modifies frozen artifacts
- Code without a task
- Task without a requirement
- Public API without a contract
- Entity without a data-model definition
- Scope expansion beyond approved architecture

Result: GOVERNANCE FAILURE — reject the change.
