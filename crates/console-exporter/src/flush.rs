//! Flush strategies for the console exporter.

/// Flush strategy trait for controlling when to flush output.
pub trait FlushStrategy: Send + Sync {
    /// Determines if a flush should occur based on write count.
    fn should_flush(&self, write_count: usize) -> bool;

    /// Determines if flushing should occur on shutdown.
    fn on_shutdown(&self) -> bool;
}

/// Immediate flush strategy - flushes after every write.
pub struct ImmediateFlush;

impl FlushStrategy for ImmediateFlush {
    fn should_flush(&self, _write_count: usize) -> bool {
        true
    }

    fn on_shutdown(&self) -> bool {
        false
    }
}

/// On shutdown flush strategy - flushes only on shutdown.
pub struct OnShutdownFlush;

impl FlushStrategy for OnShutdownFlush {
    fn should_flush(&self, _write_count: usize) -> bool {
        false
    }

    fn on_shutdown(&self) -> bool {
        true
    }
}

/// Batch flush strategy - flushes after a certain number of writes.
pub struct BatchFlush {
    threshold: usize,
}

impl BatchFlush {
    /// Creates a new batch flush strategy with the given threshold.
    pub fn new(threshold: usize) -> Self {
        Self { threshold }
    }
}

impl FlushStrategy for BatchFlush {
    fn should_flush(&self, write_count: usize) -> bool {
        write_count.is_multiple_of(self.threshold)
    }

    fn on_shutdown(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- ImmediateFlush ---

    #[test]
    fn immediate_should_flush_every_write() {
        let strategy = ImmediateFlush;
        assert!(
            strategy.should_flush(0),
            "Immediate: should flush on first write"
        );
        assert!(
            strategy.should_flush(1),
            "Immediate: should flush on every write"
        );
        assert!(
            strategy.should_flush(100),
            "Immediate: should flush on 100th write"
        );
    }

    #[test]
    fn immediate_on_shutdown_returns_false() {
        let strategy = ImmediateFlush;
        assert!(
            !strategy.on_shutdown(),
            "Immediate does not flush on shutdown"
        );
    }

    // --- OnShutdownFlush ---

    #[test]
    fn on_shutdown_should_not_flush_during_writes() {
        let strategy = OnShutdownFlush;
        assert!(!strategy.should_flush(0), "OnShutdown: no flush on write");
        assert!(!strategy.should_flush(1), "OnShutdown: no flush on write");
        assert!(
            !strategy.should_flush(5000),
            "OnShutdown: no flush even after many writes"
        );
    }

    #[test]
    fn on_shutdown_flushes_on_shutdown() {
        let strategy = OnShutdownFlush;
        assert!(strategy.on_shutdown(), "OnShutdown flushes on shutdown");
    }

    // --- BatchFlush ---

    #[test]
    fn batch_flushes_at_threshold() {
        let strategy = BatchFlush::new(5);

        // Note: 0 is a multiple of 5, so write_count=0 triggers flush
        assert!(
            strategy.should_flush(0),
            "write 0: 0 is multiple of 5, flushes"
        );
        assert!(!strategy.should_flush(1), "write 1: should not flush");
        assert!(!strategy.should_flush(4), "write 4: should not flush");

        // Should flush at threshold boundary
        assert!(
            strategy.should_flush(5),
            "write 5: should flush (multiple of 5)"
        );
        assert!(!strategy.should_flush(6), "write 6: should not flush");
        assert!(
            strategy.should_flush(10),
            "write 10: should flush (multiple of 5)"
        );
        assert!(!strategy.should_flush(11), "write 11: should not flush");
    }

    #[test]
    fn batch_threshold_of_one_flushes_every_write() {
        let strategy = BatchFlush::new(1);
        assert!(
            strategy.should_flush(0),
            "threshold 1: every write flushes, including 0"
        );
        assert!(strategy.should_flush(1), "threshold 1: flush at write 1");
        assert!(strategy.should_flush(42), "threshold 1: flush at write 42");
    }

    #[test]
    fn batch_flushes_on_shutdown() {
        let strategy = BatchFlush::new(10);
        assert!(strategy.on_shutdown(), "Batch flushes on shutdown");
    }

    #[test]
    fn batch_flush_is_send_and_sync() {
        // Compile-time check: FlushStrategy trait requires Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ImmediateFlush>();
        assert_send_sync::<OnShutdownFlush>();
        assert_send_sync::<BatchFlush>();
    }
}
