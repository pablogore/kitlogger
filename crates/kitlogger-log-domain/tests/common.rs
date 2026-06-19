//! Shared test utilities for kitlogger-log-domain integration tests.
#![allow(dead_code)]

use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use kitlogger_log_domain::{
    EmitError, LogAttribute, LogAttributeValue, LogContext, LogRecord, Logger, LoggerFactory,
    Severity,
};

pub fn make_attr(key: &str, val: &str) -> LogAttribute {
    LogAttribute::new(key.to_string(), LogAttributeValue::String(val.to_string())).unwrap()
}

pub fn merge_contexts(factory_ctx: &LogContext, extra: Option<LogContext>) -> LogContext {
    let extra_ctx = extra.unwrap_or_default();
    let mut merged = LogContext::new();
    for a in factory_ctx.attributes() {
        if extra_ctx
            .attributes()
            .iter()
            .any(|ea| ea.name() == a.name())
        {
            continue;
        }
        let _ = merged.add_attribute(a.clone());
    }
    for a in extra_ctx.attributes() {
        let _ = merged.add_attribute(a.clone());
    }
    merged
}

pub struct RecordingLogger {
    pub name: String,
    pub context: LogContext,
    pub emitted: Mutex<Vec<(Severity, String, Vec<LogAttribute>)>>,
}

impl RecordingLogger {
    pub fn new(name: &str, context: LogContext) -> Arc<Self> {
        Arc::new(RecordingLogger {
            name: name.to_string(),
            context,
            emitted: Mutex::new(Vec::new()),
        })
    }

    pub fn emitted_attrs(&self) -> Vec<Vec<LogAttribute>> {
        self.emitted
            .lock()
            .unwrap()
            .iter()
            .map(|(_, _, attrs)| attrs.clone())
            .collect()
    }
}

impl Logger for RecordingLogger {
    fn name(&self) -> &str {
        &self.name
    }

    fn log(
        &self,
        severity: Severity,
        message: &str,
        attributes: &[LogAttribute],
    ) -> Result<(), EmitError> {
        let mut all_attrs: Vec<LogAttribute> = self.context.attributes().to_vec();
        all_attrs.extend_from_slice(attributes);
        LogRecord::new(
            SystemTime::now(),
            severity,
            message.to_string(),
            all_attrs.clone(),
        )
        .map_err(EmitError::from)?;
        self.emitted
            .lock()
            .unwrap()
            .push((severity, message.to_string(), all_attrs));
        Ok(())
    }
}

pub struct MockFactory {
    pub default_context: LogContext,
}

impl MockFactory {
    pub fn new(default_context: LogContext) -> Self {
        MockFactory { default_context }
    }

    /// Returns a concrete `Arc<RecordingLogger>` for attribute inspection in tests.
    pub fn create_recording_logger(
        &self,
        name: &str,
        extra_context: Option<LogContext>,
    ) -> Arc<RecordingLogger> {
        let merged = merge_contexts(&self.default_context, extra_context);
        RecordingLogger::new(name, merged)
    }
}

impl LoggerFactory for MockFactory {
    fn create_logger(&self, name: &str) -> Arc<dyn Logger> {
        RecordingLogger::new(name, self.default_context.clone())
    }

    fn create_logger_with_context(
        &self,
        name: &str,
        default_context: Option<LogContext>,
    ) -> Arc<dyn Logger> {
        let merged = merge_contexts(&self.default_context, default_context);
        RecordingLogger::new(name, merged)
    }
}
