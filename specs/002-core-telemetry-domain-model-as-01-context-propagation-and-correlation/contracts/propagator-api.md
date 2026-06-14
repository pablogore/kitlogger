# Propagator API Contract

## Carrier Abstraction

### Injector

```rust
pub trait Injector {
    fn set(&mut self, key: &str, value: &str);
}
```

Sets a key-value pair on the transport carrier. Called by propagators during injection.

### Extractor

```rust
pub trait Extractor {
    fn get(&self, key: &str) -> Option<&str>;
    fn get_all(&self, key: &str) -> Vec<&str>;
}
```

Reads values from a transport carrier. Called by propagators during extraction.

### Propagator

```rust
pub trait Propagator {
    type Context;
    fn inject(&self, carrier: &mut dyn Injector, context: &Self::Context);
    fn extract(&self, carrier: &dyn Extractor) -> Option<Self::Context>;
    fn fields(&self) -> &'static [&'static str];
}
```

Generic context propagation contract.

`extract` returns `None` when extraction fails — the carrier contains no valid context data or the data is malformed. Implementations must not panic on malformed input. A `None` return signals that no context was available; callers should fall back to a fresh/default context as appropriate for their domain.

### MapCarrier

```rust
pub struct MapCarrier { /* HashMap<String, Vec<String>> */ }
impl Injector for MapCarrier { /* set */ }
impl Extractor for MapCarrier { /* get, get_all */ }
```

Reference carrier implementation for testing and in-process propagation.

## Propagator Implementations

### TraceContextPropagator

| Property | Value |
|----------|-------|
| Context type | `TraceContext` |
| Inject header | `traceparent: {version}-{trace_id}-{span_id}-{trace_flags}` |
| Extract from | `traceparent` header (W3C format) |
| Fields | `["traceparent", "tracestate", "parent-span-id"]` |

The `parent-span-id` header is a non-W3C extension that preserves `parent_span_id` (defined in the data model as `Option<SpanId>`). The W3C `traceparent` format has no field for parent span ID, so it is serialized as a separate header. When present, it takes precedence over any parent span relationship implied by the receiving span. Consumers that do not understand this convention silently ignore it.

### CorrelationPropagator

| Property | Value |
|----------|-------|
| Context type | `CorrelationIdentifier` |
| Inject header | `correlation-id: {uuid}` |
| Extract from | `correlation-id` header |
| Fields | `["correlation-id"]` |

### BaggagePropagator

| Property | Value |
|----------|-------|
| Context type | `Baggage` |
| Inject header | `baggage: {entries}` (W3C Baggage format) |
| Extract from | `baggage` header |
| Fields | `["baggage"]` |

## Serialization Formats

### W3C Trace Context (traceparent)

```
Format: {version}-{trace_id}-{span_id}-{trace_flags}
Example: 00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01
```

### W3C Trace Context (tracestate)

```
Format: {vendor}={value},{vendor}={value}
Example: congo=t61rcWkgMzE
```

### Correlation ID

```
Format: {uuid_v7_string}
Example: 018f3a6b-7c5b-7b00-9c8a-2b7a9e8f1c3d
```

### Parent Span ID

```
Format: {16-char-hex-span-id}
Example: 00f067aa0ba902b7
```

Non-W3C extension header. Serialized alongside `traceparent` when the TraceContext carries a `parent_span_id`. Preserves the parent-child span relationship across propagation hops.

### W3C Baggage

```
Format: {key}={value},{key}={value};{prop}
Example: userId=alice,serverNode=DF28
```
