//! Telemetry batch types for organizing telemetry data.
//!
//! This module defines the [`TelemetryBatch`] struct which is used to
//! organize telemetry data into batches for transport.



use serde::{Deserialize, Serialize};

use crate::error::{TransportError, TransportResult};

/// A batch of telemetry data.
///
/// This struct contains traces, metrics, and logs that are grouped together
/// for transport. It enforces that at least one of these categories must be
/// present to avoid empty batches.
///
/// # Examples
///
/// ```rust
/// use as_02::{TelemetryBatch, TransportResult};
///
/// // This will succeed
/// let batch = TelemetryBatch::new(
///     "resource1".to_string(),
///     vec!["trace1".to_string()],
///     vec![],
///     vec![],
/// );
///
/// // This will fail because all categories are empty
/// let empty_batch = TelemetryBatch::new(
///     "resource1".to_string(),
///     vec![],
///     vec![],
///     vec![],
/// );
/// assert!(empty_batch.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryBatch {
    /// The resource identifier for this batch.
    pub resource: String,

    /// The traces in this batch.
    pub traces: Vec<String>,

    /// The metrics in this batch.
    pub metrics: Vec<String>,

    /// The logs in this batch.
    pub logs: Vec<String>,
}

impl TelemetryBatch {
    /// Creates a new telemetry batch.
    ///
    /// # Arguments
    ///
    /// * `resource` - The resource identifier for this batch
    /// * `traces` - The traces in this batch
    /// * `metrics` - The metrics in this batch
    /// * `logs` - The logs in this batch
    ///
    /// # Returns
    ///
    /// A `TransportResult` containing the new batch if at least one category
    /// is non-empty, or an error if all categories are empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use as_02::{TelemetryBatch, TransportResult};
    ///
    /// // This will succeed
    /// let batch = TelemetryBatch::new(
    ///     "resource1".to_string(),
    ///     vec!["trace1".to_string()],
    ///     vec![],
    ///     vec![],
    /// );
    /// assert!(batch.is_ok());
    ///
    /// // This will fail because all categories are empty
    /// let empty_batch = TelemetryBatch::new(
    ///     "resource1".to_string(),
    ///     vec![],
    ///     vec![],
    ///     vec![],
    /// );
    /// assert!(empty_batch.is_err());
    /// ```
    pub fn new(
        resource: String,
        traces: Vec<String>,
        metrics: Vec<String>,
        logs: Vec<String>,
    ) -> TransportResult<TelemetryBatch> {
        if traces.is_empty() && metrics.is_empty() && logs.is_empty() {
            return Err(TransportError::InvalidBatch);
        }

        Ok(TelemetryBatch {
            resource,
            traces,
            metrics,
            logs,
        })
    }
}