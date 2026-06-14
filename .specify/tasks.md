# Tasks: Architecture Specification Workflow Refactoring

**Input**: Constitutional violation report — Architecture workflow generated implementation documentation instead of domain artifacts

**Prerequisites**: Constitution v3.1 hierarchy rules

---

- [x] T001 Refactor `specify.md`, `architecture-spec-template.md`, and `spec-arquitecture.md` to comply with Constitution v3.1 — Architecture generation is now implementation-agnostic with banned-term validation, outputs only `specs/000-<domain>-architecture/spec.md`, and stops after generation. Capability generation also stops. Only Atomic Features proceed to planning/implementation.
