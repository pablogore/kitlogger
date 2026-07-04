//! Integration tests for `KITLogger`'s construction from `kit_config::LoggingConfig`
//! (capability `kitlogger-config-integration`, FR-001/FR-002/FR-003).

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

/// FR-003: this phase intentionally does not introduce emission gating. The
/// purpose of this test is to verify that `LoggingConfig.enabled` is not
/// consulted during construction or logging behavior until the dedicated
/// gating capability is implemented in a later migration phase — `log`/
/// `log_record` must return the same `Ok`/`Err` outcome regardless of
/// `.enabled`, and `from_logging_config` must never branch on it.
#[test]
fn enabled_false_does_not_change_emission_yet() {
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

    assert_eq!(
        logger_enabled.log_record(&record, None).is_ok(),
        logger_disabled.log_record(&record, None).is_ok(),
        "log_record behavior must be identical regardless of LoggingConfig.enabled"
    );
    assert_eq!(
        logger_enabled.log(Severity::Info, "hello").is_ok(),
        logger_disabled.log(Severity::Info, "hello").is_ok(),
        "log behavior must be identical regardless of LoggingConfig.enabled"
    );
}
