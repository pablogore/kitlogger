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
