# Data Model: Context Propagation and Correlation

## TraceContext

Represents the W3C Trace Context for distributed tracing.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `trace_id` | `TraceId([u8; 16])` | 128-bit globally unique trace identifier | 32 lowercase hex; all-zeros invalid |
| `span_id` | `SpanId([u8; 8])` | 64-bit current span identifier | 16 lowercase hex; all-zeros invalid |
| `parent_span_id` | `SpanId([u8; 8])` | 64-bit parent span identifier | 16 lowercase hex; zero for root span |
| `trace_flags` | `TraceFlags(u8)` | 8-bit trace flags bitfield | Bit 0: sampled, Bit 1: random-trace-id |
| `trace_state` | `TraceState` | Vendor-specific trace state entries | Max 32 entries |

### Validation Rules
- Trace ID and Span ID must not be all-zeros
- Version byte `ff` is invalid (must be `00` for v1)
- Trace flags: reserved bits must be `0`

### State Transitions
1. **Incoming**: parse from `traceparent`/`tracestate` headers
2. **Propagate**: generate new `span_id`, set `parent_span_id` to previous `span_id`
3. **Outgoing**: serialize to `traceparent`/`tracestate` headers

## CorrelationIdentifier

Links telemetry across signals (Traces, Metrics, Logs).

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `id` | `Uuid` | UUID v7 time-ordered identifier | 128-bit; globally unique |
| `created_at` | `i64` | Unix timestamp milliseconds | Extracted from UUID v7 |

### Validation Rules
- Must be a valid UUID v7 (version field = 0x7)
- Immutable once assigned

## Baggage

Application-defined key-value context for propagation.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `entries` | `Vec<BaggageEntry>` | Ordered list of baggage entries | Max 180 entries |
| `entry.key` | `String` | Baggage key | Printable ASCII, max 256 chars |
| `entry.value` | `String` | Baggage value | URL-encoded printable ASCII, max 256 chars decoded |
| `entry.properties` | `Vec<BaggageProperty>` | Optional metadata per entry | Key-value or flag properties |

### Validation Rules
- Total serialized size must not exceed 64KB
- Key must match `^[a-zA-Z0-9_\\-\\*\\/]+$` (multi-tenant: `key@system_id`)
- Value must be URL-percent-encoded

## PropagationMetadata

Transport-specific context carriage data.

| Field | Type | Description |
|-------|------|-------------|
| `carrier` | `Carrier` | Key-value store for context injection/extraction |
| `propagators` | `Vec<PropagatorKind>` | List of active propagators for this metadata |

Supported propagator kinds: `TraceContext`, `CorrelationId`, `Baggage`

## Entity Relationships

```text
TraceContext ──1:N──→ Span (transient, per-hop relationship)
CorrelationIdentifier ──1:N──→ TelemetrySignal (Trace, Metric, Log)
Baggage ──1:1──→ Operation (scoped to operation lifecycle)
PropagationMetadata ──N:1──→ Transport (one metadata per execution boundary)
```
