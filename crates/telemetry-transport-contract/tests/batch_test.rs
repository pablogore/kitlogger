use serde_test::{assert_tokens, Token};

use telemetry_transport_contract::{LogRecord, Metric, Resource, Span, TelemetryBatch};

#[test]
fn test_telemetry_batch_serde() {
    let batch = TelemetryBatch::new(
        Resource("resource1".to_string()),
        vec![Span("trace1".to_string())],
        vec![],
        vec![],
    )
    .unwrap();

    assert_tokens(
        &batch,
        &[
            Token::Struct {
                name: "TelemetryBatch",
                len: 4,
            },
            Token::Str("resource"),
            Token::NewtypeStruct { name: "Resource" },
            Token::Str("resource1"),
            Token::Str("traces"),
            Token::Seq { len: Some(1) },
            Token::NewtypeStruct { name: "Span" },
            Token::Str("trace1"),
            Token::SeqEnd,
            Token::Str("metrics"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::Str("logs"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_telemetry_batch_empty_validation() {
    let result = TelemetryBatch::new(Resource("resource1".to_string()), vec![], vec![], vec![]);

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "telemetry batch must contain at least one signal type"
    );
}

#[test]
fn test_telemetry_batch_non_empty_validation() {
    let result = TelemetryBatch::new(
        Resource("resource1".to_string()),
        vec![],
        vec![Metric("metric1".to_string())],
        vec![],
    );

    assert!(result.is_ok());
}

#[test]
fn test_telemetry_batch_only_logs() {
    let result = TelemetryBatch::new(
        Resource("resource1".to_string()),
        vec![],
        vec![],
        vec![LogRecord("log1".to_string())],
    );

    assert!(result.is_ok());
}
