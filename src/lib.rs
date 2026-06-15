//! Transport contract types for telemetry data flow.
//!
//! This crate provides the canonical transport abstraction for sending telemetry
//! data across execution boundaries. It defines the core types and traits that
//! all concrete transport implementations must satisfy.
//!
//! # Core Types
//!
//! - [`Transport`] - The main trait for sending telemetry
//! - [`PayloadEnvelope`] - Wrapper for telemetry data with metadata
//! - [`TelemetryBatch`] - Container for traces, metrics, and logs
//! - [`DeliveryMode`] - Enum representing how data was delivered
//! - [`TransportError`] - Error type for transport operations
//! - [`BackpressureSignal`] - Flow control signal for backpressure

pub mod transport;
pub mod payload;
pub mod batch;
pub mod error;

// Re-export public types for convenient access
pub use transport::Transport;
pub use transport::DeliveryMode;
pub use transport::BackpressureSignal;
pub use payload::PayloadEnvelope;
pub use batch::TelemetryBatch;
pub use error::TransportResult;
pub use error::TransportError;