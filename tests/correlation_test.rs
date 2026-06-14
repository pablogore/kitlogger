//! Tests for correlation identifier implementation

use context_propagation::correlation::CorrelationIdentifier;
use context_propagation::carrier::{MapCarrier, Propagator};

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