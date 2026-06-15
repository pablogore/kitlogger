use context_propagation::baggage::{Baggage, BaggageEntry, BaggageProperty};
use context_propagation::carrier::{Injector, MapCarrier, Propagator};
use context_propagation::correlation::CorrelationIdentifier;
use context_propagation::propagation::{
    BaggagePropagator, CorrelationPropagator, TraceContextPropagator,
};
use context_propagation::propagation_metadata::PropagationMetadata;
use context_propagation::trace_context::{SpanId, TraceContext, TraceFlags, TraceId, TraceState};
use std::str::FromStr;

// --- PropagationMetadata tests ---

#[test]
fn test_propagation_metadata_creation() {
    let meta = PropagationMetadata::new("http");
    assert_eq!(meta.transport, "http");
    assert!(meta.is_empty());
}

#[test]
fn test_propagation_metadata_add_and_get() {
    let mut meta = PropagationMetadata::new("grpc");
    meta.add("content-type", "application/grpc");
    meta.add("timeout", "30s");

    assert!(!meta.is_empty());
    assert_eq!(meta.get("content-type"), Some("application/grpc"));
    assert_eq!(meta.get("timeout"), Some("30s"));
    assert_eq!(meta.get("nonexistent"), None);
}

#[test]
fn test_propagation_metadata_keys() {
    let mut meta = PropagationMetadata::new("kafka");
    meta.add("topic", "events");
    meta.add("partition", "0");

    let keys: Vec<&String> = meta.keys().collect();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&&"topic".to_string()));
    assert!(keys.contains(&&"partition".to_string()));
}

#[test]
fn test_propagation_metadata_default() {
    let meta = PropagationMetadata::default();
    assert_eq!(meta.transport, "unknown");
    assert!(meta.is_empty());
}

#[test]
fn test_trace_context_roundtrip() {
    let trace_id = TraceId::from_str("0af7651916cd43dd8448eb211c80319c").unwrap();
    let span_id = SpanId::from_str("b7ad6b7169203331").unwrap();
    let trace_flags = TraceFlags::new(0x01);
    let mut trace_state = TraceState::new();
    trace_state.add("congo", "t61rcWkgMzE").unwrap();

    let original = TraceContext::new(trace_id, span_id, None, trace_flags, trace_state);

    let mut carrier = MapCarrier::new();
    let propagator = TraceContextPropagator::new();
    propagator.inject(&mut carrier, &original);

    let extracted = propagator.extract(&carrier);
    assert!(extracted.is_some());
    let extracted = extracted.unwrap();

    assert_eq!(extracted.trace_id, original.trace_id);
    assert_eq!(extracted.span_id, original.span_id);
    assert_eq!(extracted.trace_flags, original.trace_flags);
    assert_eq!(extracted.parent_span_id, original.parent_span_id);
    assert_eq!(extracted.trace_state, original.trace_state);
}

#[test]
fn test_trace_context_roundtrip_with_parent_span_id() {
    let trace_id = TraceId::from_str("0af7651916cd43dd8448eb211c80319c").unwrap();
    let span_id = SpanId::from_str("b7ad6b7169203331").unwrap();
    let parent_span_id = SpanId::from_str("1234567890abcdef").unwrap();
    let trace_flags = TraceFlags::new(0x01);

    let original = TraceContext::new(
        trace_id,
        span_id,
        Some(parent_span_id),
        trace_flags,
        TraceState::new(),
    );

    let mut carrier = MapCarrier::new();
    let propagator = TraceContextPropagator::new();
    propagator.inject(&mut carrier, &original);

    let extracted = propagator.extract(&carrier);
    assert!(extracted.is_some());
    let extracted = extracted.unwrap();

    assert_eq!(extracted.trace_id, original.trace_id);
    assert_eq!(extracted.span_id, original.span_id);
    assert_eq!(extracted.parent_span_id, original.parent_span_id);
    assert_eq!(extracted.trace_flags, original.trace_flags);
}

#[test]
fn test_tracestate_roundtrip() {
    let trace_id = TraceId::from_str("0af7651916cd43dd8448eb211c80319c").unwrap();
    let span_id = SpanId::from_str("b7ad6b7169203331").unwrap();
    let mut trace_state = TraceState::new();
    trace_state.add("congo", "t61rcWkgMzE").unwrap();
    trace_state.add("rojo", "00f067aa0ba902b7").unwrap();

    let original = TraceContext::new(trace_id, span_id, None, TraceFlags::new(0x01), trace_state);

    let mut carrier = MapCarrier::new();
    let propagator = TraceContextPropagator::new();
    propagator.inject(&mut carrier, &original);

    let extracted = propagator.extract(&carrier);
    assert!(extracted.is_some());
    let extracted = extracted.unwrap();

    let orig_entries = original.trace_state.entries();
    let ext_entries = extracted.trace_state.entries();
    assert_eq!(orig_entries.len(), ext_entries.len());
    for (orig_entry, ext_entry) in orig_entries.iter().zip(ext_entries.iter()) {
        assert_eq!(orig_entry, ext_entry);
    }
}

#[test]
fn test_extraction_returns_none_when_no_traceparent() {
    let carrier = MapCarrier::new();
    let propagator = TraceContextPropagator::new();
    let result = propagator.extract(&carrier);
    assert!(result.is_none());
}

#[test]
fn test_extraction_returns_none_for_malformed_traceparent() {
    let mut carrier = MapCarrier::new();
    carrier.set("traceparent", "invalid-format");
    let propagator = TraceContextPropagator::new();
    let result = propagator.extract(&carrier);
    assert!(result.is_none());
}

#[test]
fn test_extraction_returns_none_for_zero_trace_id() {
    let mut carrier = MapCarrier::new();
    carrier.set(
        "traceparent",
        "00-00000000000000000000000000000000-b7ad6b7169203331-01",
    );
    let propagator = TraceContextPropagator::new();
    let result = propagator.extract(&carrier);
    assert!(result.is_none());
}

#[test]
fn test_correlation_roundtrip() {
    let original = CorrelationIdentifier::new();

    let mut carrier = MapCarrier::new();
    let propagator = CorrelationPropagator::new();
    propagator.inject(&mut carrier, &original);

    let extracted = propagator.extract(&carrier);
    assert!(extracted.is_some());
    let extracted = extracted.unwrap();

    assert_eq!(extracted.id(), original.id());
    assert_eq!(extracted.created_at(), original.created_at());
}

#[test]
fn test_correlation_extraction_returns_none_when_missing() {
    let carrier = MapCarrier::new();
    let propagator = CorrelationPropagator::new();
    let result = propagator.extract(&carrier);
    assert!(result.is_none());
}

#[test]
fn test_correlation_extraction_returns_none_for_invalid_uuid() {
    let mut carrier = MapCarrier::new();
    carrier.set("correlation-id", "not-a-uuid");
    let propagator = CorrelationPropagator::new();
    let result = propagator.extract(&carrier);
    assert!(result.is_none());
}

#[test]
fn test_baggage_roundtrip() {
    let mut original = Baggage::new();
    let entry = BaggageEntry::new("userId".to_string(), "alice".to_string());
    original.add_entry(entry).unwrap();

    let mut carrier = MapCarrier::new();
    let propagator = BaggagePropagator::new();
    propagator.inject(&mut carrier, &original);

    let extracted = propagator.extract(&carrier);
    assert!(extracted.is_some());
    let extracted = extracted.unwrap();

    assert_eq!(extracted.entries().len(), 1);
    assert_eq!(extracted.entries()[0].key, "userId");
    assert_eq!(extracted.entries()[0].value, Some("alice".to_string()));
}

#[test]
fn test_baggage_roundtrip_with_properties() {
    let mut original = Baggage::new();
    let mut entry = BaggageEntry::new("userId".to_string(), "alice".to_string());
    entry.properties.push(BaggageProperty::KeyValue {
        key: "type".to_string(),
        value: "admin".to_string(),
    });
    entry.properties.push(BaggageProperty::Flag {
        key: "internal".to_string(),
    });
    original.add_entry(entry).unwrap();

    let mut carrier = MapCarrier::new();
    let propagator = BaggagePropagator::new();
    propagator.inject(&mut carrier, &original);

    let extracted = propagator.extract(&carrier);
    assert!(extracted.is_some());
    let extracted = extracted.unwrap();

    assert_eq!(extracted.entries().len(), 1);
    let extracted_entry = &extracted.entries()[0];
    assert_eq!(extracted_entry.key, "userId");
    assert_eq!(extracted_entry.value, Some("alice".to_string()));
    assert_eq!(extracted_entry.properties.len(), 2);
    assert_eq!(
        extracted_entry.properties[0],
        BaggageProperty::KeyValue {
            key: "type".to_string(),
            value: "admin".to_string(),
        }
    );
    assert_eq!(
        extracted_entry.properties[1],
        BaggageProperty::Flag {
            key: "internal".to_string(),
        }
    );
}

#[test]
fn test_baggage_roundtrip_with_flag_entry() {
    let mut original = Baggage::new();
    let entry = BaggageEntry::flag("test-flag".to_string());
    original.add_entry(entry).unwrap();

    let mut carrier = MapCarrier::new();
    let propagator = BaggagePropagator::new();
    propagator.inject(&mut carrier, &original);

    let extracted = propagator.extract(&carrier);
    assert!(extracted.is_some());
    let extracted = extracted.unwrap();

    assert_eq!(extracted.entries().len(), 1);
    assert_eq!(extracted.entries()[0].key, "test-flag");
    assert_eq!(extracted.entries()[0].value, None);
}

#[test]
fn test_baggage_extraction_returns_none_when_missing() {
    let carrier = MapCarrier::new();
    let propagator = BaggagePropagator::new();
    let result = propagator.extract(&carrier);
    assert!(result.is_none());
}

#[test]
fn test_baggage_multi_hop() {
    let mut baggage = Baggage::new();
    let entry = BaggageEntry::new("key1".to_string(), "val1".to_string());
    baggage.add_entry(entry).unwrap();

    let propagator = BaggagePropagator::new();

    // Hop 1: service A -> service B
    let mut carrier_a = MapCarrier::new();
    propagator.inject(&mut carrier_a, &baggage);
    let at_b = propagator.extract(&carrier_a);
    assert!(at_b.is_some());

    // Hop 2: service B adds entry, forwards to service C
    let mut at_b = at_b.unwrap();
    let entry2 = BaggageEntry::new("key2".to_string(), "val2".to_string());
    at_b.add_entry(entry2).unwrap();

    let mut carrier_b = MapCarrier::new();
    propagator.inject(&mut carrier_b, &at_b);
    let at_c = propagator.extract(&carrier_b);
    assert!(at_c.is_some());
    let at_c = at_c.unwrap();

    // Hop 3: service C verifies all entries survive
    assert_eq!(at_c.entries().len(), 2);
    assert_eq!(at_c.entries()[0].key, "key1");
    assert_eq!(at_c.entries()[0].value, Some("val1".to_string()));
    assert_eq!(at_c.entries()[1].key, "key2");
    assert_eq!(at_c.entries()[1].value, Some("val2".to_string()));
}

#[test]
fn test_trace_context_propagator() {
    let trace_id = TraceId::from_str("0af7651916cd43dd8448eb211c80319c").unwrap();
    let span_id = SpanId::from_str("b7ad6b7169203331").unwrap();
    let original = TraceContext::new(
        trace_id,
        span_id,
        None,
        TraceFlags::new(0x01),
        TraceState::new(),
    );

    let mut carrier = MapCarrier::new();
    let propagator = TraceContextPropagator::new();
    propagator.inject(&mut carrier, &original);

    let extracted = propagator.extract(&carrier);
    assert!(extracted.is_some());
    let extracted = extracted.unwrap();
    assert_eq!(extracted.trace_id, original.trace_id);
    assert_eq!(extracted.span_id, original.span_id);
    assert_eq!(extracted.trace_flags, original.trace_flags);
}

#[test]
fn test_correlation_propagator() {
    let original = CorrelationIdentifier::new();

    let mut carrier = MapCarrier::new();
    let propagator = CorrelationPropagator::new();
    propagator.inject(&mut carrier, &original);

    let extracted = propagator.extract(&carrier);
    assert!(extracted.is_some());
    let extracted = extracted.unwrap();
    assert_eq!(extracted.id(), original.id());
    assert_eq!(extracted.created_at(), original.created_at());
}
