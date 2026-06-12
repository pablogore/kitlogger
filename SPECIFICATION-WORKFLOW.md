# Specification Workflow

The repository uses a flat specification model. Every implementable atomic unit
is an independent top-level SpecKit specification.

```text
Specify Capability
    |
Architecture
    |
Expand
    |
Atomic Specifications
    |
Clarify
    |
Plan
    |
Tasks
    |
Implement
```

## Custom Skills

Only two methodology skills are custom:

- `architecture`: creates the architectural decomposition blueprint.
- `expand`: creates independent top-level specifications from that blueprint.

Clarify, plan, tasks, and implement use native SpecKit commands.

## Directory Model

```text
specs/
├── 001-capability/
│   ├── spec.md
│   ├── architecture.md
│   ├── decomposition.md
│   └── feature-index.md
├── 002-capability-01-first-atomic-spec/
│   └── spec.md
├── 003-capability-02-second-atomic-spec/
│   └── spec.md
└── 004-capability-03-third-atomic-spec/
    └── spec.md
```

Specification directories may exist only as direct children of `specs/`.

## Architecture

Architecture defines Capability boundaries, domain boundaries, constraints,
decomposition strategy, dependencies, ownership, and Atomic Specification
candidates. It does not create implementation specifications or artifacts.

## Expand

Expand allocates sequential top-level specification IDs, renders each candidate
with the standard SpecKit specification template, and updates the parent
decomposition and feature index.

Each generated specification records its parent Capability and can independently
run:

```text
/speckit.clarify
/speckit.plan
/speckit.tasks
/speckit.implement
```

## Migration Checklist

- Preserve every source `spec.md` section and clarification answer.
- Preserve architecture, plan, research, contracts, quickstart, and tasks.
- Move each legacy child unit to a unique top-level specification.
- Update parent, dependency, decomposition, and index references.
- Verify native SpecKit commands resolve each migrated specification.
- Remove empty legacy child directories and custom downstream wrappers.

## Rollback

Before migration, record a complete path and content-hash inventory. Rollback
restores files to those recorded paths and reverses only reference changes made
by the migration. Never regenerate specification content during rollback.
