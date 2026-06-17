use context_propagation::carrier::{Extractor, Injector, MapCarrier};
use telemetry_transport_contract::payload::{
    PayloadEnvelope, PropagationMetadata, TransportMetadata,
};
use telemetry_transport_contract::{Resource, Span, TelemetryBatch};

#[test]
fn test_transport_metadata_serde() {
    let metadata = TransportMetadata::now();
    let serialized = serde_json::to_string(&metadata).unwrap();
    assert!(serialized.contains("\"timestamp\""));
}

#[test]
fn test_propagation_metadata_from_as01() {
    let metadata = PropagationMetadata::new("http");
    assert_eq!(metadata.get("nonexistent"), None);
    assert!(!metadata.keys().any(|_| true));
}

#[test]
fn test_propagation_metadata_default() {
    let metadata = PropagationMetadata::default();
    assert_eq!(metadata.transport, "unknown");
    assert!(metadata.is_empty());
}

#[test]
fn test_propagation_metadata_add_and_get() {
    let mut metadata = PropagationMetadata::new("grpc");
    metadata.add("trace_id", "abc123");
    assert_eq!(metadata.get("trace_id"), Some("abc123"));
    assert_eq!(metadata.get("missing"), None);
}

#[test]
fn test_payload_envelope_serde() {
    let batch = TelemetryBatch::new(
        Resource("resource1".to_string()),
        vec![Span("trace1".to_string())],
        vec![],
        vec![],
    )
    .unwrap();

    let envelope = PayloadEnvelope {
        transport_metadata: TransportMetadata::now(),
        propagation_metadata: PropagationMetadata::new("test"),
        payload: batch,
    };

    let serialized = serde_json::to_string(&envelope).unwrap();
    assert!(serialized.contains("\"payload\""));
    assert!(serialized.contains("\"transport_metadata\""));
    assert!(serialized.contains("\"propagation_metadata\""));
}

#[test]
fn test_propagation_metadata_serde() {
    let metadata = PropagationMetadata::new("http");
    let serialized = serde_json::to_string(&metadata).unwrap();
    let deserialized: PropagationMetadata = serde_json::from_str(&serialized).unwrap();
    assert_eq!(metadata.transport, deserialized.transport);
}

#[test]
fn test_telemetry_batch_rejects_all_empty_in_payload() {
    let result = TelemetryBatch::new(Resource("resource1".to_string()), vec![], vec![], vec![]);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "telemetry batch must contain at least one signal type"
    );
}

#[test]
fn test_payload_envelope_serde_with_map_carrier() {
    let mut carrier = MapCarrier::new();
    carrier.set("trace_id", "abc123");
    carrier.set("span_id", "def456");

    let mut metadata = PropagationMetadata::new("test");
    if let Some(val) = carrier.get("trace_id") {
        metadata.add("trace_id", val);
    }
    if let Some(val) = carrier.get("span_id") {
        metadata.add("span_id", val);
    }

    let batch = TelemetryBatch::new(
        Resource("resource1".to_string()),
        vec![Span("trace1".to_string())],
        vec![],
        vec![],
    )
    .unwrap();

    let envelope = PayloadEnvelope {
        transport_metadata: TransportMetadata::now(),
        propagation_metadata: metadata,
        payload: batch,
    };

    let serialized = serde_json::to_string(&envelope).unwrap();
    let deserialized: PayloadEnvelope = serde_json::from_str(&serialized).unwrap();

    assert_eq!(
        envelope.payload.traces.len(),
        deserialized.payload.traces.len()
    );
    assert_eq!(
        envelope.propagation_metadata.transport,
        deserialized.propagation_metadata.transport
    );
    assert_eq!(
        deserialized.propagation_metadata.get("trace_id"),
        Some("abc123")
    );
}
