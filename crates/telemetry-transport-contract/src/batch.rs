use context_propagation::models::{LogRecord, Metric, Resource, Span};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Telemetry batch containing telemetry data.
///
/// This struct represents a batch of telemetry data that can be sent
/// across an execution boundary. It contains resource information and
/// collections of different types of telemetry signals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryBatch {
    /// Resource information for the telemetry data.
    pub resource: Resource,

    /// Collection of trace spans.
    pub traces: Vec<Span>,

    /// Collection of metrics.
    pub metrics: Vec<Metric>,

    /// Collection of log records.
    pub logs: Vec<LogRecord>,
}

impl TelemetryBatch {
    /// Creates a new telemetry batch.
    ///
    /// # Arguments
    /// * `resource` - Resource information for the telemetry data
    /// * `traces` - Collection of trace spans
    /// * `metrics` - Collection of metrics
    /// * `logs` - Collection of log records
    ///
    /// # Returns
    /// * `Ok(TelemetryBatch)` if at least one of traces, metrics, or logs is non-empty
    /// * `Err(TelemetryBatchError)` if all signal types are empty
    pub fn new(
        resource: Resource,
        traces: Vec<Span>,
        metrics: Vec<Metric>,
        logs: Vec<LogRecord>,
    ) -> Result<Self, TelemetryBatchError> {
        if traces.is_empty() && metrics.is_empty() && logs.is_empty() {
            return Err(TelemetryBatchError::EmptyBatch);
        }

        Ok(TelemetryBatch {
            resource,
            traces,
            metrics,
            logs,
        })
    }
}

/// Error type for telemetry batch operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TelemetryBatchError {
    /// Attempted to create a batch with all signal types empty.
    EmptyBatch,
}

impl fmt::Display for TelemetryBatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TelemetryBatchError::EmptyBatch => {
                write!(f, "telemetry batch must contain at least one signal type")
            }
        }
    }
}

impl std::error::Error for TelemetryBatchError {}
