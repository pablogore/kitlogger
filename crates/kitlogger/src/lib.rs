use async_trait::async_trait;
use console_exporter::{ConsoleExporter, ConsoleExporterImpl, OnShutdownFlush};
use kitlogger_log_domain::Severity;
use std::sync::Arc;
use std::time::SystemTime;
use telemetry_adapter_contracts::{
    AdapterError, AdapterHealth, AdapterId, AdapterResult, CommonAdapterBase, ExporterAdapter,
    HealthReport, LifecycleAdapter, TelemetryDelivery,
};
use telemetry_types::PayloadEnvelope;

pub struct KITLogger {
    exporter: Arc<ConsoleExporterImpl>,
    id: AdapterId,
}

impl Default for KITLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl KITLogger {
    pub fn new() -> Self {
        let exporter = Arc::new(ConsoleExporterImpl::with_flush_strategy(Box::new(
            OnShutdownFlush,
        )));
        Self {
            exporter,
            id: AdapterId::new("kitlogger").expect("hardcoded id should never be empty"),
        }
    }

    pub fn init(&self) -> Result<(), AdapterError> {
        self.exporter
            .init()
            .map_err(|e| AdapterError::InitializationFailed(e.to_string()))
    }

    pub fn log(&self, severity: Severity, message: &str) -> Result<(), AdapterError> {
        self.exporter
            .export(message, severity)
            .map_err(|e| AdapterError::InitializationFailed(e.to_string()))
    }

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
