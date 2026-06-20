//! Ergonomic severity macros for the KitLogger structured logging ecosystem.
//!
//! This crate exports five `macro_rules!` macros — `trace!`, `debug!`, `info!`,
//! `warn!`, and `error!` — that expand to the matching `Logger` severity method.
//! Macros are thin wrappers that build a `&[LogAttribute]` slice and forward to
//! `logger.<severity>(msg, &attrs)`, returning its `Result<(), EmitError>`.
//!
//! # Supported invocation forms
//!
//! ```ignore
//! // Simple literal message
//! info!(logger, "message");
//!
//! // Formatted message
//! info!(logger, "hello {}", name);
//!
//! // Literal + attributes
//! info!(logger, "message", service => "auth", user_id => 42i64);
//!
//! // Context only (folds LogContext into attribute slice)
//! info!(logger, &ctx, "message");
//!
//! // Context + attributes
//! info!(logger, &ctx, "message", service => "auth");
//! ```

// Re-export domain types so macro expansions using `$crate::` paths resolve,
// and so callers need not import them explicitly.
pub use kitlogger_log_domain::{
    CorrelationId, EmitError, LogAttribute, LogAttributeValue, LogContext, Logger, Severity,
    SpanId, TraceId, ValidationError,
};

/// Local conversion trait for values passed to `key => value` attribute pairs.
///
/// This is necessary because the orphan rule forbids implementing the foreign
/// trait `From<T>` for the foreign type `LogAttributeValue` from this crate.
/// A local trait on foreign types is permitted.
pub trait IntoAttributeValue {
    /// Converts `self` into a `LogAttributeValue`.
    fn into_attribute_value(self) -> LogAttributeValue;
}

impl IntoAttributeValue for &str {
    fn into_attribute_value(self) -> LogAttributeValue {
        LogAttributeValue::String(self.to_string())
    }
}

impl IntoAttributeValue for String {
    fn into_attribute_value(self) -> LogAttributeValue {
        LogAttributeValue::String(self)
    }
}

impl IntoAttributeValue for i64 {
    fn into_attribute_value(self) -> LogAttributeValue {
        LogAttributeValue::Integer(self)
    }
}

impl IntoAttributeValue for f64 {
    fn into_attribute_value(self) -> LogAttributeValue {
        LogAttributeValue::Float(self)
    }
}

impl IntoAttributeValue for bool {
    fn into_attribute_value(self) -> LogAttributeValue {
        LogAttributeValue::Boolean(self)
    }
}

// ── Internal helper macro ─────────────────────────────────────────────────────

/// Folds a LogContext into a Vec<LogAttribute> and appends inline k=>v pairs.
///
/// Internal implementation detail — not part of the public API.
#[doc(hidden)]
#[macro_export]
macro_rules! __fold_ctx {
    // ctx only (no inline attrs)
    ($ctx:expr) => {{
        let mut __v: Vec<$crate::LogAttribute> = $ctx.attributes().to_vec();
        if let Some(__id) = $ctx.correlation_id() {
            __v.push($crate::LogAttribute::new(
                "correlation_id".to_string(),
                $crate::LogAttributeValue::String(__id.to_string()),
            )?);
        }
        if let Some(__id) = $ctx.trace_id() {
            __v.push($crate::LogAttribute::new(
                "trace_id".to_string(),
                $crate::LogAttributeValue::String(__id.to_string()),
            )?);
        }
        if let Some(__id) = $ctx.span_id() {
            __v.push($crate::LogAttribute::new(
                "span_id".to_string(),
                $crate::LogAttributeValue::String(__id.to_string()),
            )?);
        }
        __v
    }};
    // ctx + inline attrs
    ($ctx:expr, $($k:ident => $v:expr),+ $(,)?) => {{
        let mut __v: Vec<$crate::LogAttribute> = $ctx.attributes().to_vec();
        if let Some(__id) = $ctx.correlation_id() {
            __v.push($crate::LogAttribute::new(
                "correlation_id".to_string(),
                $crate::LogAttributeValue::String(__id.to_string()),
            )?);
        }
        if let Some(__id) = $ctx.trace_id() {
            __v.push($crate::LogAttribute::new(
                "trace_id".to_string(),
                $crate::LogAttributeValue::String(__id.to_string()),
            )?);
        }
        if let Some(__id) = $ctx.span_id() {
            __v.push($crate::LogAttribute::new(
                "span_id".to_string(),
                $crate::LogAttributeValue::String(__id.to_string()),
            )?);
        }
        $(
            __v.push($crate::LogAttribute::new(
                stringify!($k).to_string(),
                $crate::IntoAttributeValue::into_attribute_value($v),
            )?);
        )+
        __v
    }};
}

// ── Macros ────────────────────────────────────────────────────────────────────
//
// Arm ordering within each macro is critical for disambiguation.
// Rules:
//   1. Arms with `=>` separator (attrs/ctx+attrs) have unique tokens — safe anywhere.
//   2. Format-args arm MUST come before ctx-bare arm because both have the shape
//      ($l:expr, $something, $another) — the format arm uses `$($a:expr),+` (greedy)
//      while ctx-bare has `$msg:literal` as the third token. Placing format first
//      ensures `macro!(l, "fmt {}", val)` is not mistakenly matched as ctx-bare.
//   3. Bare-message arm always last (least specific).

/// Emits an `Info`-severity log record.
///
/// See crate-level docs for all supported invocation forms.
#[macro_export]
macro_rules! info {
    // ctx + msg + attrs — unique token `=>`, always unambiguous
    ($l:expr, $ctx:expr, $msg:literal, $($k:ident => $v:expr),+ $(,)?) => {{
        let __v = $crate::__fold_ctx!($ctx, $($k => $v),+);
        $l.info($msg, &__v)
    }};
    // msg + attrs — unique token `=>`, always unambiguous
    ($l:expr, $msg:literal, $($k:ident => $v:expr),+ $(,)?) => {{
        let __attrs = [$(
            $crate::LogAttribute::new(
                stringify!($k).to_string(),
                $crate::IntoAttributeValue::into_attribute_value($v),
            )?,
        )+];
        $l.info($msg, &__attrs)
    }};
    // format args — MUST come before ctx-bare to win over ($l, $ctx:expr, $msg:literal)
    ($l:expr, $fmt:literal, $($a:expr),+ $(,)?) => {{
        $l.info(&format!($fmt, $($a),+), &[])
    }};
    // ctx + msg (no attrs) — comes after format args
    ($l:expr, $ctx:expr, $msg:literal) => {{
        let __v = $crate::__fold_ctx!($ctx);
        $l.info($msg, &__v)
    }};
    // bare message — least specific, always last
    ($l:expr, $msg:literal) => {{
        $l.info($msg, &[])
    }};
}

/// Emits a `Trace`-severity log record.
///
/// See crate-level docs for all supported invocation forms.
#[macro_export]
macro_rules! trace {
    ($l:expr, $ctx:expr, $msg:literal, $($k:ident => $v:expr),+ $(,)?) => {{
        let __v = $crate::__fold_ctx!($ctx, $($k => $v),+);
        $l.trace($msg, &__v)
    }};
    ($l:expr, $msg:literal, $($k:ident => $v:expr),+ $(,)?) => {{
        let __attrs = [$(
            $crate::LogAttribute::new(
                stringify!($k).to_string(),
                $crate::IntoAttributeValue::into_attribute_value($v),
            )?,
        )+];
        $l.trace($msg, &__attrs)
    }};
    ($l:expr, $fmt:literal, $($a:expr),+ $(,)?) => {{
        $l.trace(&format!($fmt, $($a),+), &[])
    }};
    ($l:expr, $ctx:expr, $msg:literal) => {{
        let __v = $crate::__fold_ctx!($ctx);
        $l.trace($msg, &__v)
    }};
    ($l:expr, $msg:literal) => {{
        $l.trace($msg, &[])
    }};
}

/// Emits a `Debug`-severity log record.
///
/// See crate-level docs for all supported invocation forms.
#[macro_export]
macro_rules! debug {
    ($l:expr, $ctx:expr, $msg:literal, $($k:ident => $v:expr),+ $(,)?) => {{
        let __v = $crate::__fold_ctx!($ctx, $($k => $v),+);
        $l.debug($msg, &__v)
    }};
    ($l:expr, $msg:literal, $($k:ident => $v:expr),+ $(,)?) => {{
        let __attrs = [$(
            $crate::LogAttribute::new(
                stringify!($k).to_string(),
                $crate::IntoAttributeValue::into_attribute_value($v),
            )?,
        )+];
        $l.debug($msg, &__attrs)
    }};
    ($l:expr, $fmt:literal, $($a:expr),+ $(,)?) => {{
        $l.debug(&format!($fmt, $($a),+), &[])
    }};
    ($l:expr, $ctx:expr, $msg:literal) => {{
        let __v = $crate::__fold_ctx!($ctx);
        $l.debug($msg, &__v)
    }};
    ($l:expr, $msg:literal) => {{
        $l.debug($msg, &[])
    }};
}

/// Emits a `Warn`-severity log record.
///
/// See crate-level docs for all supported invocation forms.
#[macro_export]
macro_rules! warn {
    ($l:expr, $ctx:expr, $msg:literal, $($k:ident => $v:expr),+ $(,)?) => {{
        let __v = $crate::__fold_ctx!($ctx, $($k => $v),+);
        $l.warn($msg, &__v)
    }};
    ($l:expr, $msg:literal, $($k:ident => $v:expr),+ $(,)?) => {{
        let __attrs = [$(
            $crate::LogAttribute::new(
                stringify!($k).to_string(),
                $crate::IntoAttributeValue::into_attribute_value($v),
            )?,
        )+];
        $l.warn($msg, &__attrs)
    }};
    ($l:expr, $fmt:literal, $($a:expr),+ $(,)?) => {{
        $l.warn(&format!($fmt, $($a),+), &[])
    }};
    ($l:expr, $ctx:expr, $msg:literal) => {{
        let __v = $crate::__fold_ctx!($ctx);
        $l.warn($msg, &__v)
    }};
    ($l:expr, $msg:literal) => {{
        $l.warn($msg, &[])
    }};
}

/// Emits an `Error`-severity log record.
///
/// See crate-level docs for all supported invocation forms.
#[macro_export]
macro_rules! error {
    ($l:expr, $ctx:expr, $msg:literal, $($k:ident => $v:expr),+ $(,)?) => {{
        let __v = $crate::__fold_ctx!($ctx, $($k => $v),+);
        $l.error($msg, &__v)
    }};
    ($l:expr, $msg:literal, $($k:ident => $v:expr),+ $(,)?) => {{
        let __attrs = [$(
            $crate::LogAttribute::new(
                stringify!($k).to_string(),
                $crate::IntoAttributeValue::into_attribute_value($v),
            )?,
        )+];
        $l.error($msg, &__attrs)
    }};
    ($l:expr, $fmt:literal, $($a:expr),+ $(,)?) => {{
        $l.error(&format!($fmt, $($a),+), &[])
    }};
    ($l:expr, $ctx:expr, $msg:literal) => {{
        let __v = $crate::__fold_ctx!($ctx);
        $l.error($msg, &__v)
    }};
    ($l:expr, $msg:literal) => {{
        $l.error($msg, &[])
    }};
}

// ── Unit Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // ── MockLogger ─────────────────────────────────────────────────────────────

    /// Records every call as (Severity, message, Vec<LogAttribute>).
    struct MockLogger {
        name: String,
        calls: Mutex<Vec<(Severity, String, Vec<LogAttribute>)>>,
        /// When Some, every `log` call returns this error.
        forced_error: Option<EmitError>,
    }

    impl MockLogger {
        fn new(name: &str) -> Self {
            MockLogger {
                name: name.to_string(),
                calls: Mutex::new(Vec::new()),
                forced_error: None,
            }
        }

        fn with_error(name: &str, err: EmitError) -> Self {
            MockLogger {
                name: name.to_string(),
                calls: Mutex::new(Vec::new()),
                forced_error: Some(err),
            }
        }

        fn recorded(&self) -> Vec<(Severity, String, Vec<LogAttribute>)> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl Logger for MockLogger {
        fn name(&self) -> &str {
            &self.name
        }

        fn log(
            &self,
            severity: Severity,
            message: &str,
            attributes: &[LogAttribute],
        ) -> Result<(), EmitError> {
            if let Some(ref err) = self.forced_error {
                return Err(err.clone());
            }
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

    // ── Phase 1: IntoAttributeValue trait ─────────────────────────────────────

    #[test]
    fn into_attribute_value_str() {
        assert_eq!(
            (&"prod").into_attribute_value(),
            LogAttributeValue::String("prod".to_string())
        );
    }

    #[test]
    fn into_attribute_value_string() {
        assert_eq!(
            "owned".to_string().into_attribute_value(),
            LogAttributeValue::String("owned".to_string())
        );
    }

    #[test]
    fn into_attribute_value_i64() {
        assert_eq!(42i64.into_attribute_value(), LogAttributeValue::Integer(42));
    }

    #[test]
    fn into_attribute_value_f64() {
        assert_eq!(
            0.42f64.into_attribute_value(),
            LogAttributeValue::Float(0.42)
        );
    }

    #[test]
    fn into_attribute_value_bool() {
        assert_eq!(
            true.into_attribute_value(),
            LogAttributeValue::Boolean(true)
        );
        assert_eq!(
            false.into_attribute_value(),
            LogAttributeValue::Boolean(false)
        );
    }

    // ── Phase 2: info! macro — all forms ─────────────────────────────────────

    #[test]
    fn info_bare_message() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        info!(logger, "hello")?;
        let calls = logger.recorded();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, Severity::Info);
        assert_eq!(calls[0].1, "hello");
        assert!(calls[0].2.is_empty());
        Ok(())
    }

    #[test]
    fn info_format_args() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        let name = "world";
        info!(logger, "hello {}", name)?;
        let calls = logger.recorded();
        assert_eq!(calls[0].1, "hello world");
        assert_eq!(calls[0].0, Severity::Info);
        Ok(())
    }

    #[test]
    fn info_single_attr() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        info!(logger, "login", user_id => 42i64)?;
        let calls = logger.recorded();
        assert_eq!(calls[0].0, Severity::Info);
        assert_eq!(calls[0].2.len(), 1);
        assert_eq!(calls[0].2[0].name(), "user_id");
        assert_eq!(calls[0].2[0].value(), &LogAttributeValue::Integer(42));
        Ok(())
    }

    #[test]
    fn info_multi_attr() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        info!(logger, "order", order_id => "abc", amount => 9.99f64)?;
        let calls = logger.recorded();
        assert_eq!(calls[0].2.len(), 2);
        assert_eq!(calls[0].2[0].name(), "order_id");
        assert_eq!(
            calls[0].2[0].value(),
            &LogAttributeValue::String("abc".to_string())
        );
        assert_eq!(calls[0].2[1].name(), "amount");
        assert_eq!(calls[0].2[1].value(), &LogAttributeValue::Float(9.99));
        Ok(())
    }

    #[test]
    fn info_ctx_bare() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        let ctx = LogContext::new()
            .with_correlation_id(CorrelationId::new("req-1".to_string()))
            .unwrap();
        info!(logger, &ctx, "msg")?;
        let calls = logger.recorded();
        assert_eq!(calls[0].0, Severity::Info);
        assert_eq!(calls[0].2.len(), 1);
        assert_eq!(calls[0].2[0].name(), "correlation_id");
        assert_eq!(
            calls[0].2[0].value(),
            &LogAttributeValue::String("req-1".to_string())
        );
        Ok(())
    }

    #[test]
    fn info_ctx_with_attr() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        let ctx = LogContext::new()
            .with_correlation_id(CorrelationId::new("req-2".to_string()))
            .unwrap();
        info!(logger, &ctx, "msg", service => "auth")?;
        let calls = logger.recorded();
        assert_eq!(calls[0].2.len(), 2);
        assert_eq!(calls[0].2[0].name(), "correlation_id");
        assert_eq!(calls[0].2[1].name(), "service");
        assert_eq!(
            calls[0].2[1].value(),
            &LogAttributeValue::String("auth".to_string())
        );
        Ok(())
    }

    // ── Phase 3: Context fold order ───────────────────────────────────────────

    #[test]
    fn fold_ctx_correlation_id_only() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        let ctx = LogContext::new()
            .with_correlation_id(CorrelationId::new("req-1".to_string()))
            .unwrap();
        info!(logger, &ctx, "msg")?;
        let calls = logger.recorded();
        assert_eq!(calls[0].2.len(), 1);
        assert_eq!(calls[0].2[0].name(), "correlation_id");
        assert_eq!(
            calls[0].2[0].value(),
            &LogAttributeValue::String("req-1".to_string())
        );
        Ok(())
    }

    #[test]
    fn fold_ctx_all_three_ids() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        let ctx = LogContext::new()
            .with_correlation_id(CorrelationId::new("c".to_string()))
            .unwrap()
            .with_trace_id(TraceId::new("t".to_string()))
            .unwrap()
            .with_span_id(SpanId::new("s".to_string()))
            .unwrap();
        info!(logger, &ctx, "msg")?;
        let calls = logger.recorded();
        assert_eq!(calls[0].2.len(), 3);
        assert_eq!(calls[0].2[0].name(), "correlation_id");
        assert_eq!(calls[0].2[1].name(), "trace_id");
        assert_eq!(calls[0].2[2].name(), "span_id");
        Ok(())
    }

    #[test]
    fn fold_ctx_attrs_only() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        let mut ctx = LogContext::new();
        ctx.add_attribute(
            LogAttribute::new(
                "env".to_string(),
                LogAttributeValue::String("prod".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
        ctx.add_attribute(
            LogAttribute::new(
                "region".to_string(),
                LogAttributeValue::String("us-east".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
        info!(logger, &ctx, "msg")?;
        let calls = logger.recorded();
        assert_eq!(calls[0].2.len(), 2);
        assert_eq!(calls[0].2[0].name(), "env");
        assert_eq!(calls[0].2[1].name(), "region");
        // No id keys present
        assert!(calls[0].2.iter().all(|a| a.name() != "correlation_id"));
        assert!(calls[0].2.iter().all(|a| a.name() != "trace_id"));
        assert!(calls[0].2.iter().all(|a| a.name() != "span_id"));
        Ok(())
    }

    #[test]
    fn fold_ctx_full_plus_inline() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
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
        info!(logger, &ctx, "msg", region => "eu")?;
        let calls = logger.recorded();
        assert_eq!(calls[0].2.len(), 5);
        assert_eq!(calls[0].2[0].name(), "env");
        assert_eq!(calls[0].2[1].name(), "correlation_id");
        assert_eq!(calls[0].2[2].name(), "trace_id");
        assert_eq!(calls[0].2[3].name(), "span_id");
        assert_eq!(calls[0].2[4].name(), "region");
        Ok(())
    }

    // ── Phase 4: Remaining macros ────────────────────────────────────────────

    #[test]
    fn trace_bare_message() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        trace!(logger, "msg")?;
        let calls = logger.recorded();
        assert_eq!(calls[0].0, Severity::Trace);
        assert_eq!(calls[0].1, "msg");
        Ok(())
    }

    #[test]
    fn trace_format_args() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        let n = 3u32;
        trace!(logger, "retry {}", n)?;
        assert_eq!(logger.recorded()[0].1, "retry 3");
        Ok(())
    }

    #[test]
    fn trace_single_attr() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        trace!(logger, "msg", key => "val")?;
        let calls = logger.recorded();
        assert_eq!(calls[0].0, Severity::Trace);
        assert_eq!(calls[0].2[0].name(), "key");
        Ok(())
    }

    #[test]
    fn trace_ctx_with_attr() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        let ctx = LogContext::new()
            .with_correlation_id(CorrelationId::new("c".to_string()))
            .unwrap();
        trace!(logger, &ctx, "msg", env => "prod")?;
        let calls = logger.recorded();
        assert_eq!(calls[0].0, Severity::Trace);
        assert_eq!(calls[0].2.len(), 2);
        Ok(())
    }

    #[test]
    fn debug_bare_message() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        debug!(logger, "msg")?;
        assert_eq!(logger.recorded()[0].0, Severity::Debug);
        Ok(())
    }

    #[test]
    fn debug_format_args() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        debug!(logger, "val={}", 42u32)?;
        assert_eq!(logger.recorded()[0].1, "val=42");
        Ok(())
    }

    #[test]
    fn debug_single_attr() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        debug!(logger, "msg", key => true)?;
        let calls = logger.recorded();
        assert_eq!(calls[0].2[0].value(), &LogAttributeValue::Boolean(true));
        Ok(())
    }

    #[test]
    fn debug_ctx_with_attr() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        let ctx = LogContext::new();
        debug!(logger, &ctx, "msg", flag => false)?;
        let calls = logger.recorded();
        assert_eq!(calls[0].0, Severity::Debug);
        assert_eq!(calls[0].2.len(), 1);
        Ok(())
    }

    #[test]
    fn warn_bare_message() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        warn!(logger, "msg")?;
        assert_eq!(logger.recorded()[0].0, Severity::Warn);
        Ok(())
    }

    #[test]
    fn warn_format_args() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        warn!(logger, "attempt {}", 2u32)?;
        assert_eq!(logger.recorded()[0].1, "attempt 2");
        Ok(())
    }

    #[test]
    fn warn_single_attr() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        warn!(logger, "msg", count => 5i64)?;
        assert_eq!(
            logger.recorded()[0].2[0].value(),
            &LogAttributeValue::Integer(5)
        );
        Ok(())
    }

    #[test]
    fn warn_ctx_with_attr() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        let ctx = LogContext::new()
            .with_span_id(SpanId::new("s1".to_string()))
            .unwrap();
        warn!(logger, &ctx, "degraded", region => "eu")?;
        let calls = logger.recorded();
        assert_eq!(calls[0].0, Severity::Warn);
        assert_eq!(calls[0].2.len(), 2);
        assert_eq!(calls[0].2[0].name(), "span_id");
        assert_eq!(calls[0].2[1].name(), "region");
        Ok(())
    }

    #[test]
    fn error_bare_message() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        error!(logger, "msg")?;
        assert_eq!(logger.recorded()[0].0, Severity::Error);
        Ok(())
    }

    #[test]
    fn error_format_args() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        error!(logger, "code {}", 500u32)?;
        assert_eq!(logger.recorded()[0].1, "code 500");
        Ok(())
    }

    #[test]
    fn error_single_attr() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        error!(logger, "crash", latency => 0.42f64)?;
        assert_eq!(
            logger.recorded()[0].2[0].value(),
            &LogAttributeValue::Float(0.42)
        );
        Ok(())
    }

    #[test]
    fn error_ctx_with_attr() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        let ctx = LogContext::new()
            .with_trace_id(TraceId::new("t1".to_string()))
            .unwrap();
        error!(logger, &ctx, "fatal", code => 500i64)?;
        let calls = logger.recorded();
        assert_eq!(calls[0].0, Severity::Error);
        assert_eq!(calls[0].2[0].name(), "trace_id");
        assert_eq!(calls[0].2[1].name(), "code");
        Ok(())
    }

    // ── Phase 5: Return type & error propagation ──────────────────────────────

    #[test]
    fn returns_ok_on_success() -> Result<(), EmitError> {
        let logger = MockLogger::new("l");
        let result = info!(logger, "msg");
        assert_eq!(result, Ok(()));
        Ok(())
    }

    #[test]
    fn propagates_logger_err() {
        let logger = MockLogger::with_error("l", EmitError::LoggerClosed);
        let result = info!(logger, "msg");
        assert_eq!(result, Err(EmitError::LoggerClosed));
    }

    #[test]
    fn propagates_validation_err_empty_message() {
        let logger = MockLogger::new("l");
        let result = info!(logger, "");
        assert_eq!(
            result,
            Err(EmitError::Validation(ValidationError::EmptyMessage))
        );
    }
}
