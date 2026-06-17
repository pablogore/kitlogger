use crate::BackpressureSignal;
use serde::{Deserialize, Serialize};

/// Error type for transport operations.
///
/// This enum represents various errors that can occur during telemetry transport.
/// It is non-exhaustive to allow concrete transport implementations to add
/// their own error variants without breaking changes.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportError {
    /// The operation timed out.
    Timeout,
    
    /// The transport is currently unavailable.
    Unavailable,
    
    /// The transport is experiencing backpressure.
    ///
    /// This variant contains a `BackpressureSignal` with information
    /// about when to retry the operation.
    Backpressure(BackpressureSignal),
    
    /// The payload is too large for the transport.
    PayloadTooLarge,
    
    /// The transport protocol is not supported.
    UnsupportedTransport,
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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