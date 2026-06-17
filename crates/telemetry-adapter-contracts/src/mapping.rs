/// Bidirectional entity-specific mapping contracts (Canonical ↔ OpenTelemetry).
///
/// These traits define the contract for converting between the canonical domain
/// model entities and their OpenTelemetry equivalents. Concrete implementations
/// belong to later specs.

// Placeholder types to make the contracts compile.
// Concrete implementations will use real types from AS-01 and opentelemetry crate.
pub struct Trace;
pub struct Span;
pub struct Metric;
pub struct LogRecord;
pub struct Resource;

pub struct OtelTrace;
pub struct OtelSpan;
pub struct OtelMetric;
pub struct OtelLogRecord;
pub struct OtelResource;

pub trait TraceMappingContract {
    fn to_otel(&self, trace: &Trace) -> OtelTrace;
    fn from_otel(&self, otel: OtelTrace) -> Trace;
}

pub trait SpanMappingContract {
    fn to_otel(&self, span: &Span) -> OtelSpan;
    fn from_otel(&self, otel: OtelSpan) -> Span;
}

pub trait MetricMappingContract {
    fn to_otel(&self, metric: &Metric) -> OtelMetric;
    fn from_otel(&self, otel: OtelMetric) -> Metric;
}

pub trait LogRecordMappingContract {
    fn to_otel(&self, log: &LogRecord) -> OtelLogRecord;
    fn from_otel(&self, otel: OtelLogRecord) -> LogRecord;
}

pub trait ResourceMappingContract {
    fn to_otel(&self, resource: &Resource) -> OtelResource;
    fn from_otel(&self, otel: OtelResource) -> Resource;
}
