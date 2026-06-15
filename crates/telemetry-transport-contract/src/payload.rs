use serde::{Deserialize, Serialize};

use crate::batch::TelemetryBatch;
pub use context_propagation::propagation_metadata::PropagationMetadata;

/// Metadata for transport-specific information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportMetadata {
    /// The timestamp when this metadata was created.
    pub timestamp: std::time::SystemTime,
}

impl TransportMetadata {
    /// Creates new transport metadata with the current timestamp.
    pub fn now() -> Self {
        Self {
            timestamp: std::time::SystemTime::now(),
        }
    }
}

/// A payload envelope for telemetry data.
///
/// This struct wraps telemetry data with metadata for transport.
/// It contains transport-specific metadata, propagation metadata,
/// and the actual telemetry data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayloadEnvelope {
    /// Transport-specific metadata.
    pub transport_metadata: TransportMetadata,

    /// Propagation metadata from AS-01.
    pub propagation_metadata: PropagationMetadata,

    /// The telemetry data.
    pub payload: TelemetryBatch,
}
