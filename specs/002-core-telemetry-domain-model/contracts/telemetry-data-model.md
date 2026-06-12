# Telemetry Data Model Contract

This document defines the contract for telemetry data models used in the OpenTelemetry integration.

## Trace Contract

### Fields
| Field | Type | Description |
|-------|------|-------------|
| traceId | string | 16-byte hexadecimal identifier |
| spans | Span[] | List of spans belonging to this trace |
| startTime | number | Unix timestamp when trace started |
| endTime | number | Unix timestamp when trace ended |
| attributes | Record<string, any> | Additional attributes |

## Span Contract

### Fields
| Field | Type | Description |
|-------|------|-------------|
| spanId | string | 8-byte hexadecimal identifier |
| traceId | string | Reference to parent trace |
| parentSpanId | string | Reference to parent span (optional) |
| name | string | Name of the span |
| startTime | number | Unix timestamp when span started |
| endTime | number | Unix timestamp when span ended |
| attributes | Record<string, any> | Additional attributes |
| status | { code: number, message?: string } | Span status |

## Metric Contract

### Fields
| Field | Type | Description |
|-------|------|-------------|
| name | string | Metric name |
| value | number | Metric value |
| unit | string | Unit of measurement |
| timestamp | number | Unix timestamp when recorded |
| attributes | Record<string, any> | Additional attributes |

## Log Contract

### Fields
| Field | Type | Description |
|-------|------|-------------|
| timestamp | number | Unix timestamp when log occurred |
| severity | string | Log severity level ('trace', 'debug', 'info', 'warn', 'error', 'fatal') |
| body | string | Log message |
| attributes | Record<string, any> | Additional attributes |

## Telemetry Data Contract

### Fields
| Field | Type | Description |
|-------|------|-------------|
| type | 'trace' | 'metric' | 'log' | Data type |
| data | Trace | Metric | Log | Actual data object |
| attributes | Record<string, any> | Additional attributes |

## Validation Rules

- Trace ID must be a valid 16-byte hexadecimal string
- Span ID must be a valid 8-byte hexadecimal string
- Parent Span ID must be a valid 8-byte hexadecimal string or null
- Metric name must be a non-empty string
- Metric value must be a number
- Log severity must be one of: 'trace', 'debug', 'info', 'warn', 'error', 'fatal'
- All timestamps must be valid Unix timestamps