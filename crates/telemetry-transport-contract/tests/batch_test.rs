use telemetry_transport_contract::{
    Context, InstrumentationScope, LogRecord, LogSeverity, Metric, Resource, Span, TelemetryBatch,
};

#[test]
fn test_telemetry_batch_serde() {
    let scope = InstrumentationScope::new("test".to_string());
    let batch = TelemetryBatch::new(
        Resource::new(),
        vec![Span::new(
            Context::new_root(),
            Resource::new(),
            scope,
            "trace1".to_string(),
            0,
        )],
        vec![],
        vec![],
    )
    .unwrap();

    let json = serde_json::to_string(&batch).unwrap();
    let deserialized: TelemetryBatch = serde_json::from_str(&json).unwrap();
    assert_eq!(batch.traces.len(), deserialized.traces.len());
    assert_eq!(batch.resource, deserialized.resource);
}

#[test]
fn test_telemetry_batch_empty_validation() {
    let result = TelemetryBatch::new(Resource::new(), vec![], vec![], vec![]);

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "telemetry batch must contain at least one signal type"
    );
}

#[test]
fn test_telemetry_batch_non_empty_validation() {
    let scope = InstrumentationScope::new("test".to_string());
    let result = TelemetryBatch::new(
        Resource::new(),
        vec![],
        vec![Metric::new(
            Resource::new(),
            scope,
            "metric1".to_string(),
            "".to_string(),
            "".to_string(),
        )],
        vec![],
    );

    assert!(result.is_ok());
}

#[test]
fn test_telemetry_batch_only_logs() {
    let scope = InstrumentationScope::new("test".to_string());
    let result = TelemetryBatch::new(
        Resource::new(),
        vec![],
        vec![],
        vec![LogRecord::new(
            Context::new_root(),
            Resource::new(),
            scope,
            0,
            LogSeverity::Info,
            "log1".to_string(),
        )],
    );

    assert!(result.is_ok());
}
