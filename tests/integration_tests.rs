//! Tests for the transport contract types.
//!
//! This module contains tests for the core types defined in the transport
//! contract, ensuring they behave as expected.

use std::collections::HashMap;
use std::time::Duration;

use serde_json;

use as_02::{DeliveryMode, PayloadEnvelope, TelemetryBatch, TransportError, TransportResult};
use as_02::payload::{PropagationMetadata, TransportMetadata};

#[test]
fn test_delivery_mode_serialization() {
    let modes = vec![
        DeliveryMode::FireAndForget,
        DeliveryMode::RequestResponse,
        DeliveryMode::Batch,
        DeliveryMode::Streaming,
    ];

    for mode in &modes {
        let serialized = serde_json::to_string(mode).unwrap();
        let deserialized: DeliveryMode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(*mode, deserialized);
    }
}

#[test]
fn test_telemetry_batch_rejects_all_empty() {
    let batch = TelemetryBatch::new(
        "resource".to_string(),
        vec![],
        vec![],
        vec![],
    );
    assert!(batch.is_err());
}

#[test]
fn test_telemetry_batch_accepts_non_empty() {
    let batch = TelemetryBatch::new(
        "resource".to_string(),
        vec!["trace1".to_string()],
        vec![],
        vec![],
    );
    assert!(batch.is_ok());
}

#[test]
fn test_payload_envelope_serialization() {
    let envelope = PayloadEnvelope {
        transport_metadata: TransportMetadata::now(),
        propagation_metadata: PropagationMetadata::default(),
        payload: TelemetryBatch::new(
            "resource".to_string(),
            vec!["trace1".to_string()],
            vec![],
            vec![],
        ).unwrap(),
    };

    let json = serde_json::to_string(&envelope).unwrap();
    let deserialized: PayloadEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(envelope.payload.traces.len(), deserialized.payload.traces.len());
}

#[test]
fn test_transport_error_is_non_exhaustive() {
    // This test ensures that we can match on known variants and have a wildcard
    // arm, which is important for non-exhaustive enums.
    let result: TransportResult<DeliveryMode> = Err(TransportError::Timeout);
    match result {
        Ok(_) => panic!("Expected error"),
        Err(TransportError::Timeout) => {}, // Known variant
        Err(TransportError::Backpressure(_)) => panic!("Unexpected backpressure"),
        Err(_) => {}, // Wildcard arm for future variants
    }

    // Test backpressure variant
    let result: TransportResult<DeliveryMode> = Err(TransportError::Backpressure(
        as_02::BackpressureSignal {
            retry_after: Some(Duration::from_secs(10)),
        }
    ));
    match result {
        Ok(_) => panic!("Expected error"),
        Err(TransportError::Timeout) => panic!("Unexpected timeout"),
        Err(TransportError::Backpressure(_)) => {}, // Known variant
        Err(_) => {}, // Wildcard arm for future variants
    }
}

#[test]
fn test_transport_metadata_now() {
    let metadata = TransportMetadata::now();
    assert!(metadata.timestamp.elapsed().unwrap().as_secs() < 1);
}

#[test]
fn test_propagation_metadata_default() {
    let metadata = PropagationMetadata::default();
    assert!(metadata.context.is_empty());
}