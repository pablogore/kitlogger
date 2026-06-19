# Research: Context Propagation and Correlation

## 1. Context Propagation Standard

- **Decision**: W3C Trace Context specification
- **Rationale**: Industry standard for distributed tracing context propagation; adopted by OpenTelemetry; supports trace-id, span-id, trace-flags, and tracestate
- **Alternatives considered**: N/A (industry standard, mandated by architecture for OpenTelemetry compatibility)

## 2. Correlation Identifier Strategy

- **Decision**: UUID v7 (time-ordered) for correlation identifiers
- **Rationale**: UUID v7 provides globally unique identifiers with embedded timestamps for time-ordered correlation; available via `uuid` crate with `v7` feature
- **Alternatives considered**: ULID (not in dependency tree), UUID v4 (no temporal ordering), Snowflake-style (requires coordination)

## 3. Baggage Propagation Standard

- **Decision**: W3C Baggage specification
- **Rationale**: Industry standard for application context propagation; supports key-value pairs with properties; adopted by OpenTelemetry
- **Alternatives considered**: N/A (industry standard for baggage propagation)

## 4. Carrier Abstraction

- **Decision**: Typed Injector/Extractor traits with dynamic dispatch
- **Rationale**: Allows any carrier (HTTP headers, gRPC metadata, message envelopes) to participate in propagation without coupling to specific transport
- **Alternatives considered**: Generic associated types (over-engineered for current needs), closure-based carriers (less discoverable)

## 5. Propagator Pattern

- **Decision**: Separate propagators per context type (TraceContext, Correlation, Baggage) with unified Propagator trait
- **Rationale**: Each context type has distinct serialization format and fields; unified trait enables composite propagation
- **Alternatives considered**: Single mega-propagator (violates SRP), text-only vs. binary propagators (not needed for initial scope)

## 6. Unique Identifier Generation

- **Decision**: `uuid` crate with v7 feature for Trace ID and Correlation ID generation
- **Rationale**: UUID v7 provides unique, time-ordered identifiers; no additional dependencies beyond what's already declared
- **Alternatives considered**: Random byte generation (no ordering, more complex validation), incremental counters (not distributed-safe)
