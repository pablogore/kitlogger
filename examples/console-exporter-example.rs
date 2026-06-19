//! Example demonstrating how to use the console exporter with a main KITLogger system.

use console_exporter::{ConsoleExporter, ConsoleExporterImpl, OnShutdownFlush};
use kitlogger_log_domain::{LogRecord, Severity};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    
    println!("Console exporter example completed successfully!");
    Ok(())
}