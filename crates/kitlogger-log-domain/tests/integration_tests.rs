//! Tests for the structured logging domain model.

mod common;

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use kitlogger_log_domain::{
        CorrelationId, LogAttribute, LogAttributeValue, LogContext, LogRecord, Logger, LoggerFactory,
        Severity, SpanId, TraceId, ValidationError,
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
        assert_eq!(
            "invalid".parse::<Severity>().unwrap_err(),
            ValidationError::InvalidSeverity
        );
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

    // ── LogContext integration tests ──────────────────────────────

    #[test]
    fn test_log_context_full_enrichment_pipeline() {
        let ctx = LogContext::new();
        let attr = LogAttribute::new(
            "env".to_string(),
            LogAttributeValue::String("prod".to_string()),
        )
        .unwrap();

        let enriched = ctx
            .with_attribute(attr)
            .unwrap()
            .with_correlation_id(CorrelationId::new("req-1".to_string()))
            .unwrap()
            .with_trace_id(TraceId::new("trace-abc".to_string()))
            .unwrap()
            .with_span_id(SpanId::new("span-42".to_string()))
            .unwrap();

        assert_eq!(enriched.attributes().len(), 1);
        assert_eq!(enriched.attributes()[0].name(), "env");
        assert_eq!(
            enriched.correlation_id(),
            Some(&CorrelationId::new("req-1".to_string()))
        );
        assert_eq!(
            enriched.trace_id(),
            Some(&TraceId::new("trace-abc".to_string()))
        );
        assert_eq!(
            enriched.span_id(),
            Some(&SpanId::new("span-42".to_string()))
        );
    }

    #[test]
    fn test_log_context_attribute_isolation() {
        let ctx = LogContext::new();
        let attr = LogAttribute::new(
            "env".to_string(),
            LogAttributeValue::String("prod".to_string()),
        )
        .unwrap();

        let _enriched = ctx.with_attribute(attr).unwrap();
        // Original must remain unchanged
        assert_eq!(ctx.attributes().len(), 0);
    }

    #[test]
    fn test_log_context_id_idempotency() {
        let ctx = LogContext::new();
        let cid1 = CorrelationId::new("req-1".to_string());
        let cid2 = CorrelationId::new("req-2".to_string());

        let enriched = ctx
            .with_correlation_id(cid1)
            .unwrap()
            .with_correlation_id(cid2.clone())
            .unwrap();

        assert_eq!(enriched.correlation_id(), Some(&cid2));
    }

    #[test]
    fn test_log_context_display_with_all_ids() {
        let ctx = LogContext::new();
        let display = ctx
            .with_correlation_id(CorrelationId::new("req-1".to_string()))
            .unwrap()
            .with_trace_id(TraceId::new("trace-abc".to_string()))
            .unwrap()
            .with_span_id(SpanId::new("span-42".to_string()))
            .unwrap()
            .to_string();

        assert!(display.contains("correlation_id: req-1"));
        assert!(display.contains("trace_id: trace-abc"));
        assert!(display.contains("span_id: span-42"));
    }

    // ── Phase 4 integration tests ─────────────────────────────────────────────

    use crate::common::{make_attr, MockFactory, RecordingLogger};
    use std::sync::Arc;

    // 4.1 — Factory ctx + extra ctx both appear in emitted record ─────────────
    #[test]
    fn test_factory_and_extra_context_both_in_emitted_record() {
        let mut factory_ctx = LogContext::new();
        factory_ctx
            .add_attribute(make_attr("service", "my-svc"))
            .unwrap();
        let factory = MockFactory::new(factory_ctx);

        let mut extra = LogContext::new();
        extra.add_attribute(make_attr("component", "auth")).unwrap();

        // Use concrete helper to inspect merged context attrs directly.
        let logger = factory.create_recording_logger("auth", Some(extra));
        let attr_names: Vec<&str> = logger.context.attributes().iter().map(|a| a.name()).collect();
        assert!(
            attr_names.contains(&"service"),
            "factory service attr missing: {:?}",
            attr_names
        );
        assert!(
            attr_names.contains(&"component"),
            "extra component attr missing: {:?}",
            attr_names
        );
        assert_eq!(logger.name(), "auth");
        assert!(logger.info("login", &[]).is_ok());
    }

    // 4.2 — Per-emit attributes are appended after context attributes ──────────
    #[test]
    fn test_per_emit_attrs_appended_after_context_attrs() {
        let mut ctx = LogContext::new();
        ctx.add_attribute(make_attr("service", "my-svc")).unwrap();
        let logger = RecordingLogger::new("l", ctx);

        let emit_attr = make_attr("request_id", "req-1");
        logger.info("login", &[emit_attr]).unwrap();

        let emitted = logger.emitted_attrs();
        let names: Vec<&str> = emitted[0].iter().map(|a| a.name()).collect();
        let service_pos = names.iter().position(|&n| n == "service").unwrap();
        let req_pos = names.iter().position(|&n| n == "request_id").unwrap();
        assert!(service_pos < req_pos, "context attr must precede emit attr");
    }

    // 4.3 — Logger is Send + Sync: cross-thread usage ─────────────────────────
    #[test]
    fn test_logger_is_send_sync_across_threads() {
        let logger: Arc<dyn Logger> = RecordingLogger::new("threaded", LogContext::new());
        let logger_clone = Arc::clone(&logger);
        let handle = std::thread::spawn(move || {
            logger_clone.info("from-thread", &[]).unwrap();
        });
        handle.join().expect("thread panicked");
    }

    // 4.4 — LoggerFactory is Send + Sync: cross-thread usage ──────────────────
    #[test]
    fn test_logger_factory_is_send_sync_across_threads() {
        let factory: Arc<dyn LoggerFactory> = Arc::new(MockFactory::new(LogContext::new()));
        let factory_clone = Arc::clone(&factory);
        let handle = std::thread::spawn(move || {
            let logger = factory_clone.create_logger("from-thread");
            logger.info("hello", &[]).unwrap();
        });
        handle.join().expect("thread panicked");
    }
}
