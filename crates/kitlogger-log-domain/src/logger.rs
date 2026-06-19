//! Logger trait — canonical domain contract for emitting structured log records.

use crate::{EmitError, LogAttribute, Severity};

/// A named, thread-safe emitter of structured log records.
///
/// Implementations are transport, exporter, and storage agnostic. `Logger` is
/// object-safe: all methods take concrete types (`&str`, `&[LogAttribute]`) —
/// never `impl Into<T>` or generic parameters — so `dyn Logger` compiles.
///
/// The trait is `Send + Sync` so `Arc<dyn Logger>` can be shared across threads.
pub trait Logger: Send + Sync {
    /// Returns the name of this logger.
    fn name(&self) -> &str;

    /// Emits a log record with the given severity, message, and attributes.
    ///
    /// # Errors
    ///
    /// Returns `EmitError::Validation` if the message is empty or the record
    /// fails domain validation. Returns `EmitError::LoggerClosed` if emit is
    /// attempted after shutdown.
    fn log(
        &self,
        severity: Severity,
        message: &str,
        attributes: &[LogAttribute],
    ) -> Result<(), EmitError>;

    /// Emits a `Trace`-severity record.
    fn trace(&self, message: &str, attributes: &[LogAttribute]) -> Result<(), EmitError> {
        self.log(Severity::Trace, message, attributes)
    }

    /// Emits a `Debug`-severity record.
    fn debug(&self, message: &str, attributes: &[LogAttribute]) -> Result<(), EmitError> {
        self.log(Severity::Debug, message, attributes)
    }

    /// Emits an `Info`-severity record.
    fn info(&self, message: &str, attributes: &[LogAttribute]) -> Result<(), EmitError> {
        self.log(Severity::Info, message, attributes)
    }

    /// Emits a `Warn`-severity record.
    fn warn(&self, message: &str, attributes: &[LogAttribute]) -> Result<(), EmitError> {
        self.log(Severity::Warn, message, attributes)
    }

    /// Emits an `Error`-severity record.
    fn error(&self, message: &str, attributes: &[LogAttribute]) -> Result<(), EmitError> {
        self.log(Severity::Error, message, attributes)
    }

    /// Emits a `Fatal`-severity record.
    fn fatal(&self, message: &str, attributes: &[LogAttribute]) -> Result<(), EmitError> {
        self.log(Severity::Fatal, message, attributes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LogRecord, ValidationError};
    use std::sync::{Arc, Mutex};
    use std::time::SystemTime;

    // ── Object-safety smoke tests ─────────────────────────────────────────────

    #[test]
    fn logger_is_object_safe_box() {
        // This test exists purely as a compile-time check.
        // If Logger were NOT object-safe, this line would not compile.
        let _: Option<Box<dyn Logger>> = None;
    }

    #[test]
    fn logger_is_object_safe_arc() {
        let _: Option<Arc<dyn Logger>> = None;
    }

    // ── Mock implementation ────────────────────────────────────────────────────

    /// Mock logger that records (Severity, String) tuples for verification.
    struct MockLogger {
        name: String,
        calls: Mutex<Vec<(Severity, String)>>,
    }

    impl MockLogger {
        fn new(name: &str) -> Self {
            MockLogger {
                name: name.to_string(),
                calls: Mutex::new(Vec::new()),
            }
        }

        fn recorded_calls(&self) -> Vec<(Severity, String)> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl Logger for MockLogger {
        fn name(&self) -> &str {
            &self.name
        }

        fn log(
            &self,
            severity: Severity,
            message: &str,
            _attributes: &[LogAttribute],
        ) -> Result<(), EmitError> {
            if message.is_empty() {
                return Err(EmitError::Validation(ValidationError::EmptyMessage));
            }
            self.calls
                .lock()
                .unwrap()
                .push((severity, message.to_string()));
            Ok(())
        }
    }

    #[test]
    fn name_returns_logger_name() {
        let logger = MockLogger::new("my-logger");
        assert_eq!(logger.name(), "my-logger");
    }

    // ── Convenience methods delegate to log with correct Severity ──────────────

    #[test]
    fn trace_calls_log_with_trace_severity() {
        let logger = MockLogger::new("l");
        logger.trace("msg", &[]).unwrap();
        assert_eq!(
            logger.recorded_calls(),
            vec![(Severity::Trace, "msg".to_string())]
        );
    }

    #[test]
    fn debug_calls_log_with_debug_severity() {
        let logger = MockLogger::new("l");
        logger.debug("msg", &[]).unwrap();
        assert_eq!(
            logger.recorded_calls(),
            vec![(Severity::Debug, "msg".to_string())]
        );
    }

    #[test]
    fn info_calls_log_with_info_severity() {
        let logger = MockLogger::new("l");
        logger.info("msg", &[]).unwrap();
        assert_eq!(
            logger.recorded_calls(),
            vec![(Severity::Info, "msg".to_string())]
        );
    }

    #[test]
    fn warn_calls_log_with_warn_severity() {
        let logger = MockLogger::new("l");
        logger.warn("msg", &[]).unwrap();
        assert_eq!(
            logger.recorded_calls(),
            vec![(Severity::Warn, "msg".to_string())]
        );
    }

    #[test]
    fn error_calls_log_with_error_severity() {
        let logger = MockLogger::new("l");
        logger.error("msg", &[]).unwrap();
        assert_eq!(
            logger.recorded_calls(),
            vec![(Severity::Error, "msg".to_string())]
        );
    }

    #[test]
    fn fatal_calls_log_with_fatal_severity() {
        let logger = MockLogger::new("l");
        logger.fatal("msg", &[]).unwrap();
        assert_eq!(
            logger.recorded_calls(),
            vec![(Severity::Fatal, "msg".to_string())]
        );
    }

    // ── Empty message → EmitError::Validation(ValidationError::EmptyMessage) ──

    /// Mock logger that delegates to LogRecord::new to exercise real validation.
    struct ValidatingLogger {
        name: String,
    }

    impl Logger for ValidatingLogger {
        fn name(&self) -> &str {
            &self.name
        }

        fn log(
            &self,
            severity: Severity,
            message: &str,
            attributes: &[LogAttribute],
        ) -> Result<(), EmitError> {
            LogRecord::new(
                SystemTime::now(),
                severity,
                message.to_string(),
                attributes.to_vec(),
            )
            .map(|_| ())
            .map_err(EmitError::from)
        }
    }

    #[test]
    fn empty_message_returns_emit_error_validation_empty_message() {
        let logger = ValidatingLogger {
            name: "validator".to_string(),
        };
        let result = logger.log(Severity::Info, "", &[]);
        assert_eq!(
            result,
            Err(EmitError::Validation(ValidationError::EmptyMessage))
        );
    }
}
