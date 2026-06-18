# Identifier Contracts

**SPEC_ID**: `003-structured-logging-core-as-01-structured-log-domain-model`

---

## Common Pattern

All three identifier types follow the same newtype pattern:

```rust
pub struct CorrelationId(String);
pub struct TraceId(String);
pub struct SpanId(String);
```

## Shared Interface

```rust
impl Type {
    pub fn new(id: String) -> Self;
    pub fn as_str(&self) -> &str;
}

impl Display for Type;       // formats the inner string
impl AsRef<str> for Type;    // delegates to inner string
impl From<String> for Type;  // infallible conversion
```

## Semantics

| Type | Purpose |
|------|---------|
| `CorrelationId` | Cross-service correlation |
| `TraceId` | Distributed trace association |
| `SpanId` | Span-level identification within a trace |

All identifiers are opaque strings with no required internal structure.
