# KITLogger

KITLogger is a comprehensive logging system for Rust applications with support for multiple exporters including console output.

## Crates

- `kitlogger-log-domain`: Core logging domain models
- `console-exporter`: Console output exporter
- `telemetry-adapter-contracts`: Telemetry adapter contracts
- `telemetry-types`: Telemetry data types
- `telemetry-config-semantics`: Configuration semantics
- `telemetry-transport-contract`: Transport contracts
- `context-propagation`: Context propagation utilities

## Console Exporter

The console exporter provides a simple way to output log messages to the console with different flush strategies.

### Usage

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

## Integration

The console exporter can be integrated with the main KITLogger system using the telemetry adapter registry pattern.