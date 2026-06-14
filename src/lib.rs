//! Observability telemetry library
//!
//! This crate provides foundational observability abstractions that are
//! backend-agnostic, vendor-neutral, and domain-agnostic.
//!
//! The core components are:
//! - Context: For correlation and propagation
//! - Resource: For service metadata
//! - InstrumentationScope: For library identification
//! - Span: For trace telemetry
//! - LogRecord: For log telemetry
//! - Metric: For metric telemetry
//!
//! All components are designed to be thread-safe and to avoid memory leaks
//! or resource exhaustion.

pub mod models;
pub mod traits;
pub mod noop;
pub mod api;
pub mod validation;
pub mod carrier;
pub mod trace_context;
pub mod correlation;
pub mod baggage;
pub mod propagation;

pub use trace_context::*;
pub use correlation::*;
pub use baggage::*;
pub use propagation::*;
