use serde::{Deserialize, Serialize};

use crate::error::TelemetryBatchError;

/// Placeholder — will be replaced by canonical definition from child spec.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span(pub String);

/// Placeholder — will be replaced by canonical definition from child spec.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metric(pub String);

/// Placeholder — will be replaced by canonical definition from child spec.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogRecord(pub String);

/// Origin resource/entity identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource(pub String);

/// A batch of telemetry data.
///
/// This struct contains traces, metrics, and logs that are grouped together
/// for transport. It enforces that at least one of these categories must be
/// present to avoid empty batches.
///
/// # Examples
///
/// ```rust
/// use telemetry_transport_contract::{TelemetryBatch, Span, Resource};
///
/// let batch = TelemetryBatch::new(
///     Resource("resource1".to_string()),
///     vec![Span("trace1".to_string())],
///     vec![],
///     vec![],
/// );
/// assert!(batch.is_ok());
///
/// let empty_batch = TelemetryBatch::new(
///     Resource("resource1".to_string()),
///     vec![],
///     vec![],
///     vec![],
/// );
/// assert!(empty_batch.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryBatch {
    /// The resource identifier for this batch.
    pub resource: Resource,

    /// The traces in this batch.
    pub traces: Vec<Span>,

    /// The metrics in this batch.
    pub metrics: Vec<Metric>,

    /// The logs in this batch.
    pub logs: Vec<LogRecord>,
}

impl TelemetryBatch {
    /// Creates a new telemetry batch.
    ///
    /// Returns an error if all signal types (traces, metrics, logs) are empty.
    pub fn new(
        resource: Resource,
        traces: Vec<Span>,
        metrics: Vec<Metric>,
        logs: Vec<LogRecord>,
    ) -> Result<TelemetryBatch, TelemetryBatchError> {
        if traces.is_empty() && metrics.is_empty() && logs.is_empty() {
            return Err(TelemetryBatchError);
        }

        Ok(TelemetryBatch {
            resource,
            traces,
            metrics,
            logs,
        })
    }
}
