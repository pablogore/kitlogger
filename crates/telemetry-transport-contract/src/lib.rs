//! Transport-agnostic telemetry flow contracts.
//!
//! This crate defines the contracts for transporting telemetry data
//! across execution boundaries in a transport-agnostic way.
//!
//! # Modules
//!
//! * `transport` - Defines the transport contract and related types
//! * `error` - Defines the transport error types
//!
//! `TelemetryBatch`/`PayloadEnvelope`/`TransportMetadata`/`BackpressureSignal`
//! are not defined here — per ADR-007/ADR-010, `telemetry_types` is their
//! canonical owner. This crate consumes them, it does not redefine them.

mod error;
mod transport;

mod redaction;
mod rotation;
mod sampling;

pub use error::*;
pub use transport::*;

pub use redaction::*;
pub use rotation::*;
pub use sampling::*;
