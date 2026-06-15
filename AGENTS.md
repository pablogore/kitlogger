<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan
at specs/002-core-telemetry-domain-model-as-01-context-propagation-and-correlation/plan.md

Capability Namespace Governance:
.specify/memory/capability-namespace-governance.md

Extended Governance:
.specify/memory/constitution.md
<!-- SPECKIT END -->

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
