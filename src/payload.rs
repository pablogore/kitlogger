use crate::TelemetryBatch;
use serde::{Deserialize, Serialize};

/// Metadata about the transport layer.
///
/// This struct contains information about the transport layer that
/// is used to send telemetry data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportMetadata {
    /// Timestamp of when the metadata was created.
    pub timestamp: std::time::SystemTime,
    
    /// Content type of the payload.
    pub content_type: String,
    
    /// Encoding hints for the payload.
    pub encoding: String,
}

impl TransportMetadata {
    /// Creates new transport metadata with the current timestamp.
    pub fn now() -> Self {
        TransportMetadata {
            timestamp: std::time::SystemTime::now(),
            content_type: "application/json".to_string(),
            encoding: "utf-8".to_string(),
        }
    }
}

/// Metadata for context propagation.
///
/// This struct contains information needed to propagate context
/// across execution boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropagationMetadata {
    /// Context propagation data.
    pub context: String,
}

/// Envelope for telemetry payloads.
///
/// This struct wraps the telemetry data with metadata needed for transport
/// across execution boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayloadEnvelope {
    /// Metadata about the transport layer.
    pub transport_metadata: TransportMetadata,
    
    /// Metadata for context propagation.
    pub propagation_metadata: PropagationMetadata,
    
    /// The actual telemetry data payload.
    pub payload: TelemetryBatch,
}