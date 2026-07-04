//! Time abstraction enabling deterministic testing of time-sensitive claim checks.

use chrono::{DateTime, Utc};

/// Source of the current time.
///
/// Object-safe and `Send + Sync` so it can be shared as
/// `Box<dyn Clock>` / `Arc<dyn Clock>` across threads. Injecting `Clock`
/// (instead of calling `Utc::now()` directly) makes exp/nbf/iat checks
/// deterministic in tests via `FakeClock`.
pub trait Clock: Send + Sync {
    /// Returns the current time.
    fn now(&self) -> DateTime<Utc>;
}

/// Production `Clock` backed by the system clock.
pub struct UtcClock;

impl Clock for UtcClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// A `Clock` fixed at construction time.
///
/// Public (not `#[cfg(test)]`-gated) so downstream crates that implement
/// `AuthenticationProvider` (e.g. `security-jwt`) can reuse it for their own
/// deterministic exp/nbf/iat tests instead of duplicating a fake.
pub struct FakeClock(DateTime<Utc>);

impl FakeClock {
    /// Creates a `FakeClock` fixed at `instant`.
    pub fn new(instant: DateTime<Utc>) -> Self {
        FakeClock(instant)
    }
}

impl Clock for FakeClock {
    fn now(&self) -> DateTime<Utc> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Clock, FakeClock, UtcClock};
    use chrono::{TimeZone, Utc};

    #[test]
    fn clock_is_object_safe() {
        // Compile-time check only: if Clock were NOT object-safe, this line
        // would fail to compile.
        let _: Option<Box<dyn Clock>> = None;
    }

    #[test]
    fn fake_clock_returns_the_fixed_instant_it_was_constructed_with() {
        let fixed = Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap();
        let clock = FakeClock::new(fixed);

        assert_eq!(clock.now(), fixed);
    }

    #[test]
    fn fake_clock_with_a_different_instant_returns_that_instant() {
        let fixed = Utc.with_ymd_and_hms(2030, 6, 1, 8, 30, 0).unwrap();
        let clock = FakeClock::new(fixed);

        assert_eq!(clock.now(), fixed);
    }

    #[test]
    fn utc_clock_now_returns_a_time_close_to_real_now() {
        let clock = UtcClock;
        let before = Utc::now();
        let observed = clock.now();
        let after = Utc::now();

        assert!(observed >= before && observed <= after);
    }
}
