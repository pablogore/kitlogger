use serde::{Deserialize, Serialize};
use std::fmt;

/// Telemetry batch containing telemetry data.
///
/// This struct represents a batch of telemetry data that can be sent
/// across an execution boundary. It contains resource information and
/// collections of different types of telemetry signals.
/// 
/// Note: Resource, Span, Metric, and LogRecord types are defined in AS-03.
/// This is a placeholder for the canonical types from the owning specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryBatch {
    /// Resource information for the telemetry data.
    /// 
    /// TODO: Replace with canonical Resource type from AS-03
    pub resource: String,
    
    /// Collection of trace spans.
    /// 
    /// TODO: Replace with canonical Span type from AS-03
    pub traces: Vec<String>,
    
    /// Collection of metrics.
    /// 
    /// TODO: Replace with canonical Metric type from AS-03
    pub metrics: Vec<String>,
    
    /// Collection of log records.
    /// 
    /// TODO: Replace with canonical LogRecord type from AS-03
    pub logs: Vec<String>,
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
        resource: String,
        traces: Vec<String>,
        metrics: Vec<String>,
        logs: Vec<String>,
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
            TelemetryBatchError::EmptyBatch => write!(f, "telemetry batch cannot be empty"),
        }
    }
}

impl std::error::Error for TelemetryBatchError {}