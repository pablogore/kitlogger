---
name: create-architecture-spec
description: Create one Constitution v3.1 Architecture Specification from an explicitly supplied SPEC_ID. Use when architecture.md must be created or replaced without creating branches, allocating identifiers, or modifying other specifications.
---

# Create Architecture Specification

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
- `.specify/templates/architecture-spec-template.md`
- `specs/<SPEC_ID>/spec.md`, if present
- Read-only parent, sibling, or dependency context when essential

## Allowed Write

```text
specs/<SPEC_ID>/architecture.md
```

No other file may be created or modified. Never create or switch branches.

## Procedure

1. Validate `SPEC_ID` before inspecting repository state.
2. Load the Constitution and Architecture template.
3. Create Architecture content covering boundaries, concepts, capabilities,
   relationships, and constraints.
4. Exclude requirements, stories, acceptance criteria, plans, tasks, and
   implementation details.
5. Verify the exact output path before writing.
6. Verify no other path changed after writing.

## Failure

Stop with:

```text
CONSTITUTION VIOLATION: explicit capability-scoped SPEC_ID is required.
```

Reject `specs/021-opentelemetry-integration/architecture.md` when the explicit
input is `SPEC_ID=002-opentelemetry`.
