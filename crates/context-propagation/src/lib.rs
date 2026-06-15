//! Context propagation and correlation types for telemetry data flow.
//!
//! This crate provides the canonical context propagation abstraction for
//! carrying telemetry context across execution boundaries. It defines the
//! core types and traits for W3C Trace Context, Baggage, and Correlation
//! identifier propagation.

pub mod trace_context;
pub mod correlation;
pub mod baggage;
pub mod carrier;
pub mod propagation;
pub mod propagation_metadata;
pub mod models;
pub mod traits;
pub mod noop;
pub mod api;
pub mod validation;
