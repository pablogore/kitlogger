# Design: Crate Removal

## Where `Transport` Lands, and Why It's Optional

`telemetry-adapter-contracts::Adapter` is the mandatory supertrait every registered adapter implements (`CommonAdapterBase + LifecycleAdapter + TelemetryDelivery`). `Transport` is not added to this supertrait. Reasoning: `console-exporter` and `file-exporter` (change 014) are local, synchronous sinks — they have no wire protocol to abstract over, and forcing them to depend on a `Transport` trait they'd never use would repeat exactly the mistake the Phase 4 review already corrected once (forcing local sinks through telemetry-shaped machinery they don't need). `Transport` instead becomes an available module inside `telemetry-adapter-contracts` — a toolkit a *specific* adapter implementation can reach for internally when it does need protocol-agnostic wire delivery (an `otlp-exporter` sending over gRPC, for instance), exactly mirroring how `mapping.rs`'s OTel conversion contracts are already available-but-optional, not required by `Adapter` itself.

## `TransportError` Stays Distinct From `AdapterError`

Reviewed both side by side:

- `AdapterError` (existing): `InvalidTransition`, `AlreadyRegistered`, `Frozen`, `InitializationFailed`, `FlushFailed`, `ShutdownFailed`, `DeliveryFailed`, `PartialDelivery` — all registry/lifecycle-management failures.
- `TransportError` (relocated): `Timeout`, `Unavailable`, `Backpressure`, `PayloadTooLarge`, `UnsupportedTransport` — all wire-protocol-level failures.

Zero overlap. These are genuinely different failure domains, not two models of the same concept — merging them would conflate "the registry couldn't manage this adapter" with "the network call itself failed," which are handled at different layers by different code. An adapter's `TelemetryDelivery::deliver()` implementation MAY wrap a `TransportError` inside an `AdapterError::DeliveryFailed`'s message when it internally uses `Transport` and that call fails — composition, not unification. This is a design note, not an API design; the exact wrapping mechanism is left to whichever future change implements a `Transport`-using adapter.

## `exporter-registry`'s Supersession

Confirmed before writing this proposal: `exporter-registry`'s spec (`openspec/specs/exporter-registry/spec.md`) was never mentioned in change `005-console-exporter`'s own `proposal.md`/`design.md`, no `ExporterRegistry` type was ever implemented (`rg -rn "ExporterRegistry" crates` returns zero matches), and that change's own `verify-report.md` already flagged it: *"The exporter-registry spec appears to be out of scope for this change — consider documenting this explicitly."* That documentation never happened until now.

Its model — select one exporter by string name, with a default fallback — predates and conflicts with ADR-008 §6's actual committed architecture: multiple outputs (console AND file) registered and dispatched to *simultaneously*, not one selected exporter at a time. `output-adapter-contracts` (change 014) already implements the correct model. `exporter-registry` is superseded, not reconciled — there is no behavior worth preserving from a spec that was never implemented and already contradicted the eventual architecture before this migration even began.

The historical spec file itself is not deleted (openspec convention preserves capability specs as a record; this change's delta formally empties it via `## REMOVED Requirements`, leaving the supersession traceable rather than erasing the fact it once existed).

## Final Dependency Graph (end state of the entire migration)

```
Host
  └─ kitlogger (KITLogger — full orchestrator, Phase 5)
       ├─ kitlogger-log-domain
       │     └─ shared correlation-id primitive        [ADR-009, Phase 9 — not yet done]
       ├─ kitlogger-formatter                          [unmodified throughout]
       ├─ kitlogger-sampling
       │     └─ kitlogger-log-domain (Clock only)
       ├─ kitlogger-redaction
       │     └─ kitlogger-log-domain (LogRecord)
       ├─ kit-config
       └─ output-adapter-contracts
             ├─ console-exporter
             └─ file-exporter                          [built, not yet registered — kit_config gap]

telemetry-adapter-contracts
      ├─ telemetry-types (PayloadEnvelope, TelemetryBatch, BackpressureSignal — canonical, ADR-007)
      └─ transport (Transport, DeliveryMode, TransportResult, TransportError — relocated, Phase 8)

context-propagation                                    [zero-dependent since Phase 7; ADR-009/Phase 9 gives it a role]

telemetry-transport-contract                           [REMOVED]
```

This is the target graph ADR-008's original migration plan described, fully realized — with the two corrections this session's review discipline found along the way: the Output Port is its own crate (not `telemetry-adapter-contracts`, per the Phase 4 validation), and `exporter-registry` is formally retired rather than left dangling.

## Ownership Table (final)

| Concept | Owner | Status at end of Phase 8 |
|---|---|---|
| Logging pipeline orchestration | `kitlogger` | Complete (Phase 5) |
| Log record model | `kitlogger-log-domain` | Complete, canonical throughout |
| Formatting | `kitlogger-formatter` | Complete, untouched |
| Redaction, Sampling | `kitlogger-redaction`, `kitlogger-sampling` | Complete (Phase 3) |
| Output Port + dispatch mechanism | `output-adapter-contracts` | Complete (Phase 4); registered outputs limited to console until the `kit_config::OutputTarget::File` gap closes |
| Console, File outputs | `console-exporter`, `file-exporter` | Complete (Phase 4/5) |
| Buffer, format selection | `kitlogger` (internal modules) | Complete (Phase 4/5) |
| Telemetry/OTel provider contract | `telemetry-adapter-contracts` | Complete; gains `transport` toolkit (Phase 8) |
| Correlation identity | Two competing copies | **Not yet resolved** — ADR-009/Phase 9, the one remaining open item after this change |

## Migration Strategy

1. Delete the three remaining absorbed-and-verified modules first (`sampling.rs`, `redaction.rs`, `rotation.rs` — the other five originally scoped here were already deleted by change 016) — no dependency on the `Transport` relocation.
2. Relocate `Transport`/`DeliveryMode`/`TransportResult`/`TransportError` into `telemetry-adapter-contracts` — independent of step 1.
3. Write the `exporter-registry` supersession delta — independent of steps 1–2, purely a spec-record change.
4. Only once steps 1–2 leave `telemetry-transport-contract` with zero remaining source files: delete the crate directory, its `Cargo.toml`, and its workspace member entry.

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Regression | Every crate that already tested absorbed behavior (`kitlogger-redaction`, `kitlogger-sampling`, `file-exporter`, `kitlogger-buffering`'s tests) still passes after the orphaned originals are deleted | `cargo test --workspace` |
| Unit (`telemetry-adapter-contracts`) | `Transport`/`DeliveryMode`/`TransportResult`/`TransportError` behave identically to the orphaned originals | Port the orphaned crate's own `transport.rs`/`error.rs` unit tests verbatim (their behavior is unchanged, only their location moves) |
| Structural | No crate in the workspace references `telemetry_transport_contract::` | `rg -rl "telemetry_transport_contract" crates` returns no matches |
| Structural | `exporter-registry`'s spec has no active (non-removed) requirements remaining | Inspect the merged spec post-archive |

## Open Questions

- None blocking. Whether a future `otlp-exporter` actually uses the relocated `Transport` trait, or an entirely different mechanism, is that future change's decision — this phase only makes the toolkit available in a coherent home.
