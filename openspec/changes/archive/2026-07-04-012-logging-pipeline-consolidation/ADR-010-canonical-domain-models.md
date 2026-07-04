# ADR-010: Canonical Domain Models

## Status

Accepted

## Context

The audits behind ADR-007, ADR-008, and ADR-009 independently found the same failure pattern applied to different concepts:

- `LogRecord` was initially thought to exist three times (`kitlogger_log_domain::LogRecord`, `telemetry_transport_contract::LogEvent`, `context_propagation::models::LogRecord`). Audited during Phase 9 closure (change 019): `telemetry_transport_contract::LogEvent` was a genuine duplicate, retired in Phase 6. `context_propagation::models::LogRecord` is not — side-by-side comparison found it shaped after the OpenTelemetry Logs Data Model (`resource`, `instrumentation_scope`, `context` fields with no counterpart in the domain type), a wire DTO a future OTLP exporter would construct *from* the canonical domain `LogRecord`, not a second owner of the same concept. Its `severity: LogSeverity` field is entirely subordinate to that same DTO (unused independently anywhere in `context-propagation`) and inherits the same resolution. `kitlogger_log_domain::LogRecord` remains the sole canonical domain model; ADR-010 is satisfied without further action.
- `PayloadEnvelope`/`TelemetryBatch` exist twice (`telemetry_types` — canonical per ADR-007 — and a second, independent copy in `telemetry-transport-contract`).
- `BackpressureSignal` exists twice, for the same reason — a crate that was never wired into the workspace re-derived a type ADR-007 had already declared canonical, because nothing forced it to depend on the canonical crate instead.
- `CorrelationId`/`TraceId`/`SpanId` exist twice (`kitlogger_log_domain` and `context_propagation::models`), addressed in ADR-009.
- `TelemetryConfig`'s capability flags and sampling policy overlap conceptually with `kit_config::LoggingConfig`'s enabled-state and sampling, addressed in this initiative's `telemetry-config-semantics` change.

In every case the duplication appeared because crates evolved independently, without a shared rule about who is allowed to define a given concept. The consequences are consistent across all cases: unnecessary conversions between equivalent types, parallel pipelines that don't communicate, loss of a single model identity for a concept, and behavior implemented twice with no guarantee the two implementations agree.

This pattern is not specific to logging, telemetry, or tracing. It will recur for any domain concept shared across multiple crates or bounded contexts — the same failure mode applies equally to identity/security concepts (`SecurityContext`, `Identity`, `Principal`), multi-tenancy (`TenantId`, `Permission`, `ResourceName`), or messaging (`Event`, `Message`, `Command`, `Query`). The rule below is therefore stated without reference to logging or telemetry, so it governs the whole workspace, not just this consolidation.

## Decision

Every domain concept has a single canonical owner and a single canonical model. All other crates extend behavior through traits, adapters or plugins instead of redefining equivalent types.

Concretely:

- A domain concept belongs to exactly one bounded context. That bounded context's crate is the sole owner of the model representing it.
- Every other crate that needs the concept consumes the canonical model directly, enriches it through traits, wrapper types, or adapters, or extends it through a plugin — it does not define a new type that represents the same concept under a different name.
- "The same concept under a different name" is judged by what the type represents, not by its literal shape. A type with different fields than the canonical model, but the same reason to exist, is still a violation of this rule — divergent shape is not evidence of a different concept, it is usually evidence that the duplicate evolved independently and drifted.

## Enforcement

This rule depends on review discipline, not tooling. To keep that discipline concrete rather than aspirational: every future OpenSpec proposal that introduces a new public domain model MUST declare, in its `proposal.md` (or `design.md`):

- **Canonical Owner** — which bounded context or crate owns this concept.
- **Canonical Model** — the specific type or module that represents it.
- **Consumers** — which other crates or capabilities will consume it.
- **Existing Competing Models** — any prior art already representing this concept, named explicitly, with a stated decision to extend or consume it rather than duplicate it (or, if genuinely distinct, why it does not collide with the existing model).

This is a process requirement enforced through proposal review, not through automation or CI — a proposal missing this declaration is incomplete, the same way a proposal missing a rollback plan is incomplete.

## Consequences

### Positive

- Prevents the specific failure pattern found by ADR-007/008/009 from recurring for any future concept, in any bounded context — this rule generalizes the fix rather than special-casing it to logging.
- Makes "which crate owns this concept" answerable by inspection instead of by convention or memory — the model's location *is* the answer.
- Aligns plugin/extension design with bounded contexts instead of implementation convenience: extending a concept means adding a trait impl or adapter, never adding a parallel struct.

### Negative

- Requires active review discipline: nothing today automatically catches a new type that duplicates an existing concept before it merges. Absent that discipline, this rule is aspirational rather than enforced.
- Can create friction when a consuming crate wants a field the canonical model doesn't have yet — the correct response is to extend the canonical model or wrap it, not to fork it, which is a slower path than copy-pasting a similar struct.

## Scope

This ADR is project-wide, not scoped to `kit-logger` or the logging pipeline. It applies retroactively to the findings in ADR-007, ADR-008, and ADR-009 (which are concrete instances of the rule stated here) and prospectively to every future domain concept introduced anywhere in the workspace.

## Related

- ADR-007: Shared Canonical Types — first concrete instance of this rule, scoped to telemetry envelope types.
- ADR-008: Logging Pipeline Consolidation — applies this rule to `LogRecord` and the logging pipeline as a whole.
- ADR-009: Correlation ID Unification — applies this rule to `CorrelationId`/`TraceId`/`SpanId`.
