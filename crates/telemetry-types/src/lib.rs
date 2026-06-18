//! Shared canonical types for telemetry data flow
//!
//! This crate defines the core data structures used across the telemetry
//! system, ensuring consistency and avoiding duplication between
//! different capabilities.

/// The canonical payload envelope that wraps telemetry data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PayloadEnvelope {
    /// Transport metadata associated with the payload
    pub transport_metadata: TransportMetadata,

    /// Propagation metadata for context propagation
    pub propagation_metadata: PropagationMetadata,

    /// The actual telemetry data batch
    pub payload: TelemetryBatch,
}

/// Transport metadata for telemetry data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransportMetadata {
    /// The transport protocol used (e.g., "http", "grpc", "kafka")
    pub protocol: String,

    /// The transport endpoint (e.g., URL, topic name)
    pub endpoint: String,

    /// Additional transport-specific metadata
    pub attributes: std::collections::HashMap<String, String>,
}

/// Propagation metadata for context propagation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PropagationMetadata {
    /// Context propagation headers or other metadata
    pub headers: std::collections::HashMap<String, String>,
}

/// The canonical telemetry batch containing traces, metrics, and logs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TelemetryBatch {
    /// Collection of trace data
    pub traces: Vec<TraceData>,

    /// Collection of metric data
    pub metrics: Vec<MetricData>,

    /// Collection of log data
    pub logs: Vec<LogData>,
}

/// Trace data structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TraceData {
    /// Trace identifier
    pub trace_id: String,

    /// Span identifier
    pub span_id: String,

    /// Trace data content
    pub data: String,
}

/// Metric data structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricData {
    /// Metric identifier
    pub metric_id: String,

    /// Metric value
    pub value: f64,

    /// Metric timestamp
    pub timestamp: u64,
}

/// Log data structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogData {
    /// Log identifier
    pub log_id: String,

    /// Log level
    pub level: String,

    /// Log message
    pub message: String,
}

/// Backpressure signal for flow control
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackpressureSignal {
    /// Optional retry-after hint in seconds
    pub retry_after: Option<u64>,

    /// Additional backpressure attributes
    pub attributes: std::collections::HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_envelope_serialization() {
        let envelope = PayloadEnvelope {
            transport_metadata: TransportMetadata {
                protocol: "http".to_string(),
                endpoint: "http://example.com".to_string(),
                attributes: std::collections::HashMap::new(),
            },
            propagation_metadata: PropagationMetadata {
                headers: std::collections::HashMap::new(),
            },
            payload: TelemetryBatch {
                traces: vec![TraceData {
                    trace_id: "trace1".to_string(),
                    span_id: "span1".to_string(),
                    data: "trace data".to_string(),
                }],
                metrics: vec![MetricData {
                    metric_id: "metric1".to_string(),
                    value: 42.0,
                    timestamp: 1234567890,
                }],
                logs: vec![LogData {
                    log_id: "log1".to_string(),
                    level: "info".to_string(),
                    message: "test log".to_string(),
                }],
            },
        };

        let serialized = serde_json::to_string(&envelope).unwrap();
        let deserialized: PayloadEnvelope = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.payload.traces.len(), 1);
        assert_eq!(deserialized.payload.metrics.len(), 1);
        assert_eq!(deserialized.payload.logs.len(), 1);
    }

    #[test]
    fn test_backpressure_signal_serialization() {
        let signal = BackpressureSignal {
            retry_after: Some(30),
            attributes: std::collections::HashMap::new(),
        };

        let serialized = serde_json::to_string(&signal).unwrap();
        let deserialized: BackpressureSignal = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.retry_after, Some(30));
    }
}
