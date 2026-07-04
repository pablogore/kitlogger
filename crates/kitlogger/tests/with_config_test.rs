//! Regression tests for `KITLogger`'s pre-existing constructors.
//!
//! `KITLogger::with_config(TelemetryConfig)` was retired in this change
//! (FR-004 of `kitlogger-config-integration`) — `KITLogger` no longer accepts
//! a `TelemetryConfig` nor references `EffectiveTelemetryState`/
//! `effective_state()`. See `logging_config_test.rs` for the
//! `kit_config::LoggingConfig`-based construction contract that replaces it
//! (FR-001/FR-002/FR-003).

use kitlogger::KITLogger;
use kitlogger_formatter::LogFormat;

/// Non-regression: `KITLogger::new()` still constructs successfully.
#[test]
fn kitlogger_new_still_constructs() {
    let _logger = KITLogger::new();
}

/// Non-regression: `KITLogger::with_format()` still constructs successfully.
#[test]
fn kitlogger_with_format_still_constructs() {
    let _logger = KITLogger::with_format(LogFormat::Json);
}
