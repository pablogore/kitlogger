//! Lifecycle management for the console exporter.

use crate::error::ExportError;

/// Lifecycle states for the console exporter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    /// Initial state before initialization.
    Uninitialized,
    /// Running state after initialization.
    Running,
    /// Flushing state during shutdown.
    Flushing,
    /// Shutdown state after completion.
    Shutdown,
    /// Error state when an error occurred.
    Error,
}

/// Lifecycle state machine for the console exporter.
pub struct LifecycleStateMachine {
    state: LifecycleState,
}

impl Default for LifecycleStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl LifecycleStateMachine {
    /// Creates a new lifecycle state machine in the uninitialized state.
    pub fn new() -> Self {
        Self {
            state: LifecycleState::Uninitialized,
        }
    }

    /// Transitions the state machine to a new state.
    pub fn transition_to(&mut self, target: LifecycleState) -> Result<(), ExportError> {
        match (self.state, target) {
            // From uninitialized
            (LifecycleState::Uninitialized, LifecycleState::Running) => {
                self.state = LifecycleState::Running;
                Ok(())
            }
            // From running
            (LifecycleState::Running, LifecycleState::Flushing) => {
                self.state = LifecycleState::Flushing;
                Ok(())
            }
            (LifecycleState::Running, LifecycleState::Shutdown) => {
                self.state = LifecycleState::Shutdown;
                Ok(())
            }
            // From flushing
            (LifecycleState::Flushing, LifecycleState::Shutdown) => {
                self.state = LifecycleState::Shutdown;
                Ok(())
            }
            // From any state to error
            (_, LifecycleState::Error) => {
                self.state = LifecycleState::Error;
                Ok(())
            }
            // Invalid transitions
            _ => Err(ExportError::Lifecycle(
                "Invalid lifecycle transition".to_string(),
            )),
        }
    }

    /// Returns the current lifecycle state.
    pub fn current(&self) -> LifecycleState {
        self.state
    }

    /// Checks if the exporter is initialized.
    pub fn is_initialized(&self) -> bool {
        matches!(self.state, LifecycleState::Running | LifecycleState::Flushing | LifecycleState::Shutdown | LifecycleState::Error)
    }

    /// Checks if the exporter is running.
    pub fn is_running(&self) -> bool {
        matches!(self.state, LifecycleState::Running)
    }

    /// Checks if the exporter is shutting down.
    pub fn is_shutting_down(&self) -> bool {
        matches!(self.state, LifecycleState::Flushing | LifecycleState::Shutdown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> LifecycleStateMachine {
        LifecycleStateMachine::new()
    }

    #[test]
    fn initial_state_is_uninitialized() {
        let lsm = make();
        assert_eq!(lsm.current(), LifecycleState::Uninitialized);
        assert!(!lsm.is_initialized());
        assert!(!lsm.is_running());
        assert!(!lsm.is_shutting_down());
    }

    #[test]
    fn transition_uninitialized_to_running() {
        let mut lsm = make();
        assert!(lsm.transition_to(LifecycleState::Running).is_ok());
        assert_eq!(lsm.current(), LifecycleState::Running);
        assert!(lsm.is_initialized());
        assert!(lsm.is_running());
        assert!(!lsm.is_shutting_down());
    }

    #[test]
    fn transition_running_to_flushing() {
        let mut lsm = make();
        lsm.transition_to(LifecycleState::Running).unwrap();
        assert!(lsm.transition_to(LifecycleState::Flushing).is_ok());
        assert_eq!(lsm.current(), LifecycleState::Flushing);
        assert!(lsm.is_initialized());
        assert!(!lsm.is_running());
        assert!(lsm.is_shutting_down());
    }

    #[test]
    fn transition_flushing_to_shutdown() {
        let mut lsm = make();
        lsm.transition_to(LifecycleState::Running).unwrap();
        lsm.transition_to(LifecycleState::Flushing).unwrap();
        assert!(lsm.transition_to(LifecycleState::Shutdown).is_ok());
        assert_eq!(lsm.current(), LifecycleState::Shutdown);
        assert!(lsm.is_initialized());
        assert!(!lsm.is_running());
        assert!(lsm.is_shutting_down());
    }

    #[test]
    fn transition_running_to_shutdown() {
        let mut lsm = make();
        lsm.transition_to(LifecycleState::Running).unwrap();
        assert!(lsm.transition_to(LifecycleState::Shutdown).is_ok());
        assert_eq!(lsm.current(), LifecycleState::Shutdown);
    }

    #[test]
    fn invalid_uninitialized_to_shutdown_returns_err() {
        let mut lsm = make();
        let result = lsm.transition_to(LifecycleState::Shutdown);
        assert!(result.is_err());
        assert_eq!(lsm.current(), LifecycleState::Uninitialized);
    }

    #[test]
    fn invalid_running_to_uninitialized_returns_err() {
        let mut lsm = make();
        lsm.transition_to(LifecycleState::Running).unwrap();
        let result = lsm.transition_to(LifecycleState::Uninitialized);
        assert!(result.is_err());
        assert_eq!(lsm.current(), LifecycleState::Running);
    }

    #[test]
    fn invalid_flushing_to_running_returns_err() {
        let mut lsm = make();
        lsm.transition_to(LifecycleState::Running).unwrap();
        lsm.transition_to(LifecycleState::Flushing).unwrap();
        let result = lsm.transition_to(LifecycleState::Running);
        assert!(result.is_err());
        assert_eq!(lsm.current(), LifecycleState::Flushing);
    }

    #[test]
    fn delivery_after_shutdown_is_blocked() {
        // Spec: delivery after shutdown should be blocked (part of lifecycle)
        let mut lsm = make();
        lsm.transition_to(LifecycleState::Running).unwrap();
        lsm.transition_to(LifecycleState::Shutdown).unwrap();
        assert!(!lsm.is_running());
        // is_initialized returns true for Shutdown
        assert!(lsm.is_initialized());
    }

    #[test]
    fn transition_to_error_from_any_state() {
        let mut lsm = make();
        assert!(lsm.transition_to(LifecycleState::Error).is_ok());
        assert_eq!(lsm.current(), LifecycleState::Error);

        // From running
        let mut lsm2 = make();
        lsm2.transition_to(LifecycleState::Running).unwrap();
        assert!(lsm2.transition_to(LifecycleState::Error).is_ok());
        assert_eq!(lsm2.current(), LifecycleState::Error);

        // From flushing
        let mut lsm3 = make();
        lsm3.transition_to(LifecycleState::Running).unwrap();
        lsm3.transition_to(LifecycleState::Flushing).unwrap();
        assert!(lsm3.transition_to(LifecycleState::Error).is_ok());
        assert_eq!(lsm3.current(), LifecycleState::Error);

        // From shutdown
        let mut lsm4 = make();
        lsm4.transition_to(LifecycleState::Running).unwrap();
        lsm4.transition_to(LifecycleState::Shutdown).unwrap();
        assert!(lsm4.transition_to(LifecycleState::Error).is_ok());
        assert_eq!(lsm4.current(), LifecycleState::Error);
    }
}