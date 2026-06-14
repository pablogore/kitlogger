//! Correlation identifier implementation for cross-signal correlation
//!
//! This module implements correlation identifiers that can be used to
//! correlate telemetry signals (traces, metrics, logs) across service boundaries.

use std::str::FromStr;
use uuid::Uuid;

/// Correlation identifier with creation timestamp
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CorrelationIdentifier {
    /// The UUID identifier
    pub id: Uuid,

    /// Creation timestamp in milliseconds since Unix epoch
    pub created_at: i64,
}

impl FromStr for CorrelationIdentifier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s).map_err(|e| format!("Invalid UUID format: {}", e))?;
        Ok(Self::from_uuid(uuid))
    }
}

impl CorrelationIdentifier {
    /// Create a new correlation identifier using UUID v7
    pub fn new() -> Self {
        let id = Uuid::now_v7();
        let created_at = extract_timestamp_from_uuid(&id);

        Self { id, created_at }
    }

    /// Create a correlation identifier from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        let created_at = extract_timestamp_from_uuid(&uuid);
        Self {
            id: uuid,
            created_at,
        }
    }

    /// Get the UUID of this correlation identifier
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get the creation timestamp
    pub fn created_at(&self) -> i64 {
        self.created_at
    }
}

impl Default for CorrelationIdentifier {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CorrelationIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// Extract timestamp from UUID v7
fn extract_timestamp_from_uuid(uuid: &Uuid) -> i64 {
    // For UUID v7, the timestamp is in bytes 0-7 (first 8 bytes)
    // The timestamp is stored in big-endian format as 48-bit integer
    let bytes = uuid.as_bytes();
    let timestamp_ms = ((bytes[0] as u64) << 40
        | (bytes[1] as u64) << 32
        | (bytes[2] as u64) << 24
        | (bytes[3] as u64) << 16
        | (bytes[4] as u64) << 8
        | (bytes[5] as u64)) as i64;

    timestamp_ms
}
