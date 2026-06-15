---
name: implementation-remediation
description: Remediate non-compliant implementation code to align with approved architecture artifacts. Fix code, tests, and operational documentation without modifying frozen artifacts.
---

# Implementation Remediation

Remediate non-compliant implementation to match approved architecture.

## Behavior

1. Accept architecture audit findings as input.
2. For each finding, determine the required change in `src/**`, `tests/**`, `examples/**`, or `benchmarks/**`.
3. Apply changes to align implementation with:
   - Data model definitions in `data-model.md`
   - API contracts in `contracts/**`
   - Architecture decisions in `architecture.md`
4. Verify traceability: every code change maps to a task, every task maps to a success criterion.
5. Run tests after each remediation step.

## Rules

- Never modify frozen artifacts (`spec.md`, `research.md`, `data-model.md`, `contracts/**`, `plan.md`, `tasks.md`, `tech-stack.yaml`).
- Never expand scope beyond approved implementation directories.
- Each remediation must reference the finding it resolves.
- Run full test suite before declaring completion.

## Failure

Stop with `REMEDIATION FAILURE: <reason>` when:
- Remediation requires modifying a frozen artifact.
- Tests fail after remediation.
- Traceability is broken.
