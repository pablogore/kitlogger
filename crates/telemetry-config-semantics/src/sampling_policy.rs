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