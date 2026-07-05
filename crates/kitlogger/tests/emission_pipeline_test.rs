//! Integration tests for `kitlogger-emission-pipeline`: `KITLogger::log`/
//! `log_record`'s end-to-end sequencing (enabled gate -> level filter ->
//! sample -> redact -> buffer -> format -> dispatch), per
//! `openspec/changes/015-orchestration-fold/specs/kitlogger-emission-pipeline/spec.md`.
//!
//! These tests are black-box: `Sampler`/`Redactor`/`Buffer` are concrete
//! types owned by `KITLogger`, not injected trait objects, so stage
//! invocation is observed via captured stdout/stderr output rather than
//! literal call-count spies.

use console_exporter::{BatchFlush, ConsoleExporterImpl, OnShutdownFlush};
use kit_config::{
    BufferingConfig, LogFormat as ConfigLogFormat, LogLevel, LoggingConfig, RedactionConfig,
    SamplingConfig, SamplingStrategy,
};
use kitlogger::KITLogger;
use kitlogger_log_domain::{LogAttribute, LogAttributeValue, LogRecord, Severity};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// A test writer that records output in a shared `Vec<u8>` buffer.
#[derive(Clone)]
struct TestWriter {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl TestWriter {
    fn new() -> Self {
        Self {
            buf: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn contents(&self) -> String {
        let guard = self.buf.lock().unwrap();
        String::from_utf8(guard.clone()).unwrap()
    }

    fn is_empty(&self) -> bool {
        self.buf.lock().unwrap().is_empty()
    }
}

impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Creates a `KITLogger` from `config`, wired to `TestWriter` capture
/// buffers via a pre-initialized `ConsoleExporterImpl`.
///
/// Returns `(logger, stdout_writer, stderr_writer)`.
fn make_logger(config: LoggingConfig) -> (KITLogger, TestWriter, TestWriter) {
    let stdout = TestWriter::new();
    let stderr = TestWriter::new();

    let exporter = ConsoleExporterImpl::with_flush_strategy(Box::new(OnShutdownFlush));
    exporter.set_writers(Box::new(stdout.clone()), Box::new(stderr.clone()));
    exporter.init().unwrap();

    let logger = KITLogger::from_logging_config_with_exporter(config, Arc::new(exporter))
        .expect("valid config constructs");

    (logger, stdout, stderr)
}

fn record(severity: Severity, message: &str) -> LogRecord {
    LogRecord::new(SystemTime::now(), severity, message.to_string(), vec![]).expect("valid record")
}

// ---------------------------------------------------------------------------
// FR-001: Enabled Gate
// ---------------------------------------------------------------------------

/// Scenario: disabled logging performs no further processing, even for
/// `Severity::Fatal` (which would otherwise always pass the level filter) —
/// proving this is the enabled gate short-circuiting, not the level filter.
#[test]
fn disabled_config_performs_no_processing() {
    let config = LoggingConfig {
        enabled: false,
        ..LoggingConfig::default()
    };
    let (logger, stdout, stderr) = make_logger(config);

    let result = logger.log_record(&record(Severity::Fatal, "should not appear"), None);

    assert!(
        result.is_ok(),
        "a disabled logger's log_record must not error"
    );
    assert!(
        stdout.is_empty(),
        "disabled logging must not write to stdout. Got: {:?}",
        stdout.contents()
    );
    assert!(
        stderr.is_empty(),
        "disabled logging must not write to stderr. Got: {:?}",
        stderr.contents()
    );
}

// ---------------------------------------------------------------------------
// FR-002: Level Filtering
// ---------------------------------------------------------------------------

/// Table-driven over `(LogLevel, Severity)` pairs, including `Fatal` at the
/// strictest configurable level (`Error`).
#[test]
fn level_filtering_table() {
    let cases: &[(LogLevel, Severity, bool)] = &[
        // Below threshold: dropped.
        (LogLevel::Warn, Severity::Info, false),
        (LogLevel::Error, Severity::Warn, false),
        // At or above threshold: proceeds.
        (LogLevel::Warn, Severity::Warn, true),
        (LogLevel::Warn, Severity::Error, true),
        (LogLevel::Info, Severity::Info, true),
        // Fatal always proceeds, even at the strictest configurable level.
        (LogLevel::Error, Severity::Fatal, true),
        (LogLevel::Trace, Severity::Fatal, true),
    ];

    for (level, severity, should_proceed) in cases {
        let config = LoggingConfig {
            level: *level,
            ..LoggingConfig::default()
        };
        let (logger, stdout, stderr) = make_logger(config);

        logger
            .log_record(&record(*severity, "marker-text"), None)
            .expect("log_record must not error");

        let wrote_something = !stdout.is_empty() || !stderr.is_empty();
        assert_eq!(
            wrote_something, *should_proceed,
            "level={level:?} severity={severity:?}: expected proceed={should_proceed}, got wrote_something={wrote_something}"
        );
    }
}

// ---------------------------------------------------------------------------
// FR-003: Sampling Gate
// ---------------------------------------------------------------------------

/// `SamplingStrategy::EveryNth` with `n = 2` deterministically samples the
/// 1st call and drops the 2nd (see `kitlogger-sampling`'s
/// `sample_every_nth`: `previous.is_multiple_of(n)`, `previous` starting at
/// 0). A sampled-out record must not reach redaction, buffering,
/// formatting, or dispatch — observed here as "produced no output at all".
#[test]
fn sampled_out_record_does_not_reach_later_stages() {
    let config = LoggingConfig {
        sampling: SamplingConfig {
            enabled: true,
            strategy: SamplingStrategy::EveryNth,
            n: 2,
            ..SamplingConfig::default()
        },
        ..LoggingConfig::default()
    };
    let (logger, stdout, _stderr) = make_logger(config);

    logger
        .log_record(&record(Severity::Info, "first-sampled-in"), None)
        .expect("log_record must not error");
    let after_first = stdout.contents();
    assert!(
        after_first.contains("first-sampled-in"),
        "the 1st EveryNth(2) call must be sampled in. Got: {after_first:?}"
    );

    logger
        .log_record(&record(Severity::Info, "second-sampled-out"), None)
        .expect("log_record must not error");
    let after_second = stdout.contents();
    assert_eq!(
        after_second, after_first,
        "the 2nd EveryNth(2) call must be sampled out, producing no further output"
    );
}

// ---------------------------------------------------------------------------
// FR-004: Redaction Before Buffering
// ---------------------------------------------------------------------------

/// A record with a sensitive attribute must have that attribute redacted
/// by the time it is dispatched (buffering disabled, so dispatch is
/// synchronous — observed directly in the formatted JSON output).
#[test]
fn dispatched_record_reflects_redaction() {
    let config = LoggingConfig {
        redact: RedactionConfig {
            enabled: true,
            fields: vec!["password".to_string()],
        },
        ..LoggingConfig::default()
    };
    let (logger, stdout, _stderr) = make_logger(config);

    let attr = LogAttribute::new(
        "password".to_string(),
        LogAttributeValue::string("hunter2".to_string()),
    )
    .expect("valid attribute");
    let record_with_secret = LogRecord::new(
        SystemTime::now(),
        Severity::Info,
        "login attempt".to_string(),
        vec![attr],
    )
    .expect("valid record");

    logger
        .log_record(&record_with_secret, None)
        .expect("log_record must not error");

    let out = stdout.contents();
    assert!(
        out.contains("**REDACTED**"),
        "dispatched output must contain the redaction marker. Got: {out:?}"
    );
    assert!(
        !out.contains("hunter2"),
        "dispatched output must not contain the original secret. Got: {out:?}"
    );
}

// ---------------------------------------------------------------------------
// FR-005: Buffering Defers Formatting and Dispatch
// ---------------------------------------------------------------------------

/// With buffering enabled and a batch size greater than 1, a single added
/// record must not be formatted or dispatched yet.
#[test]
fn buffering_defers_format_and_dispatch() {
    let config = LoggingConfig {
        buffering: BufferingConfig {
            enabled: true,
            batch_size: 3,
            flush_interval_ms: 60_000,
        },
        ..LoggingConfig::default()
    };
    let (logger, stdout, stderr) = make_logger(config);

    logger
        .log_record(&record(Severity::Info, "held-in-buffer"), None)
        .expect("log_record must not error");

    assert!(
        stdout.is_empty() && stderr.is_empty(),
        "a single record below batch_size must not be formatted/dispatched yet. stdout: {:?}, stderr: {:?}",
        stdout.contents(),
        stderr.contents()
    );
}

/// With buffering disabled, formatting and dispatch happen immediately for
/// each record.
#[test]
fn disabled_buffering_is_synchronous() {
    let config = LoggingConfig {
        buffering: BufferingConfig {
            enabled: false,
            ..BufferingConfig::default()
        },
        ..LoggingConfig::default()
    };
    let (logger, stdout, _stderr) = make_logger(config);

    logger
        .log_record(&record(Severity::Info, "immediate-dispatch"), None)
        .expect("log_record must not error");

    assert!(
        stdout.contents().contains("immediate-dispatch"),
        "disabled buffering must dispatch immediately. Got: {:?}",
        stdout.contents()
    );
}

/// Regression test: a batch already removed from the buffer must not lose
/// records after the first dispatch failure. Every record in the batch is
/// pulled out of the buffer via `mem::take` before any of them is
/// formatted/dispatched, so a record that isn't attempted here is lost with
/// no trace — neither still buffered nor dispatched. Forces every dispatch
/// in a 3-record batch to fail (by shutting the exporter down first) and
/// asserts all 3 were still attempted, not just the first.
#[test]
fn batch_dispatch_failure_still_attempts_every_record() {
    let config = LoggingConfig {
        buffering: BufferingConfig {
            enabled: true,
            batch_size: 3,
            flush_interval_ms: 60_000,
        },
        ..LoggingConfig::default()
    };
    let (logger, _stdout, _stderr) = make_logger(config);

    // Puts the underlying exporter into a state where every dispatch fails.
    logger
        .shutdown()
        .expect("shutdown on an empty buffer must not error");

    logger
        .log_record(&record(Severity::Info, "one"), None)
        .expect("below batch_size must not error");
    logger
        .log_record(&record(Severity::Info, "two"), None)
        .expect("below batch_size must not error");
    let result = logger.log_record(&record(Severity::Info, "three"), None);

    let err = result.expect_err("dispatch to a shut-down exporter must fail");
    let failure_count = err.to_string().matches("console:").count();
    assert_eq!(
        failure_count, 3,
        "all 3 records in the batch must be attempted even though each fails; \
         only {failure_count} attempt(s) were made. Error: {err}"
    );
}

// ---------------------------------------------------------------------------
// FR-006: Flush Drains the Pipeline
// ---------------------------------------------------------------------------

/// Records added below the flush threshold must still be formatted and
/// dispatched once `shutdown()` is called.
#[test]
fn shutdown_drains_buffered_records() {
    let config = LoggingConfig {
        buffering: BufferingConfig {
            enabled: true,
            batch_size: 100,
            flush_interval_ms: 60_000,
        },
        ..LoggingConfig::default()
    };
    let (logger, stdout, _stderr) = make_logger(config);

    logger
        .log_record(&record(Severity::Info, "record-one"), None)
        .expect("log_record must not error");
    logger
        .log_record(&record(Severity::Info, "record-two"), None)
        .expect("log_record must not error");

    assert!(
        stdout.is_empty(),
        "records below batch_size must not be dispatched before shutdown. Got: {:?}",
        stdout.contents()
    );

    logger.shutdown().expect("shutdown must not error");

    let out = stdout.contents();
    assert!(
        out.contains("record-one") && out.contains("record-two"),
        "shutdown must drain and dispatch every buffered record. Got: {out:?}"
    );
}

// ---------------------------------------------------------------------------
// FR-007 / FR-008: Formatting on Flush, Dispatch Only After Formatting
// ---------------------------------------------------------------------------

/// A flushed record must be formatted using the formatter selected via
/// `LoggingConfig.format` (not a hardcoded default) before dispatch.
#[test]
fn flushed_records_use_the_configured_format() {
    let config = LoggingConfig {
        format: ConfigLogFormat::Text,
        ..LoggingConfig::default()
    };
    let (logger, stdout, stderr) = make_logger(config);

    logger
        .log_record(&record(Severity::Info, "format-selection-marker"), None)
        .expect("log_record must not error");

    let out = stdout.contents() + &stderr.contents();
    assert!(
        out.contains("[INFO] format-selection-marker"),
        "expected Text-formatted output (LoggingConfig.format = Text). Got: {out:?}"
    );
    assert!(
        !out.trim_start().starts_with('{'),
        "expected non-JSON output when LoggingConfig.format = Text. Got: {out:?}"
    );
}

// ---------------------------------------------------------------------------
// FR-009: Default Output Registration
// ---------------------------------------------------------------------------

/// A `KITLogger` constructed from a default `LoggingConfig` registers a
/// console output, and no file-based output, by default.
#[test]
fn console_registered_by_default_no_file() {
    let config = LoggingConfig::default();
    let (logger, _stdout, _stderr) = make_logger(config);

    let ids: Vec<String> = logger
        .registered_output_ids()
        .iter()
        .map(|id| id.to_string())
        .collect();

    assert!(
        ids.iter().any(|id| id.contains("console")),
        "a console output must be registered by default. Got ids: {ids:?}"
    );
    assert!(
        !ids.iter().any(|id| id.contains("file")),
        "no file-based output must be registered by default. Got ids: {ids:?}"
    );
}

// ---------------------------------------------------------------------------
// FR-010: Single Dispatch Orchestrator
// ---------------------------------------------------------------------------

/// A dispatched record must be delivered exactly once — proxy evidence
/// against a second, parallel dispatch mechanism silently double-delivering.
#[test]
fn dispatch_delivers_exactly_once() {
    let config = LoggingConfig::default();
    let (logger, stdout, stderr) = make_logger(config);

    logger
        .log_record(&record(Severity::Info, "single-delivery-marker"), None)
        .expect("log_record must not error");

    let out = stdout.contents() + &stderr.contents();
    let occurrences = out.matches("single-delivery-marker").count();
    assert_eq!(
        occurrences, 1,
        "exactly one dispatch mechanism must deliver the record exactly once. Got: {out:?}"
    );
}

// ---------------------------------------------------------------------------
// Regression (Phase 5, task 5.3): console-exporter's own `FlushStrategy`
// (I/O-level) and the pipeline `Buffer` (application-level batching)
// compose without either masking the other.
// ---------------------------------------------------------------------------

/// `console-exporter`'s `BatchFlush(2)` counts *its own* `export` calls
/// (one per dispatched record) — a strategy entirely orthogonal to, and
/// unaware of, the pipeline `Buffer`'s batching of *pre-format* records.
/// With pipeline buffering `batch_size = 2` as well, the pipeline buffer
/// flushes both held records together as one 2-record batch, which then
/// drives `console-exporter`'s own `write_count` from 0 to 2 across two
/// `export` calls — exercising `BatchFlush`'s own threshold check at the
/// same moment the pipeline batch flushes. Both records must still reach
/// the output, uncorrupted and undropped, proving neither level silently
/// disables the other.
#[test]
fn console_flush_strategy_and_pipeline_buffer_compose() {
    let stdout = TestWriter::new();
    let stderr = TestWriter::new();

    let exporter = ConsoleExporterImpl::with_flush_strategy(Box::new(BatchFlush::new(2)));
    exporter.set_writers(Box::new(stdout.clone()), Box::new(stderr.clone()));
    exporter.init().unwrap();

    let config = LoggingConfig {
        buffering: BufferingConfig {
            enabled: true,
            batch_size: 2,
            flush_interval_ms: 60_000,
        },
        ..LoggingConfig::default()
    };
    let logger = KITLogger::from_logging_config_with_exporter(config, Arc::new(exporter))
        .expect("valid config constructs");

    logger
        .log_record(&record(Severity::Info, "combo-one"), None)
        .expect("log_record must not error");
    assert!(
        stdout.is_empty(),
        "pipeline buffer must hold the 1st record below its batch_size=2 threshold. Got: {:?}",
        stdout.contents()
    );

    logger
        .log_record(&record(Severity::Info, "combo-two"), None)
        .expect("log_record must not error");

    let out = stdout.contents();
    assert!(
        out.contains("combo-one") && out.contains("combo-two"),
        "both pipeline-batched records must reach the output once console-exporter's \
         own BatchFlush(2) threshold coincides with the pipeline buffer's flush. Got: {out:?}"
    );
}
