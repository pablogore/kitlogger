//! Tests for trace context implementation

use context_propagation::carrier::{Injector, MapCarrier, Propagator};
use context_propagation::trace_context::{SpanId, TraceContext, TraceFlags, TraceId, TraceState};
use std::str::FromStr;

#[test]
fn test_trace_context_creation() {
    let trace_id = TraceId::from_str("0af7651916cd43dd8448eb211c80319c").unwrap();
    let span_id = SpanId::from_str("b7ad6b7169203331").unwrap();
    let trace_flags = TraceFlags::new(0x01);
    let trace_state = TraceState::new();

    let trace_context = TraceContext::new(trace_id, span_id, None, trace_flags, trace_state);

    assert_eq!(trace_context.trace_id, trace_id);
    assert_eq!(trace_context.span_id, span_id);
    assert_eq!(trace_context.trace_flags, trace_flags);
}

#[test]
fn test_trace_context_from_str() {
    let traceparent = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
    let context = TraceContext::from_str(traceparent).unwrap();

    assert_eq!(
        context.trace_id.to_string(),
        "0af7651916cd43dd8448eb211c80319c"
    );
    assert_eq!(context.span_id.to_string(), "b7ad6b7169203331");
    assert_eq!(context.trace_flags.as_u8(), 0x01);
}

#[test]
fn test_trace_context_from_str_invalid() {
    let traceparent = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01-";
    let result = TraceContext::from_str(traceparent);

    assert!(result.is_err());
}

#[test]
fn test_trace_context_from_str_invalid_version() {
    let traceparent = "ff-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
    let result = TraceContext::from_str(traceparent);

    assert!(result.is_err());
}

#[test]
fn test_trace_context_from_str_invalid_trace_id() {
    let traceparent = "00-00000000000000000000000000000000-b7ad6b7169203331-01";
    let context = TraceContext::from_str(traceparent).unwrap();

    assert_eq!(
        context.trace_id.to_string(),
        "00000000000000000000000000000000"
    );
}

#[test]
fn test_trace_context_from_str_invalid_span_id() {
    let traceparent = "00-0af7651916cd43dd8448eb211c80319c-0000000000000000-01";
    let context = TraceContext::from_str(traceparent).unwrap();

    assert_eq!(context.span_id.to_string(), "0000000000000000");
}

#[test]
fn test_trace_context_roundtrip() {
    // Create a trace context
    let trace_id = TraceId::from_str("0af7651916cd43dd8448eb211c80319c").unwrap();
    let span_id = SpanId::from_str("b7ad6b7169203331").unwrap();
    let trace_flags = TraceFlags::new(0x01);
    let trace_context = TraceContext::new(trace_id, span_id, None, trace_flags, TraceState::new());

    // Create a carrier and inject the context
    let mut carrier = MapCarrier::new();
    let propagator = context_propagation::propagation::TraceContextPropagator::new();
    propagator.inject(&mut carrier, &trace_context);

    // Extract the context back
    let extracted = propagator.extract(&carrier);
    assert!(extracted.is_some());
    let extracted_context = extracted.unwrap();

    // Verify they match
    assert_eq!(trace_context.trace_id, extracted_context.trace_id);
    assert_eq!(trace_context.span_id, extracted_context.span_id);
    assert_eq!(trace_context.trace_flags, extracted_context.trace_flags);
    assert_eq!(
        trace_context.parent_span_id,
        extracted_context.parent_span_id
    );
}

#[test]
fn test_trace_context_display_format() {
    let trace_id = TraceId::from_str("0af7651916cd43dd8448eb211c80319c").unwrap();
    let span_id = SpanId::from_str("b7ad6b7169203331").unwrap();
    let trace_flags = TraceFlags::new(0x01);

    let context = TraceContext::new(trace_id, span_id, None, trace_flags, TraceState::new());

    let display_string = format!("{}", context);
    assert_eq!(
        display_string,
        "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
    );
}

#[test]
fn test_trace_context_invalid_version() {
    // Test with version 0xFF which should be invalid
    let traceparent = "ff-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
    let result = TraceContext::from_str(traceparent);
    assert!(result.is_err());
}

#[test]
fn test_multi_hop_propagation() {
    let trace_id = TraceId::from_str("0af7651916cd43dd8448eb211c80319c").unwrap();
    let span_id = SpanId::from_str("b7ad6b7169203331").unwrap();
    let original = TraceContext::new(
        trace_id,
        span_id,
        None,
        TraceFlags::new(0x01),
        TraceState::new(),
    );
    let propagator = context_propagation::propagation::TraceContextPropagator::new();

    let mut context = original.clone();
    for hop in 1..=5 {
        let mut carrier = MapCarrier::new();
        propagator.inject(&mut carrier, &context);
        let extracted = propagator.extract(&carrier);
        assert!(extracted.is_some(), "extraction failed at hop {}", hop);
        context = extracted.unwrap();

        assert_eq!(
            context.trace_id, original.trace_id,
            "trace_id mismatch at hop {}",
            hop
        );
        assert_eq!(
            context.trace_flags, original.trace_flags,
            "trace_flags mismatch at hop {}",
            hop
        );
    }
}

#[test]
fn test_malformed_context_handling() {
    let propagator = context_propagation::propagation::TraceContextPropagator::new();

    // Empty carrier
    let carrier = MapCarrier::new();
    assert!(propagator.extract(&carrier).is_none());

    // Malformed traceparent
    let mut carrier = MapCarrier::new();
    carrier.set("traceparent", "not-a-traceparent");
    assert!(propagator.extract(&carrier).is_none());

    // Wrong number of parts
    let mut carrier = MapCarrier::new();
    carrier.set("traceparent", "00-abc-def-01-extra");
    assert!(propagator.extract(&carrier).is_none());

    // Invalid version (0xFF)
    let mut carrier = MapCarrier::new();
    carrier.set(
        "traceparent",
        "ff-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01",
    );
    assert!(propagator.extract(&carrier).is_none());

    // Zero trace ID (invalid domain entity)
    let mut carrier = MapCarrier::new();
    carrier.set(
        "traceparent",
        "00-00000000000000000000000000000000-b7ad6b7169203331-01",
    );
    assert!(propagator.extract(&carrier).is_none());

    // Zero span ID (invalid domain entity)
    let mut carrier = MapCarrier::new();
    carrier.set(
        "traceparent",
        "00-0af7651916cd43dd8448eb211c80319c-0000000000000000-01",
    );
    assert!(propagator.extract(&carrier).is_none());
}

#[test]
fn test_trace_context_zero_trace_id() {
    let trace_id = TraceId::from_str("00000000000000000000000000000000").unwrap();
    let span_id = SpanId::from_str("b7ad6b7169203331").unwrap();
    let trace_flags = TraceFlags::new(0x01);

    let context = TraceContext::new(trace_id, span_id, None, trace_flags, TraceState::new());

    assert!(!context.is_valid());
}

#[test]
fn test_tracestate_max_entries() {
    let mut state = TraceState::new();
    for i in 0..32 {
        assert!(state.add(&format!("key{}", i), &format!("value{}", i)).is_ok());
    }
    let result = state.add("overflow", "value");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Maximum number of trace state entries exceeded"
    );
}
