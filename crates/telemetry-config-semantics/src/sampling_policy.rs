use crate::ConfigError;
use serde::{Deserialize, Serialize};

/// SamplingPolicyType represents the type of sampling policy to use.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SamplingPolicyType {
    AlwaysOn,
    AlwaysOff,
    TraceIdRatio,
    ParentBased,
    ConsistentProbability,
    Extension(String),
}

/// SamplingPolicy represents the sampling configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SamplingPolicy {
    /// The type of sampling policy.
    pub policy_type: SamplingPolicyType,
    /// The sampling rate for policies that use it.
    pub sampling_rate: f64,
}

impl Default for SamplingPolicy {
    fn default() -> Self {
        Self {
            policy_type: SamplingPolicyType::AlwaysOn,
            sampling_rate: 1.0,
        }
    }
}

impl SamplingPolicy {
    /// Validates that `sampling_rate` is within the inclusive range `[0.0, 1.0]`.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !(0.0..=1.0).contains(&self.sampling_rate) {
            return Err(ConfigError::InvalidSamplingRate(self.sampling_rate));
        }
        Ok(())
    }
}
