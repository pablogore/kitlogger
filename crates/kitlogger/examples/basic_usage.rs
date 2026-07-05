//! Minimal end-to-end usage of `KITLogger` through its public API:
//! construction, logging at different severities (routed to stdout/stderr
//! by `console-exporter`), inspecting registered outputs, and shutdown.
//!
//! Run with: `cargo run --example basic_usage -p kitlogger`

use kitlogger::KITLogger;
use kitlogger_log_domain::Severity;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logger = KITLogger::new();
    logger.init()?;

    println!("--- registered outputs ---");
    for id in logger.registered_output_ids() {
        println!("{id}");
    }

    println!("--- info goes to stdout, error goes to stderr ---");
    logger.log(Severity::Info, "hello from basic_usage (info)")?;
    logger.log(Severity::Error, "hello from basic_usage (error)")?;

    logger.shutdown()?;

    println!("--- logging after shutdown returns an error, not a silent no-op ---");
    match logger.log(Severity::Info, "this should fail, exporter is shut down") {
        Ok(()) => println!("unexpected: Ok after shutdown"),
        Err(e) => println!("expected error after shutdown: {e}"),
    }

    Ok(())
}
