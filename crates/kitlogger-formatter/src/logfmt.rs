//! Logfmt formatter — produces `key=value` pairs on a single line.
//!
//! Field order: ts, level, msg, logger (if any), record attrs, context attrs (excl. logger).

use crate::{logger_name, rfc3339_utc, severity_label, FormatError, RecordFormatter};
use kitlogger_log_domain::{LogAttributeValue, LogContext, LogRecord};

pub struct LogfmtFormatter;

impl RecordFormatter for LogfmtFormatter {
    fn format(
        &self,
        record: &LogRecord,
        context: Option<&LogContext>,
    ) -> Result<String, FormatError> {
        let mut pairs: Vec<String> = Vec::new();

        pairs.push(format!("ts={}", rfc3339_utc(*record.timestamp())));
        pairs.push(format!("level={}", severity_label(record.severity())));
        pairs.push(format!("msg={}", logfmt_quote(record.message())));

        if let Some(name) = logger_name(context) {
            pairs.push(format!("logger={}", logfmt_quote(name)));
        }

        for attr in record.attributes() {
            pairs.push(format!(
                "{}={}",
                attr.name(),
                attr_value_to_logfmt(attr.value())?
            ));
        }

        if let Some(ctx) = context {
            for attr in ctx.attributes() {
                if attr.name() == "logger" {
                    continue;
                }
                pairs.push(format!(
                    "{}={}",
                    attr.name(),
                    attr_value_to_logfmt(attr.value())?
                ));
            }
        }

        Ok(pairs.join(" "))
    }
}

/// Returns the logfmt-quoted representation of `s`.
///
/// Rules:
/// - If `s` contains space, `=`, or `"` → double-quote and escape inner `"` as `\"`
/// - Otherwise → bare (no quotes)
fn logfmt_quote(s: &str) -> String {
    if s.contains(' ') || s.contains('=') || s.contains('"') {
        let escaped = s.replace('"', "\\\"");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}

/// Renders a [`LogAttributeValue`] as a logfmt token.
fn attr_value_to_logfmt(v: &LogAttributeValue) -> Result<String, FormatError> {
    match v {
        LogAttributeValue::String(s) => Ok(logfmt_quote(s)),
        LogAttributeValue::Integer(n) => Ok(n.to_string()),
        LogAttributeValue::Float(f) => {
            if f.is_nan() {
                Ok("NaN".to_string())
            } else if f.is_infinite() {
                Ok("Inf".to_string())
            } else {
                Ok(f.to_string())
            }
        }
        LogAttributeValue::Boolean(b) => Ok(b.to_string()),
        LogAttributeValue::Timestamp(t) => Ok(rfc3339_utc(*t)),
        LogAttributeValue::Array(arr) => {
            // Serialize to inline JSON via serde_json.
            let json_arr: Vec<serde_json::Value> = arr
                .iter()
                .map(array_item_to_json)
                .collect::<Result<Vec<_>, _>>()?;
            serde_json::to_string(&json_arr)
                .map_err(|e| FormatError::SerializationError(e.to_string()))
        }
    }
}

/// Converts an array item to a `serde_json::Value` for inline JSON serialization.
fn array_item_to_json(v: &LogAttributeValue) -> Result<serde_json::Value, FormatError> {
    match v {
        LogAttributeValue::String(s) => Ok(serde_json::Value::String(s.clone())),
        LogAttributeValue::Integer(n) => Ok(serde_json::Value::Number((*n).into())),
        LogAttributeValue::Float(f) => {
            if f.is_nan() || f.is_infinite() {
                // Logfmt array with NaN/Inf: fallback to string representation
                Ok(serde_json::Value::String(if f.is_nan() {
                    "NaN".to_string()
                } else {
                    "Inf".to_string()
                }))
            } else {
                let n = serde_json::Number::from_f64(*f).ok_or_else(|| {
                    FormatError::RenderError(format!("cannot serialize float: {}", f))
                })?;
                Ok(serde_json::Value::Number(n))
            }
        }
        LogAttributeValue::Boolean(b) => Ok(serde_json::Value::Bool(*b)),
        LogAttributeValue::Timestamp(t) => Ok(serde_json::Value::String(rfc3339_utc(*t))),
        LogAttributeValue::Array(inner) => {
            // Nested arrays: recurse
            let items: Result<Vec<serde_json::Value>, _> =
                inner.iter().map(array_item_to_json).collect();
            Ok(serde_json::Value::Array(items?))
        }
    }
}

// ---------------------------------------------------------------------------
// Tests — Phase 5 RED → GREEN
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
    fn logfmt_full_record_with_logger() {
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
        let result = LogfmtFormatter.format(&record, Some(&ctx)).unwrap();
        assert_eq!(
            result,
            r#"ts=2026-06-20T10:00:00Z level=INFO msg="login ok" logger=auth service=api"#
        );
    }

    // Scenario: Message with spaces is quoted
    #[test]
    fn logfmt_message_with_spaces_is_quoted() {
        let record = make_record(Severity::Info, "user logged in");
        let result = LogfmtFormatter.format(&record, None).unwrap();
        assert!(result.contains(r#"msg="user logged in""#));
    }

    // Scenario: Record without context — no logger field
    #[test]
    fn logfmt_record_without_context() {
        let record = make_record(Severity::Warn, "retry");
        let result = LogfmtFormatter.format(&record, None).unwrap();
        assert!(result.starts_with("ts="));
        assert!(result.contains("level=WARN"));
        assert!(result.contains("msg=retry"));
        assert!(!result.contains("logger="));
    }

    // Scenario: Value with equals sign is quoted
    #[test]
    fn logfmt_value_with_equals_is_quoted() {
        let record = make_record_with_attrs(
            Severity::Info,
            "msg",
            vec![LogAttribute::new(
                "kv".to_string(),
                LogAttributeValue::String("k=v".to_string()),
            )
            .unwrap()],
        );
        let result = LogfmtFormatter.format(&record, None).unwrap();
        assert!(result.contains(r#"kv="k=v""#));
    }

    // Scenario: Value with embedded quote is escaped
    #[test]
    fn logfmt_value_with_embedded_quote_is_escaped() {
        let record = make_record_with_attrs(
            Severity::Info,
            "msg",
            vec![LogAttribute::new(
                "greeting".to_string(),
                LogAttributeValue::String(r#"say "hello""#.to_string()),
            )
            .unwrap()],
        );
        let result = LogfmtFormatter.format(&record, None).unwrap();
        assert!(result.contains(r#"greeting="say \"hello\"""#));
    }

    // Scenario: Simple value is bare (no quotes)
    #[test]
    fn logfmt_simple_value_is_bare() {
        let record = make_record_with_attrs(
            Severity::Info,
            "msg",
            vec![LogAttribute::new(
                "env".to_string(),
                LogAttributeValue::String("prod".to_string()),
            )
            .unwrap()],
        );
        let result = LogfmtFormatter.format(&record, None).unwrap();
        assert!(result.contains("env=prod"));
        assert!(!result.contains("env=\"prod\""));
    }

    // Scenario: String array renders as inline JSON
    #[test]
    fn logfmt_string_array_renders_as_inline_json() {
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
        let result = LogfmtFormatter.format(&record, None).unwrap();
        assert!(result.contains(r#"tags=["api","auth"]"#));
    }

    // Scenario: Integer array renders as inline JSON numbers
    #[test]
    fn logfmt_integer_array_renders_as_inline_json() {
        let arr = LogAttributeValue::array(vec![
            LogAttributeValue::Integer(200),
            LogAttributeValue::Integer(201),
            LogAttributeValue::Integer(204),
        ])
        .unwrap();
        let record = make_record_with_attrs(
            Severity::Info,
            "msg",
            vec![LogAttribute::new("codes".to_string(), arr).unwrap()],
        );
        let result = LogfmtFormatter.format(&record, None).unwrap();
        assert!(result.contains("codes=[200,201,204]"));
    }

    // Scenario: Timestamp attribute renders as RFC3339
    #[test]
    fn logfmt_timestamp_attr_renders_as_rfc3339() {
        let ts = SystemTime::UNIX_EPOCH + Duration::from_secs(1_735_689_600); // 2025-01-01T00:00:00Z
        let record = make_record_with_attrs(
            Severity::Info,
            "msg",
            vec![
                LogAttribute::new("started".to_string(), LogAttributeValue::Timestamp(ts)).unwrap(),
            ],
        );
        let result = LogfmtFormatter.format(&record, None).unwrap();
        assert!(result.contains("started=2025-01-01T00:00:00Z"));
    }

    // Scenario: All record and context attributes are present
    #[test]
    fn logfmt_all_attrs_present() {
        let record = make_record_with_attrs(
            Severity::Info,
            "msg",
            vec![
                LogAttribute::new("a".to_string(), LogAttributeValue::Integer(1)).unwrap(),
                LogAttribute::new("b".to_string(), LogAttributeValue::Integer(2)).unwrap(),
            ],
        );
        let mut ctx = LogContext::new();
        ctx.add_attribute(
            LogAttribute::new(
                "logger".to_string(),
                LogAttributeValue::String("svc".to_string()),
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
        ctx.add_attribute(
            LogAttribute::new(
                "region".to_string(),
                LogAttributeValue::String("us-east-1".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
        let result = LogfmtFormatter.format(&record, Some(&ctx)).unwrap();
        assert!(result.contains("a=1"));
        assert!(result.contains("b=2"));
        assert!(result.contains("env=prod"));
        assert!(result.contains("region=us-east-1"));
        assert!(result.contains("logger=svc"));
    }

    // Scenario: NaN/Inf Float renders as string literals
    #[test]
    fn logfmt_nan_renders_as_string() {
        let record = make_record_with_attrs(
            Severity::Info,
            "msg",
            vec![LogAttribute::new("val".to_string(), LogAttributeValue::Float(f64::NAN)).unwrap()],
        );
        let result = LogfmtFormatter.format(&record, None).unwrap();
        assert!(result.contains("val=NaN"));
    }

    #[test]
    fn logfmt_inf_renders_as_string() {
        let record = make_record_with_attrs(
            Severity::Info,
            "msg",
            vec![
                LogAttribute::new("val".to_string(), LogAttributeValue::Float(f64::INFINITY))
                    .unwrap(),
            ],
        );
        let result = LogfmtFormatter.format(&record, None).unwrap();
        assert!(result.contains("val=Inf"));
    }
}
