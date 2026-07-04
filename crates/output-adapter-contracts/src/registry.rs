//! Registry: registers outputs under a unique identifier and dispatches a
//! formatted record to all of them, aggregating per-output failures.

use kitlogger_log_domain::Severity;

use crate::output::{Output, OutputError};

/// Unique identifier for a registered output.
///
/// This is the one identity type for this bounded context (see design.md's
/// "Identity Ownership" section) — no per-implementation output ID exists,
/// and this is not a redefinition of `telemetry-adapter-contracts::AdapterId`
/// for this different context.
///
/// `OutputId` is opaque: it promises identity only, not validation or a
/// specific format. An empty string is a valid, if unhelpful, id — enforcing
/// non-emptiness (or any other shape constraint) is the host's decision, not
/// this crate's.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutputId(String);

impl OutputId {
    /// Creates a new `OutputId` from any string-like value.
    pub fn new(id: impl Into<String>) -> Self {
        OutputId(id.into())
    }
}

impl std::fmt::Display for OutputId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Error returned when registering an output fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistrationError {
    /// An output is already registered under this identifier; the
    /// originally registered output remains registered, unchanged.
    DuplicateId(OutputId),
}

impl std::fmt::Display for RegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistrationError::DuplicateId(id) => {
                write!(f, "an output is already registered under id '{id}'")
            }
        }
    }
}

impl std::error::Error for RegistrationError {}

/// The aggregate result of dispatching a record to every registered output.
#[derive(Debug)]
pub enum DispatchOutcome {
    /// Every registered output succeeded.
    AllSucceeded,
    /// Some outputs succeeded and some failed; failures name the id and
    /// the reason for each failing output.
    PartialFailure(Vec<(OutputId, OutputError)>),
    /// Every registered output failed.
    AllFailed(Vec<(OutputId, OutputError)>),
}

/// Registers outputs under a unique [`OutputId`] and dispatches a formatted
/// record to every currently registered output.
#[derive(Default)]
pub struct Registry {
    outputs: Vec<(OutputId, Box<dyn Output>)>,
}

impl Registry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers `output` under `id`. Rejects registration (FR-002) if
    /// `id` is already in use; the originally registered output remains
    /// registered, unchanged.
    pub fn register(
        &mut self,
        id: OutputId,
        output: Box<dyn Output>,
    ) -> Result<(), RegistrationError> {
        if self.outputs.iter().any(|(existing, _)| existing == &id) {
            return Err(RegistrationError::DuplicateId(id));
        }
        self.outputs.push((id, output));
        Ok(())
    }

    /// Dispatches `formatted`/`severity` to every registered output
    /// (FR-003), aggregating per-output failures without letting one
    /// failure block delivery to the others (FR-004).
    ///
    /// Dispatching to an empty registry is considered successful — there is
    /// no output to fail, so `failures` is trivially empty.
    pub fn dispatch(&self, formatted: &str, severity: Severity) -> DispatchOutcome {
        let mut failures = Vec::new();
        for (id, output) in &self.outputs {
            if let Err(e) = output.dispatch(formatted, severity) {
                failures.push((id.clone(), e));
            }
        }

        if failures.is_empty() {
            DispatchOutcome::AllSucceeded
        } else if failures.len() == self.outputs.len() {
            DispatchOutcome::AllFailed(failures)
        } else {
            DispatchOutcome::PartialFailure(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct RecordingOutput {
        received: Arc<Mutex<Vec<String>>>,
    }

    impl Output for RecordingOutput {
        fn dispatch(&self, formatted: &str, _severity: Severity) -> Result<(), OutputError> {
            self.received.lock().unwrap().push(formatted.to_string());
            Ok(())
        }
    }

    struct FailingOutput;

    impl Output for FailingOutput {
        fn dispatch(&self, _formatted: &str, _severity: Severity) -> Result<(), OutputError> {
            Err(OutputError::new("simulated failure"))
        }
    }

    #[test]
    fn duplicate_registration_rejected() {
        let mut registry = Registry::new();
        let first_received = Arc::new(Mutex::new(Vec::new()));
        registry
            .register(
                OutputId::new("primary"),
                Box::new(RecordingOutput {
                    received: first_received.clone(),
                }),
            )
            .expect("first registration should succeed");

        let second_received = Arc::new(Mutex::new(Vec::new()));
        let result = registry.register(
            OutputId::new("primary"),
            Box::new(RecordingOutput {
                received: second_received.clone(),
            }),
        );

        assert_eq!(
            result,
            Err(RegistrationError::DuplicateId(OutputId::new("primary")))
        );

        // The originally registered output remains registered, unchanged.
        registry.dispatch("still working", Severity::Info);
        assert_eq!(first_received.lock().unwrap().len(), 1);
        assert!(second_received.lock().unwrap().is_empty());
    }

    #[test]
    fn dispatch_reaches_all_registered_outputs() {
        let mut registry = Registry::new();
        let recorders: Vec<_> = (0..3).map(|_| Arc::new(Mutex::new(Vec::new()))).collect();

        for (i, received) in recorders.iter().enumerate() {
            registry
                .register(
                    OutputId::new(format!("output-{i}")),
                    Box::new(RecordingOutput {
                        received: received.clone(),
                    }),
                )
                .unwrap();
        }

        let outcome = registry.dispatch("broadcast", Severity::Warn);

        assert!(matches!(outcome, DispatchOutcome::AllSucceeded));
        for received in &recorders {
            assert_eq!(received.lock().unwrap().as_slice(), ["broadcast"]);
        }
    }

    #[test]
    fn partial_failure_does_not_block_others() {
        let mut registry = Registry::new();
        let first_received = Arc::new(Mutex::new(Vec::new()));
        let third_received = Arc::new(Mutex::new(Vec::new()));

        registry
            .register(
                OutputId::new("first"),
                Box::new(RecordingOutput {
                    received: first_received.clone(),
                }),
            )
            .unwrap();
        registry
            .register(OutputId::new("failing"), Box::new(FailingOutput))
            .unwrap();
        registry
            .register(
                OutputId::new("third"),
                Box::new(RecordingOutput {
                    received: third_received.clone(),
                }),
            )
            .unwrap();

        let outcome = registry.dispatch("record", Severity::Error);

        assert_eq!(first_received.lock().unwrap().as_slice(), ["record"]);
        assert_eq!(third_received.lock().unwrap().as_slice(), ["record"]);

        match outcome {
            DispatchOutcome::PartialFailure(failures) => {
                assert_eq!(failures.len(), 1);
                assert_eq!(failures[0].0, OutputId::new("failing"));
            }
            other => panic!("expected PartialFailure, got {other:?}"),
        }
    }

    #[test]
    fn empty_registry_dispatch_is_success() {
        let registry = Registry::new();

        let outcome = registry.dispatch("no outputs registered", Severity::Info);

        assert!(matches!(outcome, DispatchOutcome::AllSucceeded));
    }

    #[test]
    fn total_failure_is_distinguishable() {
        let mut registry = Registry::new();
        for i in 0..3 {
            registry
                .register(
                    OutputId::new(format!("failing-{i}")),
                    Box::new(FailingOutput),
                )
                .unwrap();
        }

        let outcome = registry.dispatch("record", Severity::Error);

        match outcome {
            DispatchOutcome::AllFailed(failures) => assert_eq!(failures.len(), 3),
            other => panic!("expected AllFailed, got {other:?}"),
        }
    }
}
