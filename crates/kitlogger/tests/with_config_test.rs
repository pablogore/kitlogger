//! Integration tests for `KITLogger::with_config` (FR-011).

use kitlogger::KITLogger;
use kitlogger_formatter::LogFormat;
use telemetry_config_semantics::{EffectiveTelemetryState, TelemetryConfig};

/// FR-011: with_config constructs successfully and effective_state() returns Enabled
/// for a default (valid, all-flags-true) config.
#[test]
fn with_config_default_config_returns_enabled_state() {
    let logger = KITLogger::with_config(TelemetryConfig::default());
    assert_eq!(logger.effective_state(), EffectiveTelemetryState::Enabled);
}

/// FR-011: with_config stores the evaluated effective state.
/// A config with telemetry_enabled=false (and valid sampling) → Disabled.
#[test]
fn with_config_disabled_config_returns_disabled_state() {
    let config = TelemetryConfig {
        telemetry_enabled: false,
        ..Default::default()
    };
    let logger = KITLogger::with_config(config);
    assert_eq!(logger.effective_state(), EffectiveTelemetryState::Disabled);
}

/// FR-011 non-regression: KITLogger::new() still constructs successfully.
#[test]
fn kitlogger_new_still_constructs() {
    let _logger = KITLogger::new();
}

/// FR-011 non-regression: KITLogger::with_format() still constructs successfully.
#[test]
fn kitlogger_with_format_still_constructs() {
    let _logger = KITLogger::with_format(LogFormat::Json);
}

/// Newly constructed logger via new() defaults to Enabled state.
#[test]
fn kitlogger_new_default_effective_state_is_enabled() {
    let logger = KITLogger::new();
    assert_eq!(logger.effective_state(), EffectiveTelemetryState::Enabled);
}
