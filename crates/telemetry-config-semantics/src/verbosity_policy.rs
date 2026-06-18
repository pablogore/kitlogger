use serde::{Deserialize, Serialize};

/// VerbosityLevel represents the verbosity level for telemetry signals.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerbosityLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// VerbosityPolicy represents the verbosity configuration for different telemetry signals.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerbosityPolicy {
    /// The verbosity level for trace signals.
    pub trace_level: VerbosityLevel,
    /// The verbosity level for metric signals.
    pub metric_level: VerbosityLevel,
    /// The verbosity level for log signals.
    pub log_level: VerbosityLevel,
}

impl Default for VerbosityPolicy {
    fn default() -> Self {
        Self {
            trace_level: VerbosityLevel::Info,
            metric_level: VerbosityLevel::Info,
            log_level: VerbosityLevel::Info,
        }
    }
}
