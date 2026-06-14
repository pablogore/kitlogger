//! NoOp implementations for observability telemetry
//!
//! These implementations provide no-op behavior for all telemetry components,
//! ensuring that the system can function without any actual telemetry backend.

use crate::models::{Context, Resource, InstrumentationScope, Span, LogRecord, Metric};
use crate::traits::{Logger, Tracer, Meter};

/// A no-op logger that does nothing when asked to emit logs
pub struct NoOpLogger;

impl Logger for NoOpLogger {
    fn emit(&self, _log_record: LogRecord) {
        // No-op implementation
    }
}

/// A no-op tracer that does nothing when asked to create or end spans
pub struct NoOpTracer;

impl Tracer for NoOpTracer {
    fn start_span(&self, _name: String, _context: Context) -> Span {
        // Return a default span without any actual tracing
        Span::new(
            Context::new_root(),
            Resource::new(),
            InstrumentationScope::new("unknown".to_string()),
            "no-op-span".to_string(),
            0,
        )
    }
    
    fn end_span(&self, _span: Span) {
        // No-op implementation
    }
}

/// A no-op meter that does nothing when asked to record metrics
pub struct NoOpMeter;

impl Meter for NoOpMeter {
    fn record_counter(&self, _metric: Metric, _value: f64) {
        // No-op implementation
    }
    
    fn record_gauge(&self, _metric: Metric, _value: f64) {
        // No-op implementation
    }
    
    fn record_histogram(&self, _metric: Metric, _value: f64) {
        // No-op implementation
    }
    
    fn record_up_down_counter(&self, _metric: Metric, _value: f64) {
        // No-op implementation
    }
}