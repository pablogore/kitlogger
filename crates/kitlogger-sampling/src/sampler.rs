//! Emission-volume sampling decisions.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Duration, Utc};
use kit_config::{SamplingConfig, SamplingStrategy};
use kitlogger_log_domain::Clock;

/// Width of the `RateLimit` strategy's sliding window.
const RATE_LIMIT_WINDOW: Duration = Duration::seconds(1);

/// Decides whether a given emission should proceed, per a
/// `kit_config::SamplingConfig`. Depends only on configuration and time
/// (via the injected [`Clock`]) — never on record content.
pub struct Sampler {
    config: SamplingConfig,
    clock: Arc<dyn Clock>,
    every_nth_counter: AtomicU32,
    rate_limit_window_start: Mutex<DateTime<Utc>>,
    rate_limit_window_count: AtomicU32,
}

impl Sampler {
    /// Creates a new `Sampler` from the given `SamplingConfig`, sourcing
    /// time through `clock` (used by the `RateLimit` strategy's sliding
    /// window). `clock` is `Arc`-shared (rather than owned outright) so
    /// tests can hold a reference to the same clock instance and advance it
    /// externally while the `Sampler` observes the change.
    pub fn new(config: SamplingConfig, clock: Arc<dyn Clock>) -> Self {
        let window_start = clock.now();
        Sampler {
            config,
            clock,
            every_nth_counter: AtomicU32::new(0),
            rate_limit_window_start: Mutex::new(window_start),
            rate_limit_window_count: AtomicU32::new(0),
        }
    }

    /// Returns whether the current emission should proceed.
    pub fn should_sample(&self) -> bool {
        if !self.config.enabled {
            return true;
        }

        match self.config.strategy {
            SamplingStrategy::None => true,
            SamplingStrategy::EveryNth => self.sample_every_nth(),
            SamplingStrategy::Probabilistic => fastrand::f64() < self.config.rate,
            SamplingStrategy::RateLimit => self.sample_rate_limit(),
        }
    }

    fn sample_every_nth(&self) -> bool {
        let previous = self.every_nth_counter.fetch_add(1, Ordering::Relaxed);
        previous.is_multiple_of(self.config.n)
    }

    fn sample_rate_limit(&self) -> bool {
        // Time is sourced exclusively through `self.clock` (never
        // `Instant::now()`/`SystemTime::now()` directly), satisfying FR-007.
        let now = self.clock.now();
        let mut window_start = self.rate_limit_window_start.lock().unwrap();

        if now - *window_start >= RATE_LIMIT_WINDOW {
            *window_start = now;
            self.rate_limit_window_count.store(0, Ordering::Relaxed);
        }

        let count = self.rate_limit_window_count.fetch_add(1, Ordering::Relaxed);
        count < self.config.max_events_per_second
    }
}

#[cfg(test)]
mod tests {
    use kit_config::{SamplingConfig, SamplingStrategy};
    use std::sync::Arc;

    use kitlogger_log_domain::UtcClock;

    use crate::Sampler;

    fn config(strategy: SamplingStrategy) -> SamplingConfig {
        SamplingConfig {
            enabled: true,
            strategy,
            rate: 0.1,
            n: 100,
            max_events_per_second: 500,
        }
    }

    #[test]
    fn none_strategy_always_samples() {
        let sampler = Sampler::new(config(SamplingStrategy::None), Arc::new(UtcClock));

        for _ in 0..10 {
            assert!(sampler.should_sample());
        }
    }

    #[test]
    fn every_nth_strategy_deterministic_sequence() {
        let mut cfg = config(SamplingStrategy::EveryNth);
        cfg.n = 3;
        let sampler = Sampler::new(cfg.clone(), Arc::new(UtcClock));

        let first_run: Vec<bool> = (0..6).map(|_| sampler.should_sample()).collect();
        assert_eq!(first_run.iter().filter(|&&sampled| sampled).count(), 2);

        let sampler_again = Sampler::new(cfg, Arc::new(UtcClock));
        let second_run: Vec<bool> = (0..6).map(|_| sampler_again.should_sample()).collect();
        assert_eq!(first_run, second_run);
    }

    #[test]
    fn probabilistic_strategy_within_statistical_tolerance() {
        let mut cfg = config(SamplingStrategy::Probabilistic);
        cfg.rate = 0.3;
        let sampler = Sampler::new(cfg, Arc::new(UtcClock));

        let draws = 10_000;
        let sampled = (0..draws).filter(|_| sampler.should_sample()).count();
        let observed_rate = sampled as f64 / draws as f64;

        assert!(
            (observed_rate - 0.3).abs() < 0.03,
            "observed rate {observed_rate} not within tolerance of configured rate 0.3"
        );
    }

    /// Test-only, programmatically advanceable `Clock`. `kitlogger_log_domain::FakeClock`
    /// is fixed at construction with no way to move it forward, so it cannot
    /// exercise the `RateLimit` window-boundary scenarios below. This
    /// implements the SAME canonical `Clock` trait (no competing abstraction,
    /// per ADR-010) — it just adds test-only mutability via a `Mutex`.
    struct AdvanceableClock(std::sync::Mutex<chrono::DateTime<chrono::Utc>>);

    impl AdvanceableClock {
        fn new(start: chrono::DateTime<chrono::Utc>) -> Self {
            AdvanceableClock(std::sync::Mutex::new(start))
        }

        fn advance(&self, duration: chrono::Duration) {
            let mut current = self.0.lock().unwrap();
            *current += duration;
        }
    }

    impl kitlogger_log_domain::Clock for AdvanceableClock {
        fn now(&self) -> chrono::DateTime<chrono::Utc> {
            *self.0.lock().unwrap()
        }
    }

    fn epoch() -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::UNIX_EPOCH
    }

    #[test]
    fn rate_limit_allows_decisions_within_the_limit() {
        let mut cfg = config(SamplingStrategy::RateLimit);
        cfg.max_events_per_second = 5;
        let clock = Arc::new(AdvanceableClock::new(epoch()));
        let sampler = Sampler::new(cfg, clock);

        for _ in 0..5 {
            assert!(sampler.should_sample());
        }
    }

    #[test]
    fn rate_limit_rejects_decisions_beyond_the_limit_in_the_same_window() {
        let mut cfg = config(SamplingStrategy::RateLimit);
        cfg.max_events_per_second = 5;
        let clock = Arc::new(AdvanceableClock::new(epoch()));
        let sampler = Sampler::new(cfg, clock);

        for _ in 0..5 {
            assert!(sampler.should_sample());
        }
        assert!(!sampler.should_sample());
    }

    #[test]
    fn rate_limit_resets_in_the_next_window() {
        let mut cfg = config(SamplingStrategy::RateLimit);
        cfg.max_events_per_second = 5;
        let clock = Arc::new(AdvanceableClock::new(epoch()));
        let dyn_clock: Arc<dyn kitlogger_log_domain::Clock> = clock.clone();
        let sampler = Sampler::new(cfg, dyn_clock);

        for _ in 0..5 {
            assert!(sampler.should_sample());
        }
        assert!(!sampler.should_sample());

        clock.advance(chrono::Duration::seconds(1));

        assert!(sampler.should_sample());
    }

    #[test]
    fn disabled_config_always_samples() {
        let mut cfg = config(SamplingStrategy::RateLimit);
        cfg.enabled = false;
        cfg.max_events_per_second = 1;
        let sampler = Sampler::new(cfg, Arc::new(UtcClock));

        for _ in 0..10 {
            assert!(sampler.should_sample());
        }
    }
}
