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
    fn extract(&self, carrier: &dyn Extractor) -> Self::Context;
    fn fields(&self) -> &'static [&'static str];
}
```

Generic context propagation contract.

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
| Fields | `["traceparent", "tracestate"]` |

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

### W3C Baggage

```
Format: {key}={value},{key}={value};{prop}
Example: userId=alice,serverNode=DF28
```
