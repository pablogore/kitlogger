//! The Output Port: the single seam every log output destination (console,
//! file, and future network/vendor destinations) is dispatched through.
//!
//! By the time a record reaches an output, it has already been formatted to
//! a string (pipeline order: buffer -> format -> dispatch, per ADR-008 §5)
//! — the Port never takes a raw `LogRecord`.

use kitlogger_log_domain::Severity;

/// Error returned when a single output fails to deliver a dispatched
/// record. Carries a human-readable reason only — this crate has no
/// knowledge of any specific output's failure modes (I/O, network, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputError(String);

impl OutputError {
    /// Creates a new `OutputError` from a failure reason.
    pub fn new(reason: impl Into<String>) -> Self {
        OutputError(reason.into())
    }

    /// Returns the failure reason.
    pub fn reason(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for OutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "output dispatch failed: {}", self.0)
    }
}

impl std::error::Error for OutputError {}

/// The Output Port every log destination implements.
///
/// `severity` accompanies the already-formatted payload because
/// implementations may route independently by severity — sampling,
/// alerting, or backpressure decisions that don't require re-parsing the
/// formatted string (see design.md Q1).
pub trait Output: Send + Sync {
    /// Delivers an already-formatted record and its severity to this
    /// output. Implementations MUST NOT require any additional,
    /// output-specific wrapping of the two values (FR-001).
    fn dispatch(&self, formatted: &str, severity: Severity) -> Result<(), OutputError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// A fake output recording every dispatched record it receives.
    struct RecordingOutput {
        received: Arc<Mutex<Vec<(String, Severity)>>>,
    }

    impl Output for RecordingOutput {
        fn dispatch(&self, formatted: &str, severity: Severity) -> Result<(), OutputError> {
            self.received
                .lock()
                .unwrap()
                .push((formatted.to_string(), severity));
            Ok(())
        }
    }

    #[test]
    fn conforming_output_receives_dispatch() {
        let received = Arc::new(Mutex::new(Vec::new()));
        let output = RecordingOutput {
            received: received.clone(),
        };

        output
            .dispatch("hello, world", Severity::Info)
            .expect("dispatch should succeed");

        let recorded = received.lock().unwrap();
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0], ("hello, world".to_string(), Severity::Info));
    }
}
