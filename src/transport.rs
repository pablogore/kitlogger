use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Delivery mode for telemetry transport operations.
///
/// This enum represents the different ways telemetry can be delivered
/// across an execution boundary. It is returned as a value from
/// `Transport::send()` to indicate which delivery mode was used.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryMode {
    /// Send the telemetry and forget about it.
    ///
    /// The sender does not wait for acknowledgment or response.
    FireAndForget,
    
    /// Send the telemetry and wait for a response.
    ///
    /// The sender waits for an acknowledgment or response from the receiver.
    RequestResponse,
    
    /// Send telemetry in batches.
    ///
    /// Multiple telemetry items are sent together in a batch.
    Batch,
    
    /// Send telemetry in a streaming fashion.
    ///
    /// Telemetry is sent as a continuous stream of data.
    Streaming,
}

/// Signal indicating that a transport operation is experiencing backpressure.
///
/// This structure provides information about backpressure conditions,
/// including a hint about when to retry the operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackpressureSignal {
    /// Optional duration to wait before retrying the operation.
    ///
    /// If present, this indicates how long the sender should wait
    /// before attempting to send again.
    pub retry_after: Option<Duration>,
}

/// Result type for transport operations.
pub type TransportResult<T> = Result<T, TransportError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delivery_mode_serialization() {
        let mode = DeliveryMode::FireAndForget;
        let serialized = serde_json::to_string(&mode).unwrap();
        let deserialized: DeliveryMode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(mode, deserialized);
    }

    #[test]
    fn test_backpressure_signal() {
        let signal = BackpressureSignal {
            retry_after: Some(std::time::Duration::from_secs(5)),
        };
        assert_eq!(signal.retry_after, Some(std::time::Duration::from_secs(5)));
    }
}