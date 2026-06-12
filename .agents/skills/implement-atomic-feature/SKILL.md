---
name: implement-atomic-feature
description: Implement one approved Atomic Feature using explicit SPEC_ID and AF_ID. Use when bounded tasks are ready and code changes must remain traceable without modifying specification hierarchy artifacts.
---

# Implement Atomic Feature

Require explicit `SPEC_ID` and `AF_ID`.

## Identifier Gate

- Validate `SPEC_ID` with
  `^[0-9]{3}-[a-z0-9]+(?:-[a-z0-9]+)*$`.
- Validate `AF_ID` with `^af-[0-9]{3}$`.
- Record both sources as `explicit-user-input`.
- Never derive identifiers from branch, session, feature, or counter metadata.
- Resolve exactly one Atomic Feature directory:
  `specs/<SPEC_ID>/<AF_ID>-*/`.

## Preconditions

Require expanded `spec.md`, approved `plan.md`, and validated `tasks.md`.
Reject Architecture, Capability, decomposition output, or unexpanded stubs.

## Procedure

1. Read the Atomic Feature chain and relevant design artifacts.
2. Implement tasks in dependency order.
3. Run required tests and checks.
4. Mark tasks complete only after verification succeeds.
5. Stop when requested work introduces another entity, responsibility, or
   sibling scope.

## Immutable Governance Artifacts

Do not modify:

- Architecture or Capability specifications
- `decomposition.md` or `feature-index.md`
- Atomic Feature `spec.md`
- `plan.md`

Implementation may change source, tests, supporting documentation, and task
completion markers only as authorized by `tasks.md`.
