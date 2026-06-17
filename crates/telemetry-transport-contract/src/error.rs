use std::fmt;

use serde::{Deserialize, Serialize};

use crate::transport::BackpressureSignal;

/// The result type for transport operations.
pub type TransportResult<T> = Result<T, TransportError>;

/// Errors that can occur during transport operations.
///
/// This enum represents the various errors that can occur when sending
/// telemetry data across execution boundaries. It is designed to be
/// non-exhaustive to allow concrete transport implementations to add
/// their own error variants without breaking changes.
///
/// # Examples
///
/// ```rust
/// use telemetry_transport_contract::{TransportError, TransportResult};
///
/// fn handle_result() -> TransportResult<()> {
///     Err(TransportError::Timeout)
/// }
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportError {
    /// The transport operation timed out.
    Timeout,

    /// The destination is unreachable.
    Unavailable,

    /// The transport encountered a backpressure condition.
    Backpressure(BackpressureSignal),

    /// The payload exceeds transport limits.
    PayloadTooLarge,

    /// The requested transport is not available.
    UnsupportedTransport,
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportError::Timeout => write!(f, "transport timeout"),
            TransportError::Unavailable => write!(f, "transport unavailable"),
            TransportError::Backpressure(_) => write!(f, "transport backpressure"),
            TransportError::PayloadTooLarge => write!(f, "payload too large"),
            TransportError::UnsupportedTransport => write!(f, "unsupported transport"),
        }
    }
}

impl std::error::Error for TransportError {}

/// Error returned when a `TelemetryBatch` is constructed with no signals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryBatchError;

impl fmt::Display for TelemetryBatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "telemetry batch must contain at least one signal type")
    }
}

impl std::error::Error for TelemetryBatchError {}
