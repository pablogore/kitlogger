---
name: plan-atomic-feature
description: Plan one expanded Atomic Feature using explicit SPEC_ID and AF_ID. Use when technical design artifacts are needed for exactly one validated Atomic Feature without creating tasks or branches.
---

# Plan Atomic Feature

Require explicit `SPEC_ID` and `AF_ID`.

## Identifier Gate

- Validate `SPEC_ID` with
  `^[0-9]{3}-[a-z0-9]+(?:-[a-z0-9]+)*$`.
- Validate `AF_ID` with `^af-[0-9]{3}$`.
- Record both sources as `explicit-user-input`.
- Never use branch, session, feature, or repository allocation metadata.
- Resolve exactly one expanded Atomic Feature at
  `specs/<SPEC_ID>/<AF_ID>-*/spec.md`.

## Allowed Reads

Read the Constitution, parent Architecture and Capability, decomposition,
Atomic Feature, and `.specify/templates/plan-template.md`.

## Allowed Writes

Within the matching Atomic Feature directory only:

```text
plan.md
research.md
data-model.md
quickstart.md
contracts/*
```

Create optional artifacts only when required by this Atomic Feature.

## Rules

- Every design decision traces to an Atomic Feature requirement.
- Do not include sibling scope or additional entities and responsibilities.
- Do not modify specifications or decomposition artifacts.
- Do not create tasks, branches, or implementation code.
- Stop if the source is a stub or fails atomicity validation.
