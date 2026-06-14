# Carrier Contract

The Carrier abstraction decouples context propagation from transport protocols.

## Traits

### Injector

```rust
/// Allows writing key-value pairs to a carrier (e.g., headers, metadata map).
pub trait Injector {
    /// Set a header value for the given key.
    fn set(&mut self, key: &str, value: &str);
}
```

### Extractor

```rust
/// Allows reading key-value pairs from a carrier (e.g., headers, metadata map).
pub trait Extractor {
    /// Get a header value for the given key.
    fn get(&self, key: &str) -> Option<&str>;

    /// Get all header values for the given key.
    fn get_all(&self, key: &str) -> Vec<&str>;
}
```

### Propagator (Composite)

```rust
/// Combines injection and extraction for a context type.
pub trait Propagator: Send + Sync {
    /// The type of context this propagator handles.
    type Context: Clone + Send + Sync;

    /// Inject context into a carrier for outgoing requests.
    fn inject<C: Injector + ?Sized>(&self, context: &Self::Context, carrier: &mut C);

    /// Extract context from a carrier for incoming requests.
    fn extract<C: Extractor + ?Sized>(&self, carrier: &C) -> Self::Context;

    /// The fields (header names) this propagator may read or write.
    fn fields(&self) -> Vec<&str>;
}
```

## Implementations

### MapCarrier

Generic `HashMap<String, String>` carrier for testing and transport-agnostic usage. Transport-specific carrier implementations are provided by AS-02 Transport-Agnostic Telemetry Flow.

## Contract Tests

All carrier implementations must pass:
1. Round-trip: inject then extract returns original values
2. Multi-value: multiple values for the same key are preserved
3. Empty: extracting missing key returns `None`
4. Special chars: keys and values with special characters round-trip correctly
