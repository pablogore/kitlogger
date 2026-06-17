/// Health types for adapter health reporting.
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl Default for AdapterHealth {
    fn default() -> Self {
        AdapterHealth::Unknown
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthReport {
    pub status: AdapterHealth,
    pub reason: String,
    pub timestamp: SystemTime,
}
