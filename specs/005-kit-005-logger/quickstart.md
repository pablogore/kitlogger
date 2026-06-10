# Quickstart: KIT-005 Logger API

## Add the Dependency

```toml
[dependencies]
kit-logger = { path = "crates/kit-logger" }
```

## Create a Logger

```rust
use kit_logger::{LoggerFactory, NoopLoggerFactory};

let factory = NoopLoggerFactory;
let logger = factory.create().expect("logger creation failed");
```

(For real backends, swap `NoopLoggerFactory` with a provider factory from a future KIT spec.)

## Emit Log Entries

```rust
// Via convenience methods
logger.info("Server starting on port {}", port);
logger.warn("Disk space below threshold: {}%", usage);

// Via macros (zero-cost when disabled)
log_info!(logger, "Request processed in {}ms", duration);
log_error!(logger, "Connection failed: {}", err);
```

## Add Context

```rust
use kit_logger::LoggerContext;

let ctx = LoggerContext::new()
    .with("tenant_id", "acme".into())
    .with("request_id", "req-123".into());

let request_logger = logger.with_context(ctx);
request_logger.info("Processing order");
// Fields: { tenant_id: "acme", request_id: "req-123", message: "Processing order" }
```

## Check If Enabled

```rust
if logger.enabled(&LogLevel::Debug) {
    let stats = expensive_computation();
    logger.debug("Computed stats: {:?}", stats);
}
```

## Flush

```rust
logger.flush().expect("flush failed");
```

## Handle Errors

```rust
match logger.log(&record) {
    Ok(()) => {},
    Err(LoggerError::Backend(e)) => eprintln!("Log backend unavailable: {}", e),
    Err(e) => eprintln!("Log failed: {}", e),
}
```

## Full Example

```rust
use std::sync::Arc;
use kit_logger::*;

fn main() -> Result<(), LoggerError> {
    let factory = NoopLoggerFactory;
    let logger = factory.create()?;

    let ctx = LoggerContext::new()
        .with("service", Value::String("api-gateway".into()))
        .with("version", Value::String("1.0.0".into()));

    let svc_logger = logger.with_context(ctx);
    svc_logger.info("Service initialized");

    log_info!(svc_logger, "Listening on port {}", 8080);
    Ok(())
}
```

## Next Steps

- See [contracts/logger-api.md](./contracts/logger-api.md) for the full API contract
- See [data-model.md](./data-model.md) for entity definitions
- See [plan.md](./plan.md) for the implementation plan and task breakdown
