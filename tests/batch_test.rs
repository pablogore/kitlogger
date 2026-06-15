//! Tests for the batch module.

use serde_test::{assert_tokens, Token};

use as_02::TelemetryBatch;

#[test]
fn test_telemetry_batch_serde() {
    let batch = TelemetryBatch::new(
        "resource1".to_string(),
        vec!["trace1".to_string()],
        vec![],
        vec![],
    ).unwrap();

    assert_tokens(&batch, &[
        Token::Struct { name: "TelemetryBatch", len: 4 },
        Token::Str("resource"),
        Token::Str("resource1"),
        Token::Str("traces"),
        Token::Seq { len: Some(1) },
        Token::Str("trace1"),
        Token::SeqEnd,
        Token::Str("metrics"),
        Token::Seq { len: Some(0) },
        Token::SeqEnd,
        Token::Str("logs"),
        Token::Seq { len: Some(0) },
        Token::SeqEnd,
        Token::StructEnd,
    ]);
}

#[test]
fn test_telemetry_batch_empty_validation() {
    let result = TelemetryBatch::new(
        "resource1".to_string(),
        vec![],
        vec![],
        vec![],
    );

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "invalid telemetry batch");
}

#[test]
fn test_telemetry_batch_non_empty_validation() {
    let result = TelemetryBatch::new(
        "resource1".to_string(),
        vec![],
        vec!["metric1".to_string()],
        vec![],
    );

    assert!(result.is_ok());
}