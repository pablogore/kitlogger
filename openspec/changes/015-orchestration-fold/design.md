# Design: Orchestration Fold

## Pipeline Sequencing

```
KITLogger::log / log_record
  │
  ├─ 1. Enabled gate       — LoggingConfig.enabled == false → stop, no further processing
  │
  ├─ 2. Level filter       — record.severity below LoggingConfig.level's threshold → stop
  │                           (Severity::Fatal always proceeds — see "Severity vs. LogLevel" below)
  │
  ├─ 3. Sample             — kitlogger-sampling::Sampler says no → stop
  │
  ├─ 4. Redact             — kitlogger-redaction::Redactor produces the record that continues
  │
  ├─ 5. Buffer             — kitlogger-buffering::Buffer holds the (redacted) record;
  │                           steps 6–7 happen only when the buffer flushes it
  │                           (immediately, if buffering is disabled)
  │
  ├─ 6. Format             — kitlogger-format-selection maps LoggingConfig.format to a
  │                           kitlogger_formatter::LogFormat, which selects the Formatter
  │
  └─ 7. Dispatch           — the formatted string + severity is dispatched via
                              output-adapter-contracts's registry to every registered output
                              (console-exporter only, this phase — see proposal.md)
```

Each stage is exactly the capability already frozen in changes 013/014; this design fixes only the order and the glue between them.

## Severity vs. LogLevel

`kitlogger_log_domain::Severity` has six variants (`Trace, Debug, Info, Warn, Error, Fatal`). `kit_config::LogLevel` has five (`Trace, Debug, Info, Warn, Error`) — there is no `LogLevel::Fatal`, so no operator-configured threshold can ever equal or exceed `Fatal`. Resolution: `Severity::Fatal` always passes the level filter regardless of the configured `LogLevel`. This is not a `kit_config` gap requiring a schema change — a threshold can only restrict *up to* the most severe value it can name, and `Fatal` being unconditionally at least as severe as any `LogLevel` value is inherent to the domain's own ordering, not something an operator needs a config option to express.

## Default Output Registration and the `kit_config::OutputTarget` Gap

`LoggingConfig.output.targets: Vec<OutputTarget>` selects among `Console`, `Stdout`, `Stderr` — all of which `console-exporter`'s existing stream-routing already understands. `KITLogger` translates this `Vec<OutputTarget>` into `console-exporter`'s routing configuration at construction time (a small, facade-level translation, the same kind of thing `kitlogger-format-selection` already does for `LogFormat` — not dignified as its own capability here since it is a single, narrow construction-time step, not a reusable mapping needed by more than one consumer).

There is no `OutputTarget::File` and no file-path field anywhere in `LoggingConfig`. `file-exporter` (change 014) cannot be registered from config as a result. This is tracked as an explicit, external follow-up — not solved by this change, and not worked around by inventing a kitlogger-local config source (which would violate ADR-008 §4's "`LoggingConfig` is the single configuration model" decision).

## Buffering's Effect on `log()`'s Observable Timing

Because Buffer (step 5) may defer formatting and dispatch until a flush condition is met, `KITLogger::log`/`log_record` returning `Ok(())` means "accepted into the pipeline through buffering," not "written to every output." This is a real, user-visible behavior change from `KITLogger`'s current unconditional synchronous format+export. Consequence: `KITLogger`'s already-implemented `flush()`/`shutdown()` (`LifecycleAdapter`) must be extended to guarantee the buffer's contents are flushed through formatting and dispatch before returning — otherwise records accepted but not yet flushed could be silently lost on process exit. This is stated as a requirement (`kitlogger-emission-pipeline` FR-006) rather than left as an implicit assumption.

## Dispatch Ownership Realized

Change 014 split dispatch ownership: mechanism = `output-adapter-contracts`, orchestration = `kitlogger`. This phase is where the orchestration half actually happens: `KITLogger` holds exactly one registry instance, registers `console-exporter` into it at construction, and calls its dispatch mechanism at the end of the pipeline. `telemetry_transport_contract::provider::LoggerProvider` — the orphaned type that played this same "multi-output dispatch" role — is retired; its role does not survive as a second type anywhere (ADR-010).

## Dependency Graph

```
Host
  └─ kitlogger (KITLogger — now the full orchestrator)
       ├─ kitlogger-log-domain
       ├─ kitlogger-formatter          [via kitlogger-format-selection, unchanged]
       ├─ kitlogger-sampling           [now called from log()/log_record()]
       ├─ kitlogger-redaction          [now called from log()/log_record()]
       ├─ kit-config
       ├─ output-adapter-contracts     [now actually used — registry held by KITLogger]
       ├─ console-exporter             [registered by default]
       └─ file-exporter                [built, NOT registered — blocked on kit_config gap]

telemetry-adapter-contracts, telemetry-transport-contract   [unaffected by this change]
```

The only new edges versus change 014's graph are `kitlogger → kitlogger-sampling`, `kitlogger → kitlogger-redaction`, and `kitlogger → output-adapter-contracts` actually being exercised — everything else was already a dependency, just unused until now.

## Ownership Table

| Concept | Owner | What changes in this phase |
|---|---|---|
| Pipeline sequencing (filter→sample→redact→buffer→format→dispatch) | `kitlogger` (`KITLogger`) | New — this phase's core deliverable |
| Enabled gate, level filter | `kitlogger` | New |
| Sampling decision logic | `kitlogger-sampling` (unchanged) | Now called |
| Redaction logic | `kitlogger-redaction` (unchanged) | Now called |
| Buffering logic | `kitlogger`'s internal `Buffer` module (unchanged) | Now called |
| Format selection | `kitlogger`'s internal format-selection module (unchanged) | Now called |
| Formatting | `kitlogger-formatter` (unchanged) | Now called (via selection) |
| Dispatch mechanism | `output-adapter-contracts` (unchanged) | Now called |
| Dispatch orchestration (default registration, when to invoke) | `kitlogger` | New |
| Default output set | `kitlogger` (Console only) | New — File explicitly excluded, see Gap |

## Migration Strategy

Architectural sequencing only:

1. Wire the enabled gate and level filter first — they require no dependency on Phase 3/4 capabilities and establish the pipeline's entry/exit points.
2. Wire sampling and redaction (Phase 3 capabilities) — independent of each other in terms of *how* they're called, but must execute in the fixed order (sample, then redact).
3. Wire buffering (Phase 4) — depends on 1–2 already routing a record to this point.
4. Wire format selection + dispatch (Phase 4) as the pair that fires when the buffer flushes — depends on 3.
5. Extend `flush()`/`shutdown()` to guarantee buffered records are drained through steps 4's format+dispatch.
6. Confirm `file-exporter` remains unregistered and undisturbed; confirm no second dispatch/provider type was introduced.

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | `enabled = false` short-circuits before any other stage runs | Spy/counter on each stage confirming zero invocations |
| Unit | Level filtering drops below-threshold records, always passes `Fatal` | Table-driven over all `(LogLevel, Severity)` pairs, including `Fatal` at the strictest configured `LogLevel` |
| Unit | A sampled-out record never reaches redaction, buffering, formatting, or dispatch | Spy confirming zero downstream invocations when `Sampler` returns false |
| Unit | A redacted record, not the original, is what reaches buffering | Assert buffered content reflects redaction |
| Unit | Buffering defers format+dispatch until flush; disabling buffering makes it synchronous | Assert dispatch call count/timing under both configurations |
| Unit | `flush()`/`shutdown()` drain all buffered records through format+dispatch before returning | Fill the buffer below its flush threshold, call `shutdown()`, assert every record was still dispatched |
| Unit | Console is registered by default; `file-exporter` is not | Inspect the registry's registered identifiers after construction |
| Regression | `console-exporter`'s own `FlushStrategy` and the new `Buffer` compose without either disabling the other | Exercise both configurations (e.g. `BatchFlush` + buffering enabled) and confirm expected write timing at both levels |

## Composability

This phase is the one place composability intentionally narrows: `KITLogger` is explicitly the composition root wiring the standalone capabilities together. This does not contradict changes 013/014's composability statements — those capabilities remain independently usable by a different consumer; this phase only describes *this* consumer's wiring.

## Open Questions

- Whether the buffer's deferred flush is driven by a background thread (matching the orphaned original's OS-thread + `mpsc::channel` design) or a simpler on-next-call check is an implementation decision for the apply phase, not fixed by this design — either satisfies FR-006 as long as `flush()`/`shutdown()` guarantee drainage.
