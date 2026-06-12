---
name: plan-atomic-feature
description: Plan one explicitly provided Atomic Feature as a fully isolated unit. Use when SpecKit planning artifacts must be generated only inside an existing specs/<capability>/af-###-<name>/ directory without feature-directory, branch, or numeric-prefix inference.
---

# Plan Atomic Feature

Reuse the SpecKit planning methodology and `.specify/templates/plan-template.md`
inside one isolated Atomic Feature. Do not dispatch the standard
`/speckit.plan` command, its setup script, extension hooks, or agent-context
updates because they may write outside the Atomic Feature.

## Input

Require one explicit Atomic Feature directory path, for example:

```text
specs/001-telemetry-observability/af-002-context-propagation-and-correlation
```

Resolve exactly the path supplied by the user. Do not search by `SPEC_ID`,
`AF_ID`, numeric prefix, branch, session, feature metadata, sibling directory,
or repository counter.

Canonicalize the existing directory as `AF_ROOT` and verify that it is a direct
Atomic Feature child of one Capability under the repository `specs/`
directory. Reject ambiguous, missing, inferred, or non-canonical targets.

Treat the provided path and identifiers parsed from it as explicit user input.

## Context Resolution

Parse the Capability directory name as `SPEC_ID` and the Atomic Feature
directory prefix as `AF_ID`. Record both identifier sources as
`explicit-user-input`. Validate `SPEC_ID` with
`^[0-9]{3}-[a-z0-9]+(?:-[a-z0-9]+)*$` and `AF_ID` with `^af-[0-9]{3}$`.

Require:

```text
AF_SPEC=<AF_ROOT>/spec.md
CAPABILITY_DIR=<AF_ROOT parent>
CAPABILITY_SPEC=<CAPABILITY_DIR>/spec.md
ARCHITECTURE_SPEC=<CAPABILITY_DIR>/architecture.md
```

If `AF_SPEC` is missing, stop with:

```text
ERROR:
Atomic Feature spec not found:
<AF_ROOT>/spec.md
```

Read the Atomic Feature dependencies from its `spec.md` and, when present, the
matching entries in the parent `decomposition.md` and `feature-index.md`.
Resolve dependency specifications read-only. Do not include sibling scope that
is not an explicit dependency.

Fail before planning when:

- `AF_ROOT` does not exist
- `CAPABILITY_SPEC` does not exist
- `ARCHITECTURE_SPEC` does not exist
- The resolved paths escape the repository or the explicit Capability

## Planning Workflow

Execute only the side-effect-free planning phases:

1. Load `AF_SPEC` and the Constitution.
2. Add `CAPABILITY_SPEC`, `ARCHITECTURE_SPEC`, and resolved dependencies as
   read-only planning context.
3. Read the existing resolved `plan-template`; render it directly to
   `<AF_ROOT>/plan.md`.
4. Execute the standard Technical Context, Constitution Check, research,
   design, contracts, and quickstart phases.
5. Generate `tasks.md` only when task generation is explicitly invoked.

The parent context constrains the plan but does not become additional plan
scope. Planning remains limited to the selected Atomic Feature.

## Preflight Write Manifest

Before any write:

1. Capture an in-memory baseline inventory of repository file paths, types, and
   content hashes so changes made during this run can be identified separately
   from pre-existing worktree changes.
2. Build the complete planned-write manifest.
3. Canonicalize each destination through its nearest existing parent. Reject
   any symlink component whose resolved path escapes `AF_ROOT`.
4. Require every destination to equal `AF_ROOT` or begin with
   `<AF_ROOT>/`.
5. Require every destination to match the allowed outputs below.

If any planned destination fails, stop before the first write and report:

```text
ISOLATION VIOLATION: planning attempted to write outside the Atomic Feature directory.
```

## Allowed Writes

Allow only:

```text
<AF_ROOT>/spec.md
<AF_ROOT>/plan.md
<AF_ROOT>/research.md
<AF_ROOT>/data-model.md
<AF_ROOT>/quickstart.md
<AF_ROOT>/contracts/*
<AF_ROOT>/tasks.md
```

Treat `<AF_ROOT>/spec.md` as read-only unless the user explicitly requests a
specification update; omit it from the planned-write manifest otherwise.
Create `contracts/` only when external interfaces require it. Create or update
`tasks.md` only during an explicitly invoked task generation phase.

Never create sibling Atomic Features, parent-level planning artifacts, new
specification directories, files under another specification, or any file
outside `AF_ROOT`. Never update `AGENTS.md`, branches, SpecKit state, commands,
scripts, templates, hooks, or agent context during a planning run.

## Completion Validation

After planning:

1. Compare the repository to the in-memory baseline.
2. Classify every changed path as created, modified, deleted, or renamed.
3. Canonicalize every changed path and require it to remain inside `AF_ROOT`.
4. Require every changed path to match the allowed-write manifest.
5. Verify `plan.md`, `research.md`, `data-model.md`, and `quickstart.md`.
6. Verify `contracts/` when required and `tasks.md` when task generation ran.
7. Verify no parent, sibling, or new specification directory was changed.

Print:

```text
Atomic Feature: <project-relative AF_ROOT>
Files created:
- <path>
Files modified:
- <path>
```

Also report deleted or renamed files when present.

If any path changed outside `AF_ROOT`, fail the run, list every offending path,
and do not report success. Existing unrelated files do not cause failure unless
they changed during this run.
