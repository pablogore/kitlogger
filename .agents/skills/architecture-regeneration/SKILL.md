---
name: architecture-regeneration
description: Regenerate architecture artifacts (architecture.md, decomposition.md, feature-index.md) from implementation changes when architecture drift is detected and approved.
---

# Architecture Regeneration

Regenerate architecture artifacts after approved architecture changes.

## Behavior

1. Verify an approved architecture finding exists authorizing regeneration.
2. Load the active specification's `spec.md` for scope and requirements.
3. Read implementation structure in `src/**` to understand entities, modules, and public APIs.
4. Regenerate the following artifacts under `specs/<SPEC_ID>/`:
   - `architecture.md` — updated capability boundaries, concepts, constraints, and decomposition strategy
   - `decomposition.md` — updated Atomic Specification candidates matching current implementation
   - `feature-index.md` — updated index of Atomic Specifications and their allocation
5. Preserve all existing content that remains valid.

## Rules

- Never modify frozen artifacts (`spec.md`, `research.md`, `data-model.md`, `contracts/**`, `plan.md`, `tasks.md`, `tech-stack.yaml`).
- Never modify implementation code, tests, or operational documentation.
- Only modify `architecture.md`, `decomposition.md`, and `feature-index.md`.
- Require explicit approval before making changes.

## Failure

Stop with `REGENERATION FAILURE: <reason>` when:
- No approved architecture finding exists.
- Any file outside the three allowed outputs would be modified.
