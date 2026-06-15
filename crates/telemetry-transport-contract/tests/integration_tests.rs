use std::time::Duration;

use telemetry_transport_contract::payload::{PropagationMetadata, TransportMetadata};
use telemetry_transport_contract::{
    BackpressureSignal, DeliveryMode, PayloadEnvelope, Resource, Span, TelemetryBatch,
    TransportError, TransportResult,
};

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
        Resource("resource".to_string()),
        vec![],
        vec![],
        vec![],
    );
    assert!(batch.is_err());
}

#[test]
fn test_telemetry_batch_accepts_non_empty() {
    let batch = TelemetryBatch::new(
        Resource("resource".to_string()),
        vec![Span("trace1".to_string())],
        vec![],
        vec![],
    );
    assert!(batch.is_ok());
}

#[test]
fn test_telemetry_batch_accepts_metrics_only() {
    let batch = TelemetryBatch::new(
        Resource("resource".to_string()),
        vec![],
        vec![telemetry_transport_contract::Metric("cpu".to_string())],
        vec![],
    );
    assert!(batch.is_ok());
}

#[test]
fn test_telemetry_batch_accepts_logs_only() {
    let batch = TelemetryBatch::new(
        Resource("resource".to_string()),
        vec![],
        vec![],
        vec![telemetry_transport_contract::LogRecord("log1".to_string())],
    );
    assert!(batch.is_ok());
}

#[test]
fn test_payload_envelope_serialization() {
    let envelope = PayloadEnvelope {
        transport_metadata: TransportMetadata::now(),
        propagation_metadata: PropagationMetadata::new("test"),
        payload: TelemetryBatch::new(
            Resource("resource".to_string()),
            vec![Span("trace1".to_string())],
            vec![],
            vec![],
        )
        .unwrap(),
    };

    let json = serde_json::to_string(&envelope).unwrap();
    let deserialized: PayloadEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(
        envelope.payload.traces.len(),
        deserialized.payload.traces.len()
    );
}

#[test]
fn test_transport_error_is_non_exhaustive() {
    let result: TransportResult<DeliveryMode> = Err(TransportError::Timeout);
    match result {
        Ok(_) => panic!("Expected error"),
        Err(TransportError::Timeout) => {}
        Err(TransportError::Unavailable) => panic!("Unexpected unavailable"),
        Err(TransportError::Backpressure(_)) => panic!("Unexpected backpressure"),
        Err(TransportError::PayloadTooLarge) => panic!("Unexpected payload too large"),
        Err(_) => {}
    }

    let result: TransportResult<DeliveryMode> = Err(TransportError::Backpressure(
        BackpressureSignal {
            retry_after: Some(Duration::from_secs(10)),
        },
    ));
    match result {
        Ok(_) => panic!("Expected error"),
        Err(TransportError::Timeout) => panic!("Unexpected timeout"),
        Err(TransportError::Backpressure(_)) => {}
        Err(_) => {}
    }

    let result: TransportResult<DeliveryMode> = Err(TransportError::Unavailable);
    match result {
        Ok(_) => panic!("Expected error"),
        Err(TransportError::Unavailable) => {}
        Err(_) => {}
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
    assert!(!metadata.transport.is_empty());
}
