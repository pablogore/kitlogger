//! Tests for the structured logging domain model.

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use kitlogger_log_domain::{
        CorrelationId, LogAttribute, LogAttributeValue, LogRecord, Severity, SpanId, TraceId,
        ValidationError,
    };

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Trace), "Trace");
        assert_eq!(format!("{}", Severity::Debug), "Debug");
        assert_eq!(format!("{}", Severity::Info), "Info");
        assert_eq!(format!("{}", Severity::Warn), "Warn");
        assert_eq!(format!("{}", Severity::Error), "Error");
        assert_eq!(format!("{}", Severity::Fatal), "Fatal");
    }

    #[test]
    fn test_severity_from_str() {
        assert_eq!("trace".parse::<Severity>().unwrap(), Severity::Trace);
        assert_eq!("debug".parse::<Severity>().unwrap(), Severity::Debug);
        assert_eq!("info".parse::<Severity>().unwrap(), Severity::Info);
        assert_eq!("warn".parse::<Severity>().unwrap(), Severity::Warn);
        assert_eq!("error".parse::<Severity>().unwrap(), Severity::Error);
        assert_eq!("fatal".parse::<Severity>().unwrap(), Severity::Fatal);
        assert!("invalid".parse::<Severity>().is_err());
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Trace < Severity::Debug);
        assert!(Severity::Debug < Severity::Info);
        assert!(Severity::Info < Severity::Warn);
        assert!(Severity::Warn < Severity::Error);
        assert!(Severity::Error < Severity::Fatal);
    }

    #[test]
    fn test_log_attribute_creation() {
        let attr = LogAttribute::new(
            "test_key".to_string(),
            LogAttributeValue::String("test_value".to_string()),
        )
        .unwrap();
        assert_eq!(attr.name(), "test_key");
        assert_eq!(
            attr.value(),
            &LogAttributeValue::String("test_value".to_string())
        );
    }

    #[test]
    fn test_log_attribute_invalid_name() {
        // Empty name
        assert!(LogAttribute::new(
            "".to_string(),
            LogAttributeValue::String("test".to_string()),
        )
        .is_err());

        // Uppercase first letter
        assert!(LogAttribute::new(
            "Test".to_string(),
            LogAttributeValue::String("test".to_string()),
        )
        .is_err());

        // Invalid character
        assert!(LogAttribute::new(
            "test-key".to_string(),
            LogAttributeValue::String("test".to_string()),
        )
        .is_err());

        // Reserved name
        assert!(LogAttribute::new(
            "timestamp".to_string(),
            LogAttributeValue::String("test".to_string()),
        )
        .is_err());
    }

    #[test]
    fn test_log_attribute_value_array() {
        let values = vec![
            LogAttributeValue::String("a".to_string()),
            LogAttributeValue::String("b".to_string()),
        ];
        let array = LogAttributeValue::array(values).unwrap();
        assert!(matches!(array, LogAttributeValue::Array(_)));

        // Test heterogeneous array fails
        let values = vec![
            LogAttributeValue::String("a".to_string()),
            LogAttributeValue::Integer(1),
        ];
        assert!(LogAttributeValue::array(values).is_err());
    }

    #[test]
    fn test_log_record_creation() {
        let record = LogRecord::new(
            SystemTime::now(),
            Severity::Info,
            "Test message".to_string(),
            vec![LogAttribute::new(
                "key".to_string(),
                LogAttributeValue::String("value".to_string()),
            )
            .unwrap()],
        )
        .unwrap();

        assert_eq!(record.severity(), &Severity::Info);
        assert_eq!(record.message(), "Test message");
        assert_eq!(record.attributes().len(), 1);
    }

    #[test]
    fn test_log_record_empty_message() {
        assert_eq!(
            LogRecord::new(SystemTime::now(), Severity::Info, "".to_string(), vec![],).unwrap_err(),
            ValidationError::EmptyMessage
        );
    }

    #[test]
    fn test_correlation_id() {
        let cid = CorrelationId::new("req-42".to_string());
        assert_eq!(cid.as_str(), "req-42");
        assert_eq!(cid.to_string(), "req-42");
    }

    #[test]
    fn test_trace_id() {
        let tid = TraceId::new("trace-42".to_string());
        assert_eq!(tid.as_str(), "trace-42");
        assert_eq!(tid.to_string(), "trace-42");
    }

    #[test]
    fn test_span_id() {
        let sid = SpanId::new("span-42".to_string());
        assert_eq!(sid.as_str(), "span-42");
        assert_eq!(sid.to_string(), "span-42");
    }
}
