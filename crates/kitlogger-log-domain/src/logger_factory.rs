//! LoggerFactory trait — canonical domain contract for creating named Logger instances.

use std::sync::Arc;

use crate::{LogContext, Logger};

/// A thread-safe factory for creating named [`Logger`] instances.
///
/// `LoggerFactory` is object-safe and `Send + Sync`, so it can be shared across
/// threads and held behind `Arc<dyn LoggerFactory>`.
///
/// Factories carry an optional *default context* that is merged into every logger
/// they produce. The merge strategy is last-wins: factory context attributes form
/// the base, and any logger-local attributes passed via `default_context` override
/// factory attributes with the same key.
pub trait LoggerFactory: Send + Sync {
    /// Creates a named logger with no additional context.
    ///
    /// The returned logger inherits the factory's default context (if any).
    fn create_logger(&self, name: &str) -> Arc<dyn Logger>;

    /// Creates a named logger, optionally merging additional context.
    ///
    /// When `default_context` is `Some`, its attributes are merged on top of the
    /// factory's own default context (last-wins by attribute key). When `None`,
    /// the logger inherits only the factory's default context.
    fn create_logger_with_context(
        &self,
        name: &str,
        default_context: Option<LogContext>,
    ) -> Arc<dyn Logger>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EmitError, LogAttribute, LogAttributeValue, LogContext, LogRecord, Logger, Severity,
    };
    use std::sync::Mutex;
    use std::time::SystemTime;

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Builds a `LogAttribute` with a string value — shorthand for tests.
    pub(super) fn make_attr(key: &str, val: &str) -> LogAttribute {
        LogAttribute::new(key.to_string(), LogAttributeValue::String(val.to_string()))
            .expect("attribute creation should succeed in tests")
    }

    /// Merges `factory_ctx` and `extra_ctx` using last-wins by key.
    ///
    /// Factory attrs form the base; logger-local attrs override by key.
    pub(super) fn merge_contexts(
        factory_ctx: &LogContext,
        extra: Option<LogContext>,
    ) -> LogContext {
        let extra_ctx = extra.unwrap_or_default();
        let mut merged = LogContext::new();

        // Add factory attrs first (base layer), skipping those overridden by extra.
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
        // Add logger-local attrs (override layer).
        for a in extra_ctx.attributes() {
            let _ = merged.add_attribute(a.clone());
        }
        merged
    }

    // ── Minimal mock Logger ───────────────────────────────────────────────────

    pub(super) struct RecordingLogger {
        pub(super) name: String,
        pub(super) context: LogContext,
        pub(super) emitted: Mutex<Vec<(Severity, String, Vec<LogAttribute>)>>,
    }

    impl RecordingLogger {
        pub(super) fn new(name: &str, context: LogContext) -> Arc<Self> {
            Arc::new(RecordingLogger {
                name: name.to_string(),
                context,
                emitted: Mutex::new(Vec::new()),
            })
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
            // Build the full attribute list: context attrs ++ per-emit attrs.
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

    // ── Minimal mock LoggerFactory ────────────────────────────────────────────

    pub(super) struct MockFactory {
        pub(super) default_context: LogContext,
    }

    impl MockFactory {
        pub(super) fn new(default_context: LogContext) -> Self {
            MockFactory { default_context }
        }

        /// Returns a concrete `Arc<RecordingLogger>` so tests can inspect merged attrs directly.
        pub(super) fn create_recording_logger(
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

    // ── Object-safety smoke tests ─────────────────────────────────────────────

    #[test]
    fn logger_factory_is_object_safe_box() {
        let _: Option<Box<dyn LoggerFactory>> = None;
    }

    #[test]
    fn logger_factory_is_object_safe_arc() {
        let _: Option<Arc<dyn LoggerFactory>> = None;
    }

    // ── create_logger ─────────────────────────────────────────────────────────

    #[test]
    fn create_logger_returns_logger_with_correct_name() {
        let factory = MockFactory::new(LogContext::new());
        let logger = factory.create_logger("auth");
        assert_eq!(logger.name(), "auth");
    }

    // ── create_logger_with_context(name, None) ────────────────────────────────

    #[test]
    fn create_logger_with_context_none_inherits_empty_context() {
        let factory = MockFactory::new(LogContext::new());
        let logger = factory.create_logger_with_context("svc", None);
        assert_eq!(logger.name(), "svc");
        assert!(logger.info("hello", &[]).is_ok());
    }

    // ── create_logger_with_context(name, Some(ctx)) ───────────────────────────

    #[test]
    fn create_logger_with_context_some_carries_factory_context_attrs() {
        let mut factory_ctx = LogContext::new();
        factory_ctx
            .add_attribute(make_attr("service", "my-svc"))
            .unwrap();
        let factory = MockFactory::new(factory_ctx);

        // Use the concrete helper to inspect merged context attrs without unsafe downcast.
        let logger = factory.create_recording_logger("auth", None);

        let attr_names: Vec<&str> = logger.context.attributes().iter().map(|a| a.name()).collect();
        assert!(
            attr_names.contains(&"service"),
            "factory service attr missing from logger context: {:?}",
            attr_names
        );
        assert_eq!(logger.name(), "auth");
    }

    // ── Factory default context is not modified after logger creation ─────────

    #[test]
    fn factory_default_context_not_modified_after_logger_creation() {
        let mut factory_ctx = LogContext::new();
        factory_ctx
            .add_attribute(make_attr("service", "my-svc"))
            .unwrap();
        let factory = MockFactory::new(factory_ctx);

        let mut extra = LogContext::new();
        extra.add_attribute(make_attr("component", "auth")).unwrap();
        let _ = factory.create_logger_with_context("auth", Some(extra));

        // Factory's default_context must still have only the original attr.
        assert_eq!(factory.default_context.attributes().len(), 1);
        assert_eq!(factory.default_context.attributes()[0].name(), "service");
    }

    // ── Merge semantics: factory attr + logger extra attr both present ────────

    #[test]
    fn merge_contexts_includes_both_factory_and_extra_attrs() {
        let mut factory_ctx = LogContext::new();
        factory_ctx
            .add_attribute(make_attr("service", "my-svc"))
            .unwrap();

        let mut extra = LogContext::new();
        extra.add_attribute(make_attr("component", "auth")).unwrap();

        let merged = merge_contexts(&factory_ctx, Some(extra));
        let names: Vec<&str> = merged.attributes().iter().map(|a| a.name()).collect();
        assert!(
            names.contains(&"service"),
            "factory attr missing: {:?}",
            names
        );
        assert!(
            names.contains(&"component"),
            "extra attr missing: {:?}",
            names
        );
    }

    // ── Merge semantics: last-wins on key collision ───────────────────────────

    #[test]
    fn merge_contexts_last_wins_on_key_collision() {
        let mut factory_ctx = LogContext::new();
        factory_ctx
            .add_attribute(make_attr("env", "staging"))
            .unwrap();

        let mut extra = LogContext::new();
        extra.add_attribute(make_attr("env", "prod")).unwrap();

        let merged = merge_contexts(&factory_ctx, Some(extra));
        let env_attr = merged
            .attributes()
            .iter()
            .find(|a| a.name() == "env")
            .expect("env attr must exist");
        assert_eq!(
            env_attr.value(),
            &LogAttributeValue::String("prod".to_string())
        );
    }
}
