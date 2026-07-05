//! Integration tests for `KITLogger`'s construction from `kit_config::LoggingConfig`
//! (capability `kitlogger-config-integration`, FR-001/FR-002).
//!
//! FR-003 ("No Behavioral Change from LoggingConfig Fields") is formally
//! superseded by `kitlogger-emission-pipeline` (change
//! `015-orchestration-fold`, see its proposal.md "Modified Capabilities"):
//! `LoggingConfig`'s behavioral fields — starting with `.enabled` — now
//! drive real pipeline behavior. See `emission_pipeline_test.rs` for the
//! capability that replaces FR-003's restriction.

use kit_config::{LoggingConfig, SamplingConfig, SamplingStrategy};
use kitlogger::KITLogger;
use kitlogger_log_domain::{LogRecord, Severity};
use std::time::SystemTime;

/// FR-001: a valid `LoggingConfig` constructs a usable `KITLogger`.
#[test]
fn constructs_from_valid_logging_config() {
    let config = LoggingConfig::default();
    let logger = KITLogger::from_logging_config(config);
    assert!(
        logger.is_ok(),
        "constructing from a valid LoggingConfig should succeed"
    );
}

/// FR-002: an invalid `LoggingConfig` (out-of-range Probabilistic sampling rate)
/// is rejected at construction time, surfacing the validation error.
#[test]
fn rejects_invalid_logging_config() {
    let config = LoggingConfig {
        sampling: SamplingConfig {
            enabled: true,
            strategy: SamplingStrategy::Probabilistic,
            rate: 1.5,
            ..SamplingConfig::default()
        },
        ..LoggingConfig::default()
    };

    let result = KITLogger::from_logging_config(config);
    assert!(
        result.is_err(),
        "constructing from an invalid LoggingConfig must fail"
    );
}

/// FR-003 superseded: `LoggingConfig.enabled` now gates the pipeline
/// (`kitlogger-emission-pipeline` FR-001). Both `from_logging_config`-built
/// loggers here share an un-initialized default `ConsoleExporterImpl` — a
/// disabled logger's calls short-circuit before ever touching that exporter
/// and always succeed, while an enabled logger's calls reach the exporter
/// and fail because it was never `init()`-ed. This asymmetry is the
/// intended, documented replacement for FR-003, not a bug: see
/// `emission_pipeline_test.rs::disabled_config_performs_no_processing` for
/// the capability-level test (using an initialized, capturing exporter) that
/// this regression check complements.
#[test]
fn enabled_false_now_short_circuits_before_dispatch() {
    let enabled_config = LoggingConfig::default();
    let disabled_config = LoggingConfig {
        enabled: false,
        ..LoggingConfig::default()
    };

    let logger_enabled =
        KITLogger::from_logging_config(enabled_config).expect("valid config constructs");
    let logger_disabled =
        KITLogger::from_logging_config(disabled_config).expect("valid config constructs");

    let record = LogRecord::new(
        SystemTime::UNIX_EPOCH,
        Severity::Info,
        "hello".to_string(),
        vec![],
    )
    .expect("valid record");

    assert!(
        logger_disabled.log_record(&record, None).is_ok(),
        "a disabled logger's log_record must short-circuit to Ok(()) without dispatching"
    );
    assert!(
        logger_enabled.log_record(&record, None).is_err(),
        "an enabled logger's log_record must reach the (un-initialized) exporter and fail"
    );

    assert!(
        logger_disabled.log(Severity::Info, "hello").is_ok(),
        "a disabled logger's log must short-circuit to Ok(()) without dispatching"
    );
    assert!(
        logger_enabled.log(Severity::Info, "hello").is_err(),
        "an enabled logger's log must reach the (un-initialized) exporter and fail"
    );
}
