//! Transport trait and core types for telemetry data flow.
//!
//! This module defines the [`Transport`] trait and related types that form the
//! foundation of the telemetry transport contract.
//!
//! # Transport Trait
//!
//! The [`Transport`] trait is the core abstraction for sending telemetry data
//! across execution boundaries. It is designed to be runtime-independent and
//! supports multiple delivery modes.
//!
//! # Delivery Modes
//!
//! The [`DeliveryMode`] enum represents the different ways telemetry data can
//! be delivered:
//!
//! - [`DeliveryMode::FireAndForget`] - Data is sent without waiting for a response
//! - [`DeliveryMode::RequestResponse`] - Data is sent and a response is expected
//! - [`DeliveryMode::Batch`] - Multiple telemetry items are sent together
//! - [`DeliveryMode::Streaming`] - Data is sent in a streaming fashion


use crate::payload::PayloadEnvelope;
use crate::TransportResult;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// The canonical delivery mode for telemetry data.
///
/// This enum represents how telemetry data was delivered by a transport.
/// It is returned as a value from [`Transport::send()`] rather than as an
/// associated type to allow runtime selection of delivery mode.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryMode {
    /// Data is sent without waiting for a response.
    FireAndForget,
    /// Data is sent and a response is expected.
    RequestResponse,
    /// Multiple telemetry items are sent together.
    Batch,
    /// Data is sent in a streaming fashion.
    Streaming,
}

/// A signal sent back via [`TransportError::Backpressure`] to indicate flow control.
///
/// This signal provides information about when to retry sending data,
/// helping to manage backpressure in the system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackpressureSignal {
    /// Recommended wait time before retrying.
    pub retry_after: Option<Duration>,
}

/// The main transport trait for sending telemetry data.
///
/// This trait defines the contract for sending telemetry data across execution
/// boundaries. Implementations of this trait can be any transport mechanism
/// (HTTP, gRPC, Kafka, etc.) as long as they satisfy the contract.
///
/// # Runtime Independence
///
/// The trait uses `std::future::Future` only, avoiding any dependency on
/// specific async runtimes like Tokio. This allows concrete transport
/// implementations to choose their own runtime.
///
/// # Examples
///
/// ```rust
/// use telemetry_transport_contract::{Transport, PayloadEnvelope, TransportResult, DeliveryMode};
/// use async_trait::async_trait;
///
/// struct MockTransport;
///
/// #[async_trait]
/// impl Transport for MockTransport {
///     async fn send(&self, envelope: PayloadEnvelope) -> TransportResult<DeliveryMode> {
///         Ok(DeliveryMode::FireAndForget)
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    /// Send a telemetry payload across an execution boundary.
    ///
    /// Returns the delivery mode used for the operation.
    ///
    /// # Arguments
    ///
    /// * `envelope` - The payload to send, containing telemetry data and metadata
    ///
    /// # Returns
    ///
    /// A `TransportResult` containing the delivery mode used for the operation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use telemetry_transport_contract::{Transport, TransportResult, DeliveryMode,
    ///     TelemetryBatch, Resource, Span};
    /// use telemetry_transport_contract::payload::{TransportMetadata, PropagationMetadata};
    /// use telemetry_transport_contract::payload::PayloadEnvelope;
    /// use async_trait::async_trait;
    ///
    /// struct MyTransport;
    ///
    /// #[async_trait]
    /// impl Transport for MyTransport {
    ///     async fn send(&self, envelope: PayloadEnvelope) -> TransportResult<DeliveryMode> {
    ///         let _ = envelope;
    ///         Ok(DeliveryMode::FireAndForget)
    ///     }
    /// }
    ///
    /// let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
    /// rt.block_on(async {
    ///     let transport = MyTransport;
    ///     let batch = TelemetryBatch::new(
    ///         Resource("resource-1".to_string()),
    ///         vec![Span("trace-1".to_string())],
    ///         vec![],
    ///         vec![],
    ///     ).unwrap();
    ///     let envelope = PayloadEnvelope {
    ///         transport_metadata: TransportMetadata::now(),
    ///         propagation_metadata: PropagationMetadata::default(),
    ///         payload: batch,
    ///     };
    ///     let result = transport.send(envelope).await;
    ///     match result {
    ///         Ok(mode) => println!("Delivered as {:?}", mode),
    ///         Err(e) => eprintln!("Transport error: {}", e),
    ///     }
    /// });
    /// ```
    async fn send(&self, envelope: PayloadEnvelope) -> TransportResult<DeliveryMode>;
}