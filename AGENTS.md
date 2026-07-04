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

## Stacked-PR Merge Order (Mandatory)

Before merging any PR, check whether it is itself the base branch of another
open PR: `gh pr list --search "base:<this-PR's-branch-name>"`. If one or more
open PRs target it, merge those children into the parent branch **first**,
then merge the parent into its own base — never the other way around.

GitHub does not retroactively propagate a branch's later commits into a base
it already merged into. Merging a parent before its children land means the
children's content silently never reaches the parent's base, even though
GitHub reports them as `MERGED`. This bit change 014 (Output Consolidation):
PR #35 merged before its children #36/#37, stranding their commits until a
closing PR (#40) recovered them. See issue #41 for the incident.

When opening a chain of stacked PRs, state the merge order explicitly in
each PR's description (e.g. "Merge after #36 and #37" on the parent PR) so
it's visible in review, not just inferred from branch names.
