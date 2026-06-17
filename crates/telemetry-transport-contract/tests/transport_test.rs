use async_trait::async_trait;
use serde_test::{assert_tokens, Token};

use telemetry_transport_contract::payload::{PropagationMetadata, TransportMetadata};
use telemetry_transport_contract::{
    BackpressureSignal, DeliveryMode, PayloadEnvelope, Transport, TransportError, TransportResult,
};
use telemetry_transport_contract::{Resource, Span, TelemetryBatch};

#[test]
fn test_delivery_mode_serde() {
    assert_tokens(
        &DeliveryMode::FireAndForget,
        &[Token::UnitVariant {
            name: "DeliveryMode",
            variant: "FireAndForget",
        }],
    );
    assert_tokens(
        &DeliveryMode::RequestResponse,
        &[Token::UnitVariant {
            name: "DeliveryMode",
            variant: "RequestResponse",
        }],
    );
    assert_tokens(
        &DeliveryMode::Batch,
        &[Token::UnitVariant {
            name: "DeliveryMode",
            variant: "Batch",
        }],
    );
    assert_tokens(
        &DeliveryMode::Streaming,
        &[Token::UnitVariant {
            name: "DeliveryMode",
            variant: "Streaming",
        }],
    );
}

#[test]
fn test_backpressure_signal_serde() {
    let signal = BackpressureSignal { retry_after: None };
    assert_tokens(
        &signal,
        &[
            Token::Struct {
                name: "BackpressureSignal",
                len: 1,
            },
            Token::Str("retry_after"),
            Token::None,
            Token::StructEnd,
        ],
    );
}

#[test]
fn test_transport_error_serde() {
    assert_tokens(
        &TransportError::Timeout,
        &[Token::UnitVariant {
            name: "TransportError",
            variant: "Timeout",
        }],
    );
    assert_tokens(
        &TransportError::Unavailable,
        &[Token::UnitVariant {
            name: "TransportError",
            variant: "Unavailable",
        }],
    );
    assert_tokens(
        &TransportError::PayloadTooLarge,
        &[Token::UnitVariant {
            name: "TransportError",
            variant: "PayloadTooLarge",
        }],
    );
    assert_tokens(
        &TransportError::UnsupportedTransport,
        &[Token::UnitVariant {
            name: "TransportError",
            variant: "UnsupportedTransport",
        }],
    );
}

#[test]
fn test_transport_error_backpressure_json_roundtrip() {
    let err = TransportError::Backpressure(BackpressureSignal { retry_after: None });
    let json = serde_json::to_string(&err).unwrap();
    let deserialized: TransportError = serde_json::from_str(&json).unwrap();
    assert_eq!(err, deserialized);

    let err = TransportError::Backpressure(BackpressureSignal {
        retry_after: Some(std::time::Duration::from_secs(30)),
    });
    let json = serde_json::to_string(&err).unwrap();
    let deserialized: TransportError = serde_json::from_str(&json).unwrap();
    assert_eq!(err, deserialized);
}

#[test]
fn test_transport_error_display() {
    assert_eq!(format!("{}", TransportError::Timeout), "transport timeout");
    assert_eq!(
        format!("{}", TransportError::Unavailable),
        "transport unavailable"
    );
    assert_eq!(
        format!(
            "{}",
            TransportError::Backpressure(BackpressureSignal { retry_after: None })
        ),
        "transport backpressure"
    );
    assert_eq!(
        format!("{}", TransportError::PayloadTooLarge),
        "payload too large"
    );
    assert_eq!(
        format!("{}", TransportError::UnsupportedTransport),
        "unsupported transport"
    );
}

#[test]
fn test_transport_error_is_error() {
    let error = TransportError::Timeout;
    let _error: &dyn std::error::Error = &error;
}

#[test]
fn test_telemetry_batch_error_is_error() {
    let error = telemetry_transport_contract::TelemetryBatchError;
    let _error: &dyn std::error::Error = &error;
    assert_eq!(
        error.to_string(),
        "telemetry batch must contain at least one signal type"
    );
}

struct MockTransport;

#[async_trait]
impl Transport for MockTransport {
    async fn send(&self, _envelope: PayloadEnvelope) -> TransportResult<DeliveryMode> {
        Ok(DeliveryMode::RequestResponse)
    }
}

#[tokio::test]
async fn test_mock_transport_implements_trait() {
    let transport = MockTransport;
    let envelope = PayloadEnvelope {
        transport_metadata: TransportMetadata::now(),
        propagation_metadata: PropagationMetadata::new("mock"),
        payload: TelemetryBatch::new(
            Resource("resource1".to_string()),
            vec![Span("trace1".to_string())],
            vec![],
            vec![],
        )
        .unwrap(),
    };
    let result = transport.send(envelope).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), DeliveryMode::RequestResponse);
}

#[tokio::test]
async fn test_mock_transport_returns_request_response() {
    let transport = MockTransport;
    let envelope = PayloadEnvelope {
        transport_metadata: TransportMetadata::now(),
        propagation_metadata: PropagationMetadata::new("mock"),
        payload: TelemetryBatch::new(
            Resource("resource1".to_string()),
            vec![Span("trace1".to_string())],
            vec![],
            vec![],
        )
        .unwrap(),
    };
    let result = transport.send(envelope).await;
    match result {
        Ok(DeliveryMode::RequestResponse) => {}
        Ok(other) => panic!("Expected RequestResponse, got {:?}", other),
        Err(e) => panic!("Expected Ok, got {}", e),
    }
}

struct MockTransportStreaming;

#[async_trait]
impl Transport for MockTransportStreaming {
    async fn send(&self, _envelope: PayloadEnvelope) -> TransportResult<DeliveryMode> {
        Ok(DeliveryMode::Streaming)
    }
}

#[tokio::test]
async fn test_mock_transport_returns_streaming() {
    let transport = MockTransportStreaming;
    let envelope = PayloadEnvelope {
        transport_metadata: TransportMetadata::now(),
        propagation_metadata: PropagationMetadata::new("mock"),
        payload: TelemetryBatch::new(
            Resource("resource1".to_string()),
            vec![Span("trace1".to_string())],
            vec![],
            vec![],
        )
        .unwrap(),
    };
    let result = transport.send(envelope).await;
    match result {
        Ok(DeliveryMode::Streaming) => {}
        Ok(other) => panic!("Expected Streaming, got {:?}", other),
        Err(e) => panic!("Expected Ok, got {}", e),
    }
}

struct MockTransportBatch;

#[async_trait]
impl Transport for MockTransportBatch {
    async fn send(&self, _envelope: PayloadEnvelope) -> TransportResult<DeliveryMode> {
        Ok(DeliveryMode::Batch)
    }
}

#[tokio::test]
async fn test_mock_transport_returns_batch() {
    let transport = MockTransportBatch;
    let envelope = PayloadEnvelope {
        transport_metadata: TransportMetadata::now(),
        propagation_metadata: PropagationMetadata::new("mock"),
        payload: TelemetryBatch::new(
            Resource("resource1".to_string()),
            vec![Span("trace1".to_string())],
            vec![],
            vec![],
        )
        .unwrap(),
    };
    let result = transport.send(envelope).await;
    match result {
        Ok(DeliveryMode::Batch) => {}
        Ok(other) => panic!("Expected Batch, got {:?}", other),
        Err(e) => panic!("Expected Ok, got {}", e),
    }
}

struct MockTransportUnavailable;

#[async_trait]
impl Transport for MockTransportUnavailable {
    async fn send(&self, _envelope: PayloadEnvelope) -> TransportResult<DeliveryMode> {
        Err(TransportError::Unavailable)
    }
}

#[tokio::test]
async fn test_mock_transport_returns_unavailable() {
    let transport = MockTransportUnavailable;
    let envelope = PayloadEnvelope {
        transport_metadata: TransportMetadata::now(),
        propagation_metadata: PropagationMetadata::new("mock"),
        payload: TelemetryBatch::new(
            Resource("resource1".to_string()),
            vec![Span("trace1".to_string())],
            vec![],
            vec![],
        )
        .unwrap(),
    };
    let result = transport.send(envelope).await;
    match result {
        Err(TransportError::Unavailable) => {}
        Ok(mode) => panic!("Expected Err(Unavailable), got Ok({:?})", mode),
        Err(e) => panic!("Expected Err(Unavailable), got Err({})", e),
    }
}

struct MockTransportBackpressure;

#[async_trait]
impl Transport for MockTransportBackpressure {
    async fn send(&self, _envelope: PayloadEnvelope) -> TransportResult<DeliveryMode> {
        Err(TransportError::Backpressure(BackpressureSignal {
            retry_after: Some(std::time::Duration::from_secs(30)),
        }))
    }
}

#[tokio::test]
async fn test_mock_transport_returns_backpressure() {
    let transport = MockTransportBackpressure;
    let envelope = PayloadEnvelope {
        transport_metadata: TransportMetadata::now(),
        propagation_metadata: PropagationMetadata::new("mock"),
        payload: TelemetryBatch::new(
            Resource("resource1".to_string()),
            vec![Span("trace1".to_string())],
            vec![],
            vec![],
        )
        .unwrap(),
    };
    let result = transport.send(envelope).await;
    match result {
        Err(TransportError::Backpressure(ref signal)) => {
            assert_eq!(signal.retry_after, Some(std::time::Duration::from_secs(30)));
        }
        Ok(mode) => panic!("Expected Err(Backpressure), got Ok({:?})", mode),
        Err(e) => panic!("Expected Err(Backpressure), got Err({})", e),
    }
}

struct MockTransportTimeout;

#[async_trait]
impl Transport for MockTransportTimeout {
    async fn send(&self, _envelope: PayloadEnvelope) -> TransportResult<DeliveryMode> {
        Err(TransportError::Timeout)
    }
}

#[tokio::test]
async fn test_mock_transport_returns_timeout() {
    let transport = MockTransportTimeout;
    let envelope = PayloadEnvelope {
        transport_metadata: TransportMetadata::now(),
        propagation_metadata: PropagationMetadata::new("mock"),
        payload: TelemetryBatch::new(
            Resource("resource1".to_string()),
            vec![Span("trace1".to_string())],
            vec![],
            vec![],
        )
        .unwrap(),
    };
    let result = transport.send(envelope).await;
    match result {
        Err(TransportError::Timeout) => {}
        Ok(mode) => panic!("Expected Err(Timeout), got Ok({:?})", mode),
        Err(e) => panic!("Expected Err(Timeout), got Err({})", e),
    }
}

struct MockTransportPayloadTooLarge;

#[async_trait]
impl Transport for MockTransportPayloadTooLarge {
    async fn send(&self, _envelope: PayloadEnvelope) -> TransportResult<DeliveryMode> {
        Err(TransportError::PayloadTooLarge)
    }
}

#[tokio::test]
async fn test_mock_transport_returns_payload_too_large() {
    let transport = MockTransportPayloadTooLarge;
    let envelope = PayloadEnvelope {
        transport_metadata: TransportMetadata::now(),
        propagation_metadata: PropagationMetadata::new("mock"),
        payload: TelemetryBatch::new(
            Resource("resource1".to_string()),
            vec![Span("trace1".to_string())],
            vec![],
            vec![],
        )
        .unwrap(),
    };
    let result = transport.send(envelope).await;
    match result {
        Err(TransportError::PayloadTooLarge) => {}
        Ok(mode) => panic!("Expected Err(PayloadTooLarge), got Ok({:?})", mode),
        Err(e) => panic!("Expected Err(PayloadTooLarge), got Err({})", e),
    }
}

/// Verify FR-008: AS-02 defines transport contracts only, not concrete implementations.
/// The Transport trait is protocol-agnostic — no HTTP, gRPC, or concrete transport
/// dependencies are required to implement it. This test proves that a mock transport
/// works without any concrete protocol dependency.
#[tokio::test]
async fn test_as02_defines_contracts_only() {
    let transport = MockTransport;
    let envelope = PayloadEnvelope {
        transport_metadata: TransportMetadata::now(),
        propagation_metadata: PropagationMetadata::new("abstract"),
        payload: TelemetryBatch::new(
            Resource("resource1".to_string()),
            vec![Span("trace1".to_string())],
            vec![],
            vec![],
        )
        .unwrap(),
    };
    let result = transport.send(envelope).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), DeliveryMode::RequestResponse);
}

/// Verify runtime independence: `Transport` uses `async_trait` (not `tokio::async_trait`),
/// which expands to `std::future::Future` and avoids tying implementations to Tokio.
struct NoRuntimeTransport;

#[async_trait]
impl Transport for NoRuntimeTransport {
    async fn send(&self, _envelope: PayloadEnvelope) -> TransportResult<DeliveryMode> {
        Ok(DeliveryMode::FireAndForget)
    }
}

#[tokio::test]
async fn test_no_runtime_transport() {
    let transport = NoRuntimeTransport;
    let envelope = PayloadEnvelope {
        transport_metadata: TransportMetadata::now(),
        propagation_metadata: PropagationMetadata::new("mock"),
        payload: TelemetryBatch::new(
            Resource("resource1".to_string()),
            vec![Span("trace1".to_string())],
            vec![],
            vec![],
        )
        .unwrap(),
    };
    let result = transport.send(envelope).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), DeliveryMode::FireAndForget);
}

struct MockTransportUnsupported;

#[async_trait]
impl Transport for MockTransportUnsupported {
    async fn send(&self, _envelope: PayloadEnvelope) -> TransportResult<DeliveryMode> {
        Err(TransportError::UnsupportedTransport)
    }
}

#[tokio::test]
async fn test_mock_transport_returns_unsupported() {
    let transport = MockTransportUnsupported;
    let envelope = PayloadEnvelope {
        transport_metadata: TransportMetadata::now(),
        propagation_metadata: PropagationMetadata::new("mock"),
        payload: TelemetryBatch::new(
            Resource("resource1".to_string()),
            vec![Span("trace1".to_string())],
            vec![],
            vec![],
        )
        .unwrap(),
    };
    let result = transport.send(envelope).await;
    match result {
        Err(TransportError::UnsupportedTransport) => {}
        Ok(mode) => panic!("Expected Err(UnsupportedTransport), got Ok({:?})", mode),
        Err(e) => panic!("Expected Err(UnsupportedTransport), got Err({})", e),
    }
}
