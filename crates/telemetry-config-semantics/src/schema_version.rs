use serde::{Deserialize, Serialize};

/// SchemaVersion represents the version of the telemetry configuration schema.
/// It is independent from Kit Config's pipeline version.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaVersion {
    /// The semantic version of the schema.
    pub version: String,
    /// Optional description of the schema version.
    pub description: Option<String>,
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            description: None,
        }
    }
}