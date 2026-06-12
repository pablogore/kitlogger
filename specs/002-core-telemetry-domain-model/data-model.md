# Telemetry Data Model

## Entities

### Trace
A directed acyclic graph of spans representing a logical operation.

**Fields**:
- traceId: string (16-byte identifier)
- spans: Span[] (list of spans belonging to this trace)
- startTime: number (timestamp when trace started)
- endTime: number (timestamp when trace ended)
- attributes: Record<string, any> (additional attributes)

### Span
A named, timed operation representing work done in a system.

**Fields**:
- spanId: string (8-byte identifier)
- traceId: string (reference to parent trace)
- parentSpanId: string (reference to parent span, optional)
- name: string (name of the span)
- startTime: number (timestamp when span started)
- endTime: number (timestamp when span ended)
- attributes: Record<string, any> (additional attributes)
- status: { code: number, message?: string } (span status)

### Metric
A measurement of a system's behavior over time.

**Fields**:
- name: string (metric name)
- value: number (metric value)
- unit: string (unit of measurement)
- timestamp: number (when the metric was recorded)
- attributes: Record<string, any> (additional attributes)

### Log
A record of an event that occurred in a system.

**Fields**:
- timestamp: number (when the log occurred)
- severity: string (log severity level)
- body: string (log message)
- attributes: Record<string, any> (additional attributes)

### Telemetry Data
Structured data representing system behavior including traces, metrics, and logs.

**Fields**:
- type: 'trace' | 'metric' | 'log' (data type)
- data: Trace | Metric | Log (actual data object)
- attributes: Record<string, any> (additional attributes)

## Relationships

- A Trace contains multiple Spans
- A Span belongs to one Trace
- A Span can have a parent Span
- A Metric represents a measurement over time
- A Log records an event at a specific time
- All telemetry data types can be associated with additional attributes

## Validation Rules

- Trace ID must be a valid 16-byte hexadecimal string
- Span ID must be a valid 8-byte hexadecimal string
- Parent Span ID must be a valid 8-byte hexadecimal string or null
- Metric name must be a non-empty string
- Metric value must be a number
- Log severity must be one of: 'trace', 'debug', 'info', 'warn', 'error', 'fatal'
- All timestamps must be valid Unix timestamps