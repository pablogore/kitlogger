use async_trait::async_trait;
use console_exporter::{ConsoleExporter, ConsoleExporterImpl, OnShutdownFlush};
use kit_config::{LoggingConfig, Validation, ValidationReport};
use kitlogger_formatter::{formatter_from_config, LogFormat, RecordFormatter};
use kitlogger_log_domain::{LogContext, LogRecord, Severity};
use std::sync::Arc;
use std::time::SystemTime;
use telemetry_adapter_contracts::{
    AdapterError, AdapterHealth, AdapterId, AdapterResult, CommonAdapterBase, ExporterAdapter,
    HealthReport, LifecycleAdapter, TelemetryDelivery,
};
use telemetry_types::PayloadEnvelope;

pub mod buffer;
pub mod format_selection;

pub struct KITLogger {
    exporter: Arc<ConsoleExporterImpl>,
    formatter: Box<dyn RecordFormatter>,
    id: AdapterId,
}

impl Default for KITLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl KITLogger {
    /// Creates a `KITLogger` with the default JSON formatter and a new
    /// `ConsoleExporterImpl` using `OnShutdownFlush`.
    pub fn new() -> Self {
        let exporter = Arc::new(ConsoleExporterImpl::with_flush_strategy(Box::new(
            OnShutdownFlush,
        )));
        Self {
            exporter,
            formatter: formatter_from_config(LogFormat::Json),
            id: AdapterId::new("kitlogger").expect("hardcoded id should never be empty"),
        }
    }

    /// Creates a `KITLogger` with a custom formatter selected by `format`.
    pub fn with_format(format: LogFormat) -> Self {
        let exporter = Arc::new(ConsoleExporterImpl::with_flush_strategy(Box::new(
            OnShutdownFlush,
        )));
        Self {
            exporter,
            formatter: formatter_from_config(format),
            id: AdapterId::new("kitlogger").expect("hardcoded id should never be empty"),
        }
    }

    /// Creates a `KITLogger` from a `kit_config::LoggingConfig` value.
    ///
    /// `config` is validated via `kit_config`'s `Validation` trait at
    /// construction time; an invalid config fails fast and the caller receives
    /// the `ValidationReport` describing why. No field of `config` other than
    /// what `validate()` itself inspects is consulted here — in particular,
    /// `LoggingConfig.enabled` does not gate emission yet (that gate is
    /// designed once, in a later phase, together with level filtering).
    pub fn from_logging_config(config: LoggingConfig) -> Result<Self, ValidationReport> {
        config.validate()?;
        let exporter = Arc::new(ConsoleExporterImpl::with_flush_strategy(Box::new(
            OnShutdownFlush,
        )));
        Ok(Self {
            exporter,
            formatter: formatter_from_config(LogFormat::Json),
            id: AdapterId::new("kitlogger").expect("hardcoded id should never be empty"),
        })
    }

    /// Creates a `KITLogger` wired to a pre-built exporter with a given format.
    ///
    /// Intended for testing: callers supply a `ConsoleExporterImpl` with
    /// custom `set_writers` capture buffers already attached and initialized.
    pub fn with_exporter_and_format(exporter: Arc<ConsoleExporterImpl>, format: LogFormat) -> Self {
        Self {
            exporter,
            formatter: formatter_from_config(format),
            id: AdapterId::new("kitlogger").expect("hardcoded id should never be empty"),
        }
    }

    /// Initializes the underlying console exporter.
    pub fn init(&self) -> Result<(), AdapterError> {
        self.exporter
            .init()
            .map_err(|e| AdapterError::InitializationFailed(e.to_string()))
    }

    /// Formats `record` using the configured formatter and exports the result.
    ///
    /// This is the primary entry point for structured logging.
    pub fn log_record(
        &self,
        record: &LogRecord,
        context: Option<&LogContext>,
    ) -> Result<(), AdapterError> {
        let formatted = self
            .formatter
            .format(record, context)
            .map_err(|e| AdapterError::InitializationFailed(e.to_string()))?;
        self.exporter
            .export(&formatted, *record.severity())
            .map_err(|e| AdapterError::InitializationFailed(e.to_string()))
    }

    /// Exports a raw string directly (back-compat path, no formatter involved).
    pub fn log(&self, severity: Severity, message: &str) -> Result<(), AdapterError> {
        self.exporter
            .export(message, severity)
            .map_err(|e| AdapterError::InitializationFailed(e.to_string()))
    }

    /// Shuts down the underlying console exporter.
    pub fn shutdown(&self) -> Result<(), AdapterError> {
        self.exporter
            .shutdown()
            .map_err(|e| AdapterError::ShutdownFailed(e.to_string()))
    }
}

impl CommonAdapterBase for KITLogger {
    fn id(&self) -> &AdapterId {
        &self.id
    }

    fn health(&self) -> HealthReport {
        HealthReport {
            status: AdapterHealth::Healthy,
            reason: "kitlogger".into(),
            timestamp: SystemTime::now(),
        }
    }
}

#[async_trait]
impl LifecycleAdapter for KITLogger {
    async fn flush(&self) -> AdapterResult<()> {
        self.exporter
            .flush()
            .map_err(|e| AdapterError::FlushFailed(e.to_string()))
    }

    async fn shutdown(&self) -> AdapterResult<()> {
        self.exporter
            .shutdown()
            .map_err(|e| AdapterError::ShutdownFailed(e.to_string()))
    }
}

#[async_trait]
impl TelemetryDelivery for KITLogger {
    async fn deliver(&self, _envelope: PayloadEnvelope) -> AdapterResult<()> {
        Ok(())
    }
}

#[async_trait]
impl ExporterAdapter for KITLogger {
    async fn initialize(&self) -> AdapterResult<()> {
        self.init()
    }

    async fn start(&self) -> AdapterResult<()> {
        Ok(())
    }

    async fn stop(&self) -> AdapterResult<()> {
        Ok(())
    }
}
