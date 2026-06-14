# Propagation Contract

Defines the context propagators for each context type.

## TraceContextPropagator

Implements W3C Trace Context propagation.

| Method | Behavior |
|--------|----------|
| `inject` | Sets `traceparent` and `tracestate` headers on carrier |
| `extract` | Parses `traceparent` and `tracestate` headers from carrier |
| `fields` | Returns `["traceparent", "tracestate"]` |

### Contract Tests
1. Valid `traceparent` header extracts correct trace_id, span_id, flags
2. Invalid `traceparent` (all-zeros, bad format) returns empty context
3. `tracestate` entries are preserved on extract and re-inject
4. Sampled flag is correctly read and written
5. Round-trip: inject(extract(carrier)) preserves context

## CorrelationPropagator

Propagates the cross-signal correlation identifier.

| Method | Behavior |
|--------|----------|
| `inject` | Sets `correlation-id` header on carrier |
| `extract` | Parses `correlation-id` header from carrier |
| `fields` | Returns `["correlation-id"]` |

### Contract Tests
1. Valid UUID v7 header extracts correct identifier
2. Missing header returns new generated correlation ID
3. Round-trip: inject then extract returns same UUID

## BaggagePropagator

Implements W3C Baggage propagation.

| Method | Behavior |
|--------|----------|
| `inject` | Sets `baggage` header on carrier |
| `extract` | Parses `baggage` header from carrier |
| `fields` | Returns `["baggage"]` |

### Contract Tests
1. Valid `baggage` header extracts correct key-value pairs
2. Properties are preserved on round-trip
3. Empty baggage extracts as empty entries list
4. Invalid entries are skipped without failing entire parse

## CompositePropagator

Combines multiple propagators into one injection/extraction pass.

```rust
pub struct CompositePropagator {
    propagators: Vec<Box<dyn Propagator>>,
}
```

### Contract Tests
1. All registered propagators inject their fields
2. All registered propagators extract their context
3. Adding a propagator does not affect existing propagators
