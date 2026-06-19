# Data Model: Context Propagation and Correlation

## Entities

### TraceId
16-byte identifier for a distributed trace.

| Field | Type | Description |
|-------|------|-------------|
| bytes | `[u8; 16]` | Raw trace identifier bytes |

**Validation**: Must not be all zeros. Must be exactly 16 bytes.
**Display**: 32-character lowercase hex string.

### SpanId
8-byte identifier for a single span within a trace.

| Field | Type | Description |
|-------|------|-------------|
| bytes | `[u8; 8]` | Raw span identifier bytes |

**Validation**: Must not be all zeros. Must be exactly 8 bytes.
**Display**: 16-character lowercase hex string.

### TraceFlags
8-bit flags for trace options.

| Bit | Flag | Description |
|-----|------|-------------|
| 0 | Sampled | When set, the trace should be sampled |
| 1-7 | Reserved | Must not be interpreted |

**Validation**: None beyond valid u8 range.

### TraceState
Vendor-specific trace state entries (key-value pairs).

| Field | Type | Description |
|-------|------|-------------|
| entries | `Vec<(String, String)>` | Vendor-specific key-value entries |

**Validation**: Max 32 entries. Each key max 256 chars. Each value max 256 chars.

### TraceContext
Complete W3C Trace Context representation.

| Field | Type | Description |
|-------|------|-------------|
| version | `u8` | Format version (currently 0) |
| trace_id | `TraceId` | Trace identifier |
| span_id | `SpanId` | Span identifier |
| parent_span_id | `Option<SpanId>` | Parent span identifier |
| trace_flags | `TraceFlags` | Trace options flags |
| trace_state | `TraceState` | Vendor-specific state |

**Validation**: `is_valid()` returns false if version == 0xFF, trace_id is zero, span_id is zero, or parent_span_id is zero.

### CorrelationIdentifier
Globally unique identifier for cross-signal correlation.

| Field | Type | Description |
|-------|------|-------------|
| id | `Uuid` | UUID v7 identifier |
| created_at | `i64` | Creation timestamp (ms since epoch) |

**Validation**: `is_valid()` returns false if UUID is nil.

### BaggageProperty
A property attached to a baggage entry (key-value or flag).

| Variant | Fields | Description |
|---------|--------|-------------|
| KeyValue | key: String, value: String | Key-value property |
| Flag | key: String | Flag property (key only) |

### BaggageEntry
A single baggage entry with key, optional value, and properties.

| Field | Type | Description |
|-------|------|-------------|
| key | `String` | Entry key |
| value | `Option<String>` | Entry value |
| properties | `Vec<BaggageProperty>` | Entry properties |

### Baggage
Container for W3C Baggage entries.

| Field | Type | Description |
|-------|------|-------------|
| entries | `Vec<BaggageEntry>` | Baggage entries |
| total_size | `usize` | Cumulative entry size (bytes) |

**Validation**: Max 180 entries. Max total size 64KB.

### PropagationMetadata
Transport-specific metadata required for context carriage.

| Field | Type | Description |
|-------|------|-------------|
| transport | `String` | Transport protocol name (e.g., "http", "grpc", "kafka") |
| entries | `Vec<(String, String)>` | Key-value metadata entries for the transport binding |

**Methods**: `new(transport)` creates empty metadata, `add(key, value)` appends an entry, `get(key)` retrieves first value, `keys()` iterates entry keys, `is_empty()` checks for entries. `Default` uses transport `"unknown"`.

## Relationships

```text
TraceContext
├── trace_id: TraceId (1:1)
├── span_id: SpanId (1:1)
├── parent_span_id: Option<SpanId> (0:1)
├── trace_flags: TraceFlags (1:1)
└── trace_state: TraceState (1:1)

CorrelationIdentifier (standalone, referenced by Context)

Baggage
└── entries: Vec<BaggageEntry> (1:N)
    └── properties: Vec<BaggageProperty> (1:N)
```

## Carrier & Propagator Contracts

### Injector
Trait for setting key-value pairs in a transport carrier.

| Method | Signature | Description |
|--------|-----------|-------------|
| set | `(&mut self, key: &str, value: &str)` | Set a header/field value |

### Extractor
Trait for reading values from a transport carrier.

| Method | Signature | Description |
|--------|-----------|-------------|
| get | `(&self, key: &str) -> Option<&str>` | Get first value for a key |
| get_all | `(&self, key: &str) -> Vec<&str>` | Get all values for a key |

### Propagator
Generic trait for context injection and extraction.

| Method | Signature | Description |
|--------|-----------|-------------|
| inject | `(&self, carrier: &mut dyn Injector, context: &Self::Context)` | Inject context into carrier |
| extract | `(&self, carrier: &dyn Extractor) -> Option<Self::Context>` | Extract context from carrier, returns `None` if extraction fails (e.g., missing or malformed carrier data) |
| fields | `(&self) -> &'static [&'static str]` | Fields used by this propagator |

### MapCarrier
HashMap-based carrier implementing Injector and Extractor. Keys map to `Vec<String>` values.

## State Transitions

### TraceContext Lifecycle
```text
[New Span] → inject(MapCarrier) → [Serialized Headers]
[Serialized Headers] → extract(MapCarrier) → [TraceContext]
```

### CorrelationIdentifier Lifecycle
```text
[New Operation] → generate() → [CorrelationIdentifier]
                    → inject()   → [Serialized in Carrier]
[Serialized]     → extract()    → [CorrelationIdentifier]
```

### Baggage Lifecycle
```text
[New Scope] → add_entry() → [Baggage with entries]
            → inject()   → [Serialized Baggage Header]
[Header]    → extract()  → [Baggage with entries]
```
