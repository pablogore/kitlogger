# Change Proposal: 006-log-context

## Intent

Implement AS-02 (Log Context & Enrichment) of the Structured Logging Core capability. Define the LogContext entity for scoped contextual metadata attachment and the enrichment contracts for adding attributes and identifiers to logging scopes without modifying previously emitted log records.

This is the second atomic specification within KIT-005 (Structured Logging Core), building on AS-01 (Structured Log Domain Model) which is already implemented and verified.

## Scope

### In Scope

- LogContext entity as an immutable set of attributes and metadata
- Display implementation for LogContext
- Enrichment methods: `with_attribute`, `with_correlation_id`, `with_trace_id`, `with_span_id`
- Enrichment creates a new context without modifying the original
- All enrichment operations return `Result<Self, ValidationError>` for consistency
- New `ValidationError` variant for enrichment failures
- `Default` trait implementation (empty context)
- Duplicate attribute names rejected with error
- Empty LogContext (`::new()` / `::default()`) valid as starting point
- Integration tests

### Out of Scope

- Logger and LoggerFactory interfaces (AS-03 — separate change)
- Serialization contracts (AS-04 — separate change 007-serialization-contracts)
- Configuration integration (AS-05)
- OTel/W3C Baggage integration
- Exporter-specific behavior
- Any runtime formatting, transport, or storage

## Approach

Add LogContext and enrichment contracts to the existing `kitlogger-log-domain` crate, which already contains all AS-01 types (LogRecord, Severity, LogAttribute, etc.).

### Crate Structure

**Crate**: `crates/kitlogger-log-domain/` (existing workspace member)

| New File | Contents |
|----------|----------|
| `src/log_context.rs` | LogContext struct, constructor, Display, Default |
| `src/enrichment.rs` | Trait and/or inherent methods for enrichment |
| `src/lib.rs` | Add modules + re-exports |

### Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Duplicate attribute names | Error (`EnrichmentError` variant) | Clear failure path, consistent with domain validation pattern |
| Empty LogContext | Valid starting point | Needed for scopes without context; Default impl provides this |
| Enrichment return type | `Result<Self, ValidationError>` | Consistent with LogRecord::new, LogAttribute::new |
| Scope semantics | Passive (data only, no Logger ref) | Avoids circular dependency, more testable and composable |
| Display | Implement for debugging | Zero cost, useful for observability |
| Default trait | Implement (empty context) | Convenience, consistent with recent KITLogger Default |
| OTel integration | Future | Avoids premature coupling to external tracing systems |
| EnrichmentError variant | New variant on ValidationError | Distinguishes enrichment failures from construction failures |

### Affected Files

| File | Change |
|------|--------|
| `crates/kitlogger-log-domain/src/lib.rs` | Add modules for log_context and enrichment |
| `crates/kitlogger-log-domain/src/log_context.rs` | **NEW** — LogContext entity |
| `crates/kitlogger-log-domain/src/enrichment.rs` | **NEW** — Enrichment logic |
| `crates/kitlogger-log-domain/src/validation.rs` | Add `EnrichmentError(String)` variant |
| `crates/kitlogger-log-domain/tests/integration_tests.rs` | Add context + enrichment tests |

## Architecture Context

```
AS-01 (Structured Log Domain Model) ──── Implemented (kitlogger-log-domain)
  │
  ▼
AS-02 (Log Context & Enrichment) ◄──── THIS CHANGE
  │
  ▼
AS-03 (Logger Contracts) ◄──────────── Next after this change
```

LogContext is consumed by Logger at record emission time (AS-03). For this change, LogContext is designed as a standalone passive entity without knowledge of Logger, factories, or emission.

## Risks

1. **Duplicate name detection requires allocation** — Storing attributes as `Vec<LogAttribute>` and checking for duplicates on insertion is O(n). For logging contexts (typically <10 attributes), this is fine.
2. **EnrichmentError variant broadens ValidationError** — Currently a construction-error enum. Adding enrichment errors means consumers need to match more variants. Mitigation: use `#[non_exhaustive]` if the enum is public, or keep enrichment errors as a separate concern.
3. **Integration with future Logger (AS-03)** — The enrichment API must be compatible with Logger's expected consumption pattern. Mitigation: design enrichment as methods on LogContext itself (not a separate trait), which gives Logger maximum flexibility.

## Estimated Size

~3-4 new files, ~300-460 lines new code, ~100-150 lines tests.
