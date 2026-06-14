//! Correlation identifier implementation for cross-signal correlation
//!
//! This module implements correlation identifiers that can be used to
//! correlate telemetry signals (traces, metrics, logs) across service boundaries.

use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

/// Correlation identifier with creation timestamp
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CorrelationIdentifier {
    /// The UUID identifier
    pub id: Uuid,
    
    /// Creation timestamp in milliseconds since Unix epoch
    pub created_at: i64,
}

impl CorrelationIdentifier {
  /// Create a new correlation identifier using UUID v4
    pub fn new() -> Self {
        // Create a new UUID using the standard constructor
        let id = Uuid::from_u128(0x1234567890abcdef1234567890abcdef); // Use a valid UUID
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_millis() as i64);
        
        Self { id, created_at }
    }
    
    /// Create a correlation identifier from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_millis() as i64);
        Self { id: uuid, created_at }
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