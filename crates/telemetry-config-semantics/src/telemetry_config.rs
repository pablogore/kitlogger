use crate::ConfigError;
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

/// TelemetryConfig represents the top-level telemetry configuration.
///
/// Scope (post Phase 1 of the Logging Pipeline Consolidation, ADR-008 §4): this
/// type is the source of plugin-enablement flags for a future Plugin layer
/// (Migration Plan Phase 10). It does not model, and MUST NOT be used to derive,
/// "is logging enabled" — that concept is owned exclusively by the Logging
/// domain's `kit_config::LoggingConfig`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelemetryConfig {
    /// Whether the plugin layer's telemetry subsystem (tracing, metrics, and
    /// propagation collectively) is enabled. This is the plugin layer's master
    /// switch (Migration Plan Phase 10) — it MUST NOT be read as, or used to
    /// derive, "is logging enabled"; that concept is owned exclusively by the
    /// Logging domain's `kit_config::LoggingConfig.enabled` (materialized and
    /// validated by `kit-config`).
    /// Accepts the legacy `"enabled"` key via serde alias for backward compatibility.
    #[serde(alias = "enabled")]
    pub telemetry_enabled: bool,
    /// Whether distributed tracing is enabled.
    #[serde(default = "default_true")]
    pub tracing_enabled: bool,
    /// Whether metrics collection is enabled.
    #[serde(default = "default_true")]
    pub metrics_enabled: bool,
    /// Whether context propagation (e.g. W3C TraceContext) is enabled.
    #[serde(default = "default_true")]
    pub propagation_enabled: bool,
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
            telemetry_enabled: true,
            tracing_enabled: true,
            metrics_enabled: true,
            propagation_enabled: true,
            exporters: Some(vec![crate::ExporterConfig::default()]),
            resources: Some(crate::ResourceConfig::default()),
            verbosity: Some(crate::VerbosityPolicy::default()),
            schema_version: crate::SchemaVersion::default(),
        }
    }
}

impl TelemetryConfig {
    /// Validates the configuration.
    ///
    /// FR-008 (trace-ratio sampling validation) was removed in the Logging
    /// Pipeline Consolidation's Phase 1 (ADR-008 §4, ADR-010): `sampling_rate`
    /// duplicated a concept `kit_config::LoggingConfig.sampling` already owns.
    /// With that check gone, `TelemetryConfig` has no remaining fields that
    /// require validation, so this always returns `Ok(())`. The method is kept
    /// as a stable extension point for future plugin-layer fields (Migration
    /// Plan Phase 10) that may introduce their own invariants.
    pub fn validate(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}
