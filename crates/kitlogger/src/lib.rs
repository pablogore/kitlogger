use async_trait::async_trait;
use console_exporter::{ConsoleExporter, ConsoleExporterImpl, OnShutdownFlush};
use kit_config::{LogLevel, LoggingConfig, Validation, ValidationReport};
use kitlogger_formatter::{formatter_from_config, LogFormat, RecordFormatter};
use kitlogger_log_domain::{Clock, LogContext, LogRecord, Severity, UtcClock};
use kitlogger_redaction::Redactor;
use kitlogger_sampling::Sampler;
use output_adapter_contracts::{DispatchOutcome, Output, OutputError, OutputId, Registry};
use std::sync::Arc;
use std::time::SystemTime;
use telemetry_adapter_contracts::{
    AdapterError, AdapterHealth, AdapterId, AdapterResult, CommonAdapterBase, ExporterAdapter,
    HealthReport, LifecycleAdapter, TelemetryDelivery,
};
use telemetry_types::PayloadEnvelope;

// `file-exporter` is a dependency of this crate per proposal.md ("built,
// not yet registered") — see design.md's "A Gap Found" section for why it
// cannot be wired into the dispatch registry in this phase. It is
// deliberately unreferenced below; this comment (not a `use`) is the only
// acknowledgment of the dependency edge, per task 5.4's constraint.

pub mod buffer;
pub mod format_selection;

/// Maps a `kit_config::LogLevel` (five variants) to the `Severity` (six
/// variants) it names as the level-filter threshold.
///
/// `kit_config::LogLevel` has no `Fatal` variant — there is no
/// `LogLevel` value this function could map to `Severity::Fatal` as a
/// threshold, which is exactly why `passes_level_filter` never needs a
/// separate `Fatal`-always-proceeds branch: `Severity::Fatal` already
/// compares `>=` any threshold this function can produce.
fn level_floor(level: LogLevel) -> Severity {
    match level {
        LogLevel::Trace => Severity::Trace,
        LogLevel::Debug => Severity::Debug,
        LogLevel::Info => Severity::Info,
        LogLevel::Warn => Severity::Warn,
        LogLevel::Error => Severity::Error,
    }
}

/// Thin, `kitlogger`-local translation from `ConsoleExporterImpl` to the
/// generic `output_adapter_contracts::Output` Port the dispatch registry
/// requires.
///
/// `ConsoleExporterImpl` already implements `Output` directly (see
/// console-exporter's `exporter.rs`) — this wrapper exists only because
/// `Registry::register` takes ownership of a `Box<dyn Output>`, while
/// `KITLogger` also needs to keep calling `init`/`flush`/`shutdown` on the
/// SAME exporter instance directly. Sharing it via `Arc` (rather than
/// registering a second, separately owned `ConsoleExporterImpl`) is what
/// keeps this a single dispatch mechanism (FR-010) instead of two.
struct ConsoleOutputAdapter(Arc<ConsoleExporterImpl>);

impl Output for ConsoleOutputAdapter {
    fn dispatch(&self, formatted: &str, severity: Severity) -> Result<(), OutputError> {
        self.0
            .export(formatted, severity)
            .map_err(|e| OutputError::new(e.to_string()))
    }
}

pub struct KITLogger {
    exporter: Arc<ConsoleExporterImpl>,
    formatter: Box<dyn RecordFormatter>,
    id: AdapterId,
    /// Materialized from `LoggingConfig` at construction (per ADR-010,
    /// config models are construction-time only — the runtime retains just
    /// the state it actually executes against, not the whole config).
    enabled: bool,
    level: LogLevel,
    buffering_enabled: bool,
    sampler: Sampler,
    redactor: Redactor,
    buffer: buffer::Buffer,
    /// The sole dispatch mechanism (FR-010) — `KITLogger` holds exactly one
    /// `output_adapter_contracts::Registry` instance. No second
    /// registry/`LoggerProvider`-shaped type exists anywhere in this crate.
    registry: Registry,
    /// Tracks which `OutputId`s were registered into `registry`, for
    /// inspection (see `registered_output_ids`) — `Registry` itself does
    /// not expose enumeration, and this crate must not modify that
    /// already-frozen capability (see proposal.md's "Out of Scope").
    ///
    /// INVARIANT: this must always list exactly the `OutputId`s registered
    /// into `registry`, in the same order — the two are updated together in
    /// `build`. There is currently only one call site (`build`) that
    /// registers anything, so this holds trivially; if a future change adds
    /// a second call site that registers into `registry` (e.g. optionally
    /// registering `file-exporter`), that call site MUST push the same
    /// `OutputId` here, or `registered_output_ids()` will silently lie
    /// about what's actually registered.
    output_ids: Vec<OutputId>,
}

impl Default for KITLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl KITLogger {
    /// Shared construction path: every public constructor below funnels
    /// through here so `Sampler`/`Redactor` (and, from Phase 3/4 onward,
    /// `Buffer`/the dispatch registry) are always built consistently from
    /// `config`, regardless of which constructor a caller used. Always uses
    /// `UtcClock` — production code has no path to construct a `KITLogger`
    /// with any other clock (see `build_with_clock`, below, for why that's
    /// deliberate).
    fn build(
        config: LoggingConfig,
        exporter: Arc<ConsoleExporterImpl>,
        formatter: Box<dyn RecordFormatter>,
    ) -> Self {
        let clock: Arc<dyn Clock> = Arc::new(UtcClock);
        let enabled = config.enabled;
        let level = config.level;
        let buffering_enabled = config.buffering.enabled;
        let sampler = Sampler::new(config.sampling, clock.clone());
        let redactor = Redactor::new(config.redact);
        let buffer = buffer::Buffer::new(config.buffering, clock);

        // FR-009: register a console output by default; explicitly do NOT
        // register a file-based one (see the module-level comment on
        // `file-exporter`'s dependency edge above).
        let mut registry = Registry::new();
        let console_id = OutputId::new("console");
        registry
            .register(
                console_id.clone(),
                Box::new(ConsoleOutputAdapter(exporter.clone())),
            )
            .expect("a freshly built registry has no prior registration under 'console'");

        Self {
            exporter,
            formatter,
            id: AdapterId::new("kitlogger").expect("hardcoded id should never be empty"),
            enabled,
            level,
            buffering_enabled,
            sampler,
            redactor,
            buffer,
            registry,
            output_ids: vec![console_id],
        }
    }

    /// `build`'s clock-parameterized core. Only compiled under
    /// `#[cfg(test)]` so it cannot become a second, production-reachable
    /// construction path that bypasses `UtcClock` — `build` (above) is
    /// non-test code's only route to this logic, and always supplies
    /// `UtcClock` inline. Tests reach this directly to exercise time-driven
    /// behavior (e.g. `Buffer`'s time-based flush) deterministically,
    /// without a real sleep.
    #[cfg(test)]
    fn build_with_clock(
        config: LoggingConfig,
        exporter: Arc<ConsoleExporterImpl>,
        formatter: Box<dyn RecordFormatter>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        let enabled = config.enabled;
        let level = config.level;
        let buffering_enabled = config.buffering.enabled;
        let sampler = Sampler::new(config.sampling, clock.clone());
        let redactor = Redactor::new(config.redact);
        let buffer = buffer::Buffer::new(config.buffering, clock);

        // FR-009: register a console output by default; explicitly do NOT
        // register a file-based one (see the module-level comment on
        // `file-exporter`'s dependency edge above).
        let mut registry = Registry::new();
        let console_id = OutputId::new("console");
        registry
            .register(
                console_id.clone(),
                Box::new(ConsoleOutputAdapter(exporter.clone())),
            )
            .expect("a freshly built registry has no prior registration under 'console'");

        Self {
            exporter,
            formatter,
            id: AdapterId::new("kitlogger").expect("hardcoded id should never be empty"),
            enabled,
            level,
            buffering_enabled,
            sampler,
            redactor,
            buffer,
            registry,
            output_ids: vec![console_id],
        }
    }

    /// Creates a `KITLogger` with the default JSON formatter and a new
    /// `ConsoleExporterImpl` using `OnShutdownFlush`.
    pub fn new() -> Self {
        let exporter = Arc::new(ConsoleExporterImpl::with_flush_strategy(Box::new(
            OnShutdownFlush,
        )));
        Self::build(
            LoggingConfig::default(),
            exporter,
            formatter_from_config(LogFormat::Json),
        )
    }

    /// Creates a `KITLogger` with a custom formatter selected by `format`.
    pub fn with_format(format: LogFormat) -> Self {
        let exporter = Arc::new(ConsoleExporterImpl::with_flush_strategy(Box::new(
            OnShutdownFlush,
        )));
        Self::build(
            LoggingConfig::default(),
            exporter,
            formatter_from_config(format),
        )
    }

    /// Creates a `KITLogger` from a `kit_config::LoggingConfig` value.
    ///
    /// `config` is validated via `kit_config`'s `Validation` trait at
    /// construction time; an invalid config fails fast and the caller receives
    /// the `ValidationReport` describing why.
    pub fn from_logging_config(config: LoggingConfig) -> Result<Self, ValidationReport> {
        config.validate()?;
        let exporter = Arc::new(ConsoleExporterImpl::with_flush_strategy(Box::new(
            OnShutdownFlush,
        )));
        let formatter = formatter_from_config(format_selection::map_log_format(config.format));
        Ok(Self::build(config, exporter, formatter))
    }

    /// Creates a `KITLogger` from a `kit_config::LoggingConfig` value, wired
    /// to a caller-supplied exporter.
    ///
    /// Intended for testing: combines `from_logging_config`'s config-driven
    /// pipeline behavior with `with_exporter_and_format`'s ability to supply
    /// a pre-initialized `ConsoleExporterImpl` with capture buffers attached
    /// — the two constructors below cannot each individually provide both
    /// levers a pipeline test needs (a fully specified `LoggingConfig`, and
    /// an observable exporter).
    pub fn from_logging_config_with_exporter(
        config: LoggingConfig,
        exporter: Arc<ConsoleExporterImpl>,
    ) -> Result<Self, ValidationReport> {
        config.validate()?;
        let formatter = formatter_from_config(format_selection::map_log_format(config.format));
        Ok(Self::build(config, exporter, formatter))
    }

    /// Creates a `KITLogger` wired to a pre-built exporter with a given format.
    ///
    /// Intended for testing: callers supply a `ConsoleExporterImpl` with
    /// custom `set_writers` capture buffers already attached and initialized.
    pub fn with_exporter_and_format(exporter: Arc<ConsoleExporterImpl>, format: LogFormat) -> Self {
        Self::build(
            LoggingConfig::default(),
            exporter,
            formatter_from_config(format),
        )
    }

    /// Initializes the underlying console exporter.
    pub fn init(&self) -> Result<(), AdapterError> {
        self.exporter
            .init()
            .map_err(|e| AdapterError::InitializationFailed(e.to_string()))
    }

    /// Formats `record` using the configured formatter and exports the result.
    ///
    /// This is the primary entry point for structured logging. Per
    /// `kitlogger-emission-pipeline`, `record` passes, in order: the enabled
    /// gate (FR-001), the level filter (FR-002), the sampling gate (FR-003),
    /// redaction (FR-004), and buffering (FR-005) before formatting/dispatch
    /// — a call dropped by the enabled gate, the level filter, or sampling
    /// returns `Ok(())` without touching redaction, buffering, the
    /// formatter, or the exporter. A call that reaches buffering but does
    /// not yet trigger a flush also returns `Ok(())` — "accepted into the
    /// pipeline through buffering," not "written to every output" (see
    /// design.md's "Buffering's Effect on `log()`'s Observable Timing").
    pub fn log_record(
        &self,
        record: &LogRecord,
        context: Option<&LogContext>,
    ) -> Result<(), AdapterError> {
        if !self.passes_enabled_gate() || !self.passes_level_filter(record.severity()) {
            return Ok(());
        }
        if !self.sampler.should_sample() {
            return Ok(());
        }

        let redacted = self.redactor.redact(record);
        match self.buffer.add(redacted) {
            Some(batch) => self.format_and_dispatch(&batch, context),
            // FR-005: the buffer flushes on whichever of size or elapsed
            // time is reached first. `add` above only ever resolves the
            // size condition; the time condition is checked here, on every
            // call that doesn't already trigger a size-based flush, so
            // `BufferingConfig.flush_interval_ms` has an actual effect.
            None => match self.buffer.try_flush() {
                Some(batch) => self.format_and_dispatch(&batch, None),
                None => Ok(()),
            },
        }
    }

    /// Exports a raw string directly (back-compat path).
    ///
    /// Delegates entirely into `log_record`'s pipeline (constructing a
    /// minimal `LogRecord` from `severity`/`message`) so there is exactly
    /// one implementation of the enabled gate / level filter / sampling /
    /// redaction / buffering / format / dispatch sequence, not two
    /// duplicated ones. This means `log()` is no longer a raw,
    /// formatter-free passthrough — per `kitlogger-emission-pipeline`,
    /// both entry points share the full pipeline.
    pub fn log(&self, severity: Severity, message: &str) -> Result<(), AdapterError> {
        let record = LogRecord::new(SystemTime::now(), severity, message.to_string(), Vec::new())
            .map_err(|e| AdapterError::InitializationFailed(e.to_string()))?;
        self.log_record(&record, None)
    }

    /// Formats and dispatches every record in `batch` (a buffer flush's
    /// output). `context` is only meaningful for `batch` as a whole when it
    /// is guaranteed to describe every record in it — true only when
    /// buffering is disabled, in which case `Buffer::add` always returns a
    /// single-element batch containing exactly the record `context`
    /// describes. For a deferred, multi-record flush, no single record's
    /// context is knowable from here, so formatting proceeds without one.
    ///
    /// `batch` has already been removed from the buffer by the caller (via
    /// `Buffer::add`/`try_flush`/`drain`, all of which `mem::take` their
    /// internal storage) — there is no path back into the buffer for a
    /// record once it reaches this method. A failure on one record MUST
    /// NOT stop the remaining records in `batch` from being attempted, or
    /// they would be lost with no trace: neither re-buffered nor dispatched.
    /// Every record is attempted regardless of earlier failures; failures
    /// are collected and reported together once the whole batch has been
    /// tried.
    fn format_and_dispatch(
        &self,
        batch: &[LogRecord],
        context: Option<&LogContext>,
    ) -> Result<(), AdapterError> {
        let context = if self.buffering_enabled {
            None
        } else {
            context
        };
        let mut failures = Vec::new();
        for record in batch {
            match self.formatter.format(record, context) {
                Ok(formatted) => {
                    if let Err(e) = self.dispatch(&formatted, *record.severity()) {
                        failures.push(e.to_string());
                    }
                }
                Err(e) => failures.push(e.to_string()),
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(AdapterError::InitializationFailed(failures.join("; ")))
        }
    }

    /// FR-010: the one place a formatted record reaches an output — always
    /// through `self.registry`, `KITLogger`'s sole dispatch mechanism.
    fn dispatch(&self, formatted: &str, severity: Severity) -> Result<(), AdapterError> {
        match self.registry.dispatch(formatted, severity) {
            DispatchOutcome::AllSucceeded => Ok(()),
            DispatchOutcome::PartialFailure(failures) | DispatchOutcome::AllFailed(failures) => {
                let reasons = failures
                    .iter()
                    .map(|(id, err)| format!("{id}: {err}"))
                    .collect::<Vec<_>>()
                    .join("; ");
                Err(AdapterError::InitializationFailed(reasons))
            }
        }
    }

    /// Returns the `OutputId`s currently registered into this `KITLogger`'s
    /// dispatch registry — inspection support for FR-009's contract
    /// ("console is present, file is not").
    pub fn registered_output_ids(&self) -> &[OutputId] {
        &self.output_ids
    }

    /// FR-001: `LoggingConfig.enabled == false` blocks all further pipeline
    /// processing.
    fn passes_enabled_gate(&self) -> bool {
        self.enabled
    }

    /// FR-002: a record proceeds only if its severity is at or above the
    /// threshold named by `LoggingConfig.level`. `Severity::Fatal` always
    /// proceeds — it is `Severity`'s maximal variant (see `severity.rs`'s
    /// `Ord` derive), so it is >= every threshold `level_floor` can produce,
    /// with no special-case branch required.
    fn passes_level_filter(&self, severity: &Severity) -> bool {
        *severity >= level_floor(self.level)
    }

    /// Flushes the underlying console exporter, having first drained and
    /// dispatched every record currently held in the buffer (FR-006) — a
    /// caller observing this call's return MUST see every record accepted
    /// so far as formatted and dispatched, not silently pending.
    pub fn flush(&self) -> Result<(), AdapterError> {
        self.drain_buffer()?;
        self.exporter
            .flush()
            .map_err(|e| AdapterError::FlushFailed(e.to_string()))
    }

    /// Shuts down the underlying console exporter, having first drained and
    /// dispatched every record currently held in the buffer (FR-006) — see
    /// `flush`'s doc comment; the same guarantee applies here.
    pub fn shutdown(&self) -> Result<(), AdapterError> {
        self.drain_buffer()?;
        self.exporter
            .shutdown()
            .map_err(|e| AdapterError::ShutdownFailed(e.to_string()))
    }

    /// FR-006: force-drains every record currently held in the buffer
    /// (regardless of whether its own flush conditions have been met) and
    /// formats/dispatches all of them. No single record's `LogContext` is
    /// available at this point (drained records may have been added by
    /// many different calls with different contexts, or none at all), so
    /// formatting proceeds without one — the same rule `format_and_dispatch`
    /// already applies to any multi-record, buffering-enabled flush.
    fn drain_buffer(&self) -> Result<(), AdapterError> {
        let drained = self.buffer.drain();
        if drained.is_empty() {
            return Ok(());
        }
        self.format_and_dispatch(&drained, None)
    }
}

impl CommonAdapterBase for KITLogger {
    fn id(&self) -> &AdapterId {
        &self.id
    }

    fn health(&self) -> HealthReport {
        HealthReport {
            status: AdapterHealth::Healthy,
            reason: "kitlogger".into(),
            timestamp: SystemTime::now(),
        }
    }
}

#[async_trait]
impl LifecycleAdapter for KITLogger {
    async fn flush(&self) -> AdapterResult<()> {
        // Delegates to the inherent `flush` (fully qualified to avoid
        // recursing into this same trait method) so buffer drainage
        // (FR-006) has exactly one implementation.
        KITLogger::flush(self)
    }

    async fn shutdown(&self) -> AdapterResult<()> {
        KITLogger::shutdown(self)
    }
}

#[async_trait]
impl TelemetryDelivery for KITLogger {
    async fn deliver(&self, _envelope: PayloadEnvelope) -> AdapterResult<()> {
        Ok(())
    }
}

#[async_trait]
impl ExporterAdapter for KITLogger {
    async fn initialize(&self) -> AdapterResult<()> {
        self.init()
    }

    async fn start(&self) -> AdapterResult<()> {
        Ok(())
    }

    async fn stop(&self) -> AdapterResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Duration as ChronoDuration, Utc};
    use kit_config::BufferingConfig;
    use std::sync::Mutex as StdMutex;

    /// Test-only, programmatically advanceable `Clock` — same rationale as
    /// `buffer::Buffer`'s own test-only clock: the canonical `FakeClock` is
    /// immutable after construction and can't exercise interval-elapsed
    /// scenarios. Implements the same canonical `Clock` trait (no
    /// competing abstraction, per ADR-010).
    struct AdvanceableClock(StdMutex<DateTime<Utc>>);

    impl AdvanceableClock {
        fn new(start: DateTime<Utc>) -> Self {
            AdvanceableClock(StdMutex::new(start))
        }

        fn advance(&self, duration: ChronoDuration) {
            let mut current = self.0.lock().unwrap();
            *current += duration;
        }
    }

    impl Clock for AdvanceableClock {
        fn now(&self) -> DateTime<Utc> {
            *self.0.lock().unwrap()
        }
    }

    /// A fake `Output` that always fails, counting how many times it was
    /// actually invoked.
    struct CountingFailingOutput(Arc<std::sync::atomic::AtomicUsize>);

    impl Output for CountingFailingOutput {
        fn dispatch(&self, _formatted: &str, _severity: Severity) -> Result<(), OutputError> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err(OutputError::new("simulated failure"))
        }
    }

    /// Regression test for the batch-abort-on-first-error bug: a batch is
    /// already removed from `Buffer` (via `mem::take`) before any of its
    /// records are formatted/dispatched, so a record `format_and_dispatch`
    /// doesn't attempt is lost with no trace. Registers a counting fake
    /// `Output` directly (bypassing `console-exporter` entirely) so the
    /// assertion is a call count, not a string match on an error's wording.
    #[test]
    fn batch_dispatch_failure_still_attempts_every_record() {
        let attempts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut registry = Registry::new();
        registry
            .register(
                OutputId::new("failing"),
                Box::new(CountingFailingOutput(attempts.clone())),
            )
            .unwrap();

        let exporter = Arc::new(ConsoleExporterImpl::with_flush_strategy(Box::new(
            OnShutdownFlush,
        )));
        exporter.init().unwrap();

        let config = LoggingConfig {
            buffering: BufferingConfig {
                enabled: true,
                batch_size: 3,
                flush_interval_ms: 60_000,
            },
            ..LoggingConfig::default()
        };
        let clock: Arc<dyn Clock> = Arc::new(UtcClock);
        let logger = KITLogger {
            registry,
            output_ids: vec![OutputId::new("failing")],
            ..KITLogger::build_with_clock(
                config,
                exporter,
                formatter_from_config(LogFormat::Json),
                clock,
            )
        };

        let record_of = |msg: &str| {
            LogRecord::new(
                SystemTime::now(),
                Severity::Info,
                msg.to_string(),
                Vec::new(),
            )
            .unwrap()
        };

        logger.log_record(&record_of("one"), None).unwrap();
        logger.log_record(&record_of("two"), None).unwrap();
        let result = logger.log_record(&record_of("three"), None);

        assert!(
            result.is_err(),
            "dispatch to a permanently-failing output must return Err"
        );
        assert_eq!(
            attempts.load(std::sync::atomic::Ordering::SeqCst),
            3,
            "all 3 records in the batch must be attempted even though each fails"
        );
    }

    struct VecWriter(Arc<StdMutex<Vec<u8>>>);
    impl std::io::Write for VecWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// Regression test for FR-005: `log_record` must check the buffer's
    /// time-based flush condition, not just its size-based one, on every
    /// call — otherwise `BufferingConfig.flush_interval_ms` has no effect
    /// in the actual pipeline (only `buffer::Buffer`'s own unit tests would
    /// ever exercise `try_flush`). Uses an advanceable clock to move time
    /// forward deterministically, without a real sleep.
    #[test]
    fn log_record_checks_time_based_flush_when_size_has_not_been_reached() {
        let stdout = Arc::new(StdMutex::new(Vec::new()));

        let exporter = Arc::new(ConsoleExporterImpl::with_flush_strategy(Box::new(
            OnShutdownFlush,
        )));
        exporter.set_writers(
            Box::new(VecWriter(stdout.clone())),
            Box::new(VecWriter(stdout.clone())),
        );
        exporter.init().unwrap();

        let config = LoggingConfig {
            buffering: BufferingConfig {
                enabled: true,
                batch_size: 100, // never reached in this test
                flush_interval_ms: 50,
            },
            ..LoggingConfig::default()
        };
        let clock = Arc::new(AdvanceableClock::new(DateTime::UNIX_EPOCH));
        let logger = KITLogger::build_with_clock(
            config,
            exporter,
            formatter_from_config(LogFormat::Json),
            clock.clone(),
        );

        let first = LogRecord::new(
            SystemTime::now(),
            Severity::Info,
            "held-then-time-flushed".to_string(),
            Vec::new(),
        )
        .unwrap();
        logger.log_record(&first, None).unwrap();
        assert!(
            stdout.lock().unwrap().is_empty(),
            "a single record below batch_size must not be dispatched yet"
        );

        clock.advance(ChronoDuration::milliseconds(51));

        let second = LogRecord::new(
            SystemTime::now(),
            Severity::Info,
            "second".to_string(),
            Vec::new(),
        )
        .unwrap();
        logger.log_record(&second, None).unwrap();

        let out = String::from_utf8(stdout.lock().unwrap().clone()).unwrap();
        assert!(
            out.contains("held-then-time-flushed") && out.contains("second"),
            "elapsed flush_interval_ms must trigger a flush via the opportunistic \
             try_flush check, dispatching both buffered records. Got: {out:?}"
        );
    }
}
