//! Transport-agnostic telemetry flow contracts.
//!
//! This crate defines the contracts for transporting telemetry data
//! across execution boundaries in a transport-agnostic way.
//!
//! # Modules
//!
//! * `transport` - Defines the transport contract and related types
//! * `error` - Defines the transport error types
//! * `batch` - Defines the telemetry batch structure
//! * `payload` - Defines the payload envelope structure

mod transport;
mod error;
mod batch;
mod payload;

pub use transport::*;
pub use batch::*;
pub use payload::*;