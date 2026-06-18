use serde::{Deserialize, Serialize};

/// TelemetryConfig represents the top-level telemetry configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled.
    pub enabled: bool,
    /// The sampling policy configuration.
    pub sampling: Option<crate::SamplingPolicy>,
    /// The exporter configurations.
    pub exporters: Option<Vec<crate::ExporterConfig>>,
    /// The resource configuration.
    pub resources: Option<crate::ResourceConfig>,
    /// The verbosity policy configuration.
    pub verbosity: Option<crate::VerbosityPolicy>,
    /// The schema version of this configuration.
    pub schema_version: crate::SchemaVersion,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling: Some(crate::SamplingPolicy::default()),
            exporters: Some(vec![crate::ExporterConfig::default()]),
            resources: Some(crate::ResourceConfig::default()),
            verbosity: Some(crate::VerbosityPolicy::default()),
            schema_version: crate::SchemaVersion::default(),
        }
    }
}
