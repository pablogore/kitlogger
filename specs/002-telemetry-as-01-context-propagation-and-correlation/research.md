# Research: Context Propagation and Correlation

## W3C Trace Context

- **Decision**: Adopt W3C Trace Context v1 as the canonical trace context format
- **Rationale**: Industry standard, OpenTelemetry-compatible, supported by all major observability platforms
- **Alternatives considered**: N/A (mandated by parent capability's OpenTelemetry-compatible model)

### Key details
- `traceparent` header: `00-{trace-id-32hex}-{span-id-16hex}-{flags-2hex}` (55 chars fixed)
- `tracestate` header: comma-separated `key=value` vendor entries
- Trace ID: 128-bit (16 bytes), Span ID: 64-bit (8 bytes)
- Trace flags: bitfield (bit 0 = sampled, bit 1 = random-trace-id in Level 2)
- Stack-allocated parsing for `traceparent` (fixed length); `tracestate` requires heap for variable entries
- Rust: `TraceId([u8; 16])`, `SpanId([u8; 8])`, `TraceFlags(u8)` with bitmask operations

## W3C Baggage

- **Decision**: Adopt W3C Baggage as the canonical baggage format
- **Rationale**: Complements W3C Trace Context, OpenTelemetry-compatible, standard key-value propagation
- **Alternatives considered**: N/A (mandated by OpenTelemetry-compatible model)

### Key details
- Header: `baggage: key1=value1,key2=value2`
- Key: printable ASCII, max 256 chars, `key` or `key@system_id` for multi-tenant
- Value: URL-percent-encoded printable ASCII, max 256 chars after decoding
- Properties: optional per-entry metadata `;property` or `;key=value`
- Max 180 entries across 64KB total header size recommended

## Correlation Identifier Generation

- **Decision**: Use UUID v7 (time-ordered UUID) for correlation identifiers
- **Rationale**: Time-sortable, globally unique, 128-bit, no central coordination needed
- **Alternatives considered**: UUID v4 (not sortable), ULID (non-standard), Snowflake (requires coordination)

### Key details
- 128-bit identifier: 48-bit Unix timestamp ms + 74-bit random + 2-bit version
- Time-sortable: enables chronological ordering without separate timestamp
- Rust: `uuid` crate with `uuid::Uuid::new_v7()` (v7 feature gate)
- Constraint: UUID v7 is not yet stabilized in all UUID library versions; use `uuid` crate >= 1.3 with `v7` feature

## Propagation Abstraction

- **Decision**: Carrier pattern for inject/extract abstraction
- **Rationale**: Matches OpenTelemetry `TextMapPropagator` pattern, transport-agnostic
- **Alternatives considered**: Direct header manipulation (tightly coupled to transport)

### Key details
- `Injector` trait: write key-value pairs to a carrier
- `Extractor` trait: read key-value pairs from a carrier
- `Propagator` trait: inject/extract context from a carrier
- One propagator per context type (TraceContext, Baggage, Correlation)
- Composite propagator for combined injection/extraction

## Technology Declaration Check

All technologies used:
- **Rust** ✅ declared in `.specify/tech-stack.yaml`
- **Tokio** ✅ declared in `.specify/tech-stack.yaml` (runtime)
- **cargo test** ✅ declared in `.specify/tech-stack.yaml` (testing)
- **OpenTelemetry** ✅ declared in `.specify/tech-stack.yaml` (observability)

No technology violations.
