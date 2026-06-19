use console_exporter::{ConsoleExporter, ConsoleExporterImpl, OnShutdownFlush};
use kitlogger_log_domain::{LogRecord, Severity};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exporter = ConsoleExporterImpl::with_flush_strategy(Box::new(OnShutdownFlush));

    exporter.init()?;

    let record = LogRecord::new(
        std::time::SystemTime::now(),
        Severity::Info,
        "Hello from console-exporter example!".to_string(),
        vec![],
    )?;

    exporter.export(record.message(), *record.severity())?;
    exporter.shutdown()?;

    println!("Example completed successfully!");
    Ok(())
}
