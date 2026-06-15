//! Transport error types for telemetry data flow.
//!
//! This module defines the [`TransportError`] enum and [`TransportResult`] type alias
//! which are used to represent errors that can occur during transport operations.

use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::transport::BackpressureSignal;

/// The result type for transport operations.
///
/// This type is a convenience alias for `Result<T, TransportError>`.
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
/// use as_02::{TransportError, TransportResult};
///
/// fn handle_result() -> TransportResult<()> {
///     // Simulate an error
///     Err(TransportError::Timeout)
/// }
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportError {
    /// The transport timed out.
    Timeout,

    /// The transport encountered a backpressure condition.
    ///
    /// This variant contains a [`BackpressureSignal`] that provides
    /// information about when to retry sending data.
    Backpressure(BackpressureSignal),

    /// The transport encountered an invalid batch.
    ///
    /// This error is returned when a [`TelemetryBatch`] is created
    /// with all categories empty.
    InvalidBatch,

    /// The transport encountered an unknown error.
    ///
    /// This variant is used for errors that don't fit into the other
    /// categories.
    Unknown,
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportError::Timeout => write!(f, "transport timeout"),
            TransportError::Backpressure(_) => write!(f, "transport backpressure"),
            TransportError::InvalidBatch => write!(f, "invalid telemetry batch"),
            TransportError::Unknown => write!(f, "unknown transport error"),
        }
    }
}

impl std::error::Error for TransportError {}