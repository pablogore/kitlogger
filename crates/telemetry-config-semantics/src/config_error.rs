/// ConfigError represents errors that can occur during telemetry configuration validation.
/// No external dependencies — implements Display and Error manually.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    /// The sampling rate is outside the valid inclusive range [0.0, 1.0].
    InvalidSamplingRate(f64),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidSamplingRate(rate) => {
                write!(
                    f,
                    "invalid sampling rate: {rate}; expected a value in the inclusive range [0.0, 1.0]"
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}
