# Design: Logging Pipeline Consolidation

## Technical Approach (this change: Phases 1–2)

### Phase 1 — Configuration Ownership Reconciliation

`telemetry-config-semantics::TelemetryConfig` is scoped down, not merged into `kit_config` and not deprecated outright:

- Removed: `correlation_enabled`, `SamplingPolicy`/`SamplingPolicyType`/`sampling_rate` — canonical ownership of these concepts is resolved by ADR-008 §4 and ADR-010; this phase executes that resolution, it does not re-derive it.
- Removed as a consequence: `EffectiveTelemetryState::Fallback`. Its only trigger was `sampling_rate` validation failure (see `openspec/specs/telemetry-config-semantics/spec.md` FR-007/FR-008). With that validation gone, nothing can produce `Fallback` — it would be dead code if kept.
- Retained, redocumented as plugin-layer-scoped (Migration Plan Phase 10, not logging behavior): `telemetry_enabled` (redefined as the plugin layer's master switch, not "is logging enabled"), `tracing_enabled`, `metrics_enabled`, `propagation_enabled`, `exporters`, `resources`, `verbosity`, `schema_version`.
- `EffectiveTelemetryState`/`effective_state()` itself is retained as a type/method in `telemetry-config-semantics` (it may still serve a future `AdapterRegistry::health()`-style concern) — only `KITLogger`'s dependency on it is removed.

### Phase 2 — Facade Config Wiring

- `kitlogger`'s Cargo.toml gains a direct path dependency on `kit-config` (the same cross-repo path pattern `telemetry-transport-contract` already uses: `{ path = "../../../kit-config/crates/kit-config", ... }`).
- `KITLogger` gains a construction path accepting `LoggingConfig` — the Logging domain's canonical configuration model — materialized and validated by `kit-config`'s `Validation` trait at construction time (fail fast, no new validation logic invented).
- `KITLogger::with_config(TelemetryConfig)` is removed outright — after Phase 1, it triggers no behavior beyond a call to `effective_state()` that no longer exists on its path.
- No pipeline behavior changes: `LoggingConfig.level`, `.sampling`, `.redact`, `.buffering`, `.rotation`, `.output` are all reachable but not consulted by any runtime code path yet. `LoggingConfig.enabled` specifically is *not* wired into any gate here — that gate is designed once, in Phase 5, together with level filtering, rather than built ad hoc now and rebuilt later.

## Architecture Constraints (immutable, from ADR-008/009/010 — govern this change and every later phase)

1. `kit-logger` maintains exactly one logging pipeline (ADR-008 §1).
2. `telemetry-transport-contract` does not survive as a parallel pipeline (ADR-008 §2).
3. `kitlogger_log_domain::LogRecord` is the single canonical log record model (ADR-008 §3, ADR-010).
4. `LoggingConfig` is the single configuration model, owned by the Logging bounded context; `kit-config` materializes and validates it but does not own it (ADR-008 §4) — this change is the first step of establishing that.
5. Pipeline order is filter → sample → redact → buffer (raw record) → format → dispatch (ADR-008 §5) — not built yet, but no earlier phase may build a piece of it out of order.
6. Observability consumes `LogRecord` via adapters/plugins; never a stage inside the internal pipeline (ADR-008 §6).
7. Exactly one `CorrelationId`/`TraceId`/`SpanId`, shared by `kitlogger-log-domain` and `context-propagation`, neither depending on the other (ADR-009) — a later change, not this one.
8. Every domain concept has exactly one canonical owner (ADR-010) — the acceptance gate for every phase, including this one: this change's own removal of `correlation_enabled`/`SamplingPolicy` is itself an application of this rule.

## Full Migration Roadmap (context — only Phases 1–2 are executed by this change)

| Phase | Name | Status | Notes |
|---|---|---|---|
| 0 | Governance Bootstrap | Superseded by adopting openspec | ADR-010 review gate now enforced via openspec's spec-delta mechanism itself |
| 1 | Configuration Ownership Reconciliation | **This change** | `telemetry-config-semantics` spec delta below |
| 2 | Facade Config Wiring | **This change** | `kitlogger-config-integration` spec below |
| 3 | Leaf Capability Absorption: Redaction + Sampling | Future change | Gated on this change landing |
| 4 | Buffering, Output, Formatter Consolidation | Future change | Gated on Phase 3. Crate-boundary guidance: no reflexive crate creation — `Buffer` exists only for the logging pipeline and `Rotation` exists only for `FileOutput`; both land as internal modules of `kitlogger` or the outputs crate, not standalone crates, unless reuse is demonstrated. Contrast with Phase 3's `Redactor`/`Sampler`, which warranted their own crates as genuinely independent, reusable concepts |
| 5 | Orchestration Fold (+ `LoggingConfig.enabled` gate, level filtering) | Future change | Gated on Phases 3–4 |
| 6 | Record Model Retirement (`LogEvent` → `LogRecord`) | Future change | Gated on Phase 5 |
| 7 | Transport/Envelope Cleanup | Future change, parallel track | No dependency on Phases 1–6 |
| 8 | Crate Removal (`telemetry-transport-contract`) | Future change, join point | Gated on Phases 6 AND 7 |
| 9 | Correlation ID Unification (ADR-009) | Future change, parallel track | Gated only by: must land before Phase 10 |
| 10 | Plugin Enablement (OTLP, metrics, tracing-correlation) | Future change, out of current scope | Gated on Phases 8 AND 9 |

## Dependency Graph (target end state, all phases)

```
Host
  └─ kit-config (materializes/validates LoggingConfig)             [Keep]
       └─ kitlogger (facade — KITLogger)                          [this change: gains config wiring]
            ├─ kitlogger-log-domain      (LogRecord, LogContext)   [Keep — canonical]
            │     └─ shared correlation-id primitive               [new — ADR-009, Phase 9]
            ├─ kitlogger-formatter        (Formatter impls)        [Keep — extended, Phase 4]
            ├─ console-exporter           (Console output)         [Keep — extended, Phase 4]
            ├─ telemetry-adapter-contracts (AdapterRegistry)       [Keep — activated, Phase 5/10]
            │     └─ telemetry-types      (PayloadEnvelope, etc.)  [Keep — sole envelope model]
            └─ telemetry-config-semantics                          [this change: scoped down]

context-propagation                                                [Keep — becomes zero-dependent at Phase 7 (change 017), status to confirm]
      └─ shared correlation-id primitive                           [new — ADR-009, Phase 9]

telemetry-transport-contract                                       [REMOVED — Phase 8]
```

## Interfaces / Contracts

This change does not introduce new public API surface beyond what the two spec deltas below require: `KITLogger` becomes constructible from `kit_config::LoggingConfig`, and `TelemetryConfig`'s public field set shrinks. No new traits, no new crates.

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit (`telemetry-config-semantics`) | `TelemetryConfig` no longer exposes `correlation_enabled`/`sampling` | Compile-fail test / field-absence assertion |
| Unit (`telemetry-config-semantics`) | `EffectiveTelemetryState` has exactly 3 variants | Exhaustive `match` compiles without a `Fallback` arm |
| Unit (`telemetry-config-semantics`) | Retained flags still default to `true` and round-trip via serde | Existing FR-001/FR-012-style tests, updated field lists |
| Unit (`kitlogger`) | `KITLogger` constructs from valid `LoggingConfig` | New test against `kit_config::LoggingConfig::default()` (or a minimal valid instance) |
| Unit (`kitlogger`) | `KITLogger` construction rejects invalid `LoggingConfig` | New test using a `LoggingConfig` that fails `Validation` (e.g. out-of-range `Probabilistic` sampling rate) |
| Regression (`kitlogger`) | `KITLogger::log`/`log_record` behavior unchanged | Existing behavioral tests pass unmodified |
| Removal (`kitlogger`) | `with_config(TelemetryConfig)` no longer compiles | `kitlogger/tests/with_config_test.rs` rewritten, not left referencing the removed method |

## Migration / Rollout

No data migration. Both affected crates (`telemetry-config-semantics`, `kitlogger`) are internal-only with no external published consumers — see Rollback Plan in `proposal.md`.

## Open Questions

- None blocking this change. Phase 3 onward (redaction, sampling, buffering, output, formatter, orchestration) will each raise their own design questions when scoped as their own changes.
