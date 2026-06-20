//! `kitlogger-formatter` — stateless log record formatters for KitLogger.
//!
//! Exposes the `RecordFormatter` trait, `LogFormat` enum, `formatter_from_config`
//! factory, and four concrete implementations:
//! - [`JsonFormatter`]
//! - [`HumanReadableFormatter`]
//! - [`TextFormatter`]
//! - [`LogfmtFormatter`]

mod error;
pub mod human;
pub mod json;
pub mod logfmt;
pub mod text;

pub use error::FormatError;
pub use human::HumanReadableFormatter;
pub use json::JsonFormatter;
pub use logfmt::LogfmtFormatter;
pub use text::TextFormatter;

use kitlogger_log_domain::{LogContext, LogRecord, Severity};
use std::time::SystemTime;

// ---------------------------------------------------------------------------
// Public trait
// ---------------------------------------------------------------------------

/// Stateless formatter that converts a [`LogRecord`] into a `String`.
///
/// All implementations MUST be deterministic: the same inputs MUST always
/// produce the same output. The trait is object-safe so callers can store
/// `Box<dyn RecordFormatter>`.
pub trait RecordFormatter: Send + Sync {
    /// Format `record` (with optional `context`) into a string.
    ///
    /// # Errors
    ///
    /// Returns [`FormatError`] when a value cannot be serialized or rendered.
    /// MUST NOT panic.
    fn format(
        &self,
        record: &LogRecord,
        context: Option<&LogContext>,
    ) -> Result<String, FormatError>;
}

// ---------------------------------------------------------------------------
// LogFormat enum
// ---------------------------------------------------------------------------

/// Selects which concrete formatter to use.
#[derive(Clone, Debug, PartialEq)]
pub enum LogFormat {
    /// JSON object output (single line, no trailing newline).
    Json,
    /// Human-readable output for developer consoles.
    HumanReadable,
    /// Compact `[LEVEL] logger: message` output.
    Text,
    /// Logfmt `key=value` pairs.
    Logfmt,
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

/// Returns the formatter that corresponds to `format`.
pub fn formatter_from_config(format: LogFormat) -> Box<dyn RecordFormatter> {
    match format {
        LogFormat::Json => Box::new(JsonFormatter),
        LogFormat::HumanReadable => Box::new(HumanReadableFormatter),
        LogFormat::Text => Box::new(TextFormatter),
        LogFormat::Logfmt => Box::new(LogfmtFormatter),
    }
}

// ---------------------------------------------------------------------------
// Shared private helpers
// ---------------------------------------------------------------------------

/// Maps a [`Severity`] to its uppercase ASCII label.
///
/// `Severity::Display` yields title-case; we map explicitly to uppercase.
pub(crate) fn severity_label(s: &Severity) -> &'static str {
    match s {
        Severity::Trace => "TRACE",
        Severity::Debug => "DEBUG",
        Severity::Info => "INFO",
        Severity::Warn => "WARN",
        Severity::Error => "ERROR",
        Severity::Fatal => "FATAL",
    }
}

/// Formats a [`SystemTime`] as an RFC3339 UTC string with seconds precision.
///
/// Output format: `YYYY-MM-DDTHH:MM:SSZ`
///
/// Implemented without `chrono` via epoch arithmetic only.
pub(crate) fn rfc3339_utc(t: SystemTime) -> String {
    // Seconds since UNIX epoch (1970-01-01T00:00:00Z).
    let secs = t
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Decompose into calendar fields (Gregorian proleptic calendar, UTC).
    let (year, month, day, hour, minute, second) = epoch_secs_to_datetime(secs);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, minute, second
    )
}

/// Renders a [`LogAttributeValue`] as a human-readable string.
///
/// - Strings are bare (no quotes).
/// - Arrays render as inline JSON.
/// - NaN/Inf floats render as literal `"NaN"` / `"Inf"`.
pub(crate) fn attr_value_to_human(v: &kitlogger_log_domain::LogAttributeValue) -> String {
    use kitlogger_log_domain::LogAttributeValue;
    match v {
        LogAttributeValue::String(s) => s.clone(),
        LogAttributeValue::Integer(n) => n.to_string(),
        LogAttributeValue::Float(f) => {
            if f.is_nan() {
                "NaN".to_string()
            } else if f.is_infinite() {
                "Inf".to_string()
            } else {
                f.to_string()
            }
        }
        LogAttributeValue::Boolean(b) => b.to_string(),
        LogAttributeValue::Timestamp(t) => rfc3339_utc(*t),
        LogAttributeValue::Array(arr) => {
            // Best-effort inline JSON; silently use debug repr on failure.
            let items: Vec<serde_json::Value> = arr
                .iter()
                .map(|item| match item {
                    LogAttributeValue::String(s) => serde_json::Value::String(s.clone()),
                    LogAttributeValue::Integer(n) => serde_json::Value::Number((*n).into()),
                    LogAttributeValue::Boolean(b) => serde_json::Value::Bool(*b),
                    LogAttributeValue::Float(f) => serde_json::Number::from_f64(*f)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::String(f.to_string())),
                    LogAttributeValue::Timestamp(t) => serde_json::Value::String(rfc3339_utc(*t)),
                    LogAttributeValue::Array(_) => serde_json::Value::String(format!("{:?}", item)),
                })
                .collect();
            serde_json::to_string(&items).unwrap_or_else(|_| format!("{:?}", arr))
        }
    }
}

/// Extracts the logger name from `LogContext.attributes` by looking for an
/// attribute named `"logger"` whose value is a `String`.
///
/// Returns `None` when the context is absent or no such attribute exists.
pub(crate) fn logger_name(ctx: Option<&LogContext>) -> Option<&str> {
    use kitlogger_log_domain::LogAttributeValue;
    ctx?.attributes().iter().find_map(|attr| {
        if attr.name() == "logger" {
            if let LogAttributeValue::String(s) = attr.value() {
                return Some(s.as_str());
            }
        }
        None
    })
}

// ---------------------------------------------------------------------------
// Epoch arithmetic helpers (no external deps)
// ---------------------------------------------------------------------------

/// Converts seconds since UNIX epoch to `(year, month, day, hour, min, sec)` in UTC.
fn epoch_secs_to_datetime(secs: u64) -> (u32, u32, u32, u32, u32, u32) {
    let second = (secs % 60) as u32;
    let minutes = secs / 60;
    let minute = (minutes % 60) as u32;
    let hours = minutes / 60;
    let hour = (hours % 24) as u32;
    let days = hours / 24; // days since 1970-01-01

    // Compute year from day count using the Gregorian calendar.
    // A 400-year cycle has 97 leap years → 146 097 days.
    // Shift epoch by 719 468 days so that day 0 is 0000-03-01.
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z % 146_097; // day of era [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // year of era [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year starting 2000-03-01 [0, 365]
    let mp = (5 * doy + 2) / 153; // month of period [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // day [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // month [1, 12]
    let y = if m <= 2 { y + 1 } else { y };

    (y as u32, m as u32, d as u32, hour, minute, second)
}

// ---------------------------------------------------------------------------
// Tests — Phase 1 RED → GREEN
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use kitlogger_log_domain::{LogAttribute, LogAttributeValue, LogContext, LogRecord, Severity};
    use std::time::{Duration, SystemTime};

    // Helper: build a minimal LogRecord.
    fn make_record(severity: Severity, message: &str) -> LogRecord {
        LogRecord::new(
            SystemTime::UNIX_EPOCH + Duration::from_secs(1_781_949_600), // 2026-06-20T10:00:00Z
            severity,
            message.to_string(),
            vec![],
        )
        .unwrap()
    }

    // -----------------------------------------------------------------------
    // LogFormat variants are all distinct
    // -----------------------------------------------------------------------

    #[test]
    fn log_format_variants_are_distinct() {
        assert_ne!(LogFormat::Json, LogFormat::HumanReadable);
        assert_ne!(LogFormat::Json, LogFormat::Text);
        assert_ne!(LogFormat::Json, LogFormat::Logfmt);
        assert_ne!(LogFormat::HumanReadable, LogFormat::Text);
        assert_ne!(LogFormat::HumanReadable, LogFormat::Logfmt);
        assert_ne!(LogFormat::Text, LogFormat::Logfmt);
    }

    // -----------------------------------------------------------------------
    // formatter_from_config returns without panic for each variant
    // -----------------------------------------------------------------------

    #[test]
    fn formatter_from_config_json_does_not_panic() {
        let record = make_record(Severity::Info, "test");
        let formatter = formatter_from_config(LogFormat::Json);
        // Must not panic — result may be Ok or Err
        let _ = formatter.format(&record, None);
    }

    #[test]
    fn formatter_from_config_human_readable_does_not_panic() {
        let record = make_record(Severity::Info, "test");
        let formatter = formatter_from_config(LogFormat::HumanReadable);
        let _ = formatter.format(&record, None);
    }

    #[test]
    fn formatter_from_config_text_does_not_panic() {
        let record = make_record(Severity::Info, "test");
        let formatter = formatter_from_config(LogFormat::Text);
        let _ = formatter.format(&record, None);
    }

    #[test]
    fn formatter_from_config_logfmt_does_not_panic() {
        let record = make_record(Severity::Info, "test");
        let formatter = formatter_from_config(LogFormat::Logfmt);
        let _ = formatter.format(&record, None);
    }

    // -----------------------------------------------------------------------
    // severity_label returns uppercase
    // -----------------------------------------------------------------------

    #[test]
    fn severity_label_trace() {
        assert_eq!(severity_label(&Severity::Trace), "TRACE");
    }

    #[test]
    fn severity_label_debug() {
        assert_eq!(severity_label(&Severity::Debug), "DEBUG");
    }

    #[test]
    fn severity_label_info() {
        assert_eq!(severity_label(&Severity::Info), "INFO");
    }

    #[test]
    fn severity_label_warn() {
        assert_eq!(severity_label(&Severity::Warn), "WARN");
    }

    #[test]
    fn severity_label_error() {
        assert_eq!(severity_label(&Severity::Error), "ERROR");
    }

    #[test]
    fn severity_label_fatal() {
        assert_eq!(severity_label(&Severity::Fatal), "FATAL");
    }

    // -----------------------------------------------------------------------
    // rfc3339_utc formats a known epoch
    // -----------------------------------------------------------------------

    #[test]
    fn rfc3339_utc_unix_epoch() {
        // 0 seconds → 1970-01-01T00:00:00Z
        let t = SystemTime::UNIX_EPOCH;
        assert_eq!(rfc3339_utc(t), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn rfc3339_utc_known_timestamp() {
        // 2026-06-20T10:00:00Z = 1_781_949_600 seconds since epoch
        let t = SystemTime::UNIX_EPOCH + Duration::from_secs(1_781_949_600);
        assert_eq!(rfc3339_utc(t), "2026-06-20T10:00:00Z");
    }

    // -----------------------------------------------------------------------
    // logger_name extraction
    // -----------------------------------------------------------------------

    #[test]
    fn logger_name_none_when_no_context() {
        assert!(logger_name(None).is_none());
    }

    #[test]
    fn logger_name_found_in_context() {
        let mut ctx = LogContext::new();
        ctx.add_attribute(
            LogAttribute::new(
                "logger".to_string(),
                LogAttributeValue::String("auth".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(logger_name(Some(&ctx)), Some("auth"));
    }

    #[test]
    fn logger_name_absent_when_key_missing() {
        let mut ctx = LogContext::new();
        ctx.add_attribute(
            LogAttribute::new(
                "env".to_string(),
                LogAttributeValue::String("prod".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
        assert!(logger_name(Some(&ctx)).is_none());
    }
}
