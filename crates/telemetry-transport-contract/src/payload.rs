use crate::TelemetryBatch;
use context_propagation::propagation_metadata::PropagationMetadata;
use serde::{Deserialize, Serialize};

/// Metadata about the transport layer.
///
/// This struct contains information about the transport layer that
/// is used to send telemetry data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransportMetadata {
    /// Timestamp of when the metadata was created.
    pub timestamp: std::time::SystemTime,

    /// Content type of the payload.
    pub content_type: String,

    /// Encoding hints for the payload.
    pub encoding: String,
}

impl Default for TransportMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl TransportMetadata {
    /// Creates new transport metadata.
    pub fn new() -> Self {
        TransportMetadata {
            timestamp: std::time::SystemTime::now(),
            content_type: String::new(),
            encoding: String::new(),
        }
    }

    /// Creates new transport metadata with current timestamp.
    pub fn now() -> Self {
        TransportMetadata {
            timestamp: std::time::SystemTime::now(),
            content_type: String::new(),
            encoding: String::new(),
        }
    }
}

/// Envelope for telemetry payloads.
///
/// This struct wraps the telemetry data with metadata needed for transport
/// across execution boundaries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayloadEnvelope {
    /// Metadata about the transport layer.
    pub transport_metadata: TransportMetadata,

    /// Metadata for context propagation.
    pub propagation_metadata: PropagationMetadata,

    /// The actual telemetry data payload.
    pub payload: TelemetryBatch,
}
