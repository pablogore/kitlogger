use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ResourceConfig represents the resource configuration for telemetry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceConfig {
    /// The name of the service.
    pub service_name: String,
    /// The version of the service.
    pub service_version: String,
    /// The deployment environment.
    pub deployment_environment: String,
    /// Additional attributes for the resource.
    pub attributes: HashMap<String, String>,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            service_name: "".to_string(),
            service_version: "unknown".to_string(),
            deployment_environment: "development".to_string(),
            attributes: HashMap::new(),
        }
    }
}