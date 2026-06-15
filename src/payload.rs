//! Payload envelope types for telemetry data.
//!
//! This module defines the [`PayloadEnvelope`] struct which wraps telemetry
//! data with metadata for transport.

use serde::{Deserialize, Serialize};

use crate::batch::TelemetryBatch;
use crate::transport::{BackpressureSignal, DeliveryMode};

/// Metadata for transport-specific information.
///
/// This struct contains information that is specific to the transport layer
/// and is used to provide context about how the data was sent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportMetadata {
    /// The timestamp when this metadata was created.
    pub timestamp: std::time::SystemTime,
}

impl TransportMetadata {
    /// Creates new transport metadata with the current timestamp.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use as_02::TransportMetadata;
    ///
    /// let metadata = TransportMetadata::now();
    /// assert!(metadata.timestamp.elapsed().unwrap().as_secs() < 1);
    /// ```
    pub fn now() -> Self {
        Self {
            timestamp: std::time::SystemTime::now(),
        }
    }
}

/// Metadata for propagation information.
///
/// This struct contains information that is used to propagate context
/// across execution boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropagationMetadata {
    /// The propagation context.
    pub context: std::collections::HashMap<String, String>,
}

impl Default for PropagationMetadata {
    /// Creates default propagation metadata with an empty context.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use as_02::PropagationMetadata;
    ///
    /// let metadata = PropagationMetadata::default();
    /// assert!(metadata.context.is_empty());
    /// ```
    fn default() -> Self {
        Self {
            context: std::collections::HashMap::new(),
        }
    }
}

/// A payload envelope for telemetry data.
///
/// This struct wraps telemetry data with metadata for transport.
/// It contains transport-specific metadata, propagation metadata,
/// and the actual telemetry data.
///
/// # Examples
///
/// ```rust
/// use as_02::{PayloadEnvelope, TelemetryBatch, TransportMetadata, PropagationMetadata};
///
/// let envelope = PayloadEnvelope {
///     transport_metadata: TransportMetadata::now(),
///     propagation_metadata: PropagationMetadata::default(),
///     payload: TelemetryBatch::new("resource1".to_string(), vec![], vec![], vec![]).unwrap(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayloadEnvelope {
    /// Transport-specific metadata.
    pub transport_metadata: TransportMetadata,

    /// Propagation metadata.
    pub propagation_metadata: PropagationMetadata,

    /// The telemetry data.
    pub payload: TelemetryBatch,
}