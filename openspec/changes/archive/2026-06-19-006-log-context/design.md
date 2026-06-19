# Design: Log Context & Enrichment (006-log-context)

## Technical Approach

Add a `LogContext` struct and enrichment API to the existing `kitlogger-log-domain` crate. LogContext is an immutable, passive metadata container holding a set of `LogAttribute` values plus optional correlation/trace/span identifiers. Enrichment methods produce new `LogContext` instances — the original is never mutated. All enrichment paths return `Result<Self, ValidationError>`. Follows the same patterns as `LogRecord` (getters, `Vec`-backed storage, per-file module structure).

## Architecture Decisions

### Decision: File Structure

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Single `log_context.rs` (struct + all impls) | Follows existing pattern; no cross-module visibility issues | **Selected** |
| Split `log_context.rs` + `enrichment.rs` | Inherent methods across modules require `pub(crate)` fields, breaking encapsulation | Rejected |

**Rationale**: Enrichment methods are inherent, not a trait. Splitting impl blocks across modules forces exposing private fields. Single file matches `log_record.rs` conventions.

### Decision: Duplicate Name Detection

| Option | Tradeoff | Decision |
|--------|----------|----------|
| O(n) `any()` scan per `with_attribute` | Simple, zero allocation; fine for <10 attributes | **Selected** |
| `HashMap`-backed index | Overkill for small contexts; adds memory overhead | Rejected |

**Rationale**: Contexts carry <10 attributes in practice. Linear scan is fast, allocation-free, and keeps the struct dependency-free.

### Decision: ID Enrichment Return Type

| Option | Tradeoff | Decision |
|--------|----------|----------|
| Return `Result` (never Err) | Uniform API surface; future-proof for validation | **Selected** |
| Return `Self` | Truthful API but inconsistent with `with_attribute` | Rejected |

**Rationale**: Spec mandates `Result` for all enrichment methods. Uniform return type simplifies consumer error handling in Logger (AS-03).

## Data Flow

```
Caller (app/middleware)
  │
  ▼
LogContext::new() —► empty LogContext { attrs: [], ids: None }
  │
  ▼  .with_attribute(attr)
  │  ── duplicate name? ──► Err(ValidationError::EnrichmentError)
  │  ── OK ──► new LogContext with attr appended
  │
  ▼  .with_correlation_id(id) / .with_trace_id(id) / .with_span_id(id)
  │  ──► new LogContext with identifier set (replaces if already set)
  │
  ▼  consumed by Logger::log() — AS-03 (future change)
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `crates/kitlogger-log-domain/src/log_context.rs` | Create | LogContext struct, new(), getters, Display, Default, all enrichment methods |
| `crates/kitlogger-log-domain/src/validation.rs` | Modify | Add `EnrichmentError(String)` variant + Display arm |
| `crates/kitlogger-log-domain/src/lib.rs` | Modify | Add `mod log_context;` + `pub use log_context::LogContext;` |
| `crates/kitlogger-log-domain/tests/integration_tests.rs` | Modify | Add context + enrichment integration tests |

## Interfaces / Contracts

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct LogContext {
    attributes: Vec<LogAttribute>,
    correlation_id: Option<CorrelationId>,
    trace_id: Option<TraceId>,
    span_id: Option<SpanId>,
}

impl LogContext {
    /// Empty context with no attributes or identifiers.
    pub fn new() -> Self;

    // Getters — exposed for Logger consumption (AS-03)
    pub fn attributes(&self) -> &[LogAttribute];
    pub fn correlation_id(&self) -> Option<&CorrelationId>;
    pub fn trace_id(&self) -> Option<&TraceId>;
    pub fn span_id(&self) -> Option<&SpanId>;

    // Enrichment — all return Result, all produce new instances
    pub fn with_attribute(&self, attr: LogAttribute) -> Result<Self, ValidationError>;
    pub fn with_correlation_id(&self, id: CorrelationId) -> Result<Self, ValidationError>;
    pub fn with_trace_id(&self, id: TraceId) -> Result<Self, ValidationError>;
    pub fn with_span_id(&self, id: SpanId) -> Result<Self, ValidationError>;
}

impl Default for LogContext {}  // delegates to new()

impl Display for LogContext {
    // Format: LogContext { attr1: "val1", attr2: 42, correlation_id: "abc-123" }
}
```

**ValidationError addition** (in `crates/kitlogger-log-domain/src/validation.rs`):

```rust
pub enum ValidationError {
    EmptyMessage,
    InvalidSeverity,
    InvalidAttributeName(String),
    InvalidAttributeValue(String),
    /// Enrichment operation failed (e.g. duplicate attribute name)
    EnrichmentError(String),   // NEW — ordered last
}
```

Display output for `EnrichmentError(s)`: `"Enrichment error: {s}"`

## Testing Strategy

| Layer | What | Approach |
|-------|------|----------|
| Unit | `new()` / `Default` | Assert empty attrs, no identifiers |
| Unit | `with_attribute` | Assert attr present in returned context |
| Unit | Duplicate name rejection | Assert `Err(ValidationError::EnrichmentError(...))` |
| Unit | Enrichment immutability | Call `with_attribute` on original, verify original unchanged |
| Unit | ID enrichment | Assert returned context has the identifier |
| Unit | ID idempotency | Set same ID twice, verify last-wins |
| Unit | Display (empty, with attrs, with IDs) | Match expected format strings |
| Unit | Getters | Assert correct values and types |
| Integration | Full enrichment pipeline | Chain multiple with_* calls, verify final state |

## Migration / Rollout

No migration required. This is purely additive — no existing code consumes LogContext yet.

## Open Questions

None. All decisions are resolved by spec and existing codebase patterns.
