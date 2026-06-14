---
name: architecture
description: Create or update the architecture and flat Atomic Specification blueprint for one explicit Capability specification. Use before expand to define boundaries, dependencies, ownership, and top-level specification candidates without creating specifications, plans, tasks, branches, or nested directories.
---

# Architecture

Require `SPEC_ID` as explicit user input.

## Identifier Gate

1. Reject missing or empty `SPEC_ID`.
2. Require `SPEC_ID` to match `^[0-9]{3}-[a-z0-9]+(?:-[a-z0-9]+)*$`.
3. Record its source as `explicit-user-input`.
4. Never derive or alter it from Git branches, sessions, feature numbers,
   repository counters, sibling directories, or prior specifications.
5. Reject any target whose first directory below `specs/` is not exactly
   `SPEC_ID`.

## Allowed Reads

- `.specify/memory/constitution.md`
- `assets/architecture-spec-template.md`
- `specs/<SPEC_ID>/spec.md`
- Read-only parent, sibling, or dependency context when essential

## Allowed Writes

```text
specs/<SPEC_ID>/architecture.md
specs/<SPEC_ID>/decomposition.md
specs/<SPEC_ID>/feature-index.md
```

Do not create any other file or directory. Never create or switch branches.

## Procedure

1. Validate `SPEC_ID` before inspecting repository state.
2. Require `specs/<SPEC_ID>/spec.md`.
3. Load the Constitution and Architecture template.
4. Write `architecture.md` covering:
   - Capability and domain boundaries
   - Concepts, constraints, and ownership boundaries
   - Decomposition strategy
   - Atomic Specification dependency graph
   - Atomic Specification candidates
5. For each candidate define:
   - Local candidate key
   - Name
   - One responsibility
   - Dependencies
   - Ownership boundary
6. Write the same candidate set to `decomposition.md` and `feature-index.md`.
7. Exclude implementation artifacts, plans, tasks, code, branches, and
   directory allocation.
8. Verify only the allowed files changed.

## Candidate Rules

- Describe candidates only as Atomic Specifications.
- Keep each candidate independently evolvable and implementable.
- Split candidates with multiple entities, responsibilities, contracts,
  lifecycles, or integration points.
- Assign stable capability-local candidate keys such as `AS-01`, `AS-02`.
- Do not assign repository specification numbers; `expand` owns allocation.
- Produce a blueprint suitable for deterministic expansion.

## Forbidden Outputs

Never create child specification directories below the Capability or any
top-level specification directory. Architecture produces documents only.

## Failure

Stop with:

```text
CONSTITUTION VIOLATION: explicit capability-scoped SPEC_ID is required.
```

Stop when the Capability spec is missing or when any requested write falls
outside the three allowed files.
