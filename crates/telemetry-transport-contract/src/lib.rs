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

pub mod payload;

mod batch;
mod error;
mod transport;

// Re-export types from context-propagation crate
pub use context_propagation::models::{
    AttributeValue, Context, InstrumentationScope, LogRecord, LogSeverity, Metric, Resource, Span,
    SpanStatus,
};
pub use context_propagation::propagation_metadata::PropagationMetadata;

pub use batch::*;
pub use error::*;
pub use payload::*;
pub use transport::*;
