//! Console exporter implementation.

use std::sync::Arc;

use crate::{
    error::ExportError,
    flush::FlushStrategy,
    lifecycle::LifecycleStateMachine,
    stream_router::{LevelStreamMapping, StreamRouter},
};
use kitlogger_log_domain::Severity;

/// Console exporter trait for exporting log messages.
pub trait ConsoleExporter: Send + Sync {
    /// Exports a formatted log message.
    fn export(&self, msg: &str, severity: Severity) -> Result<(), ExportError>;
    /// Flushes pending output.
    fn flush(&self) -> Result<(), ExportError>;
    /// Shuts down the exporter.
    fn shutdown(&self) -> Result<(), ExportError>;
}

/// Console exporter implementation.
pub struct ConsoleExporterImpl {
    router: Arc<std::sync::Mutex<StreamRouter>>,
    lifecycle: Arc<std::sync::Mutex<LifecycleStateMachine>>,
    flush_strategy: Box<dyn FlushStrategy>,
    write_count: Arc<std::sync::atomic::AtomicUsize>,
}

impl Default for ConsoleExporterImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleExporterImpl {
    /// Creates a new console exporter with default settings.
    pub fn new() -> Self {
        Self::with_flush_strategy(Box::new(crate::flush::ImmediateFlush))
    }

    /// Creates a new console exporter with a custom flush strategy.
    pub fn with_flush_strategy(flush_strategy: Box<dyn FlushStrategy>) -> Self {
        let router = Arc::new(std::sync::Mutex::new(StreamRouter::new()));
        let lifecycle = Arc::new(std::sync::Mutex::new(LifecycleStateMachine::new()));
        let write_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        Self {
            router,
            lifecycle,
            flush_strategy,
            write_count,
        }
    }

    /// Initializes the console exporter.
    pub fn init(&self) -> Result<(), ExportError> {
        let mut lifecycle = self
            .lifecycle
            .lock()
            .map_err(|_| ExportError::Lifecycle("Failed to acquire lifecycle lock".to_string()))?;
        lifecycle.transition_to(crate::lifecycle::LifecycleState::Running)?;
        Ok(())
    }

    /// Sets a custom level-to-stream mapping.
    pub fn set_mapping(&self, mapping: LevelStreamMapping) {
        let mut router = self.router.lock().unwrap();
        router.set_mapping(mapping);
    }

    /// Sets custom writers for stdout and stderr.
    pub fn set_writers(
        &self,
        stdout: Box<dyn std::io::Write + Send>,
        stderr: Box<dyn std::io::Write + Send>,
    ) {
        let mut router = self.router.lock().unwrap();
        router.set_writers(stdout, stderr);
    }
}

/// Conforms `ConsoleExporterImpl` to the generic Output Port defined by
/// `output-adapter-contracts`, in addition to its existing `ConsoleExporter`
/// trait. Both traits intentionally delegate to the same `export` method
/// below, so console output behavior remains defined in a single place.
impl output_adapter_contracts::Output for ConsoleExporterImpl {
    fn dispatch(
        &self,
        formatted: &str,
        severity: Severity,
    ) -> Result<(), output_adapter_contracts::OutputError> {
        self.export(formatted, severity)
            .map_err(|e| output_adapter_contracts::OutputError::new(e.to_string()))
    }
}

impl ConsoleExporter for ConsoleExporterImpl {
    fn export(&self, msg: &str, severity: Severity) -> Result<(), ExportError> {
        // Check if we're initialized
        let lifecycle = self
            .lifecycle
            .lock()
            .map_err(|_| ExportError::Lifecycle("Failed to acquire lifecycle lock".to_string()))?;
        if !lifecycle.is_running() {
            return Err(ExportError::Lifecycle(
                "Exporter is not running".to_string(),
            ));
        }

        // Write the message
        let mut router = self
            .router
            .lock()
            .map_err(|_| ExportError::Lifecycle("Failed to acquire router lock".to_string()))?;
        router.write(msg, severity)?;

        // Increment write count and check if we should flush
        let write_count = self
            .write_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if self.flush_strategy.should_flush(write_count) {
            self.flush()?;
        }

        Ok(())
    }

    fn flush(&self) -> Result<(), ExportError> {
        // In a real implementation, this would flush the underlying streams
        // For now, we just return Ok(())
        Ok(())
    }

    fn shutdown(&self) -> Result<(), ExportError> {
        // Transition to flushing state
        let mut lifecycle = self
            .lifecycle
            .lock()
            .map_err(|_| ExportError::Lifecycle("Failed to acquire lifecycle lock".to_string()))?;
        lifecycle.transition_to(crate::lifecycle::LifecycleState::Flushing)?;

        // If flush strategy requires flushing on shutdown, do it
        if self.flush_strategy.on_shutdown() {
            self.flush()?;
        }

        // Transition to shutdown state
        lifecycle.transition_to(crate::lifecycle::LifecycleState::Shutdown)?;
        Ok(())
    }
}
