use serde::{Deserialize, Serialize};

/// CapabilityState represents the enabled/disabled state of an individual telemetry capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityState {
    Enabled,
    Disabled,
}
