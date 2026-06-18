/// Health types for adapter health reporting.
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AdapterHealth {
    Healthy,
    Degraded,
    Unhealthy,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthReport {
    pub status: AdapterHealth,
    pub reason: String,
    pub timestamp: SystemTime,
}
