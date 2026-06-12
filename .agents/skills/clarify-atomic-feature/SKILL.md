---
name: clarify-atomic-feature
description: Clarify one expanded Atomic Feature from explicit SPEC_ID and AF_ID. Use to resolve bounded ambiguities without adding entities, responsibilities, sub-features, or changing decomposition.
---

# Clarify Atomic Feature

Require explicit `SPEC_ID` and `AF_ID`.

## Identifier Gate

- Validate `SPEC_ID` with
  `^[0-9]{3}-[a-z0-9]+(?:-[a-z0-9]+)*$`.
- Validate `AF_ID` with `^af-[0-9]{3}$`.
- Resolve only `specs/<SPEC_ID>/<AF_ID>-*/spec.md`.
- Record both identifier sources as `explicit-user-input`.
- Never derive identifiers from branch, session, feature, or counter metadata.

## Procedure

1. Read the Constitution, parent Capability, decomposition, and expanded Atomic
   Feature.
2. Ask at most five questions that materially affect the existing entity,
   responsibility, requirements, constraints, or acceptance criteria.
3. Apply accepted answers only to the matching Atomic Feature `spec.md`.
4. Keep exactly one story and no more than eight functional requirements.
5. Revalidate atomicity after each write.

Do not modify Architecture, Capability, decomposition, index, siblings, plans,
tasks, or implementation files. If clarification requires broader scope, stop
and route to `decompose-capability`.
