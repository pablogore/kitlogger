---
name: decompose-capability
description: Decompose one explicitly identified Capability into capability-local Atomic Feature Stubs. Use when local af identifiers and decomposition artifacts must be created without repository-wide counters or feature branches.
---

# Decompose Capability

Require explicit `SPEC_ID`.

## Identifier Gate

1. Validate `SPEC_ID` with
   `^[0-9]{3}-[a-z0-9]+(?:-[a-z0-9]+)*$`.
2. Record its source as `explicit-user-input`.
3. Never inspect branches, sessions, sibling specifications, or repository
   counters to derive identifiers.
4. Restrict every generated path to `specs/<SPEC_ID>/`.

## Required Inputs

- `specs/<SPEC_ID>/spec.md`
- `specs/<SPEC_ID>/architecture.md`
- `.specify/memory/constitution.md`

## Allowed Writes

```text
specs/<SPEC_ID>/decomposition.md
specs/<SPEC_ID>/feature-index.md
specs/<SPEC_ID>/af-###-slug/spec.md
```

Atomic Feature directories are direct children of the Capability. Never create
a `features/` wrapper.

## Local IDs

- Begin at `af-001` for this Capability's decomposition.
- Increment only within the current Capability.
- Match `^af-[0-9]{3}$`.
- Never consult another Capability to choose an ID.
- A rerun may preserve matching IDs from this Capability's own decomposition,
  but may not use repository-wide allocation state.

## Decomposition Heuristics

Before creating an Atomic Feature, evaluate whether the candidate:

1. Contains multiple major domain entities.
2. Contains multiple independently implementable or independently evolvable
   concerns.
3. Would likely require more than one pull request.
4. Would likely require more than fifteen implementation tasks.
5. Contains a section that could reasonably become its own Atomic Feature.

If any answer is yes, continue decomposing the candidate before assigning its
final Atomic Feature identity. Do not stop at a subsystem, complete domain
model, collection of unrelated entities, or broad architectural concept.
Cohesion does not override this rule: a cohesive candidate with multiple
independently evolvable domain entities or concerns must still be decomposed.

Prefer boundaries centered on one entity, one responsibility, one contract,
one lifecycle, or one integration point. Target Small or Medium estimated size.
Treat Large as a signal that further decomposition is required.

Reject broad candidates such as:

- `Core Telemetry Domain Model` when it includes Resource, Span, Trace, and
  Metric
- `Context Propagation and Correlation` when propagation and correlation can
  evolve independently
- `Transport-Agnostic Telemetry Flow` when it includes transport context,
  context carrier, and middleware integration

## Stub Rules

Each stub contains only identity, parent, one primary entity, one
responsibility, estimated size, and dependencies. Do not generate stories,
requirements, acceptance criteria, plans, tasks, contracts, or code.

## Validation

Before finalizing, verify every Atomic Feature has:

- One primary entity
- One primary responsibility
- No second domain entity or concern that can evolve independently
- A scope implementable in one pull request
- An expected implementation size of no more than fifteen tasks
- No obvious child Atomic Features remaining
- A Small or Medium estimated size

If any check fails, continue decomposition and validate the resulting
candidates again.

Reject:

- Any path whose first directory below `specs/` differs from `SPEC_ID`
- Any branch-, session-, feature-, or counter-derived identifier
- Any Atomic Feature with multiple entities or responsibilities
- Any Large Atomic Feature
- Any Atomic Feature with independently implementable or independently
  evolvable child concerns, even when they are cohesive
- Cyclic dependencies
- Any write outside the allowed set
