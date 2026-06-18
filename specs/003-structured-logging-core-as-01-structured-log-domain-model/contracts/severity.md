# Severity Contract

**SPEC_ID**: `003-structured-logging-core-as-01-structured-log-domain-model`

---

## Enum Definition

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}
```

## Traits

```rust
impl Display for Severity;      // "Trace", "Debug", "Info", "Warn", "Error", "Fatal"
impl FromStr for Severity;      // Err = (), case-insensitive
```

## Ordering

`Trace < Debug < Info < Warn < Error < Fatal`
