//! Context propagation and correlation types for telemetry data flow.
//!
//! This crate provides the canonical context propagation abstraction for
//! carrying telemetry context across execution boundaries. It defines the
//! core types and traits for W3C Trace Context, Baggage, and Correlation
//! identifier propagation.

pub mod api;
pub mod baggage;
pub mod carrier;
pub mod correlation;
pub mod models;
pub mod noop;
pub mod propagation;
pub mod propagation_metadata;
pub mod trace_context;
pub mod traits;
pub mod validation;
