//! Canonical structured logging domain model for KitLogger.
//!
//! This crate defines the core data types and validation rules for structured logs.
//! It is the foundational layer for all logging components in the KitLogger ecosystem.

pub mod correlation_id;
pub mod log_attribute;
pub mod log_attribute_value;
pub mod log_record;
pub mod severity;
pub mod span_id;
pub mod trace_id;
pub mod validation;

pub use correlation_id::CorrelationId;
pub use log_attribute::LogAttribute;
pub use log_attribute_value::LogAttributeValue;
pub use log_record::LogRecord;
pub use severity::Severity;
pub use span_id::SpanId;
pub use trace_id::TraceId;
pub use validation::ValidationError;
