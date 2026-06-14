use crate::baggage::{Baggage, BaggageEntry, BaggageProperty};
use crate::carrier::{Extractor, Injector, Propagator};
use crate::correlation::CorrelationIdentifier;
use crate::trace_context::{SpanId, TraceContext, TraceState};
use std::str::FromStr;

/// TraceContextPropagator for W3C Trace Context propagation
pub struct TraceContextPropagator;

impl TraceContextPropagator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TraceContextPropagator {
    fn default() -> Self {
        Self::new()
    }
}

impl Propagator for TraceContextPropagator {
    type Context = TraceContext;

    fn inject(&self, carrier: &mut dyn Injector, context: &Self::Context) {
        let traceparent = format!(
            "{:02x}-{}-{}-{:02x}",
            context.version, context.trace_id, context.span_id, context.trace_flags
        );
        carrier.set("traceparent", &traceparent);

        if !context.trace_state.entries().is_empty() {
            let tracestate_str = context
                .trace_state
                .entries()
                .iter()
                .map(|(key, value)| format!("{}={}", key, value))
                .collect::<Vec<_>>()
                .join(",");
            carrier.set("tracestate", &tracestate_str);
        }

        if let Some(parent_span_id) = &context.parent_span_id {
            carrier.set("parent-span-id", &parent_span_id.to_string());
        }
    }

    fn extract(&self, carrier: &dyn Extractor) -> Option<Self::Context> {
        let traceparent = carrier.get("traceparent")?;

        let context = TraceContext::from_str(traceparent).ok()?;

        let context = if let Some(tracestate_str) = carrier.get("tracestate") {
            let mut trace_state = TraceState::new();
            for entry in tracestate_str.split(',') {
                if let Some(pos) = entry.find('=') {
                    let key = &entry[..pos].trim();
                    let value = &entry[pos + 1..].trim();
                    if !key.is_empty() {
                        trace_state.add(key, value).ok();
                    }
                }
            }
            TraceContext {
                trace_state,
                ..context
            }
        } else {
            context
        };

        let context = if let Some(parent_id_str) = carrier.get("parent-span-id") {
            if let Ok(parent_id) = SpanId::from_str(parent_id_str) {
                TraceContext {
                    parent_span_id: Some(parent_id),
                    ..context
                }
            } else {
                context
            }
        } else {
            context
        };

        if !context.is_valid() {
            return None;
        }

        Some(context)
    }

    fn fields(&self) -> &'static [&'static str] {
        &["traceparent", "tracestate", "parent-span-id"]
    }
}

/// CorrelationPropagator for correlation identifier propagation
pub struct CorrelationPropagator;

impl CorrelationPropagator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CorrelationPropagator {
    fn default() -> Self {
        Self::new()
    }
}

impl Propagator for CorrelationPropagator {
    type Context = CorrelationIdentifier;

    fn inject(&self, carrier: &mut dyn Injector, context: &Self::Context) {
        carrier.set("correlation-id", &context.id().to_string());
    }

    fn extract(&self, carrier: &dyn Extractor) -> Option<Self::Context> {
        let id_str = carrier.get("correlation-id")?;
        let uuid = id_str.parse::<uuid::Uuid>().ok()?;
        Some(CorrelationIdentifier::from_uuid(uuid))
    }

    fn fields(&self) -> &'static [&'static str] {
        &["correlation-id"]
    }
}

/// BaggagePropagator for W3C Baggage propagation
pub struct BaggagePropagator;

impl BaggagePropagator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BaggagePropagator {
    fn default() -> Self {
        Self::new()
    }
}

impl Propagator for BaggagePropagator {
    type Context = Baggage;

    fn inject(&self, carrier: &mut dyn Injector, context: &Self::Context) {
        let baggage_str = context
            .entries()
            .iter()
            .map(|entry| {
                let mut s = if let Some(value) = &entry.value {
                    format!("{}={}", entry.key, value)
                } else {
                    entry.key.clone()
                };
                for prop in &entry.properties {
                    match prop {
                        BaggageProperty::KeyValue { key, value } => {
                            s.push_str(&format!(";{}={}", key, value));
                        }
                        BaggageProperty::Flag { key } => {
                            s.push_str(&format!(";{}", key));
                        }
                    }
                }
                s
            })
            .collect::<Vec<_>>()
            .join(",");

        if !baggage_str.is_empty() {
            carrier.set("baggage", &baggage_str);
        }
    }

    fn extract(&self, carrier: &dyn Extractor) -> Option<Self::Context> {
        let baggage_str = carrier.get("baggage")?;
        let mut baggage = Baggage::new();

        for raw_entry in baggage_str.split(',') {
            let raw_entry = raw_entry.trim();
            if raw_entry.is_empty() {
                continue;
            }

            let mut parts = raw_entry.split(';');

            let key_value = parts.next().unwrap_or("");
            if key_value.is_empty() {
                continue;
            }

            let (key, value) = if let Some(pos) = key_value.find('=') {
                let k = &key_value[..pos];
                let v = &key_value[pos + 1..];
                (k.to_string(), Some(v.to_string()))
            } else {
                (key_value.to_string(), None)
            };

            let mut entry = if let Some(v) = value {
                BaggageEntry::new(key, v)
            } else {
                BaggageEntry::flag(key)
            };

            for prop_str in parts {
                let prop_str = prop_str.trim();
                if prop_str.is_empty() {
                    continue;
                }
                if let Some(pos) = prop_str.find('=') {
                    let pk = &prop_str[..pos];
                    let pv = &prop_str[pos + 1..];
                    entry.properties.push(BaggageProperty::KeyValue {
                        key: pk.to_string(),
                        value: pv.to_string(),
                    });
                } else {
                    entry.properties.push(BaggageProperty::Flag {
                        key: prop_str.to_string(),
                    });
                }
            }

            baggage.add_entry(entry).ok();
        }

        if baggage.entries().is_empty() {
            return None;
        }

        Some(baggage)
    }

    fn fields(&self) -> &'static [&'static str] {
        &["baggage"]
    }
}
