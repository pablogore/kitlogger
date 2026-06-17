# Technology Declaration Harness

Every specification processed by `/speckit.plan`, `/speckit.tasks`, or
`/speckit.implement` must contain `tech-stack.yaml`.

The required declarations are:

```yaml
language: <explicit value>
runtime: <explicit value>
testing: <explicit value>
```

Add frameworks, databases, transports, package managers, SDKs, cloud providers,
deployment targets, and serialization technologies when the specification uses
them. Values may be scalars or YAML lists. Nested mappings are intentionally not
accepted, keeping the declaration auditable.

Before a workflow can write, the harness reads:

- `.specify/memory/constitution.md`
- the active `spec.md`
- current or parent capability and architecture context
- `plan.md`, `tasks.md`, `research.md`, `data-model.md`, `quickstart.md`
- files under `contracts/`

The run stops when `tech-stack.yaml` is missing, a required declaration is
empty, capability/architecture context cannot be resolved, or a recognized
technology reference is not declared. Parent context is resolved only from
explicit metadata or an exact `Specification ID` entry in one capability's
`feature-index.md` or `decomposition.md`; numeric prefixes are never used.
The validator does not create or modify files.

Use the validator directly with:

```sh
scripts/validate-tech-stack.sh \
  --feature-dir specs/<SPEC_ID> \
  --phase plan
```

The declaration is an allowlist, not a recommendation engine. Architecture,
planning, task generation, contracts, quickstarts, and implementation must not
derive technologies from domain terminology.
