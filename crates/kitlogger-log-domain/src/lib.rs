//! Canonical structured logging domain model for KitLogger.
//!
//! This crate defines the core data types and validation rules for structured logs.
//! It is the foundational layer for all logging components in the KitLogger ecosystem.

pub mod correlation_id;
pub mod emit_error;
pub mod log_attribute;
pub mod log_attribute_value;
pub mod log_context;
pub mod log_record;
pub mod logger;
pub mod logger_factory;
pub mod severity;
pub mod span_id;
pub mod trace_id;
pub mod validation;

pub use correlation_id::CorrelationId;
pub use emit_error::EmitError;
pub use log_attribute::LogAttribute;
pub use log_attribute_value::LogAttributeValue;
pub use log_context::LogContext;
pub use log_record::LogRecord;
pub use logger::Logger;
pub use logger_factory::LoggerFactory;
pub use severity::Severity;
pub use span_id::SpanId;
pub use trace_id::TraceId;
pub use validation::ValidationError;
