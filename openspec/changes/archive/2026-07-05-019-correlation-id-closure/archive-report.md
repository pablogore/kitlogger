# Archive Report: Correlation ID Closure (Change 019, Migration Plan Phase 9)

## Status

Shipped. All tasks in `tasks.md` complete (`[x]`). Merged into `develop` via PR #56.

## What shipped

- Zero code-behavior changes, as scoped: `kitlogger_log_domain::{CorrelationId, TraceId, SpanId}` and `context-propagation`'s identifier types remain exactly as they were.
- Added a one-line doc-comment cross-reference in each of `kitlogger-log-domain`'s three identifier files, pointing a future reader to `context-propagation`'s richer, W3C Trace Context-compliant types for wire-level interop:
  - `crates/kitlogger-log-domain/src/correlation_id.rs:8` → `context_propagation::correlation::CorrelationIdentifier`
  - `crates/kitlogger-log-domain/src/trace_id.rs:8` → `context_propagation::trace_context::TraceId`
  - `crates/kitlogger-log-domain/src/span_id.rs:8` → `context_propagation::trace_context::SpanId`
- Confirmed no crate in the workspace converts between the two identifier families (`rg -rn "context_propagation" crates/kitlogger-log-domain` and `rg -rn "kitlogger_log_domain" crates/context-propagation` both empty) — this amendment regresses nothing.

## Why no shared type

ADR-009 originally called for unifying these identifiers into one shared, extracted type, based on comparing `kitlogger-log-domain`'s newtypes only against `context_propagation::models`' bare type aliases. Reading the rest of `context-propagation` (`correlation.rs`, `trace_context.rs`) found a full W3C Trace Context implementation — byte-array IDs, `TraceFlags`, `TraceState`, `traceparent` validation, UUID-v7 correlation IDs — with no counterpart in `kitlogger-log-domain`'s simple, unvalidated `String` newtypes. ADR-009 was amended (see `openspec/changes/archive/2026-07-04-012-logging-pipeline-consolidation/ADR-009-correlation-id-unification.md`, "Amendment" section) rather than executed as originally written: the two identifier families serve different layers (log-line tagging vs. wire-level interop) and unifying them would force one to compromise the other.

## Verification

- `cargo doc -p kitlogger-log-domain` — new doc comments render without warnings.
- No spec merge required — no capability was introduced or modified; nothing in any existing spec asserted the original shared-crate plan as a requirement.

## Migration status after this change

This closes Migration Plan Phase 9. The only remaining open item from ADR-008/ADR-009's original migration plan is Phase 10 (Plugin Enablement), explicitly out of scope for the entire initiative per ADR-008's own migration sequencing.
