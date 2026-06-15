//! Tests for the transport module.

use serde_test::{assert_tokens, Token};

use as_02::{DeliveryMode, BackpressureSignal, TransportError, TransportResult};

#[test]
fn test_delivery_mode_serde() {
    assert_tokens(&DeliveryMode::FireAndForget, &[Token::UnitVariant { name: "DeliveryMode", variant: "FireAndForget" }]);
    assert_tokens(&DeliveryMode::RequestResponse, &[Token::UnitVariant { name: "DeliveryMode", variant: "RequestResponse" }]);
    assert_tokens(&DeliveryMode::Batch, &[Token::UnitVariant { name: "DeliveryMode", variant: "Batch" }]);
    assert_tokens(&DeliveryMode::Streaming, &[Token::UnitVariant { name: "DeliveryMode", variant: "Streaming" }]);
}

#[test]
fn test_backpressure_signal_serde() {
    let signal = BackpressureSignal {
        retry_after: None,
    };
    assert_tokens(&signal, &[
        Token::Struct { name: "BackpressureSignal", len: 1 },
        Token::Str("retry_after"),
        Token::None,
        Token::StructEnd,
    ]);
}

#[test]
fn test_transport_error_serde() {
    assert_tokens(&TransportError::Timeout, &[Token::UnitVariant { name: "TransportError", variant: "Timeout" }]);
    assert_tokens(&TransportError::InvalidBatch, &[Token::UnitVariant { name: "TransportError", variant: "InvalidBatch" }]);
    assert_tokens(&TransportError::Unknown, &[Token::UnitVariant { name: "TransportError", variant: "Unknown" }]);
}

#[test]
fn test_transport_error_display() {
    assert_eq!(format!("{}", TransportError::Timeout), "transport timeout");
    assert_eq!(format!("{}", TransportError::Backpressure(BackpressureSignal { retry_after: None })), "transport backpressure");
    assert_eq!(format!("{}", TransportError::InvalidBatch), "invalid telemetry batch");
    assert_eq!(format!("{}", TransportError::Unknown), "unknown transport error");
}

#[test]
fn test_transport_error_is_error() {
    let error = TransportError::Timeout;
    let _error: &dyn std::error::Error = &error;
}