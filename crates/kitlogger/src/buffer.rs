//! Pre-format record batching, deferring format + dispatch cost to flush
//! time.
//!
//! Per ADR-008 §5 (filter -> sample -> redact -> buffer -> format ->
//! dispatch), this buffer holds raw, pre-format `LogRecord`s — it has no
//! knowledge of formatting or output destinations. Internal to `kitlogger`
//! (not its own crate — see design.md Q4). Not yet wired into `KITLogger`'s
//! emission path; that is Phase 5 (Orchestration Fold), a separate future
//! change.

use std::sync::Mutex;

use chrono::{DateTime, Duration, Utc};
use kit_config::BufferingConfig;
use kitlogger_log_domain::{Clock, LogRecord};

/// Batches pre-format records per `BufferingConfig`, flushing on whichever
/// of size or elapsed time is reached first.
pub struct Buffer {
    config: BufferingConfig,
    clock: std::sync::Arc<dyn Clock>,
    records: Mutex<Vec<LogRecord>>,
    window_start: Mutex<Option<DateTime<Utc>>>,
}

impl Buffer {
    /// Creates a new `Buffer` from `config`, sourcing the passage of time
    /// exclusively through `clock` (FR-006) — never
    /// `Instant::now()`/`SystemTime::now()` directly.
    pub fn new(config: BufferingConfig, clock: std::sync::Arc<dyn Clock>) -> Self {
        Buffer {
            config,
            clock,
            records: Mutex::new(Vec::new()),
            window_start: Mutex::new(None),
        }
    }

    /// Adds `record`.
    ///
    /// When buffering is disabled (FR-003), `record` is returned
    /// immediately as a single-element batch — no accumulation occurs.
    /// When enabled, `record` is held until `BufferingConfig.batch_size`
    /// records have accumulated (FR-001), at which point the whole batch
    /// is returned in insertion order (FR-004).
    pub fn add(&self, record: LogRecord) -> Option<Vec<LogRecord>> {
        if !self.config.enabled {
            return Some(vec![record]);
        }

        let mut records = self.records.lock().unwrap();
        let mut window_start = self.window_start.lock().unwrap();

        if records.is_empty() {
            *window_start = Some(self.clock.now());
        }
        records.push(record);

        if records.len() >= self.config.batch_size {
            *window_start = None;
            return Some(std::mem::take(&mut records));
        }

        None
    }

    /// Flushes the currently held batch if `BufferingConfig.flush_interval_ms`
    /// has elapsed since the first unflushed record was added (FR-002),
    /// regardless of how many records have accumulated since. Returns
    /// `None` if nothing is held, or if the interval has not yet elapsed.
    pub fn try_flush(&self) -> Option<Vec<LogRecord>> {
        let mut records = self.records.lock().unwrap();
        let mut window_start = self.window_start.lock().unwrap();

        let started_at = (*window_start)?;
        let elapsed = self.clock.now() - started_at;

        if elapsed >= Duration::milliseconds(self.config.flush_interval_ms as i64) {
            *window_start = None;
            return Some(std::mem::take(&mut records));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kitlogger_log_domain::LogAttribute;
    use std::sync::{Arc, Mutex as StdMutex};
    use std::time::SystemTime;

    fn config(enabled: bool, batch_size: usize, flush_interval_ms: u64) -> BufferingConfig {
        BufferingConfig {
            enabled,
            batch_size,
            flush_interval_ms,
        }
    }

    fn record(message: &str) -> LogRecord {
        LogRecord::new(
            SystemTime::now(),
            kitlogger_log_domain::Severity::Info,
            message.to_string(),
            Vec::<LogAttribute>::new(),
        )
        .unwrap()
    }

    /// Test-only, programmatically advanceable `Clock`. The canonical
    /// `kitlogger_log_domain::FakeClock` is immutable after construction
    /// and therefore cannot exercise interval-elapsed scenarios. This
    /// implements the SAME canonical `Clock` trait (no competing
    /// abstraction, per ADR-010) — it just adds test-only mutability.
    struct AdvanceableClock(StdMutex<DateTime<Utc>>);

    impl AdvanceableClock {
        fn new(start: DateTime<Utc>) -> Self {
            AdvanceableClock(StdMutex::new(start))
        }

        fn advance(&self, duration: Duration) {
            let mut current = self.0.lock().unwrap();
            *current += duration;
        }
    }

    impl Clock for AdvanceableClock {
        fn now(&self) -> DateTime<Utc> {
            *self.0.lock().unwrap()
        }
    }

    fn epoch() -> DateTime<Utc> {
        DateTime::UNIX_EPOCH
    }

    #[test]
    fn batch_size_triggers_flush() {
        let clock = Arc::new(AdvanceableClock::new(epoch()));
        let buffer = Buffer::new(config(true, 3, 60_000), clock);

        assert!(buffer.add(record("one")).is_none());
        assert!(buffer.add(record("two")).is_none());
        let flushed = buffer.add(record("three")).expect("batch size reached");

        assert_eq!(flushed.len(), 3);
    }

    #[test]
    fn flush_interval_triggers_flush_before_batch_size() {
        let clock = Arc::new(AdvanceableClock::new(epoch()));
        let buffer = Buffer::new(config(true, 100, 50), clock.clone());

        assert!(buffer.add(record("one")).is_none());
        assert!(buffer.add(record("two")).is_none());

        // No time-based flush yet: interval hasn't elapsed.
        assert!(buffer.try_flush().is_none());

        clock.advance(Duration::milliseconds(50));

        let flushed = buffer
            .try_flush()
            .expect("interval elapsed, should flush despite not reaching batch_size");
        assert_eq!(flushed.len(), 2);
    }

    #[test]
    fn disabled_buffering_passes_through_immediately() {
        let clock = Arc::new(AdvanceableClock::new(epoch()));
        let buffer = Buffer::new(config(false, 100, 60_000), clock);

        let flushed = buffer
            .add(record("solo"))
            .expect("disabled buffering must pass through immediately");
        assert_eq!(flushed.len(), 1);
        assert_eq!(flushed[0].message(), "solo");
    }

    #[test]
    fn flushed_batch_preserves_insertion_order() {
        let clock = Arc::new(AdvanceableClock::new(epoch()));
        let buffer = Buffer::new(config(true, 3, 60_000), clock);

        buffer.add(record("first"));
        buffer.add(record("second"));
        let flushed = buffer.add(record("third")).unwrap();

        let messages: Vec<&str> = flushed.iter().map(|r| r.message()).collect();
        assert_eq!(messages, ["first", "second", "third"]);
    }
}
