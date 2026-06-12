---
name: create-capability-spec
description: Create one Constitution v3.1 Capability Specification from explicit SPEC_ID and parent architecture input. Use when a bounded Capability must be written without allocating identifiers or generating Atomic Features.
---

# Create Capability Specification

Require explicit `SPEC_ID` and `PARENT_ARCHITECTURE`.

## Identifier Gate

1. Validate `SPEC_ID` with
   `^[0-9]{3}-[a-z0-9]+(?:-[a-z0-9]+)*$`.
2. Record its source as `explicit-user-input`.
3. Never derive it from branch, session, feature, counter, or directory scans.
4. Require `PARENT_ARCHITECTURE` to reference an existing Architecture
   Specification.
5. Reject any output outside `specs/<SPEC_ID>/`.

## Allowed Reads

- `.specify/memory/constitution.md`
- `.specify/templates/capability-spec-template.md`
- The explicit parent Architecture Specification
- Read-only sibling Capability context when required for boundaries

## Allowed Write

```text
specs/<SPEC_ID>/spec.md
```

Never create branches, Atomic Feature IDs, decomposition files, plans, or tasks.

## Procedure

1. Validate both explicit inputs.
2. Load the Constitution, parent Architecture, and Capability template.
3. Define one bounded grouped concern, scope, non-scope, concepts, and
   relationships.
4. Exclude stories, acceptance criteria, requirements, plans, tasks, and
   implementation details.
5. Verify only the allowed file changes.

If the requested path has a numbered prefix different from `SPEC_ID`, stop with
a constitutional violation.
