# Active Change

**Change**: `005-console-exporter`
**Phase**: Spec complete — implementation not started
**Engram**: `sdd/005-console-exporter/*`

Run `/sdd-status 005-console-exporter` to see full artifact state.

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
