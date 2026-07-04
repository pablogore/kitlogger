# ADR-008: Logging Pipeline Consolidation

## Status

Accepted

## Context

The workspace contains two non-communicating logging pipelines:

- **Pipeline A** (production): `kitlogger` (facade) → `kitlogger-log-domain` + `kitlogger-formatter` + `console-exporter` + `telemetry-adapter-contracts` + `telemetry-types` + `telemetry-config-semantics`.
- **Pipeline B** (orphaned): `telemetry-transport-contract`, reachable only from its own tests, reimplementing `Logger`/`Formatter`/`Output`/`Sampler`/`Buffer`/`Redactor` from scratch on top of `kit_config::LoggingConfig`.

A full responsibility-by-responsibility audit (see engram `architecture/telemetry-transport-contract-fate`) found that `telemetry-transport-contract` mixes two unrelated bounded contexts:

1. A generic transport/envelope contract (`batch.rs`, `payload.rs`, `transport.rs`, `error.rs`) that duplicates types already owned by `telemetry-types` per **ADR-007 (Accepted)** — including a second, independent copy of `BackpressureSignal`, which ADR-007 already declared canonical in `telemetry-types`. This is not a new duplication risk; it is proof that an unreachable crate can silently violate an already-accepted ADR, because nothing forced it to depend on the canonical crate.
2. A complete, self-contained logging pipeline (`event.rs`, `formatter.rs`, `output.rs`, `sampling.rs`, `redaction.rs`, `rotation.rs`, `buffering.rs`, `provider.rs`, `logger.rs`) that plays the same role as `KITLogger`, and contains capabilities — sampling, redaction, buffering, file rotation — that exist nowhere else in the workspace.

`LoggingConfig` (materialized by `kit-config`) is not yet consumed by the production facade. `telemetry-transport-contract` is the *only* crate in the workspace depending on `kit-config` today, and it reaches across the filesystem into the sibling `kit-config` repo to do so.

## Decision

1. **`kit-logger` maintains exactly one logging pipeline.** `KITLogger` (in the `kitlogger` crate) is the single facade a host imports. No second `Logger` type is introduced or kept.
2. **`telemetry-transport-contract` does not survive as a parallel pipeline.** Every module is resolved to exactly one of three outcomes — never left as-is, never copy-pasted wholesale:
   - **Absorb the responsibility** into `kit-logger` core (sampling, redaction, buffering, file output + rotation, multi-output dispatch, formatting, and the filter→sample→redact→buffer→format→dispatch orchestration currently in `logger.rs`). Absorbing means the *behavior* is preserved and re-implemented against the framework's existing conventions (traits, error handling, testing patterns already established in `kitlogger-formatter`/`console-exporter`) — not that the orphaned source is transplanted verbatim.
   - **Delete** anything that is a duplicate of a decision already made elsewhere (`ConsoleOutput` vs. `console-exporter`; `event::LogEvent` vs. `kitlogger_log_domain::LogRecord`; `batch.rs`/`payload.rs`/`BackpressureSignal` vs. `telemetry-types` per ADR-007).
   - **Transfer** anything that belongs to a different bounded context to that context's own roadmap, not into `kit-logger` (`transport.rs`'s `Transport` trait / `DeliveryMode` semantics → `telemetry-adapter-contracts`, as input to its own future decision, not an automatic migration).
3. **`kitlogger_log_domain::LogRecord` is the single canonical log record model.** No other "log event"/"log record" shape is introduced or kept (this retires `telemetry-transport-contract::LogEvent` and does not create a new one).
4. **`LoggingConfig` is the single configuration model owned by the Logging bounded context.** `kit-config` is the configuration framework: it materializes `LoggingConfig` (loading it from TOML/YAML/env) and validates it — `kit-config` does not own the model's shape or semantics; that ownership belongs to Logging. The dependency on `kit-config` moves to originate at `kitlogger` (the facade), not at an unreachable crate. `telemetry-config-semantics::TelemetryConfig`'s overlapping concepts (enabled-flags, sampling) must be reconciled against `LoggingConfig` before or alongside this move, per the prior audit (engram `architecture/kitlogger-kit-config-readiness`).
5. **Pipeline order is filter → sample → redact → buffer (raw record) → format → dispatch.** Formatting happens after filtering/sampling/redaction (don't format what will be dropped or must still be redacted as structured data), and buffering holds the raw record, deferring format+output cost to flush time.
6. **Observability consumes `LogRecord` through adapters/plugins; it is never a stage inside the internal pipeline.** "Outputs" and "Plugins" are the same seam, not two sequential stages — Console and File are the built-in plugins dispatched through the same registry (`telemetry-adapter-contracts::AdapterRegistry`) that OTLP/metrics/tracing-correlation exporters will use. `KITLogger` already implements `ExporterAdapter`/`TelemetryDelivery`; today `deliver()` is a no-op — this decision activates that seam rather than inventing a new one.

## Consequences

### Positive

- One pipeline, one config domain, one record type — no future contributor can accidentally build against the wrong `Logger`/`LogEvent`/`LoggingConfig` again.
- Real capability gaps (sampling, redaction, buffering, file output) get filled using logic that already exists and was already validated by tests in the orphaned crate, without dragging in its transport/envelope half.
- The plugin seam (`AdapterRegistry`) gets exercised for the first time, since Console/File become its first real registered adapters instead of being hardcoded inside `KITLogger`.

### Negative

- `kitlogger` gains a new external dependency (`kit-config`, cross-repo) that it didn't have before.
- `telemetry-config-semantics` must be reconciled (deprecated, merged, or scoped down) before `kit-config` becomes canonical — this is a prerequisite, not a side effect, and blocks step 4 until resolved.
- Existing tests inside `telemetry-transport-contract` (`tests/{batch_test,integration_tests,payload_test,transport_test}.rs`) will need to move or be rewritten against their new homes; none of them exercise the code from the perspective of the production facade today, so none can be preserved unchanged.

## Migration Sequencing

Architectural ordering only (see engram `architecture/telemetry-transport-contract-fate` for the full component-by-component table, and this initiative's `design.md` for the executable migration plan):

1. Reconcile `telemetry-config-semantics` vs. `kit_config::LoggingConfig` overlap.
2. Wire `kit-config` into `kitlogger` directly.
3. Absorb leaf capabilities with no dependents yet: redaction, then sampling.
4. Absorb buffering, then output (retire `ConsoleOutput`, adopt `FileOutput` + `RotationManager`), then formatter reconciliation.
5. Fold `logger.rs`'s orchestration sequencing into `KITLogger` itself; retire `LoggerProvider` as a standalone type.
6. Retire `event::LogEvent`, re-pointing everything migrated above onto `kitlogger_log_domain::LogRecord`/`LogContext`.
7. Delete `batch.rs`/`payload.rs`/the duplicate `BackpressureSignal` immediately (independent of the rest — no dependency on steps 1–6).
8. Delete the `telemetry-transport-contract` crate once every module is empty.
9. Only after step 8: build Logger Plugins (OTLP, metrics, tracing-correlation) against the now-unified emission point.

## Related

- ADR-007: Shared Canonical Types (`telemetry-types`) — this decision is what `telemetry-transport-contract`'s transport/envelope half violated by existing in isolation.
- ADR-009: Correlation ID Unification — must land before any tracing-correlation plugin is built (see step 9 above); otherwise the same "two independent identifier types" problem this crate had with `PayloadEnvelope`/`BackpressureSignal` repeats with `CorrelationId`.
- ADR-010: Canonical Domain Models — the general rule this ADR is a concrete instance of.
- engram `architecture/telemetry-transport-contract-fate` — full component classification table and reasoning.
- engram `architecture/kitlogger-kit-config-readiness` — prior audit that first identified the two disconnected logger facades and the `telemetry-config-semantics` overlap.
- `openspec/specs/telemetry-config-semantics/spec.md` — canonical spec this initiative's first change (this one) modifies.
