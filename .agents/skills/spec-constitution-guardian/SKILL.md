---
name: spec-constitution-guardian
description: Audit repository-native specification skills for Constitution v3.1 identifier, routing, and write-boundary compliance. Use when skills are added or changed, or when legacy SpecKit assumptions may have returned.
---

# Spec Constitution Guardian

Audit every `SKILL.md` under `.agents/skills/` that reads or writes `specs/`.

## Required Checks

For every specification workflow skill:

1. Require explicit `SPEC_ID`.
2. Require the capability-scoped format
   `^[0-9]{3}-[a-z0-9]+(?:-[a-z0-9]+)*$`.
3. Require identifier provenance to be `explicit-user-input`.
4. Reject branch-, session-, feature-, and repository-counter-derived IDs.
5. Reject feature branches and repository-wide allocation.
6. Reject writes whose first directory below `specs/` differs from `SPEC_ID`.
7. Require Atomic Feature IDs to match `^af-[0-9]{3}$` and remain local to one
   Capability.
8. Reject `features/<FEATURE_ID>`, generic Feature Specifications, multi-story
   specifications, and multi-entity Atomic Features.

## Validation Command

Run:

```sh
.agents/skills/spec-constitution-guardian/scripts/validate-skills.sh
```

Do not modify files under `specs/` during an audit.

Report every audited skill, legacy assumption, identifier source, required
fix, validation rule, compliant example, and rejected example.
