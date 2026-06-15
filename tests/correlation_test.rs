//! Tests for correlation identifier implementation

use context_propagation::carrier::{MapCarrier, Propagator};
use context_propagation::correlation::CorrelationIdentifier;
use context_propagation::models::{
    Context, InstrumentationScope, LogRecord, LogSeverity, Resource, Span,
};

#[test]
fn test_correlation_generation() {
    let correlation_id = CorrelationIdentifier::new();

    // Verify it's a valid UUID (not nil)
    assert!(!correlation_id.id().is_nil());

    // Verify it has a valid timestamp
    assert!(correlation_id.created_at() > 0);
}

#[test]
fn test_correlation_roundtrip() {
    // Create a correlation identifier
    let original_id = CorrelationIdentifier::new();

    // Create a carrier and inject the context
    let mut carrier = MapCarrier::new();
    let propagator = context_propagation::propagation::CorrelationPropagator::new();
    propagator.inject(&mut carrier, &original_id);

    // Extract the context back
    let extracted = propagator.extract(&carrier);
    assert!(extracted.is_some());
    let extracted_id = extracted.unwrap();

    // Verify they match
    assert_eq!(original_id.id(), extracted_id.id());
    assert_eq!(original_id.created_at(), extracted_id.created_at());
}

#[test]
fn test_cross_signal_correlation() {
    let correlation_id = CorrelationIdentifier::new();
    let correlation_id_str = correlation_id.id().to_string();

    let resource = Resource::new();
    let scope = InstrumentationScope::new("test".to_string());

    let trace_id = [1u8; 16];
    let span_id = [1u8; 8];

    let context = Context {
        trace_id,
        span_id,
        correlation_id: Some(correlation_id_str.clone()),
    };

    // Create a Span with the correlation ID
    let span = Span::new(
        context.clone(),
        resource.clone(),
        scope.clone(),
        "test-span".to_string(),
        1000,
    );
    assert_eq!(
        span.context.correlation_id,
        Some(correlation_id_str.clone())
    );

    // Create a LogRecord with the same correlation ID
    let log = LogRecord::new(
        context.clone(),
        resource.clone(),
        scope.clone(),
        1000,
        LogSeverity::Info,
        "test-log".to_string(),
    );
    assert_eq!(log.context.correlation_id, Some(correlation_id_str.clone()));

    // Verify the same correlation ID links all three signals
    assert_eq!(span.context.correlation_id, log.context.correlation_id);
}

#[test]
fn test_correlation_validity() {
    let id = CorrelationIdentifier::new();
    assert!(id.is_valid());

    // A manually constructed nil UUID should be invalid
    let invalid = CorrelationIdentifier {
        id: uuid::Uuid::nil(),
        created_at: 0,
    };
    assert!(!invalid.is_valid());
}

#[test]
fn test_correlation_from_uuid_rejects_nil() {
    let result = CorrelationIdentifier::from_uuid(uuid::Uuid::nil());
    assert!(result.is_none());

    let result = CorrelationIdentifier::from_uuid(uuid::Uuid::now_v7());
    assert!(result.is_some());
}

#[test]
fn test_correlation_from_str_rejects_nil() {
    use std::str::FromStr;
    let nil_str = "00000000-0000-0000-0000-000000000000";
    let result = CorrelationIdentifier::from_str(nil_str);
    assert!(result.is_err());
}

#[test]
fn test_correlation_from_str_invalid() {
    use std::str::FromStr;
    let result = CorrelationIdentifier::from_str("not-a-uuid");
    assert!(result.is_err());

    let result = CorrelationIdentifier::from_str("");
    assert!(result.is_err());
}

#[test]
fn test_correlation_timestamp_extraction() {
    let id = CorrelationIdentifier::new();

    // created_at should be a recent Unix timestamp in milliseconds
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    // Allow 5 seconds skew for test execution
    let diff = (now_ms - id.created_at()).abs();
    assert!(diff < 5000, "timestamp diff too large: {}ms", diff);
    assert!(id.created_at() > 0);
}

#[test]
fn test_correlation_from_uuid_preserves_timestamp() {
    let original = CorrelationIdentifier::new();
    let id = original.id();

    // from_uuid should preserve the same timestamp
    let restored = CorrelationIdentifier::from_uuid(*id).unwrap();
    assert_eq!(original.created_at(), restored.created_at());
    assert_eq!(original.id(), restored.id());
}

#[test]
fn test_correlation_serde_roundtrip() {
    let original = CorrelationIdentifier::new();

    let serialized = serde_json::to_string(&original).expect("serialize");
    let deserialized: CorrelationIdentifier =
        serde_json::from_str(&serialized).expect("deserialize");

    assert_eq!(original.id(), deserialized.id());
    assert_eq!(original.created_at(), deserialized.created_at());
}
