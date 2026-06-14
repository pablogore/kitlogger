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

pub use kit_config::{
    BufferingConfig, CategoriesConfig, CategoryConfig, ConfigError, ConfigModule,
    ConfigurationProfile, ConfigurationSource, CorrelationConfig, Extension, LogFormat,
    LogLevel, LoggingConfig, OutputConfig, OutputTarget, RedactionConfig, RetentionConfig,
    RotationConfig, SamplingConfig, SamplingStrategy, StructuredConfig, TimestampConfig,
    TimestampFormat, Validation, ValidationError, ValidationReport,
};

pub use kit_config::modules;

pub use api::{Logger, LoggerBuilder, LoggerProvider, LogEvent};
