---
name: expand
description: Expand a Capability architecture blueprint into independent top-level SpecKit specifications. Use after architecture to allocate sequential specification numbers, create one standard spec.md per Atomic Specification candidate, and update the parent decomposition index without creating nested directories.
---

# Expand Atomic Specifications

Require explicit `PARENT_SPEC_ID`.

## Identifier Gate

1. Validate `PARENT_SPEC_ID` with
   `^[0-9]{3}-[a-z0-9]+(?:-[a-z0-9]+)*$`.
2. Record its source as `explicit-user-input`.
3. Never derive the parent from branch, session, feature, or repository
   metadata.
4. Resolve exactly `specs/<PARENT_SPEC_ID>/`.
5. Reject nested directories below the parent as expansion targets.

## Required Reads

- `.specify/memory/constitution.md`
- `.specify/templates/spec-template.md`
- `specs/<PARENT_SPEC_ID>/spec.md`
- `specs/<PARENT_SPEC_ID>/architecture.md`
- `specs/<PARENT_SPEC_ID>/decomposition.md`
- `specs/<PARENT_SPEC_ID>/feature-index.md`

## Allowed Writes

```text
specs/<PARENT_SPEC_ID>/decomposition.md
specs/<PARENT_SPEC_ID>/feature-index.md
specs/<ALLOCATED_SPEC_ID>/spec.md
```

The parent Capability spec and architecture are read-only.

## Allocation

1. Read only direct child directories of `specs/` matching `^[0-9]{3}-`.
2. Find the highest numeric prefix and reserve the next contiguous numbers for
   all selected candidates before writing anything.
3. Preserve candidate order from `feature-index.md`.
4. Build each identifier as:

   ```text
   <NNN>-<capability-key>-<candidate-key>-<candidate-slug>
   ```

5. Reject collisions and gaps introduced during the current allocation.
6. Never use Git branches or session metadata as identifier sources.

## Generation

For every selected candidate:

1. Create one direct child of `specs/`.
2. Create `spec.md` from the standard SpecKit specification template.
3. Include:
   - `SPEC_ID`
   - Parent Capability name and `PARENT_SPEC_ID`
   - Candidate key
   - Scope and non-scope
   - One responsibility
   - Dependencies using top-level specification IDs
   - Acceptance criteria
4. Preserve all source blueprint content; do not invent additional candidates.
5. Update `decomposition.md` and `feature-index.md` with allocated IDs and
   top-level paths.

## Rules

- Every generated Atomic Specification is a first-class SpecKit specification.
- Generate no more than one independently testable story per specification.
- Keep each specification within fifteen expected implementation tasks.
- Do not create plans, tasks, contracts, code, branches, or implementation
  artifacts.
- Do not modify existing specifications unless the user explicitly requests
  re-expansion.
- Stop before writing if any candidate cannot be mapped to exactly one
  top-level specification.

## Forbidden Outputs

Never create specification directories below the parent Capability. Every
generated specification must be a direct child of `specs/`.
