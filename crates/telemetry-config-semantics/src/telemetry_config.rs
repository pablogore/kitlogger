use crate::{ConfigError, EffectiveTelemetryState};
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

/// TelemetryConfig represents the top-level telemetry configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled overall.
    /// Accepts the legacy `"enabled"` key via serde alias for backward compatibility.
    #[serde(alias = "enabled")]
    pub telemetry_enabled: bool,
    /// Whether distributed tracing is enabled.
    #[serde(default = "default_true")]
    pub tracing_enabled: bool,
    /// Whether metrics collection is enabled.
    #[serde(default = "default_true")]
    pub metrics_enabled: bool,
    /// Whether correlation ID propagation is enabled.
    #[serde(default = "default_true")]
    pub correlation_enabled: bool,
    /// Whether context propagation (e.g. W3C TraceContext) is enabled.
    #[serde(default = "default_true")]
    pub propagation_enabled: bool,
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
            telemetry_enabled: true,
            tracing_enabled: true,
            metrics_enabled: true,
            correlation_enabled: true,
            propagation_enabled: true,
            sampling: Some(crate::SamplingPolicy::default()),
            exporters: Some(vec![crate::ExporterConfig::default()]),
            resources: Some(crate::ResourceConfig::default()),
            verbosity: Some(crate::VerbosityPolicy::default()),
            schema_version: crate::SchemaVersion::default(),
        }
    }
}

impl TelemetryConfig {
    /// Computes the effective telemetry state based on configuration flags and validation.
    ///
    /// Evaluation order (Fallback is checked FIRST per FR-007):
    /// 1. If sampling is present and invalid → `Fallback`
    /// 2. If `telemetry_enabled` is false → `Disabled`
    /// 3. If all four capability flags are true → `Enabled`
    /// 4. Otherwise → `Partial`
    pub fn effective_state(&self) -> EffectiveTelemetryState {
        // Fallback first: invalid sampling always surfaces regardless of other flags.
        if let Some(ref s) = self.sampling {
            if s.validate().is_err() {
                return EffectiveTelemetryState::Fallback;
            }
        }

        if !self.telemetry_enabled {
            return EffectiveTelemetryState::Disabled;
        }

        let all_capabilities = self.tracing_enabled
            && self.metrics_enabled
            && self.correlation_enabled
            && self.propagation_enabled;

        if all_capabilities {
            EffectiveTelemetryState::Enabled
        } else {
            EffectiveTelemetryState::Partial
        }
    }

    /// Validates the configuration, returning an error if any field is out of range.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if let Some(ref s) = self.sampling {
            s.validate()?;
        }
        Ok(())
    }
}
