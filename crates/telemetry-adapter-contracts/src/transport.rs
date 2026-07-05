//! Protocol-agnostic transport strategy, relocated from the orphaned
//! `telemetry-transport-contract` crate per ADR-008's Phase 7 handoff
//! (finally executed here, in Phase 8).
//!
//! `Transport`/`DeliveryMode`/`TransportResult`/`TransportError` are an
//! optional toolkit inside this crate, not a required part of the `Adapter`
//! supertrait — `console-exporter`/`file-exporter` have no wire protocol to
//! abstract over and never depend on this module. A future network-facing
//! adapter (most naturally `otlp-exporter`) is the intended consumer.
//!
//! `TransportError` is deliberately distinct from `AdapterError`: they cover
//! different failure domains (wire-level delivery vs. registry/lifecycle
//! management) — see `design.md` for change 018's reasoning.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use telemetry_types::{BackpressureSignal, PayloadEnvelope};

/// Delivery mode for telemetry transport operations.
///
/// This enum represents the different ways telemetry can be delivered
/// across an execution boundary. It is returned as a value from
/// `Transport::send()` to indicate which delivery mode was used.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryMode {
    /// Send the telemetry and forget about it.
    ///
    /// The sender does not wait for acknowledgment or response.
    FireAndForget,

    /// Send the telemetry and wait for a response.
    ///
    /// The sender waits for an acknowledgment or response from the receiver.
    RequestResponse,

    /// Send telemetry in batches.
    ///
    /// Multiple telemetry items are sent together in a batch.
    Batch,

    /// Send telemetry in a streaming fashion.
    ///
    /// Telemetry is sent as a continuous stream of data.
    Streaming,
}

/// Error type for transport operations.
///
/// This enum represents various errors that can occur during telemetry transport.
/// It is non-exhaustive to allow concrete transport implementations to add
/// their own error variants without breaking changes.
///
/// Does not derive `PartialEq`/`Eq`: `telemetry_types::BackpressureSignal` (used
/// by the `Backpressure` variant) does not implement either.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportError {
    /// The operation timed out.
    Timeout,

    /// The transport is currently unavailable.
    Unavailable,

    /// The transport is experiencing backpressure.
    ///
    /// This variant contains a `BackpressureSignal` with information
    /// about when to retry the operation.
    Backpressure(BackpressureSignal),

    /// The payload is too large for the transport.
    PayloadTooLarge,

    /// The transport protocol is not supported.
    UnsupportedTransport,
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::Timeout => write!(f, "transport timeout"),
            TransportError::Unavailable => write!(f, "transport unavailable"),
            TransportError::Backpressure(_) => write!(f, "transport backpressure"),
            TransportError::PayloadTooLarge => write!(f, "payload too large"),
            TransportError::UnsupportedTransport => write!(f, "unsupported transport"),
        }
    }
}

impl std::error::Error for TransportError {}

/// Result type for transport operations.
pub type TransportResult<T> = Result<T, TransportError>;

/// Transport trait for sending telemetry payloads.
///
/// This trait defines the contract for transport implementations.
/// It is protocol-agnostic — implementors can use HTTP, gRPC, in-memory,
/// or any other transport mechanism.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a telemetry payload and return the delivery mode used.
    async fn send(&self, envelope: PayloadEnvelope) -> TransportResult<DeliveryMode>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_delivery_mode_serialization() {
        let mode = DeliveryMode::FireAndForget;
        let serialized = serde_json::to_string(&mode).unwrap();
        let deserialized: DeliveryMode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(mode, deserialized);
    }

    /// Regression test for the `telemetry_types::BackpressureSignal` repoint
    /// (change 017): `TransportError::Backpressure` must still construct,
    /// round-trip through serde, `Display`, and satisfy `std::error::Error`.
    #[test]
    fn backpressure_variant_survives_the_canonical_type_repoint() {
        let mut attributes = HashMap::new();
        attributes.insert("reason".to_string(), "rate_limited".to_string());
        let error = TransportError::Backpressure(BackpressureSignal {
            retry_after: Some(30),
            attributes,
        });

        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: TransportError = serde_json::from_str(&serialized).unwrap();
        match deserialized {
            TransportError::Backpressure(signal) => {
                assert_eq!(signal.retry_after, Some(30));
                assert_eq!(
                    signal.attributes.get("reason"),
                    Some(&"rate_limited".to_string())
                );
            }
            other => panic!("expected Backpressure, got {other:?}"),
        }

        assert_eq!(error.to_string(), "transport backpressure");
        let _: &dyn std::error::Error = &error;
    }

    /// Regression test for the `telemetry_types::PayloadEnvelope` repoint
    /// (change 017): `Transport` must still be implementable, and `send()`
    /// must still be callable, with the canonical envelope type.
    struct MockTransport;

    #[async_trait]
    impl Transport for MockTransport {
        async fn send(&self, _envelope: PayloadEnvelope) -> TransportResult<DeliveryMode> {
            Ok(DeliveryMode::FireAndForget)
        }
    }

    #[tokio::test]
    async fn transport_trait_is_implementable_with_the_canonical_envelope_type() {
        use telemetry_types::{PropagationMetadata, TelemetryBatch, TransportMetadata};

        let envelope = PayloadEnvelope {
            transport_metadata: TransportMetadata {
                protocol: "memory".to_string(),
                endpoint: "test".to_string(),
                attributes: Default::default(),
            },
            propagation_metadata: PropagationMetadata {
                headers: Default::default(),
            },
            payload: TelemetryBatch {
                traces: vec![],
                metrics: vec![],
                logs: vec![],
            },
        };

        let result = MockTransport.send(envelope).await;

        match result {
            Ok(mode) => assert_eq!(mode, DeliveryMode::FireAndForget),
            Err(e) => panic!("expected Ok(FireAndForget), got Err({e})"),
        }
    }
}
