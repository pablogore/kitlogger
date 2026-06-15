---
name: architecture-audit
description: Audit implementation compliance against approved architecture artifacts. Verify/ that code adheres to contracts, data models, and governance rules without modifying any frozen artifacts.
---

# Architecture Audit

Audit implementation compliance against approved architecture artifacts.

## Behavior

1. Load the active specification's architecture artifacts (`spec.md`, `data-model.md`, `contracts/**`, `plan.md`).
2. Compare implementation in `src/**`, `tests/**` against contracts and data models.
3. Check governance compliance:
   - Frozen artifacts are immutable (`spec.md`, `research.md`, `data-model.md`, `contracts/**`, `plan.md`, `tasks.md`, `tech-stack.yaml`).
   - Scope is limited to approved directories (`src/**`, `tests/**`, `examples/**`, `benchmarks/**`).
   - Traceability chain is complete: Requirement → Success Criterion → Task → Code → Test.
   - All public APIs are documented in contracts.
   - All entities are defined in data-model.md.
4. Report findings per category: requirement, contract, blocker, remediation.

## Output

```text
## Audit Report

### Objectives
- [x] / [ ] Objective 1: description
- [ ] / [ ] Objective N: description

### Findings
- F-01 [requirement|contract|blocker|remediation]: description
```

## Rules

- Never modify architecture artifacts.
- Never modify implementation code.
- Report findings only.
- If compliant, return `COMPLIANT — zero findings`.
- If non-compliant, report all findings without remediation.

## Failure

Stop with `AUDIT FAILURE: <reason>` when architecture artifacts are missing.
