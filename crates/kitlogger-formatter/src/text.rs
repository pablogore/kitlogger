//! Text formatter — produces compact `[LEVEL] logger: message` strings.

use crate::{logger_name, severity_label, FormatError, RecordFormatter};
use kitlogger_log_domain::{LogContext, LogRecord};

pub struct TextFormatter;

impl RecordFormatter for TextFormatter {
    fn format(
        &self,
        record: &LogRecord,
        context: Option<&LogContext>,
    ) -> Result<String, FormatError> {
        let level = severity_label(record.severity());
        let msg = record.message();

        let output = match logger_name(context) {
            Some(name) => format!("[{}] {}: {}", level, name, msg),
            None => format!("[{}] {}", level, msg),
        };

        Ok(output)
    }
}

// ---------------------------------------------------------------------------
// Tests — Phase 4 RED → GREEN
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use kitlogger_log_domain::{LogAttribute, LogAttributeValue, LogContext, LogRecord, Severity};
    use std::time::{Duration, SystemTime};

    fn make_record(severity: Severity, message: &str) -> LogRecord {
        LogRecord::new(
            SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000),
            severity,
            message.to_string(),
            vec![],
        )
        .unwrap()
    }

    fn make_record_with_attrs(
        severity: Severity,
        message: &str,
        attrs: Vec<LogAttribute>,
    ) -> LogRecord {
        LogRecord::new(
            SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000),
            severity,
            message.to_string(),
            attrs,
        )
        .unwrap()
    }

    fn make_ctx_with_logger(name: &str) -> LogContext {
        let mut ctx = LogContext::new();
        ctx.add_attribute(
            LogAttribute::new(
                "logger".to_string(),
                LogAttributeValue::String(name.to_string()),
            )
            .unwrap(),
        )
        .unwrap();
        ctx
    }

    // Scenario: logger present
    #[test]
    fn text_with_logger() {
        let record = make_record(Severity::Info, "login ok");
        let ctx = make_ctx_with_logger("auth");
        let result = TextFormatter.format(&record, Some(&ctx)).unwrap();
        assert_eq!(result, "[INFO] auth: login ok");
    }

    // Scenario: no context
    #[test]
    fn text_without_context() {
        let record = make_record(Severity::Warn, "slow query");
        let result = TextFormatter.format(&record, None).unwrap();
        assert_eq!(result, "[WARN] slow query");
    }

    // Scenario: context without logger key — no colon prefix
    #[test]
    fn text_context_without_logger() {
        let mut ctx = LogContext::new();
        ctx.add_attribute(
            LogAttribute::new(
                "env".to_string(),
                LogAttributeValue::String("prod".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
        let record = make_record(Severity::Info, "message");
        let result = TextFormatter.format(&record, Some(&ctx)).unwrap();
        assert_eq!(result, "[INFO] message");
    }

    // Scenario: attrs are NOT present in output
    #[test]
    fn text_attrs_not_in_output() {
        let record = make_record_with_attrs(
            Severity::Info,
            "msg",
            vec![
                LogAttribute::new(
                    "service".to_string(),
                    LogAttributeValue::String("api".to_string()),
                )
                .unwrap(),
                LogAttribute::new("retries".to_string(), LogAttributeValue::Integer(3)).unwrap(),
            ],
        );
        let mut ctx = LogContext::new();
        ctx.add_attribute(
            LogAttribute::new(
                "logger".to_string(),
                LogAttributeValue::String("auth".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
        ctx.add_attribute(
            LogAttribute::new(
                "env".to_string(),
                LogAttributeValue::String("prod".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
        let result = TextFormatter.format(&record, Some(&ctx)).unwrap();
        assert_eq!(result, "[INFO] auth: msg");
        assert!(!result.contains("service"));
        assert!(!result.contains("retries"));
        assert!(!result.contains("env"));
    }

    // Scenario: all six severity variants
    #[test]
    fn text_trace_severity() {
        let r = make_record(Severity::Trace, "t");
        assert_eq!(TextFormatter.format(&r, None).unwrap(), "[TRACE] t");
    }

    #[test]
    fn text_debug_severity() {
        let r = make_record(Severity::Debug, "t");
        assert_eq!(TextFormatter.format(&r, None).unwrap(), "[DEBUG] t");
    }

    #[test]
    fn text_info_severity() {
        let r = make_record(Severity::Info, "t");
        assert_eq!(TextFormatter.format(&r, None).unwrap(), "[INFO] t");
    }

    #[test]
    fn text_warn_severity() {
        let r = make_record(Severity::Warn, "t");
        assert_eq!(TextFormatter.format(&r, None).unwrap(), "[WARN] t");
    }

    #[test]
    fn text_error_severity() {
        let r = make_record(Severity::Error, "t");
        assert_eq!(TextFormatter.format(&r, None).unwrap(), "[ERROR] t");
    }

    #[test]
    fn text_fatal_severity() {
        let r = make_record(Severity::Fatal, "t");
        assert_eq!(TextFormatter.format(&r, None).unwrap(), "[FATAL] t");
    }
}
