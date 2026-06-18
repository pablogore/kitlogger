use std::sync::Arc;
use std::time::SystemTime;

use async_trait::async_trait;
use telemetry_types::{PayloadEnvelope, PropagationMetadata, TelemetryBatch, TransportMetadata};

use telemetry_adapter_contracts::mapping::{
    LogRecord, Metric, OtelLogRecord, OtelMetric, OtelResource, OtelSpan, OtelTrace, Resource,
    Span, Trace,
};
use telemetry_adapter_contracts::{
    Adapter, AdapterHealth, AdapterId, AdapterResult, CommonAdapterBase, ExporterAdapter,
    HealthReport, LifecycleAdapter, LogRecordMappingContract, MetricMappingContract,
    ProviderAdapter, ResourceMappingContract, SpanMappingContract, TelemetryDelivery,
    TraceMappingContract,
};

/// Mock adapter for testing all base traits and both provider/exporter traits.
struct MockAdapter {
    id: AdapterId,
    health: AdapterHealth,
}

impl MockAdapter {
    fn new(id: AdapterId) -> Self {
        MockAdapter {
            id,
            health: AdapterHealth::Healthy,
        }
    }
}

#[async_trait]
impl CommonAdapterBase for MockAdapter {
    fn id(&self) -> &AdapterId {
        &self.id
    }

    fn health(&self) -> HealthReport {
        HealthReport {
            status: self.health.clone(),
            reason: String::new(),
            timestamp: SystemTime::now(),
        }
    }
}

#[async_trait]
impl LifecycleAdapter for MockAdapter {
    async fn flush(&self) -> AdapterResult<()> {
        Ok(())
    }

    async fn shutdown(&self) -> AdapterResult<()> {
        Ok(())
    }
}

#[async_trait]
impl TelemetryDelivery for MockAdapter {
    async fn deliver(&self, _envelope: PayloadEnvelope) -> AdapterResult<()> {
        Ok(())
    }
}

#[async_trait]
impl ProviderAdapter for MockAdapter {
    async fn initialize(&self) -> AdapterResult<()> {
        Ok(())
    }

    async fn start(&self) -> AdapterResult<()> {
        Ok(())
    }

    async fn stop(&self) -> AdapterResult<()> {
        Ok(())
    }
}

#[async_trait]
impl ExporterAdapter for MockAdapter {
    async fn initialize(&self) -> AdapterResult<()> {
        Ok(())
    }

    async fn start(&self) -> AdapterResult<()> {
        Ok(())
    }

    async fn stop(&self) -> AdapterResult<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_mock_adapter_implements_all_traits() {
    let id = AdapterId::new("test-adapter").unwrap();
    let adapter = MockAdapter::new(id.clone());

    assert_eq!(adapter.id(), &id);
    let report = adapter.health();
    assert_eq!(report.status, AdapterHealth::Healthy);
}

#[tokio::test]
async fn test_mock_adapter_arc_compatible() {
    let id = AdapterId::new("arc-adapter").unwrap();
    let adapter: Arc<dyn Adapter> = Arc::new(MockAdapter::new(id.clone()));

    // All operations callable through Arc<dyn Adapter>
    assert_eq!(adapter.id(), &id);
    let report = adapter.health();
    assert_eq!(report.status, AdapterHealth::Healthy);
    assert!(adapter.flush().await.is_ok());
    assert!(adapter.shutdown().await.is_ok());
    // Create a mock PayloadEnvelope for testing
    let envelope = PayloadEnvelope {
        transport_metadata: TransportMetadata {
            protocol: "test".to_string(),
            endpoint: "test".to_string(),
            attributes: std::collections::HashMap::new(),
        },
        propagation_metadata: PropagationMetadata {
            headers: std::collections::HashMap::new(),
        },
        payload: TelemetryBatch {
            traces: vec![],
            metrics: vec![],
            logs: vec![],
        },
    };
    assert!(adapter.deliver(envelope).await.is_ok());
}

struct MockTraceMapper;

impl TraceMappingContract for MockTraceMapper {
    fn to_otel(&self, _trace: &Trace) -> OtelTrace {
        OtelTrace
    }
    fn from_otel(&self, _otel: OtelTrace) -> Trace {
        Trace
    }
}

struct MockSpanMapper;
impl SpanMappingContract for MockSpanMapper {
    fn to_otel(&self, _span: &Span) -> OtelSpan {
        OtelSpan
    }
    fn from_otel(&self, _otel: OtelSpan) -> Span {
        Span
    }
}

struct MockMetricMapper;
impl MetricMappingContract for MockMetricMapper {
    fn to_otel(&self, _metric: &Metric) -> OtelMetric {
        OtelMetric
    }
    fn from_otel(&self, _otel: OtelMetric) -> Metric {
        Metric
    }
}

struct MockLogRecordMapper;
impl LogRecordMappingContract for MockLogRecordMapper {
    fn to_otel(&self, _log: &LogRecord) -> OtelLogRecord {
        OtelLogRecord
    }
    fn from_otel(&self, _otel: OtelLogRecord) -> LogRecord {
        LogRecord
    }
}

struct MockResourceMapper;
impl ResourceMappingContract for MockResourceMapper {
    fn to_otel(&self, _resource: &Resource) -> OtelResource {
        OtelResource
    }
    fn from_otel(&self, _otel: OtelResource) -> Resource {
        Resource
    }
}

#[test]
fn test_trace_mapping_roundtrip() {
    let mapper = MockTraceMapper;
    let trace = Trace;
    let otel = mapper.to_otel(&trace);
    let _roundtrip: Trace = mapper.from_otel(otel);
}

#[test]
fn test_span_mapping_roundtrip() {
    let mapper = MockSpanMapper;
    let span = Span;
    let otel = mapper.to_otel(&span);
    let _roundtrip: Span = mapper.from_otel(otel);
}

#[test]
fn test_metric_mapping_roundtrip() {
    let mapper = MockMetricMapper;
    let metric = Metric;
    let otel = mapper.to_otel(&metric);
    let _roundtrip: Metric = mapper.from_otel(otel);
}

#[test]
fn test_log_record_mapping_roundtrip() {
    let mapper = MockLogRecordMapper;
    let log = LogRecord;
    let otel = mapper.to_otel(&log);
    let _roundtrip: LogRecord = mapper.from_otel(otel);
}

#[test]
fn test_resource_mapping_roundtrip() {
    let mapper = MockResourceMapper;
    let resource = Resource;
    let otel = mapper.to_otel(&resource);
    let _roundtrip: Resource = mapper.from_otel(otel);
}
