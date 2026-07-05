use crate::TransportError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use telemetry_types::PayloadEnvelope;

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

/// Result type for transport operations.
pub type TransportResult<T> = Result<T, TransportError>;

/// Transport trait for sending telemetry payloads.
///
/// This trait defines the contract for transport implementations.
/// It is protocol-agnostic — implementors can use HTTP, gRPC, in-memory,
/// or any other transport mechanism.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a telemetry payload and return the delivery mode used.
    async fn send(&self, envelope: PayloadEnvelope) -> TransportResult<DeliveryMode>;
}

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
}
