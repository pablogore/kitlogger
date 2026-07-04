# Tasks: Correlation ID Closure

No `design.md` — the design reasoning is the ADR-009 Amendment itself. No `spec.md` — no capability is introduced or modified; nothing in any existing spec asserted the original shared-crate plan as a requirement, so there is nothing to write a delta against.

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | < 20 |
| 400-line budget risk | None |
| Chained PRs recommended | No |

## Phase 1: Verification

- [ ] 1.1 Confirm no crate in the workspace currently converts between `kitlogger_log_domain::{CorrelationId, TraceId, SpanId}` and any `context-propagation` identifier type (`rg -rn "context_propagation" crates/kitlogger-log-domain` and `rg -rn "kitlogger_log_domain" crates/context-propagation` both return no matches) — confirms this amendment regresses nothing.

## Phase 2: Documentation

- [ ] 2.1 Add a doc-comment line to `crates/kitlogger-log-domain/src/correlation_id.rs` noting: for wire-level, W3C Trace Context-compliant correlation identifiers, see `context_propagation::correlation::CorrelationIdentifier` — this type is intentionally a separate, simpler concept for log-line tagging (ADR-009 Amendment).
- [ ] 2.2 Add the equivalent doc-comment line to `crates/kitlogger-log-domain/src/trace_id.rs`, referencing `context_propagation::trace_context::TraceId`.
- [ ] 2.3 Add the equivalent doc-comment line to `crates/kitlogger-log-domain/src/span_id.rs`, referencing `context_propagation::trace_context::SpanId`.

## Phase 3: Close

- [ ] 3.1 Run `cargo doc -p kitlogger-log-domain` — confirm the new doc comments render without warnings.
- [ ] 3.2 Confirm this closes Migration Plan Phase 9 — the ADR-008/ADR-009 migration initiative has no remaining open phases except Phase 10 (Plugin Enablement), explicitly out of scope for the entire initiative per ADR-008's own migration sequencing.
