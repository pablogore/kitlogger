# Data Model: KIT-005 Logger API

## Overview

KIT-005 introduces seven new types/entities. Three (LogLevel, LogRecord, Value) are reused from KIT-001 and described here for reference only.

## Entities

### LogLevel (from KIT-001)

The severity classification of a log entry. Reused from KIT-001, not redefined.

**Variants**: `Trace`, `Debug`, `Info`, `Warn`, `Error`

**Operations**:
- Equality and ordering (Trace < Debug < Info < Warn < Error)
- Serialization / Deserialization (roundtrip stable)
- Display (human-readable output)
- Case-insensitive parsing from text

---

### Value (from KIT-001)

A structured field value carried by log entries. Reused from KIT-001, not redefined.

**Variants**: `String(String)`, `Bool(bool)`, `I64(i64)`, `U64(u64)`, `F64(f64)`

**Traits**: Clone, Serialize, Deserialize, Display

---

### LogRecord (from KIT-001)

An immutable structured log entry. Reused from KIT-001, not redefined.

**Fields**:

| Field | Type | Description |
|---|---|---|
| `level` | `LogLevel` | Severity of the entry |
| `target` | `String` | Source component identifier (e.g., module path) |
| `message` | `String` | Human-readable log message |
| `timestamp` | `SystemTime` | When the entry was created |
| `fields` | `BTreeMap<String, Value>` | Structured key-value data, ordered by key |

**Constraints**:
- Immutable after construction (all fields set at creation)
- Field ordering is deterministic (sorted by key via BTreeMap)
- No backend-specific fields

---

### Logger (KIT-005)

The primary logging interface. A trait, not a struct.

**Type**: `trait Logger: Send + Sync`

**Methods**:

| Method | Signature | Description |
|---|---|---|
| `enabled` | `fn enabled(&self, level: &LogLevel) -> bool` | Query if a severity level is active |
| `log` | `fn log(&self, record: &LogRecord) -> Result<(), LoggerError>` | Record a log entry |
| `flush` | `fn flush(&self) -> Result<(), LoggerError>` | Flush pending entries |
| `with_context` | `fn with_context(&self, context: LoggerContext) -> Arc<dyn Logger>` | Create a context-wrapped logger |
| `trace` | `fn trace(&self, message: &str)` | Convenience: log at Trace level |
| `debug` | `fn debug(&self, message: &str)` | Convenience: log at Debug level |
| `info` | `fn info(&self, message: &str)` | Convenience: log at Info level |
| `warn` | `fn warn(&self, message: &str)` | Convenience: log at Warn level |
| `error` | `fn error(&self, message: &str)` | Convenience: log at Error level |

**Constraints**:
- Object-safe: `Arc<dyn Logger>`, `Box<dyn Logger>` compile
- Thread-safe: `Send + Sync`
- Provider-agnostic: no backend types in method signatures
- Convenience methods have default implementations calling `log()`

---

### LoggerFactory (KIT-005)

Creates logger instances.

**Type**: `trait LoggerFactory: Send + Sync`

**Methods**:

| Method | Signature | Description |
|---|---|---|
| `create` | `fn create(&self) -> Result<Arc<dyn Logger>, LoggerError>` | Create a new logger instance |

**Constraints**:
- Thread-safe: `Send + Sync`
- Abstract: no hardcoded backend or configuration types
- Returns provider-agnostic `Arc<dyn Logger>`

---

### LoggerContext (KIT-005)

Immutable builder for contextual metadata.

**Type**: `struct LoggerContext`

**Fields**:

| Field | Type | Description |
|---|---|---|
| `fields` | `BTreeMap<String, Value>` | Contextual key-value pairs, ordered by key |

**Methods**:

| Method | Signature | Description |
|---|---|---|
| `new` | `fn new() -> Self` | Create an empty context |
| `with` | `fn with(self, key: &str, value: Value) -> Self` | Add a field (consumes, returns new) |
| `is_empty` | `fn is_empty(&self) -> bool` | True if no fields |

**Constraints**:
- Immutable: each `.with()` produces a new instance
- Deterministic ordering: fields stored in BTreeMap
- Convertible to/from BTreeMap iteration

---

### ContextLogger (KIT-005)

A Logger implementation that wraps another logger and injects context fields.

**Type**: `struct ContextLogger`

**Fields**:

| Field | Type | Description |
|---|---|---|
| `inner` | `Arc<dyn Logger>` | The wrapped logger |
| `context` | `LoggerContext` | Context fields to inject |

**Behavior**:
- `enabled()`: delegates to `inner.enabled()`
- `log(record)`: merges context fields into record, then delegates to `inner.log()`
- `flush()`: delegates to `inner.flush()`
- `with_context(new_ctx)`: returns a new ContextLogger wrapping *this* ContextLogger (nested contexts merge on each delegation)

**Merge algorithm**:
```
fn merge(record: &LogRecord, context: &LoggerContext) -> LogRecord:
    let mut merged_fields = context.fields.clone();   // start with context fields
    for (k, v) in record.fields:                      // overlay record fields (wins on conflict)
        merged_fields.insert(k, v);
    return LogRecord { fields: merged_fields, ..record }
```

---

### NoopLogger (KIT-005)

A Logger implementation that silently discards all data.

**Type**: `struct NoopLogger`

**Behavior**:
- `enabled()`: always returns `false`
- `log()`: returns `Ok(())` without recording
- `flush()`: returns `Ok(())` without action
- `with_context(ctx)`: returns `Arc::new(ContextLogger::new(Arc::new(NoopLogger), ctx))`

---

### LoggerError (KIT-005)

Typed error for logging failures.

**Type**: `enum LoggerError`

**Variants**:

| Variant | Data | Description |
|---|---|---|
| `Configuration` | `String` | Invalid configuration or factory setup failure |
| `Backend` | `Box<dyn Error + Send + Sync>` | Backend write or flush failure |
| `Serialization` | `Box<dyn Error + Send + Sync>` | Log record formatting or serialization failure |

**Constraints**:
- Non-exhaustive (extensible without breaking change)
- Implements `std::error::Error` and `Display`
- Derived via `thiserror::Error`
