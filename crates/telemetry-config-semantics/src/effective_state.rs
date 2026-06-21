use serde::{Deserialize, Serialize};

/// EffectiveTelemetryState represents the computed state of the telemetry subsystem
/// after evaluating all configuration flags and validation constraints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectiveTelemetryState {
    /// All capability flags are true and configuration is valid.
    Enabled,
    /// `telemetry_enabled` is false (and sampling is valid).
    Disabled,
    /// `telemetry_enabled` is true but at least one capability flag is false.
    Partial,
    /// Configuration is invalid (e.g., sampling rate out of range). Checked before all other states.
    Fallback,
}
