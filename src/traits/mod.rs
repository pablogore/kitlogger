//! Traits for observability telemetry
//!
//! This module defines the core traits that define the behavior of
//! telemetry components in the observability system.

use crate::models::{Context, Span, LogRecord, Metric};

/// A logger for emitting log records
pub trait Logger {
    /// Emits a log record
    fn emit(&self, log_record: LogRecord);
}

/// A tracer for creating and managing spans
pub trait Tracer {
    /// Creates a new span
    fn start_span(&self, name: String, context: Context) -> Span;
    
    /// Ends a span
    fn end_span(&self, span: Span);
}

/// A meter for recording metrics
pub trait Meter {
    /// Records a counter value
    fn record_counter(&self, metric: Metric, value: f64);
    
    /// Records a gauge value
    fn record_gauge(&self, metric: Metric, value: f64);
    
    /// Records a histogram value
    fn record_histogram(&self, metric: Metric, value: f64);
    
    /// Records an up-down counter value
    fn record_up_down_counter(&self, metric: Metric, value: f64);
}