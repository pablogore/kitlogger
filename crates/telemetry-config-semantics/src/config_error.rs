/// ConfigError represents errors that can occur during telemetry configuration validation.
///
/// Empty by construction: `TelemetryConfig::validate()` currently has nothing left to
/// validate (its former sampling-rate check was removed in the Logging Pipeline
/// Consolidation's Phase 1 — see `telemetry_config.rs`). An uninhabited enum makes the
/// "this can never fail" guarantee explicit at the type level rather than carrying a
/// variant nothing can construct. Add a variant here only when a real invariant needs one.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {}
    }
}

impl std::error::Error for ConfigError {}
