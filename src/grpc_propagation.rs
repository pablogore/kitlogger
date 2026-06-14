//! gRPC metadata propagation implementations for telemetry context
//!
//! This module contains implementations of propagators specifically for gRPC metadata.

use crate::baggage::{Baggage, BaggageEntry};
use crate::carrier::{Extractor, Injector, Propagator};
use crate::correlation::CorrelationIdentifier;
use crate::trace_context::{TraceContext, TraceId, SpanId, TraceFlags};
use std::str::FromStr;

/// GrpcTraceContextPropagator for W3C Trace Context propagation over gRPC metadata
pub struct GrpcTraceContextPropagator;

impl GrpcTraceContextPropagator {
    /// Create a new GrpcTraceContextPropagator
    pub fn new() -> Self {
        Self
    }
}

impl Default for GrpcTraceContextPropagator {
    fn default() -> Self {
        Self::new()
    }
}

impl Propagator for GrpcTraceContextPropagator {
    type Context = TraceContext;

    fn inject(&self, carrier: &mut dyn Injector, context: &Self::Context) {
        let traceparent = format!(
            "{:02x}-{}-{}-{:02x}",
            context.version,
            context.trace_id,
            context.span_id,
            context.trace_flags
        );
        carrier.set("traceparent", &traceparent);
    }

    fn extract(&self, carrier: &dyn Extractor) -> Self::Context {
        if let Some(traceparent) = carrier.get("traceparent") {
            if let Ok(context) = TraceContext::from_str(traceparent) {
                return context;
            }
        }

        TraceContext::new(
            TraceId::new([0; 16]),
            SpanId::new([0; 8]),
            None,
            TraceFlags::new(0),
            Default::default(),
        )
    }

    fn fields(&self) -> &'static [&'static str] {
        &["traceparent", "tracestate"]
    }
}

/// GrpcCorrelationPropagator for correlation identifier propagation over gRPC metadata
pub struct GrpcCorrelationPropagator;

impl GrpcCorrelationPropagator {
    /// Create a new GrpcCorrelationPropagator
    pub fn new() -> Self {
        Self
    }
}

impl Default for GrpcCorrelationPropagator {
    fn default() -> Self {
        Self::new()
    }
}

impl Propagator for GrpcCorrelationPropagator {
    type Context = CorrelationIdentifier;

    fn inject(&self, carrier: &mut dyn Injector, context: &Self::Context) {
        carrier.set("correlation-id", &context.id().to_string());
    }

    fn extract(&self, carrier: &dyn Extractor) -> Self::Context {
        if let Some(id_str) = carrier.get("correlation-id") {
            if let Ok(uuid) = id_str.parse::<uuid::Uuid>() {
                return CorrelationIdentifier::from_uuid(uuid);
            }
        }

        CorrelationIdentifier::new()
    }

    fn fields(&self) -> &'static [&'static str] {
        &["correlation-id"]
    }
}

/// GrpcBaggagePropagator for W3C Baggage propagation over gRPC metadata
pub struct GrpcBaggagePropagator;

impl GrpcBaggagePropagator {
    /// Create a new GrpcBaggagePropagator
    pub fn new() -> Self {
        Self
    }
}

impl Default for GrpcBaggagePropagator {
    fn default() -> Self {
        Self::new()
    }
}

impl Propagator for GrpcBaggagePropagator {
    type Context = Baggage;

    fn inject(&self, carrier: &mut dyn Injector, context: &Self::Context) {
        let baggage_str = context
            .entries()
            .iter()
            .map(|entry| {
                if let Some(value) = &entry.value {
                    format!("{}={}", entry.key, value)
                } else {
                    entry.key.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(",");

        if !baggage_str.is_empty() {
            carrier.set("baggage", &baggage_str);
        }
    }

    fn extract(&self, carrier: &dyn Extractor) -> Self::Context {
        if let Some(baggage_str) = carrier.get("baggage") {
            let mut baggage = Baggage::new();
            for entry in baggage_str.split(',') {
                if let Some(pos) = entry.find('=') {
                    let key = &entry[..pos];
                    let value = &entry[pos + 1..];
                    let baggage_entry =
                        BaggageEntry::new(key.to_string(), value.to_string());
                    baggage.add_entry(baggage_entry).ok();
                } else {
                    let baggage_entry = BaggageEntry::flag(entry.to_string());
                    baggage.add_entry(baggage_entry).ok();
                }
            }
            return baggage;
        }

        Baggage::new()
    }

    fn fields(&self) -> &'static [&'static str] {
        &["baggage"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::carrier::MapCarrier;
    use crate::trace_context::TraceState;

    #[test]
    fn test_trace_context_propagation() {
        let mut carrier = MapCarrier::new();
        let propagator = GrpcTraceContextPropagator::new();

        let trace_id =
            TraceId::from_str("0af7651916cd43dd8448eb211c80319c").unwrap();
        let span_id = SpanId::from_str("b7ad6b7169203331").unwrap();
        let trace_flags = TraceFlags::new(0x01);
        let trace_state = TraceState::new();
        let trace_context = TraceContext::new(
            trace_id,
            span_id,
            None,
            trace_flags,
            trace_state,
        );

        propagator.inject(&mut carrier, &trace_context);

        let extracted = propagator.extract(&carrier);
        assert_eq!(trace_context, extracted);
    }

    #[test]
    fn test_correlation_identifier_propagation() {
        let mut carrier = MapCarrier::new();
        let propagator = GrpcCorrelationPropagator::new();

        let correlation_id = CorrelationIdentifier::new();

        propagator.inject(&mut carrier, &correlation_id);

        let extracted = propagator.extract(&carrier);
        assert_eq!(correlation_id, extracted);
    }

    #[test]
    fn test_baggage_propagation() {
        let mut carrier = MapCarrier::new();
        let propagator = GrpcBaggagePropagator::new();

        let mut baggage = Baggage::new();
        let entry =
            BaggageEntry::new("test-key".to_string(), "test-value".to_string());
        baggage.add_entry(entry).unwrap();

        propagator.inject(&mut carrier, &baggage);

        let extracted = propagator.extract(&carrier);
        assert_eq!(baggage, extracted);
    }

    #[test]
    fn test_extract_empty() {
        let carrier = MapCarrier::new();

        let tc_propagator = GrpcTraceContextPropagator::new();
        let corr_propagator = GrpcCorrelationPropagator::new();
        let baggage_propagator = GrpcBaggagePropagator::new();

        let trace_context = tc_propagator.extract(&carrier);
        assert!(!trace_context.is_valid());

        let correlation_id = corr_propagator.extract(&carrier);
        assert_ne!(correlation_id.id().to_string(), "");

        let baggage = baggage_propagator.extract(&carrier);
        assert!(baggage.entries().is_empty());
    }
}
