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
        if uuid.is_nil() {
            return Err("Correlation identifier UUID must not be nil".to_string());
        }
        Self::from_uuid(uuid)
            .ok_or_else(|| "Correlation identifier UUID must not be nil".to_string())
    }
}

impl CorrelationIdentifier {
    /// Create a new correlation identifier using UUID v7
    pub fn new() -> Self {
        let id = Uuid::now_v7();
        let created_at = extract_timestamp_from_uuid(&id);

        Self { id, created_at }
    }

    /// Create a correlation identifier from an existing UUID.
    /// Returns None if the UUID is nil.
    pub fn from_uuid(uuid: Uuid) -> Option<Self> {
        if uuid.is_nil() {
            return None;
        }
        let created_at = extract_timestamp_from_uuid(&uuid);
        Some(Self {
            id: uuid,
            created_at,
        })
    }

    /// Check if this correlation identifier is valid
    pub fn is_valid(&self) -> bool {
        !self.id.is_nil()
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

/// Extract timestamp from UUID v7 using the uuid crate's official API.
/// Returns Unix timestamp in milliseconds.
fn extract_timestamp_from_uuid(uuid: &Uuid) -> i64 {
    match uuid.get_timestamp() {
        Some(ts) => {
            let (secs, nsecs) = ts.to_unix();
            (secs as i64) * 1000 + (nsecs as i64) / 1_000_000
        }
        None => {
            // Fallback for non-timestamp UUIDs: extract the embedded timestamp
            // bytes manually. This should not happen with v7 UUIDs, but we
            // handle it defensively.
            let bytes = uuid.as_bytes();
            ((bytes[0] as u64) << 40
                | (bytes[1] as u64) << 32
                | (bytes[2] as u64) << 24
                | (bytes[3] as u64) << 16
                | (bytes[4] as u64) << 8
                | (bytes[5] as u64)) as i64
        }
    }
}
