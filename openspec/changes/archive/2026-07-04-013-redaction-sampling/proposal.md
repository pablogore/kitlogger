# Proposal: Redaction & Sampling (Migration Plan Phase 3)

## Intent

ADR-008 identifies `Redactor` (`telemetry-transport-contract/src/redaction.rs`) and `Sampler` (`telemetry-transport-contract/src/sampling.rs`) as unique, validated capabilities with no counterpart anywhere in the live `kitlogger` pipeline — they exist only inside the orphaned crate being dismantled. This proposal absorbs both responsibilities into two new canonical crates, per ADR-008's Migration Plan Phase 3 ("Leaf Capability Absorption").

These two capabilities are bundled into one change because they are both leaf capabilities gated only by Phase 2 (already landed), with no dependency on each other — reviewing them together matches how they'll actually land (independent PRs, same review cycle), without implying they share a domain.

## ADR-010 Domain Model Declaration

Per ADR-010's Enforcement section, both new public domain models this proposal introduces are declared here:

| | Redaction | Sampling |
|---|---|---|
| **Canonical Owner** | New crate `kitlogger-redaction` | New crate `kitlogger-sampling` |
| **Canonical Model** | `Redactor` — decides which `LogRecord` attribute values are sensitive and replaces them | `Sampler` — decides whether a given emission should proceed, per `kit_config::SamplingConfig` |
| **Consumers** | `kitlogger` (once Phase 5 folds pipeline orchestration in) | `kitlogger` (same) |
| **Existing Competing Models** | `telemetry_transport_contract::redaction::Redactor` — the orphaned original. This change re-implements its validated behavior in the new canonical home; the orphaned version is deleted in Phase 7/8, not kept. | `telemetry_transport_contract::sampling::Sampler` — same treatment. |

Neither capability has a competing model anywhere in the *live* pipeline — only in the crate already scheduled for removal.

## Scope

### In Scope

- New crate `kitlogger-redaction`: a `Redactor` that, given `kit_config::RedactionConfig`, identifies sensitive attributes according to the configured field identifiers and produces a `LogRecord` with matching attribute values replaced by a fixed redaction marker — without mutating the input record (`LogRecord` is immutable per its own domain contract). The matching algorithm is a `design.md` decision, not part of this proposal's contract.
- New crate `kitlogger-sampling`: a `Sampler` that, given `kit_config::SamplingConfig`, decides whether a given emission should proceed, implementing all four strategies already validated in the orphaned original: `None` (always proceed), `Probabilistic` (random draw against `rate`), `EveryNth` (deterministic counter), `RateLimit` (sliding time window against `max_events_per_second`).
- Both crates depend on `kit_config` directly (already reachable workspace-wide since Phase 2). `kitlogger-redaction` additionally depends on `kitlogger-log-domain` for `LogRecord`/`LogAttribute`/`LogAttributeValue`. `kitlogger-sampling` also depends on `kitlogger-log-domain`, but only for its existing `Clock` abstraction (`RateLimit`'s time source, injectable for testability) — sampling decisions still do not depend on record content, matching the orphaned original's behavior and this change's Content Independence requirement.

### Out of Scope

- Wiring `Redactor`/`Sampler` into `KITLogger`'s emission path. That is Migration Plan Phase 5 (Orchestration Fold), which folds filter → sample → redact → buffer → format → dispatch into `KITLogger` all at once, so the sequencing is designed once. This change produces the two capabilities standalone and tested; it does not call them from anywhere yet.
- Buffering, output, formatter reconciliation (Phase 4).
- Deleting `telemetry-transport-contract`'s own `redaction.rs`/`sampling.rs` (Phase 7/8 — the orphaned crate is not touched by this change).
- Any change to `kit_config` itself (sibling repo, read-only reference).
- Level filtering (folded into Phase 5 alongside the `LoggingConfig.enabled` gate, per change 012's design.md).

## Capabilities

### New Capabilities

- `kitlogger-redaction`: sensitive-attribute redaction over `LogRecord`, driven by `kit_config::RedactionConfig`.
- `kitlogger-sampling`: emission-volume sampling decisions, driven by `kit_config::SamplingConfig`.

### Modified Capabilities

- None.

## Approach

Both crates re-implement the *behavior* already validated in the orphaned crate's tests — not its code verbatim (ADR-008: "absorbing means the behavior is preserved and re-implemented against the framework's existing conventions"). `Redactor` operates on the canonical `LogRecord` (immutable, so redaction produces a new record rather than mutating fields in place — the orphaned original operated on the now-retired `LogEvent`'s mutable `HashMap<String, serde_json::Value>`, which is not available here and would not respect `LogRecord`'s immutability contract even if it were). `Sampler` is a pure decision component with no record dependency, matching its original design exactly.

Redaction and sampling are independent of each other; they do not need to agree on evaluation order between themselves — the *pipeline's* order (sample before redact) is a Phase 5 concern, not something either capability enforces on its own. Both crates are fully reusable and do not assume they will be used exclusively by `KITLogger` — this leaves the door open for future pipelines or hosts to depend on either capability independently.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/kitlogger-redaction/` | New | New crate: `Redactor`, `RedactionConfig` consumption, tests |
| `crates/kitlogger-sampling/` | New | New crate: `Sampler`, `SamplingConfig` consumption, tests |
| `Cargo.toml` (workspace) | Modified | Add both new crates to `members` |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `LogRecord`'s immutability means redaction must copy/rebuild rather than mutate — different mechanism than the orphaned original | Medium | Explicit requirement (FR-002 in `kitlogger-redaction`'s spec) states this; not left to implementer discretion |
| Two new crates with no consumer yet (`KITLogger` doesn't call them until Phase 5) | Low | Expected and scoped — matches Phase 3's "leaf capability absorption" framing; each crate is independently tested against its own spec, not against integration behavior that doesn't exist yet |
| Sampler's `RateLimit` strategy is time-sensitive (sliding window) | Medium | Existing orphaned implementation's tests already validate this behavior; re-implementation must preserve the same window semantics, verified in this change's own tests, not assumed |

## Rollback Plan

Both crates are additive with zero consumers in this change (nothing depends on `kitlogger-redaction`/`kitlogger-sampling` yet). Reverting removes the two crates and their workspace member entries — isolated, non-breaking.

## Dependencies

- ADR-008 (Migration Plan Phase 3), ADR-010 (Enforcement declaration above).
- `kit_config::{RedactionConfig, SamplingConfig, SamplingStrategy}` (external, sibling repo, read-only reference).
- `kitlogger_log_domain::{LogRecord, LogAttribute, LogAttributeValue}` (canonical domain model, read-only reference for `kitlogger-redaction`).
- `kitlogger_log_domain::Clock` (existing canonical time-source abstraction, read-only reference for `kitlogger-sampling`'s `RateLimit` strategy — reused per ADR-010 rather than redefined).

## Success Criteria

- [ ] `kitlogger-redaction` redacts attribute values identified as sensitive according to the configured field identifiers, leaving all other attributes untouched.
- [ ] `kitlogger-redaction` never mutates its input `LogRecord`; it returns a new one.
- [ ] `kitlogger-sampling` implements all four `SamplingStrategy` variants (`None`, `Probabilistic`, `EveryNth`, `RateLimit`) matching the orphaned original's validated behavior.
- [ ] Neither crate is called from `KITLogger` yet (explicitly out of scope — confirmed by absence of any new dependency edge from `kitlogger` to either crate in this change).
- [ ] Both crates' test suites pass; no regressions elsewhere in the workspace.
- [ ] No two canonical models remain for the same concept within the scope of this change (ADR-010).
