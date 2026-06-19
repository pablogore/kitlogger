# Console Exporter

A console exporter for KITLogger that outputs log messages to the console.

## Features

- Supports different flush strategies (immediate, on-shutdown, batch)
- Level-based stream routing (info to stdout, error to stderr)
- Proper lifecycle management (init, run, flush, shutdown)
- Thread-safe implementation

## Usage

```rust
use console_exporter::{ConsoleExporter, ConsoleExporterImpl, OnShutdownFlush};
use kitlogger_log_domain::{LogRecord, Severity};
use std::sync::Arc;

// Create a console exporter with on-shutdown flush strategy
let exporter = Arc::new(ConsoleExporterImpl::with_flush_strategy(
    Box::new(OnShutdownFlush)
));

// Initialize the exporter
exporter.init()?;

// Create a log record
let log_record = LogRecord::new(
    "test message".to_string(),
    Severity::Info,
    std::time::SystemTime::now(),
    vec![],
);

// Export the log record
exporter.export(&log_record)?;

// Shutdown the exporter
exporter.shutdown()?;
```

## Flush Strategies

- `ImmediateFlush`: Flushes immediately after each write
- `OnShutdownFlush`: Flushes only on shutdown
- `BatchFlush`: Flushes in batches based on configured batch size

## Integration with KITLogger

The console exporter can be integrated with the main KITLogger system by using it as an exporter in the telemetry adapter registry.