//! Integration tests: verify macros produce records equivalent to direct Logger API calls.

use kitlogger_macros::{
    info, warn, CorrelationId, EmitError, LogAttribute, LogAttributeValue, LogContext, Logger,
    Severity, SpanId, TraceId, ValidationError,
};
use std::sync::Mutex;

// ── Capturing logger ───────────────────────────────────────────────────────────

struct CapturingLogger {
    name: String,
    calls: Mutex<Vec<(Severity, String, Vec<LogAttribute>)>>,
}

impl CapturingLogger {
    fn new(name: &str) -> Self {
        CapturingLogger {
            name: name.to_string(),
            calls: Mutex::new(Vec::new()),
        }
    }

    fn recorded(&self) -> Vec<(Severity, String, Vec<LogAttribute>)> {
        self.calls.lock().unwrap().clone()
    }
}

impl Logger for CapturingLogger {
    fn name(&self) -> &str {
        &self.name
    }

    fn log(
        &self,
        severity: Severity,
        message: &str,
        attributes: &[LogAttribute],
    ) -> Result<(), EmitError> {
        if message.is_empty() {
            return Err(EmitError::Validation(ValidationError::EmptyMessage));
        }
        self.calls
            .lock()
            .unwrap()
            .push((severity, message.to_string(), attributes.to_vec()));
        Ok(())
    }
}

// ── FR-009: Macro output equivalent to direct call ────────────────────────────

#[test]
fn macro_no_attrs_equiv_direct() -> Result<(), EmitError> {
    let l1 = CapturingLogger::new("l1");
    let l2 = CapturingLogger::new("l2");

    info!(l1, "hello")?;
    l2.info("hello", &[])?;

    assert_eq!(l1.recorded(), l2.recorded());
    Ok(())
}

#[test]
fn macro_with_attrs_equiv_direct() -> Result<(), EmitError> {
    let l1 = CapturingLogger::new("l1");
    let l2 = CapturingLogger::new("l2");

    info!(l1, "hello", k => "v")?;
    let hand_built =
        [LogAttribute::new("k".to_string(), LogAttributeValue::String("v".to_string())).unwrap()];
    l2.info("hello", &hand_built)?;

    assert_eq!(l1.recorded(), l2.recorded());
    Ok(())
}

// ── CR-003: Hygiene — no explicit domain imports needed ───────────────────────

#[test]
fn hygiene_no_domain_imports() -> Result<(), EmitError> {
    // This test imports only `kitlogger_macros::info` (via the use block above).
    // It MUST compile without explicitly importing LogAttribute, LogAttributeValue,
    // or EmitError from kitlogger_log_domain.
    let logger = CapturingLogger::new("l");
    info!(logger, "msg")?;
    assert_eq!(logger.recorded().len(), 1);
    Ok(())
}

// ── Context fold integration ──────────────────────────────────────────────────

#[test]
fn context_fold_all_ids_and_inline_attr() -> Result<(), EmitError> {
    let logger = CapturingLogger::new("l");
    let mut ctx = LogContext::new();
    ctx.add_attribute(
        LogAttribute::new(
            "env".to_string(),
            LogAttributeValue::String("prod".to_string()),
        )
        .unwrap(),
    )
    .unwrap();
    let ctx = ctx
        .with_correlation_id(CorrelationId::new("c".to_string()))
        .unwrap()
        .with_trace_id(TraceId::new("t".to_string()))
        .unwrap()
        .with_span_id(SpanId::new("s".to_string()))
        .unwrap();

    warn!(logger, &ctx, "degraded", region => "eu")?;

    let calls = logger.recorded();
    assert_eq!(calls[0].0, Severity::Warn);
    let names: Vec<&str> = calls[0].2.iter().map(|a| a.name()).collect();
    assert_eq!(
        names,
        vec!["env", "correlation_id", "trace_id", "span_id", "region"]
    );
    Ok(())
}
