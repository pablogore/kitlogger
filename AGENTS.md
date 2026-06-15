<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan
at specs/002-core-telemetry-domain-model-as-02-transport-agnostic-telemetry-flow/plan.md
<!-- SPECKIT END -->

<!-- ANCHORED SUMMARY -->
# Anchored Summary

## Session Context
- **ACTIVE_SPEC_ID**: `002-core-telemetry-domain-model-as-02-transport-agnostic-telemetry-flow`
- **Branch**: `main`
- **Goal**: AS-02 plan — generate research, data model, contracts, quickstart (post-clarification regeneration)

## Status
- **Architecture**: Pre-plan — Transport contracts owned by AS-02; concrete transports are separate specs
- **Plan**: Complete — research.md (10 decisions), data-model.md (7 entities), contracts/transport-api.md, quickstart.md (6 scenarios), plan.md regenerated
- **Implementation**: Not started
- **Governance**: Frozen artifacts immutable after approval

## Key Decisions
- Transport trait with `std::future::Future` only (no Tokio dependency)
- DeliveryMode returned as enum value, not associated type
- Serde derives on PayloadEnvelope and TelemetryBatch
- Non-exhaustive TransportError enum with Backpressure variant
- Backpressure belongs to TransportError::Backpressure, not DeliveryMode
- Concrete carriers (HttpHeaderCarrier, GrpcMetadataCarrier) belong to child transport binding specs
- AS-02 uses MapCarrier from AS-01 for mock-based testing
- TelemetryBatch constructor rejects all-empty batches
- Execution boundaries are informative only
- Manual Error/Display impls (no thiserror — undeclared dependency)

## Generated Artifacts
- `research.md` — 10 research decisions resolved (updated with 8 clarifications)
- `data-model.md` — 7 entities defined (DeliveryMode, BackpressureSignal, TelemetryBatch, PayloadEnvelope, TransportResult, TransportError, Transport trait)
- `contracts/transport-api.md` — Transport trait, no concrete carriers (re-exported from AS-01)
- `quickstart.md` — 6 validation scenarios, mock-based, no concrete transports
- `plan.md` — Technical Context, Constitution Check, project structure (no carrier_ext.rs)<!-- /ANCHORED SUMMARY -->

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

## Traceability Chain
Requirement → Success Criterion → Task → Code → Test

Missing any link → STOP → Traceability Gap Report → no implementation.

## Governance Failure Conditions
- Implementation modifies frozen artifacts
- Code without a task
- Task without a requirement
- Public API without a contract
- Entity without a data-model definition
- Scope expansion beyond approved architecture

Result: GOVERNANCE FAILURE — reject the change.
