//! Tests for the payload module.

use serde_test::{assert_tokens, Token};

use as_02::payload::{PayloadEnvelope, TransportMetadata, PropagationMetadata};
use as_02::batch::TelemetryBatch;

#[test]
fn test_transport_metadata_serde() {
    let metadata = TransportMetadata::now();
    // We can't test the exact token because timestamp is dynamic
    // But we can at least make sure it serializes
    let serialized = serde_json::to_string(&metadata).unwrap();
    assert!(serialized.contains("\"timestamp\""));
}

#[test]
fn test_propagation_metadata_serde() {
    let metadata = PropagationMetadata::default();
    assert_tokens(&metadata, &[
        Token::Struct { name: "PropagationMetadata", len: 1 },
        Token::Str("context"),
        Token::Map { len: Some(0) },
        Token::MapEnd,
        Token::StructEnd,
    ]);
}

#[test]
fn test_payload_envelope_serde() {
    let batch = TelemetryBatch::new(
        "resource1".to_string(),
        vec!["trace1".to_string()],
        vec![],
        vec![],
    ).unwrap();

    let envelope = PayloadEnvelope {
        transport_metadata: TransportMetadata::now(),
        propagation_metadata: PropagationMetadata::default(),
        payload: batch,
    };

    // We can't test the exact tokens because timestamp is dynamic
    // But we can at least make sure it serializes
    let serialized = serde_json::to_string(&envelope).unwrap();
    assert!(serialized.contains("\"payload\""));
    assert!(serialized.contains("\"transport_metadata\""));
    assert!(serialized.contains("\"propagation_metadata\""));
}