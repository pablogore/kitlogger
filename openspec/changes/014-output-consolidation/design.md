# Design: Output Subsystem Consolidation

## Q1 — Who owns the Output abstraction?

**`output-adapter-contracts` (new crate).** Not `telemetry-adapter-contracts`.

This was validated, not assumed, in a prior review this session by reading `telemetry-adapter-contracts`'s actual source: its own `lib.rs` doc comment self-identifies as "telemetry provider abstraction"; it has a `mapping.rs` module of pure Canonical↔OpenTelemetry conversion contracts (`TraceMappingContract`, `SpanMappingContract`, `MetricMappingContract`, `LogRecordMappingContract`, `ResourceMappingContract`) that no local sink would ever call; and its one required delivery method, `TelemetryDelivery::deliver(&self, envelope: PayloadEnvelope)`, takes a cross-signal OTel-style batch (traces + metrics + logs together) — an unnatural shape for "write one already-formatted line to stdout or a file."

Per ADR-010 ("every domain concept has exactly one canonical owner... divergent shape is not evidence of a different concept, it is usually evidence the duplicate evolved independently and drifted" — inverted here: *convergent naming* (`Adapter`, `Exporter`, `Registry` appearing in both) is not evidence of the *same* concept either, when the actual payload shape and stated purpose diverge this much). Output and Telemetry are different bounded contexts that happen to share a structural pattern (registry + adapter + dispatch). `telemetry-adapter-contracts` remains exactly what it is, untouched, consumed only by whichever future adapter chooses to speak OTel (most naturally, `otlp-exporter`).

Why the Port carries `Severity` alongside an already-formatted payload, given formatting has already happened: severity accompanies the formatted payload because output destinations may route or prioritize records independently of formatting — e.g. `console-exporter` splits stdout/stderr by severity today, and a future output could sample, alert, or apply backpressure differently per severity, none of which requires re-parsing the formatted string. `Severity` remains part of the canonical logging domain (`kitlogger-log-domain`) and is not duplicated inside the formatted representation or redefined by `output-adapter-contracts`.

## Q2 — How should output implementations be organized?

One crate per implementation, each depending only on `output-adapter-contracts` (the Port) and `kitlogger-log-domain` (for `Severity`) — never on each other, never on `kitlogger`.

| Destination | Crate | This change |
|---|---|---|
| Console | `console-exporter` | Existing crate, gains the Port implementation |
| File | `file-exporter` | New |
| OTLP | `otlp-exporter` | Out of scope — would additionally depend on `telemetry-adapter-contracts` internally for OTel mapping/batching, while still implementing the same Output Port outward-facing |
| Loki, Sentry, CloudWatch, Elastic, S3, Kafka, NATS | `loki-exporter`, `sentry-exporter`, `cloudwatch-exporter`, `elastic-exporter`, `s3-exporter`, `kafka-exporter`, `nats-exporter` | Out of scope — none are OTel-native, none would need `telemetry-adapter-contracts` |

This is why "future outputs require no architectural redesign" (a stated constraint): adding any of the out-of-scope destinations is "add a new crate depending on the existing, unchanged Port" — zero impact on `console-exporter`, `file-exporter`, `output-adapter-contracts`, or `kitlogger`.

## Q3 — Where does Rotation belong?

**Internal module of `file-exporter`.** Not its own crate.

Bounded-context reasoning: Rotation has exactly one consumer (file output) and no reuse case anywhere in the current or planned roster — S3 has lifecycle policies, CloudWatch has retention settings, Kafka has topic retention, none of which are "the same concept" as rotating a local log file by size/age/backup-count. It fails the reuse test the same way `Buffer` does (Q4) — a concept that exists only because one other concept exists is that concept's internal detail, not a bounded context of its own. Two divergent rotation algorithms existed in the orphaned crate (`rotation::RotationManager`'s numbered-backup-chain vs. `output::FileOutput::rotate()`'s inline single-backup version); only `RotationManager`'s is ported.

## Q4 — Where does Buffer belong?

**Internal module of `kitlogger`.** Not its own crate, not part of any output crate.

Bounded-context reasoning: Buffer batches *raw, pre-format* records before the pipeline reaches Format and Dispatch (ADR-008 §5: filter → sample → redact → **buffer** → format → dispatch). It has exactly one consumer — the logging pipeline itself — and is not specific to any one output destination (a host with both console and file outputs active shares one buffer, not one per output). It composes with, but does not replace, `console-exporter`'s existing `FlushStrategy` (batching cadence vs. flush-trigger policy are adjacent, distinct concerns). Per the crate-boundary principle already established for this migration (change 012's design.md, Phase 4 note): no reflexive crate creation — a single-consumer concept stays internal until reuse is demonstrated.

## Q5 — Formatter ownership

**Formatter remains fully independent. It does not become an Output concern.**

`openspec/specs/formatter-contract/spec.md` already has an accepted, closed-dependency requirement: *"`kitlogger-formatter` MUST depend only on `kitlogger-log-domain`, `serde_json`, and `thiserror`... MUST NOT depend on any exporter crate or I/O crate."* Making `kitlogger-formatter` aware of `kit_config::LogFormat` would add a dependency outside that closed list — a direct violation of an already-accepted contract (the same class of mistake this initiative found and corrected once already with `telemetry-config-semantics` FR-011 in change 012). This proposal makes **zero changes** to `formatter-contract`.

Instead, the mapping from `kit_config::LogFormat` to `kitlogger_formatter::LogFormat` is a new, small capability — `kitlogger-format-selection` — owned by `kitlogger` itself, the one crate that already depends on both `kit_config` (since Phase 2) and `kitlogger-formatter` (since the beginning). This preserves `kitlogger-formatter`'s independence exactly as already accepted, while still letting a host select a format via `LoggingConfig.format`.

The two enums don't share variant names, so the mapping itself is a judgment call, not a mechanical rename — recorded here as a design decision (not asserted by the spec, which only requires completeness and determinism, not this specific pairing):

| `kit_config::LogFormat` | → | `kitlogger_formatter::LogFormat` | Rationale |
|---|---|---|---|
| `Json` | → | `Json` | Exact conceptual match |
| `Text` | → | `Text` | Exact conceptual match |
| `Pretty` | → | `HumanReadable` | Both describe a human-oriented, non-compact rendering |
| `Compact` | → | `Logfmt` | `Logfmt`'s single-line `key=value` rendering is the closest existing match to "compact" among the four available variants |

## Q6 — Dispatch ownership

Split ownership, stated precisely so nothing is ambiguous:

- **Dispatch mechanism** (register N outputs; deliver a formatted record to all of them; aggregate per-output failures) is owned by **`output-adapter-contracts`**. This mirrors, at the Output bounded context, exactly the pattern `telemetry-adapter-contracts::AdapterRegistry`/`deliver_to_all` already established at the Telemetry bounded context — the same *pattern*, a different *owner*, because it's a different *domain* (Q1).
- **Dispatch orchestration** (deciding *when* in the pipeline sequence dispatch fires, and *which* outputs are registered by default) is owned by **`kitlogger`**. `kitlogger` registers `console-exporter` and `file-exporter` by default; it does not register `otlp-exporter`/etc. — those remain opt-in, keeping `kitlogger`'s own default dependency footprint from growing with every future destination.

This is not two owners of the same concept — it is the standard Ports & Adapters split between the mechanism (Port + Registry) and the composition root that wires and invokes it, and it is not implemented by this change (see below) — only its ownership is declared here.

## What This Change Does NOT Implement

Per Migration Plan Phase 5 (Orchestration Fold), none of the following are wired into `KITLogger`'s actual `log`/`log_record` execution path by this change — they are built standalone and tested, exactly as Phase 3 (change 013) did for `Redactor`/`Sampler`:

- `KITLogger` does not yet call `Buffer`.
- `KITLogger` does not yet call the `kitlogger-format-selection` mapping.
- `KITLogger` does not yet register anything into an `output-adapter-contracts` registry or call it.
- `LoggingConfig.enabled` and level filtering remain un-gated, as already decided in change 012's design.md.

## Ownership Table (complete, per Acceptance Criteria)

| Concept | Owner (crate) | Consumers |
|---|---|---|
| Output Port + dispatch mechanism | `output-adapter-contracts` (new) | `console-exporter`, `file-exporter`, future adapters |
| Console output implementation | `console-exporter` (existing, modified) | `kitlogger` (registers by default, Phase 5) |
| File output implementation | `file-exporter` (new) | `kitlogger` (registers by default, Phase 5) |
| Rotation | `file-exporter` (internal module) | `file-exporter` itself only |
| Buffer | `kitlogger` (internal module) | `kitlogger`'s own pipeline (Phase 5) |
| Formatter (trait, impls, `LogFormat` enum) | `kitlogger-formatter` (existing, **unmodified**) | `kitlogger` (via the new mapping), any future direct consumer |
| `kit_config::LogFormat` → `kitlogger_formatter::LogFormat` mapping | `kitlogger` (internal module, new capability `kitlogger-format-selection`) | `kitlogger`'s own construction path |
| Dispatch orchestration (when/what) | `kitlogger` | — |
| Telemetry/OTel provider contract | `telemetry-adapter-contracts` (existing, **unmodified**) | Future `otlp-exporter` only |

### Identity Ownership

Registration identity (the value used to uniquely address a registered output, FR-002) is owned by the **Registry**, not by Output, Adapter, or Exporter as concepts. There is exactly one identity type for this bounded context, defined where the registry itself lives (`output-adapter-contracts`) — not a per-implementation `ConsoleOutputId`, nor a redefinition of `telemetry-adapter-contracts::AdapterId` for this different context. Any future output implementation registers using that one identity type; none defines its own.

## Duplicate Implementations That Disappear

| Orphaned original | Disposition |
|---|---|
| `telemetry_transport_contract::output::ConsoleOutput` | Deleted (Phase 7/8) — no replacement code needed, `console-exporter` already covers this role |
| `telemetry_transport_contract::output::FileOutput` | Deleted (Phase 7/8) — behavior absorbed into `file-exporter` |
| `telemetry_transport_contract::rotation::RotationManager` | Deleted (Phase 7/8) — algorithm absorbed into `file-exporter`'s internal rotation module |
| `telemetry_transport_contract::output::FileOutput::rotate()` (the second, divergent inline rotation algorithm) | Deleted outright — not absorbed anywhere, `RotationManager`'s version is the one that survives |
| `telemetry_transport_contract::buffering::Buffer` | Deleted (Phase 7/8) — behavior absorbed into `kitlogger`'s internal `Buffer` module |
| `telemetry_transport_contract::formatter::*` | Deleted (Phase 7/8) — no replacement code needed, `kitlogger-formatter` already covers this role |
| `telemetry_transport_contract::provider::LoggerProvider` | Not part of this change — retired in Phase 5 alongside the orchestration fold, once dispatch orchestration actually lands in `kitlogger` |

## Dependency Graph

```
Host
  └─ kitlogger (facade — gains internal modules: Buffer, LogFormat-selection; no wiring yet)
       ├─ kitlogger-log-domain
       ├─ kitlogger-formatter          [unmodified — closed dependency list preserved]
       ├─ kitlogger-sampling
       ├─ kitlogger-redaction
       ├─ kit-config
       └─ (Phase 5, not this change) output-adapter-contracts
             ├─ console-exporter        [gains: Output Port implementation]
             │     └─ kitlogger-log-domain (Severity)
             └─ file-exporter           [new; owns Rotation internally]
                   └─ kitlogger-log-domain (Severity)

telemetry-adapter-contracts   [unchanged; not a dependency of kitlogger, console-exporter, or file-exporter]
      └─ telemetry-types

telemetry-transport-contract  [unchanged by this proposal; still scheduled for removal at Phase 8]
```

Note the `kitlogger → output-adapter-contracts` edge is parenthetically marked "Phase 5, not this change" — `output-adapter-contracts` is built and `console-exporter`/`file-exporter` implement it in this change, but `kitlogger` does not yet depend on or call any of it. The edge exists architecturally (this design fixes where it will point) without being exercised by code until Phase 5.

## Phase Breakdown (within this change)

Four independent work units — none depends on another landing first, since none are wired together yet:

1. `output-adapter-contracts` — the Port + registry mechanism.
2. `file-exporter` — depends only on `output-adapter-contracts` existing (to implement its Port) and `kitlogger-log-domain`.
3. `console-exporter`'s Port implementation — depends only on `output-adapter-contracts` existing.
4. `kitlogger`'s two new internal modules (`Buffer`, `kitlogger-format-selection`) — depend on nothing new; `kit_config` and `kitlogger-formatter` are already `kitlogger` dependencies since Phase 2.

Unit 1 is a soft prerequisite for Units 2–3 (they implement its Port), but Unit 4 can proceed in full parallel with all three.

## Migration Strategy

Architectural sequencing only:

1. Land `output-adapter-contracts` first (Units 2–3 need its Port to implement against).
2. Land `file-exporter` and `console-exporter`'s Port implementation in either order, or in parallel — they don't depend on each other.
3. Land `kitlogger`'s `Buffer` and `kitlogger-format-selection` modules independently of 1–2.
4. None of this is wired into `KITLogger`'s execution path — that is Phase 5, a separate future change, gated on this one plus change 013 (Redaction/Sampling) both being complete.

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit (`output-adapter-contracts`) | Registering an output, then dispatching, reaches every registered output | Register N fake outputs, dispatch once, assert each received the record |
| Unit (`output-adapter-contracts`) | Per-output delivery failure is reported without blocking delivery to other outputs | One fake output errors, others still receive the record; aggregated result reflects the partial failure |
| Unit (`file-exporter`) | Rotation triggers at the configured size/age/backup-count boundary | Table-driven test against `kit_config::RotationConfig` variants |
| Unit (`file-exporter`) | Rotation algorithm matches `RotationManager`'s validated numbered-backup-chain behavior, not the discarded inline single-backup version | Test ported directly from the orphaned crate's existing coverage |
| Unit (`console-exporter`) | Existing behavior unchanged after gaining the new Port implementation | Full existing test suite passes unmodified |
| Unit (`kitlogger`, `Buffer`) | Records are held and flushed per `kit_config::BufferingConfig` (`batch_size`, `flush_interval_ms`) without being called from anywhere yet | Direct unit tests against the module, satisfying Rust's dead-code usage analysis |
| Unit (`kitlogger`, format-selection) | Every `kit_config::LogFormat` variant maps to the correct `kitlogger_formatter::LogFormat` variant | Exhaustive table over all `kit_config::LogFormat` variants |
| Regression | `formatter-contract`'s existing test suite is untouched and still passes | No source change in `kitlogger-formatter` |

## Composability

`output-adapter-contracts`, `file-exporter`, and `console-exporter`'s Port implementation are fully reusable and do not assume `KITLogger` is their only consumer — matching the same composability stance already declared for `kitlogger-redaction`/`kitlogger-sampling` in change 013.

## Open Questions

- None blocking this change. The exact shape console-exporter's two traits (existing `ConsoleExporter`, new Port) take relative to each other is an implementation decision for the apply phase, not a boundary question — this design only requires that no second, divergent write path results.
