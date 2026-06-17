use std::time::{SystemTime, UNIX_EPOCH};

use kit_config::{LogFormat, TimestampFormat};

use crate::event::LogEvent;

pub trait Formatter: Send + Sync {
    fn format(&self, event: &LogEvent) -> String;
}

pub struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn format(&self, event: &LogEvent) -> String {
        let mut fields = event.fields.clone();
        fields.insert("level".to_string(), serde_json::json!(event.level));
        fields.insert("message".to_string(), serde_json::json!(event.message));
        fields.insert("target".to_string(), serde_json::json!(event.target));
        fields.insert("timestamp".to_string(), serde_json::json!(format_timestamp(&event.timestamp, &TimestampFormat::Rfc3339)));

        if let Some(cid) = &event.correlation_id {
            fields.insert("correlation_id".to_string(), serde_json::json!(cid));
        }

        serde_json::to_string(&fields).unwrap_or_else(|_| "{}".to_string())
    }
}

pub struct PrettyFormatter;

impl Formatter for PrettyFormatter {
    fn format(&self, event: &LogEvent) -> String {
        let ts = format_timestamp(&event.timestamp, &TimestampFormat::Rfc3339);
        format!(
            "[{}] {:5} [{}] {}",
            ts,
            format_level(&event.level),
            event.target,
            event.message,
        )
    }
}

pub struct CompactFormatter;

impl Formatter for CompactFormatter {
    fn format(&self, event: &LogEvent) -> String {
        let ts = format_timestamp(&event.timestamp, &TimestampFormat::UnixMillis);
        format!("{} {} {} | {}", ts, format_level_short(&event.level), event.target, event.message)
    }
}

pub struct TextFormatter;

impl Formatter for TextFormatter {
    fn format(&self, event: &LogEvent) -> String {
        format!("[{}] {}: {}", format_level(&event.level), event.target, event.message)
    }
}

pub fn formatter_from_config(format: &LogFormat) -> Box<dyn Formatter> {
    match format {
        LogFormat::Json => Box::new(JsonFormatter),
        LogFormat::Pretty => Box::new(PrettyFormatter),
        LogFormat::Compact => Box::new(CompactFormatter),
        LogFormat::Text => Box::new(TextFormatter),
    }
}

fn format_timestamp(time: &SystemTime, fmt: &TimestampFormat) -> String {
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    match fmt {
        TimestampFormat::Rfc3339 => {
            let secs = duration.as_secs();
            let nanos = duration.subsec_nanos();
            chrono_from_unix(secs, nanos)
        }
        TimestampFormat::Rfc3339Nano => {
            let secs = duration.as_secs();
            let nanos = duration.subsec_nanos();
            format_nano_timestamp(secs, nanos)
        }
        TimestampFormat::Unix => duration.as_secs().to_string(),
        TimestampFormat::UnixMillis => duration.as_millis().to_string(),
        TimestampFormat::Custom => duration.as_secs().to_string(),
    }
}

fn chrono_from_unix(secs: u64, _nanos: u32) -> String {
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    let (year, month, day) = days_to_date(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

fn format_nano_timestamp(secs: u64, nanos: u32) -> String {
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    let (year, month, day) = days_to_date(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09}Z",
        year, month, day, hours, minutes, seconds, nanos
    )
}

fn days_to_date(days: u64) -> (u64, u64, u64) {
    let mut y = 1970i64;
    let mut d = days as i64;

    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }

    let months_days = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut m = 0u64;
    for (i, &md) in months_days.iter().enumerate() {
        if d < md {
            m = i as u64;
            break;
        }
        d -= md;
    }

    (y as u64, m + 1, (d + 1) as u64)
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn format_level(level: &kit_config::LogLevel) -> String {
    match level {
        kit_config::LogLevel::Trace => "TRACE".to_string(),
        kit_config::LogLevel::Debug => "DEBUG".to_string(),
        kit_config::LogLevel::Info => "INFO".to_string(),
        kit_config::LogLevel::Warn => "WARN".to_string(),
        kit_config::LogLevel::Error => "ERROR".to_string(),
    }
}

fn format_level_short(level: &kit_config::LogLevel) -> &'static str {
    match level {
        kit_config::LogLevel::Trace => "T",
        kit_config::LogLevel::Debug => "D",
        kit_config::LogLevel::Info => "I",
        kit_config::LogLevel::Warn => "W",
        kit_config::LogLevel::Error => "E",
    }
}
