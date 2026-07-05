//! Integration tests for the KITLogger formatting pipeline.
//!
//! Validates that `KITLogger::log_record` routes a `LogRecord` through the
//! formatter and delivers the formatted string to the console exporter.

use console_exporter::{ConsoleExporterImpl, OnShutdownFlush};
use kitlogger::KITLogger;
use kitlogger_formatter::LogFormat;
use kitlogger_log_domain::{LogAttribute, LogAttributeValue, LogContext, LogRecord, Severity};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

// ---------------------------------------------------------------------------
// Shared test helpers
// ---------------------------------------------------------------------------

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
}

impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Fixed timestamp: 2026-06-20T10:00:00Z = 1_781_949_600 seconds since epoch.
fn ts() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_781_949_600)
}

/// Creates a `KITLogger` wired to `TestWriter` buffers.
///
/// Returns `(logger, stdout_writer, stderr_writer)`.
fn make_logger_with_capture(format: LogFormat) -> (KITLogger, TestWriter, TestWriter) {
    let stdout = TestWriter::new();
    let stderr = TestWriter::new();

    let exporter = ConsoleExporterImpl::with_flush_strategy(Box::new(OnShutdownFlush));
    exporter.set_writers(Box::new(stdout.clone()), Box::new(stderr.clone()));
    exporter.init().unwrap();

    let logger = KITLogger::with_exporter_and_format(Arc::new(exporter), format);

    (logger, stdout, stderr)
}

// ---------------------------------------------------------------------------
// Spec: log_record routes LogRecord → formatter → exporter
// ---------------------------------------------------------------------------

/// Scenario: JSON format — record without context produces JSON with ts/level/msg keys.
#[test]
fn log_record_json_format_produces_json_output() {
    let (logger, stdout, _stderr) = make_logger_with_capture(LogFormat::Json);

    let record =
        LogRecord::new(ts(), Severity::Info, "login ok".to_string(), vec![]).expect("valid record");

    logger
        .log_record(&record, None)
        .expect("log_record should succeed");

    let out = stdout.contents();
    assert!(
        out.contains(r#""level":"INFO""#),
        "JSON output should contain level field. Got: {out:?}"
    );
    assert!(
        out.contains(r#""msg":"login ok""#),
        "JSON output should contain msg field. Got: {out:?}"
    );
}

/// Scenario: Text format — record with logger context produces `[INFO] auth: login ok`.
#[test]
fn log_record_text_format_with_logger_context() {
    let (logger, stdout, _stderr) = make_logger_with_capture(LogFormat::Text);

    let record =
        LogRecord::new(ts(), Severity::Info, "login ok".to_string(), vec![]).expect("valid record");

    let mut ctx = LogContext::new();
    ctx.add_attribute(
        LogAttribute::new(
            "logger".to_string(),
            LogAttributeValue::String("auth".to_string()),
        )
        .unwrap(),
    )
    .unwrap();

    logger
        .log_record(&record, Some(&ctx))
        .expect("log_record should succeed");

    let out = stdout.contents();
    assert!(
        out.contains("[INFO] auth: login ok"),
        "Text output should match [INFO] auth: login ok. Got: {out:?}"
    );
}

/// Scenario: HumanReadable format — record with attrs writes ts + level + message.
#[test]
fn log_record_human_readable_format_basic() {
    let (logger, stdout, _stderr) = make_logger_with_capture(LogFormat::HumanReadable);

    let attrs = vec![LogAttribute::new(
        "service".to_string(),
        LogAttributeValue::String("api".to_string()),
    )
    .unwrap()];
    let record =
        LogRecord::new(ts(), Severity::Info, "login ok".to_string(), attrs).expect("valid record");

    logger
        .log_record(&record, None)
        .expect("log_record should succeed");

    let out = stdout.contents();
    assert!(
        out.contains("INFO"),
        "HumanReadable output should contain INFO. Got: {out:?}"
    );
    assert!(
        out.contains("login ok"),
        "HumanReadable output should contain message. Got: {out:?}"
    );
    assert!(
        out.contains("service=api"),
        "HumanReadable output should contain attrs. Got: {out:?}"
    );
}

/// Scenario: Logfmt format — produces key=value pairs.
/// Note: Warn severity routes to stderr by default mapping.
#[test]
fn log_record_logfmt_format_produces_kv_pairs() {
    let (logger, _stdout, stderr) = make_logger_with_capture(LogFormat::Logfmt);

    let record = LogRecord::new(ts(), Severity::Warn, "slow query".to_string(), vec![])
        .expect("valid record");

    logger
        .log_record(&record, None)
        .expect("log_record should succeed");

    let out = stderr.contents();
    assert!(
        out.contains("level=WARN"),
        "Logfmt output should contain level=WARN. Got: {out:?}"
    );
    assert!(
        out.contains(r#"msg="slow query""#),
        "Logfmt output should contain msg field. Got: {out:?}"
    );
}

/// Scenario: Error severity is routed to stderr.
#[test]
fn log_record_error_severity_goes_to_stderr() {
    let (logger, _stdout, stderr) = make_logger_with_capture(LogFormat::Text);

    let record = LogRecord::new(ts(), Severity::Error, "db failure".to_string(), vec![])
        .expect("valid record");

    logger
        .log_record(&record, None)
        .expect("log_record should succeed");

    let out = stderr.contents();
    assert!(
        out.contains("db failure"),
        "Error severity should route to stderr. Got: {out:?}"
    );
}

/// Scenario: `log()` back-compat method still works.
///
/// Since change 015-orchestration-fold, `log()` delegates into
/// `log_record`'s full pipeline (it is no longer a raw, formatter-free
/// passthrough) — this test only asserts the message still reaches the
/// output, not that it is unformatted.
#[test]
fn log_compat_method_still_works() {
    let (logger, stdout, _stderr) = make_logger_with_capture(LogFormat::Text);
    logger
        .log(Severity::Info, "raw passthrough")
        .expect("log should succeed");
    let out = stdout.contents();
    assert!(
        out.contains("raw passthrough"),
        "Back-compat log() should still write to output. Got: {out:?}"
    );
}
