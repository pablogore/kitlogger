use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use kit_config::{SamplingConfig, SamplingStrategy};

pub struct Sampler {
    config: SamplingConfig,
    counter: AtomicU32,
    window_start: Mutex<std::time::Instant>,
    window_count: AtomicU32,
}

impl Sampler {
    pub fn new(config: SamplingConfig) -> Self {
        Self {
            counter: AtomicU32::new(0),
            window_start: Mutex::new(std::time::Instant::now()),
            window_count: AtomicU32::new(0),
            config,
        }
    }

    pub fn should_sample(&self) -> bool {
        if !self.config.enabled || self.config.strategy == SamplingStrategy::None {
            return true;
        }

        match self.config.strategy {
            SamplingStrategy::Probabilistic => {
                fastrand::f64() < self.config.rate
            }
            SamplingStrategy::EveryNth => {
                let prev = self.counter.fetch_add(1, Ordering::Relaxed);
                prev.is_multiple_of(self.config.n)
            }
            SamplingStrategy::RateLimit => {
                let mut window = self.window_start.lock().unwrap();
                let now = std::time::Instant::now();
                if now.duration_since(*window).as_secs() >= 1 {
                    *window = now;
                    self.window_count.store(0, Ordering::Relaxed);
                }
                let count = self.window_count.fetch_add(1, Ordering::Relaxed);
                count < self.config.max_events_per_second
            }
            SamplingStrategy::None => true,
        }
    }
}
