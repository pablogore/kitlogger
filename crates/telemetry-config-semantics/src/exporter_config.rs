use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CompressionType represents the compression type for exporter endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompressionType {
    None,
    Gzip,
}

/// ExporterConfig represents the configuration for a telemetry exporter.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExporterConfig {
    /// The type of exporter to use.
    pub exporter_type: String,
    /// The endpoint to send telemetry data to.
    pub endpoint: Option<String>,
    /// The compression type to use.
    pub compression: CompressionType,
    /// Headers to include in requests.
    pub headers: HashMap<String, String>,
    /// The timeout in seconds for requests.
    pub timeout_secs: u64,
    /// Additional settings for the exporter.
    pub settings: HashMap<String, String>,
}

impl Default for ExporterConfig {
    fn default() -> Self {
        Self {
            exporter_type: "console".to_string(),
            endpoint: None,
            compression: CompressionType::None,
            headers: HashMap::new(),
            timeout_secs: 30,
            settings: HashMap::new(),
        }
    }
}
