//! Human-readable formatter — produces developer-friendly single-line log strings.
//!
//! Format: `<ts>  <LEVEL> [<logger>] <message>  key=val ...`

use crate::{
    attr_value_to_human, logger_name, rfc3339_utc, severity_label, FormatError, RecordFormatter,
};
use kitlogger_log_domain::{LogContext, LogRecord};

pub struct HumanReadableFormatter;

impl RecordFormatter for HumanReadableFormatter {
    fn format(
        &self,
        record: &LogRecord,
        context: Option<&LogContext>,
    ) -> Result<String, FormatError> {
        let ts = rfc3339_utc(*record.timestamp());
        let level = severity_label(record.severity());
        let msg = record.message();
        let logger = logger_name(context);

        // Core: ts  LEVEL [logger] message
        let prefix = match logger {
            Some(name) => format!("{}  {} [{}] {}", ts, level, name, msg),
            None => format!("{}  {} {}", ts, level, msg),
        };

        // Collect attribute pairs
        let mut parts: Vec<String> = Vec::new();
        for attr in record.attributes() {
            parts.push(format!(
                "{}={}",
                attr.name(),
                attr_value_to_human(attr.value())
            ));
        }
        if let Some(ctx) = context {
            for attr in ctx.attributes() {
                if attr.name() == "logger" {
                    continue;
                }
                parts.push(format!(
                    "{}={}",
                    attr.name(),
                    attr_value_to_human(attr.value())
                ));
            }
        }

        if parts.is_empty() {
            Ok(prefix)
        } else {
            Ok(format!("{}  {}", prefix, parts.join(" ")))
        }
    }
}

// ---------------------------------------------------------------------------
// Tests — Phase 3 RED → GREEN
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use kitlogger_log_domain::{LogAttribute, LogAttributeValue, LogContext, LogRecord, Severity};
    use std::time::{Duration, SystemTime};

    const EPOCH_2026: u64 = 1_781_949_600; // 2026-06-20T10:00:00Z

    fn make_record_with_attrs(
        severity: Severity,
        message: &str,
        attrs: Vec<LogAttribute>,
    ) -> LogRecord {
        LogRecord::new(
            SystemTime::UNIX_EPOCH + Duration::from_secs(EPOCH_2026),
            severity,
            message.to_string(),
            attrs,
        )
        .unwrap()
    }

    fn make_record(severity: Severity, message: &str) -> LogRecord {
        make_record_with_attrs(severity, message, vec![])
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

    // Scenario: Full record with logger — matches spec literal
    #[test]
    fn human_full_record_with_logger() {
        let record = make_record_with_attrs(
            Severity::Info,
            "login ok",
            vec![LogAttribute::new(
                "service".to_string(),
                LogAttributeValue::String("api".to_string()),
            )
            .unwrap()],
        );
        let ctx = make_ctx_with_logger("auth");
        let result = HumanReadableFormatter.format(&record, Some(&ctx)).unwrap();
        assert_eq!(
            result,
            "2026-06-20T10:00:00Z  INFO [auth] login ok  service=api"
        );
    }

    // Scenario: Record without context — no bracket
    #[test]
    fn human_record_without_context_no_bracket() {
        let record = make_record(Severity::Warn, "slow query");
        let result = HumanReadableFormatter.format(&record, None).unwrap();
        assert!(result.contains("WARN"));
        assert!(result.contains("slow query"));
        assert!(!result.contains('['));
    }

    // Scenario: Context without logger — no bracket
    #[test]
    fn human_context_without_logger_no_bracket() {
        let mut ctx = LogContext::new();
        ctx.add_attribute(
            LogAttribute::new(
                "env".to_string(),
                LogAttributeValue::String("prod".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
        let record = make_record(Severity::Info, "msg");
        let result = HumanReadableFormatter.format(&record, Some(&ctx)).unwrap();
        assert!(!result.contains('['));
        assert!(result.contains("env=prod"));
    }

    // Scenario: No attributes — no trailing spaces or separators
    #[test]
    fn human_no_attrs_no_trailing_content() {
        let record = make_record(Severity::Info, "hello");
        let result = HumanReadableFormatter.format(&record, None).unwrap();
        // Should end with "hello" and nothing else
        assert!(result.ends_with("hello"));
    }

    // Scenario: Array attribute renders as inline JSON
    #[test]
    fn human_array_attr_renders_inline_json() {
        let arr = LogAttributeValue::array(vec![
            LogAttributeValue::String("api".to_string()),
            LogAttributeValue::String("auth".to_string()),
        ])
        .unwrap();
        let record = make_record_with_attrs(
            Severity::Info,
            "msg",
            vec![LogAttribute::new("tags".to_string(), arr).unwrap()],
        );
        let result = HumanReadableFormatter.format(&record, None).unwrap();
        assert!(result.contains(r#"tags=["api","auth"]"#));
    }

    // Scenario: NaN Float renders as literal "NaN"
    #[test]
    fn human_nan_float_renders_as_string() {
        let record = make_record_with_attrs(
            Severity::Info,
            "msg",
            vec![LogAttribute::new("val".to_string(), LogAttributeValue::Float(f64::NAN)).unwrap()],
        );
        let result = HumanReadableFormatter.format(&record, None).unwrap();
        assert!(result.contains("val=NaN"));
    }

    // Scenario: Inf Float renders as literal "Inf"
    #[test]
    fn human_inf_float_renders_as_string() {
        let record = make_record_with_attrs(
            Severity::Info,
            "msg",
            vec![
                LogAttribute::new("val".to_string(), LogAttributeValue::Float(f64::INFINITY))
                    .unwrap(),
            ],
        );
        let result = HumanReadableFormatter.format(&record, None).unwrap();
        assert!(result.contains("val=Inf"));
    }
}
