# Proposal: Output Subsystem Consolidation (Migration Plan Phase 4)

## Intent

ADR-008 already decided `telemetry-transport-contract` disappears and that its unique capabilities are absorbed while its duplicates are deleted. Phase 3 (change 013) absorbed the leaf capabilities (Redaction, Sampling). This proposal absorbs the remaining Output-side capabilities — Console output, File output, Rotation, Buffer, Formatter/config integration, and dispatch — and freezes their ownership so that adding OTLP/Loki/Sentry/CloudWatch/Kafka/NATS later requires zero architectural rediscussion.

A prior review (this session, not yet an ADR) validated against the actual source of `telemetry-adapter-contracts` and found it is a **Telemetry/OTel-provider contract** (its own doc comment says so; `mapping.rs` converts Canonical↔OTel entities; `TelemetryDelivery::deliver` takes a cross-signal `PayloadEnvelope`), not a generic Output Port. Forcing Console/File/Kafka/S3/etc. through it would mean wrapping every local, synchronous write in OTel-shaped batch semantics they don't need. This proposal therefore introduces a dedicated, generic Output Port — it does not reuse `telemetry-adapter-contracts` for this purpose, and does not modify it.

## ADR-010 Domain Model Declaration

Two new public domain models are introduced. Per ADR-010's Enforcement section:

| | Output Port & Registry | Buffer |
|---|---|---|
| **Canonical Owner** | New crate `output-adapter-contracts` | `kitlogger` (internal module — not its own crate; see Design Q4) |
| **Canonical Model** | An `Output`-shaped port every destination implements, plus a registry that dispatches a formatted record to all registered outputs | A batching stage that holds raw, pre-format records and defers format+write cost to flush time |
| **Consumers** | `console-exporter`, `file-exporter` (this change); future `otlp-exporter`, `loki-exporter`, `sentry-exporter`, `cloudwatch-exporter`, `elastic-exporter`, `s3-exporter`, `kafka-exporter`, `nats-exporter` (out of scope, but must not require redesign) | `kitlogger`'s own pipeline sequencing (wired in Phase 5) |
| **Existing Competing Models** | `telemetry_transport_contract::output::{Output, ConsoleOutput, FileOutput}` — the orphaned originals, retired per ADR-008 (`ConsoleOutput` → Replace by `console-exporter`; `FileOutput` → Migrate into `file-exporter`). `telemetry_adapter_contracts::{Adapter, ExporterAdapter, AdapterRegistry}` was evaluated and rejected as a home for this concept — see Intent. | `telemetry_transport_contract::buffering::Buffer` — the orphaned original, retired the same way (Migrate) |

One new small capability is also introduced without a competing model to declare against, because none exists: `kitlogger-format-selection` (mapping `kit_config::LogFormat` to `kitlogger_formatter::LogFormat`) — see Design Q5 for why this is not a modification to the existing, accepted `formatter-contract` capability.

## Scope

### In Scope

- New crate `output-adapter-contracts`: the Output Port every destination implements, and a registry mechanism that dispatches an already-formatted record to all registered outputs, aggregating per-output failures. Depends on `kitlogger-log-domain` only for `Severity` (routing), not `LogRecord` — by the time a record reaches an output, it has already been formatted to a string (pipeline order: Buffer → Format → Dispatch).
- New crate `file-exporter`: a file-based output implementing the new Port, with file rotation as an internal module (not a separate crate).
- Retire `telemetry_transport_contract::output::ConsoleOutput` (no replacement code needed — `console-exporter` already covers this role) and `telemetry_transport_contract::output::FileOutput`/`rotation::RotationManager` (replaced by `file-exporter`, converging on `RotationManager`'s numbered-backup-chain algorithm, the more complete of the two divergent rotation implementations found in the orphaned crate).
- New capability `kitlogger-buffering`: a batching stage living inside `kitlogger` itself (not a new crate — see Design Q4), holding raw records and deferring format+output cost to flush time, composing with `console-exporter`'s existing `FlushStrategy` rather than replacing it.
- New capability `kitlogger-format-selection`: a small mapping, owned by `kitlogger`, from `kit_config::LogFormat` to `kitlogger_formatter::LogFormat` — see Design Q5 for why this cannot live inside `kitlogger-formatter` itself.
- `console-exporter` adjusted to implement the new Output Port (in addition to, or replacing, its existing internal `ConsoleExporter` trait — an implementation detail for the apply phase, not decided here).

### Out of Scope

- Designing OTLP, Loki, Sentry, CloudWatch, Elastic, S3, Kafka, or NATS exporters. This proposal only ensures the architecture requires no redesign to add them later (see Design Q2).
- Wiring any of this into `KITLogger`'s actual `log`/`log_record` execution path. That is Migration Plan Phase 5 (Orchestration Fold), which folds filter → sample → redact → buffer → format → dispatch into `KITLogger` as one piece, together with the `LoggingConfig.enabled` gate and level filtering. This change produces every capability standalone and tested; none are called from `KITLogger` yet — matching how Phase 3 (change 013) landed Redaction/Sampling.
- Any change to `kit_config`, `telemetry-adapter-contracts`, or `formatter-contract` (all read-only references; `formatter-contract`'s existing, accepted "MUST depend only on kitlogger-log-domain, serde_json, thiserror" requirement is explicitly preserved, not touched).
- Deleting `telemetry-transport-contract` itself (Phase 8).

## Capabilities

### New Capabilities

- `output-adapter-contracts`: the Output Port and dispatch-mechanism registry.
- `file-exporter`: file-based output implementation, including rotation.
- `kitlogger-buffering`: pre-format record batching, internal to `kitlogger`.
- `kitlogger-format-selection`: `kit_config::LogFormat` → `kitlogger_formatter::LogFormat` mapping, internal to `kitlogger`.

### Modified Capabilities

- None. `formatter-contract`, `console-exporter-core`, `console-stream-router`, and `telemetry-adapter-contracts` are all read/consulted but not modified — confirmed explicitly in Design.

## Approach

Every new capability re-implements *behavior* already validated in the orphaned crate (`ConsoleOutput`'s role, `FileOutput`, `RotationManager`, `Buffer`) or fills a genuine gap (the Output Port itself, which never existed generically before) — not transplanted code, per ADR-008. `console-exporter` and `file-exporter` are siblings under one Port; neither depends on the other, and neither depends on `kitlogger`. `kitlogger-format-selection` exists specifically because `formatter-contract`'s dependency boundary is already accepted and closed (`kitlogger-log-domain`, `serde_json`, `thiserror` — no `kit_config`) — the mapping has to live in the one crate that already knows about both `kit_config` and `kitlogger-formatter`, which is `kitlogger`.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/output-adapter-contracts/` | New | Output Port trait + registry mechanism |
| `crates/file-exporter/` | New | File output + rotation (internal module) |
| `crates/console-exporter/` | Modified | Implements the new Output Port |
| `crates/kitlogger/` | Modified | Gains `Buffer` (internal module) and the `LogFormat` mapping (internal module) — neither called from the emission path yet |
| `Cargo.toml` (workspace) | Modified | Add `output-adapter-contracts`, `file-exporter` to `members` |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `console-exporter` implementing a second trait (the new Port) alongside its existing `ConsoleExporter` trait could create two divergent write paths if not reconciled carefully | Medium | Explicit task to confirm a single write path underlies both, not two independent code paths |
| `Buffer`/format-selection landing as unused internal modules of `kitlogger` before Phase 5 wires them may trigger dead-code warnings | Low | Exercised by their own unit tests within the `kitlogger` crate, which satisfies Rust's usage analysis; no `#[allow(dead_code)]` needed |
| Two divergent rotation algorithms existed in the orphaned crate (`RotationManager` vs. `FileOutput::rotate`'s inline version) | Low | Explicit task: only `RotationManager`'s numbered-backup-chain algorithm is ported; the inline version's test coverage is confirmed subsumed, not silently dropped |
| A future OTLP/Kafka/etc. exporter turns out to need something the Output Port doesn't provide, forcing a Port change later | Low | Port is intentionally minimal (formatted string + severity) — the same shape `console-exporter` already uses successfully; richer needs (batching, retries) are the *adapter's* internal concern, not the Port's, matching Ports & Adapters |

## Rollback Plan

`output-adapter-contracts` and `file-exporter` are additive with zero consumers in this change (nothing depends on them yet outside their own tests). `console-exporter`'s Port implementation and `kitlogger`'s new internal modules are also unreferenced from any execution path. Reverting removes the two new crates, their workspace member entries, and the two new internal modules — isolated, non-breaking.

## Dependencies

- ADR-008 (Migration Plan Phase 4), ADR-010 (Enforcement declaration above).
- Prior architecture reviews (this session): Output crate boundary validation, and the `telemetry-adapter-contracts` ownership validation that rejected it as the Output Port.
- `openspec/specs/formatter-contract/spec.md` — read, confirmed unmodified.
- `kitlogger_log_domain::Severity` (canonical, read-only reference for `output-adapter-contracts`).

## Success Criteria

- [ ] `output-adapter-contracts` exists with an Output Port and a registry mechanism; depends only on `kitlogger-log-domain` (for `Severity`).
- [ ] `file-exporter` exists, implements the Port, and owns rotation internally — no standalone `Rotation` crate.
- [ ] `console-exporter` implements the same Port `file-exporter` does.
- [ ] `kitlogger` gains `Buffer` and the `LogFormat` mapping as internal modules, called from nowhere yet.
- [ ] `formatter-contract`'s existing, accepted dependency-boundary requirement is unchanged and unviolated (`kit_config` does not appear in `kitlogger-formatter`'s dependencies).
- [ ] `telemetry_transport_contract::{output, rotation, buffering}` have no unique, unmigrated logic left after this change (they may still exist as source until Phase 8 deletes the crate, but nothing in them is still the "only" implementation of anything).
- [ ] No two canonical models remain for the same concept within the scope of this change (ADR-010).
