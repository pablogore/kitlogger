# Logger API Contract

## Version: 1.0.0 (Draft)

## Crate: `kit-logger`

The `kit-logger` crate provides the public logging API. It has zero dependencies on any logging backend, serialization framework, or observability pipeline.

## Public API Surface

### Traits

#### `Logger`

```rust
pub trait Logger: Send + Sync {
    /// Returns true if the specified severity level is enabled.
    fn enabled(&self, level: &LogLevel) -> bool;

    /// Records a log entry. Returns Ok(()) on success, or an error
    /// describing the failure.
    fn log(&self, record: &LogRecord) -> Result<(), LoggerError>;

    /// Flushes any buffered log entries.
    fn flush(&self) -> Result<(), LoggerError>;

    /// Creates a new logger that injects `context` fields into every
    /// log entry before delegating to this logger. The original logger
    /// is unchanged.
    fn with_context(&self, context: LoggerContext) -> Arc<dyn Logger>;

    // --- Default convenience methods ---

    fn trace(&self, message: &str) { ... }
    fn debug(&self, message: &str) { ... }
    fn info(&self, message: &str) { ... }
    fn warn(&self, message: &str) { ... }
    fn error(&self, message: &str) { ... }
}
```

**Contract**:
- `enabled()` MUST be idempotent and cheap (no I/O, no allocation)
- `log()` MUST accept `&LogRecord` without modifying it
- `log()` MUST NOT panic under any circumstance
- `flush()` MUST drain all buffered entries, returning errors if any fail
- `with_context()` MUST NOT modify `self`; returned logger is independent
- Convenience methods MUST check `enabled()` before constructing a LogRecord
- Convenience methods MUST call `log()` internally
- All methods MUST be thread-safe (callable from any thread concurrently)

#### `LoggerFactory`

```rust
pub trait LoggerFactory: Send + Sync {
    fn create(&self) -> Result<Arc<dyn Logger>, LoggerError>;
}
```

**Contract**:
- `create()` MUST return a fully initialized, usable logger
- `create()` MAY return `Err(LoggerError::Configuration)` if initialization fails
- Multiple calls to `create()` MUST produce independent logger instances
- `create()` MUST NOT panic

### Structs

#### `LoggerContext`

```rust
pub struct LoggerContext { /* fields private */ }

impl LoggerContext {
    pub fn new() -> Self;
    pub fn with(self, key: &str, value: Value) -> Self;
    pub fn is_empty(&self) -> bool;
}
```

**Contract**:
- `new()` returns an empty context (no fields)
- `.with()` consumes `self` and returns a new `LoggerContext` (Builder pattern)
- Multiple `.with()` calls are commutative with respect to the final field set
- Field ordering is deterministic (sorted by key)
- No method mutates `self` — all produce new instances

#### `NoopLogger`

```rust
pub struct NoopLogger;

impl Logger for NoopLogger { ... }
```

**Contract**:
- `enabled()` always returns `false`
- `log()` always returns `Ok(())` — silently discards
- `flush()` always returns `Ok(())`
- `with_context(ctx)` returns `Arc::new(ContextLogger::new(...))`
- Zero allocation except for `with_context()`

### Enums

#### `LoggerError`

```rust
#[non_exhaustive]
pub enum LoggerError {
    Configuration(String),
    Backend(Box<dyn Error + Send + Sync>),
    Serialization(Box<dyn Error + Send + Sync>),
}
```

**Contract**:
- All variants implement `Display` with a human-readable message
- All variants implement `std::error::Error`
- `Configuration` errors are recoverable (retry with valid config)
- `Backend` and `Serialization` wrap inner errors without data loss
- The enum is `#[non_exhaustive]` — match with a wildcard arm

### Macros

```rust
macro_rules! log_trace   { ($logger:expr, $($arg:tt)+) => { ... } }
macro_rules! log_debug   { ($logger:expr, $($arg:tt)+) => { ... } }
macro_rules! log_info    { ($logger:expr, $($arg:tt)+) => { ... } }
macro_rules! log_warn    { ($logger:expr, $($arg:tt)+) => { ... } }
macro_rules! log_error   { ($logger:expr, $($arg:tt)+) => { ... } }
```

**Contract**:
- All macros accept any expression that implements `Logger`
- Message argument is `format_args!` syntax (`"hello {}", name`)
- If `enabled()` returns false, the message expression is NOT evaluated
- Macros expand to an `if enabled() { logger.log(...); }` block
- The target is captured via `module_path!()` at the call site

## Usage Examples

### Basic logging

```rust
use kit_logger::*;

let logger: Arc<dyn Logger> = factory.create()?;
logger.info("Application started");
logger.warn("Configuration missing, using defaults");
```

### Contextual logging

```rust
let ctx = LoggerContext::new()
    .with("tenant_id", Value::String("acme".into()))
    .with("request_id", Value::String("req-123".into()));

let request_logger = logger.with_context(ctx);
request_logger.info("Processing request");
// Entry carries: { tenant_id: "acme", request_id: "req-123", message: "..." }
```

### Macros

```rust
log_info!(logger, "User {} logged in", user.name);
log_error!(logger, "Failed to process order {}", order_id);
```

## Versioning

This API contract follows semantic versioning. Breaking changes include:
- Removing or renaming any public item
- Adding required methods to `Logger` or `LoggerFactory` (without defaults)
- Changing method signatures
- Adding required variants to `LoggerError`
