---
name: generate-atomic-feature-tasks
description: Generate bounded implementation tasks for one planned Atomic Feature using explicit SPEC_ID and AF_ID. Use when tasks.md must trace to one Atomic Feature and contain no more than fifteen tasks.
---

# Generate Atomic Feature Tasks

Require explicit `SPEC_ID` and `AF_ID`.

## Identifier Gate

- Validate `SPEC_ID` with
  `^[0-9]{3}-[a-z0-9]+(?:-[a-z0-9]+)*$`.
- Validate `AF_ID` with `^af-[0-9]{3}$`.
- Record both sources as `explicit-user-input`.
- Never derive identifiers from branch, session, feature, or counter metadata.
- Resolve exactly one Atomic Feature directory:
  `specs/<SPEC_ID>/<AF_ID>-*/`.

## Required Reads

- Expanded `spec.md`
- Approved `plan.md`
- Optional design artifacts in the same Atomic Feature directory
- `.specify/templates/tasks-template.md`

## Allowed Write

```text
specs/<SPEC_ID>/<AF_ID>-*/tasks.md
```

## Rules

- Generate at most fifteen tasks.
- Map every task to one or more Atomic Feature requirement IDs.
- Cover every requirement.
- Include exact file paths or validation commands.
- Do not create story phases, priorities, entities, responsibilities, branches,
  specifications, plans, or implementation code.
- Reject any task outside the Atomic Feature boundary.
