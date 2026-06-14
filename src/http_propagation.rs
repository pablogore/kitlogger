//! HTTP propagation implementations for telemetry context
//!
//! This module contains implementations of propagators specifically for HTTP headers.

use crate::carrier::{Injector, Extractor, Propagator};
use crate::trace_context::{TraceContext, TraceId, SpanId, TraceFlags};
use crate::correlation::CorrelationIdentifier;
use crate::baggage::{Baggage, BaggageEntry};
use std::str::FromStr;

/// HTTPTraceContextPropagator for W3C Trace Context propagation over HTTP
pub struct HttpTraceContextPropagator;

impl HttpTraceContextPropagator {
    /// Create a new HttpTraceContextPropagator
    pub fn new() -> Self {
        Self
    }
}

impl Default for HttpTraceContextPropagator {
    fn default() -> Self {
        Self::new()
    }
}

impl Propagator for HttpTraceContextPropagator {
    type Context = TraceContext;
    
    fn inject(&self, carrier: &mut dyn Injector, context: &Self::Context) {
        // Inject traceparent header
        let traceparent = format!(
            "{:02x}-{}-{}-{:02x}",
            context.version,
            context.trace_id,
            context.span_id,
            context.trace_flags
        );
        carrier.set("traceparent", &traceparent);
        
        // Inject tracestate header if needed
        if !context.trace_state.entries().is_empty() {
            // Serialize tracestate entries into a comma-separated string
            let tracestate_str = context.trace_state.entries()
                .iter()
                .map(|(key, value)| format!("{}={}", key, value))
                .collect::<Vec<_>>()
                .join(",");
            carrier.set("tracestate", &tracestate_str);
        }
    }
    
    fn extract(&self, carrier: &dyn Extractor) -> Self::Context {
        // Extract traceparent header
        if let Some(traceparent) = carrier.get("traceparent") {
            // Parse the traceparent header
            if let Ok(context) = TraceContext::from_str(traceparent) {
                return context;
            }
        }
        
        // Return empty context if parsing fails
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

/// HttpCorrelationPropagator for correlation identifier propagation over HTTP
pub struct HttpCorrelationPropagator;

impl HttpCorrelationPropagator {
    /// Create a new HttpCorrelationPropagator
    pub fn new() -> Self {
        Self
    }
}

impl Default for HttpCorrelationPropagator {
    fn default() -> Self {
        Self::new()
    }
}

impl Propagator for HttpCorrelationPropagator {
    type Context = CorrelationIdentifier;
    
    fn inject(&self, carrier: &mut dyn Injector, context: &Self::Context) {
        // Inject correlation-id header
        carrier.set("correlation-id", &context.id().to_string());
    }
    
    fn extract(&self, carrier: &dyn Extractor) -> Self::Context {
        // Extract correlation-id header
        if let Some(id_str) = carrier.get("correlation-id") {
            if let Ok(uuid) = id_str.parse::<uuid::Uuid>() {
                return CorrelationIdentifier::from_uuid(uuid);
            }
        }
        
        // Generate a new correlation identifier if none found
        CorrelationIdentifier::new()
    }
    
    fn fields(&self) -> &'static [&'static str] {
        &["correlation-id"]
    }
}

/// HttpBaggagePropagator for W3C Baggage propagation over HTTP
pub struct HttpBaggagePropagator;

impl HttpBaggagePropagator {
    /// Create a new HttpBaggagePropagator
    pub fn new() -> Self {
        Self
    }
}

impl Default for HttpBaggagePropagator {
    fn default() -> Self {
        Self::new()
    }
}

impl Propagator for HttpBaggagePropagator {
    type Context = Baggage;
    
    fn inject(&self, carrier: &mut dyn Injector, context: &Self::Context) {
        // Inject baggage header
        // Serialize baggage entries into a comma-separated string
        let baggage_str = context.entries()
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
        // Extract baggage header
        if let Some(baggage_str) = carrier.get("baggage") {
            let mut baggage = Baggage::new();
            // Parse baggage entries from comma-separated string
            for entry in baggage_str.split(',') {
                if let Some(pos) = entry.find('=') {
                    let key = &entry[..pos];
                    let value = &entry[pos + 1..];
                    let baggage_entry = BaggageEntry::new(key.to_string(), value.to_string());
                    baggage.add_entry(baggage_entry).unwrap(); // Safe to unwrap since we're just testing
                } else {
                    // Flag entry (no value)
                    let baggage_entry = BaggageEntry::flag(entry.to_string());
                    baggage.add_entry(baggage_entry).unwrap(); // Safe to unwrap since we're just testing
                }
            }
            return baggage;
        }
        
        // Return empty baggage if none found
        Baggage::new()
    }
    
    fn fields(&self) -> &'static [&'static str] {
        &["baggage"]
    }
}