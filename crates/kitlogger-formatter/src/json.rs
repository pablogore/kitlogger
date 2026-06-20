//! JSON formatter — serializes log records to a single-line JSON object.
//!
//! Field order is preserved: ts, level, msg, logger (if any), record attrs, context attrs.
//! We build an ordered `Vec<(key, Value)>` and serialize it manually to avoid
//! `BTreeMap`-based alphabetical reordering from `serde_json::Map`.

use serde_json::Value;

use crate::{logger_name, rfc3339_utc, severity_label, FormatError, RecordFormatter};
use kitlogger_log_domain::{LogAttributeValue, LogContext, LogRecord};

pub struct JsonFormatter;

impl RecordFormatter for JsonFormatter {
    fn format(
        &self,
        record: &LogRecord,
        context: Option<&LogContext>,
    ) -> Result<String, FormatError> {
        let mut pairs: Vec<(String, Value)> = Vec::new();

        // Fixed fields in spec order
        pairs.push((
            "ts".to_string(),
            Value::String(rfc3339_utc(*record.timestamp())),
        ));
        pairs.push((
            "level".to_string(),
            Value::String(severity_label(record.severity()).to_string()),
        ));
        pairs.push((
            "msg".to_string(),
            Value::String(record.message().to_string()),
        ));

        // Optional logger field
        if let Some(name) = logger_name(context) {
            pairs.push(("logger".to_string(), Value::String(name.to_string())));
        }

        // Record attributes
        for attr in record.attributes() {
            let v = attr_value_to_json(attr.value())?;
            pairs.push((attr.name().to_string(), v));
        }

        // Context attributes (excluding "logger")
        if let Some(ctx) = context {
            for attr in ctx.attributes() {
                if attr.name() == "logger" {
                    continue;
                }
                let v = attr_value_to_json(attr.value())?;
                pairs.push((attr.name().to_string(), v));
            }
        }

        serialize_ordered_object(&pairs)
    }
}

/// Serializes an ordered list of `(key, Value)` pairs as a JSON object string.
///
/// Preserves insertion order — avoids `BTreeMap` reordering from `serde_json::Map`.
fn serialize_ordered_object(pairs: &[(String, Value)]) -> Result<String, FormatError> {
    let mut out = String::from("{");
    for (i, (k, v)) in pairs.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let key_json =
            serde_json::to_string(k).map_err(|e| FormatError::SerializationError(e.to_string()))?;
        let val_json =
            serde_json::to_string(v).map_err(|e| FormatError::SerializationError(e.to_string()))?;
        out.push_str(&key_json);
        out.push(':');
        out.push_str(&val_json);
    }
    out.push('}');
    Ok(out)
}

fn attr_value_to_json(v: &LogAttributeValue) -> Result<Value, FormatError> {
    match v {
        LogAttributeValue::String(s) => Ok(Value::String(s.clone())),
        LogAttributeValue::Integer(n) => Ok(Value::Number((*n).into())),
        LogAttributeValue::Float(f) => {
            if f.is_nan() || f.is_infinite() {
                return Err(FormatError::RenderError(format!(
                    "non-finite float value: {}",
                    f
                )));
            }
            let n = serde_json::Number::from_f64(*f).ok_or_else(|| {
                FormatError::RenderError(format!("cannot serialize float: {}", f))
            })?;
            Ok(Value::Number(n))
        }
        LogAttributeValue::Boolean(b) => Ok(Value::Bool(*b)),
        LogAttributeValue::Timestamp(t) => Ok(Value::String(rfc3339_utc(*t))),
        LogAttributeValue::Array(arr) => {
            let items: Result<Vec<Value>, FormatError> =
                arr.iter().map(attr_value_to_json).collect();
            Ok(Value::Array(items?))
        }
    }
}

// ---------------------------------------------------------------------------
// Tests — Phase 2 RED → GREEN
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use kitlogger_log_domain::{LogAttribute, LogAttributeValue, LogContext, LogRecord, Severity};
    use std::time::{Duration, SystemTime};

    // 2026-06-20T10:00:00Z
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

    // Scenario: Full record with logger in context — exact spec literal
    #[test]
    fn json_full_record_with_logger() {
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
        let result = JsonFormatter.format(&record, Some(&ctx)).unwrap();
        assert_eq!(
            result,
            r#"{"ts":"2026-06-20T10:00:00Z","level":"INFO","msg":"login ok","logger":"auth","service":"api"}"#
        );
    }

    // Scenario: Record without context — no "logger" key
    #[test]
    fn json_record_without_context_no_logger_key() {
        let record = make_record(Severity::Warn, "slow query");
        let result = JsonFormatter.format(&record, None).unwrap();
        assert!(result.contains(r#""level":"WARN""#));
        assert!(result.contains(r#""msg":"slow query""#));
        assert!(!result.contains(r#""logger""#));
    }

    // Scenario: Context with no logger attribute — no "logger" key, env present
    #[test]
    fn json_context_without_logger_key_absent() {
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
        let result = JsonFormatter.format(&record, Some(&ctx)).unwrap();
        assert!(!result.contains(r#""logger""#));
        assert!(result.contains(r#""env":"prod""#));
    }

    // Scenario: Boolean and integer attributes produce native JSON types
    #[test]
    fn json_boolean_and_integer_native_types() {
        let record = make_record_with_attrs(
            Severity::Info,
            "msg",
            vec![
                LogAttribute::new("retries".to_string(), LogAttributeValue::Integer(3)).unwrap(),
                LogAttribute::new("cached".to_string(), LogAttributeValue::Boolean(false)).unwrap(),
            ],
        );
        let result = JsonFormatter.format(&record, None).unwrap();
        assert!(result.contains(r#""retries":3"#));
        assert!(result.contains(r#""cached":false"#));
    }

    // Scenario: Timestamp attribute renders as RFC3339 string
    #[test]
    fn json_timestamp_attr_renders_as_rfc3339() {
        let ts = SystemTime::UNIX_EPOCH + Duration::from_secs(1_735_689_600); // 2025-01-01T00:00:00Z
        let record = make_record_with_attrs(
            Severity::Info,
            "msg",
            vec![
                LogAttribute::new("started".to_string(), LogAttributeValue::Timestamp(ts)).unwrap(),
            ],
        );
        let result = JsonFormatter.format(&record, None).unwrap();
        assert!(result.contains(r#""started":"2025-01-01T00:00:00Z""#));
    }

    // Scenario: Array attribute produces JSON array
    #[test]
    fn json_array_attr_produces_json_array() {
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
        let result = JsonFormatter.format(&record, None).unwrap();
        assert!(result.contains(r#""tags":["api","auth"]"#));
    }

    // Scenario: NaN Float returns Err(FormatError)
    #[test]
    fn json_nan_float_returns_error() {
        let record = make_record_with_attrs(
            Severity::Info,
            "msg",
            vec![LogAttribute::new("val".to_string(), LogAttributeValue::Float(f64::NAN)).unwrap()],
        );
        let result = JsonFormatter.format(&record, None);
        assert!(result.is_err());
    }

    // Scenario: Inf Float returns Err(FormatError)
    #[test]
    fn json_inf_float_returns_error() {
        let record = make_record_with_attrs(
            Severity::Info,
            "msg",
            vec![
                LogAttribute::new("val".to_string(), LogAttributeValue::Float(f64::INFINITY))
                    .unwrap(),
            ],
        );
        let result = JsonFormatter.format(&record, None);
        assert!(result.is_err());
    }

    // Triangulation: severity Fatal renders as "FATAL"
    #[test]
    fn json_fatal_severity_renders_as_fatal() {
        let record = make_record(Severity::Fatal, "crash");
        let result = JsonFormatter.format(&record, None).unwrap();
        assert!(result.contains(r#""level":"FATAL""#));
    }
}
