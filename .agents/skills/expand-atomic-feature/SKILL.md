---
name: expand-atomic-feature
description: Expand one Constitution v3.1 Atomic Feature Stub using explicit SPEC_ID and AF_ID. Use when a local stub must become a complete single-entity specification without changing decomposition or creating plans.
---

# Expand Atomic Feature

Require explicit `SPEC_ID` and `AF_ID`.

## Identifier Gate

1. Validate `SPEC_ID` with
   `^[0-9]{3}-[a-z0-9]+(?:-[a-z0-9]+)*$`.
2. Validate `AF_ID` with `^af-[0-9]{3}$`.
3. Record both sources as `explicit-user-input`.
4. Never derive either input from branch, session, feature, or repository
   counter metadata.
5. Resolve exactly one directory matching
   `specs/<SPEC_ID>/<AF_ID>-*/`.

## Required Reads

- `.specify/memory/constitution.md`
- `.specify/templates/atomic-feature-spec-template.md`
- Parent Capability and Architecture
- Parent `decomposition.md` and `feature-index.md`
- The matching Atomic Feature Stub

## Allowed Writes

- The matching `specs/<SPEC_ID>/<AF_ID>-*/spec.md`
- Only the matching status entry in `specs/<SPEC_ID>/feature-index.md`

## Rules

- Preserve local ID, parent, primary entity, responsibility, and dependencies.
- Generate exactly one story and no more than eight functional requirements.
- Keep estimated implementation at fifteen tasks or fewer and one pull request.
- Do not create entities, responsibilities, sub-features, plans, tasks,
  branches, contracts, or implementation artifacts.
- If expansion exceeds the stub boundary, stop and route to decomposition.
