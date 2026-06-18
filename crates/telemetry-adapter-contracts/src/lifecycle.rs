use crate::error::{AdapterError, AdapterResult};

/// Canonical lifecycle states for adapter state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    Registered,
    Initialized,
    Started,
    Stopping,
    Stopped,
    Shutdown,
}

/// Adapter lifecycle state machine with explicit transition matrix.
pub struct AdapterLifecycle {
    state: LifecycleState,
}

impl Default for AdapterLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl AdapterLifecycle {
    pub fn new() -> Self {
        AdapterLifecycle {
            state: LifecycleState::Registered,
        }
    }

    pub fn state(&self) -> LifecycleState {
        self.state
    }

    pub fn transition(&mut self, to: LifecycleState) -> AdapterResult<()> {
        let from = self.state;
        let valid = match (from, to) {
            // Normal startup
            (LifecycleState::Registered, LifecycleState::Initialized) => true,
            (LifecycleState::Initialized, LifecycleState::Started) => true,
            // Graceful stop
            (LifecycleState::Started, LifecycleState::Stopping) => true,
            (LifecycleState::Stopping, LifecycleState::Stopped) => true,
            // Startup failure transitions
            (LifecycleState::Registered, LifecycleState::Shutdown) => true,
            (LifecycleState::Initialized, LifecycleState::Shutdown) => true,
            // Flush-failed during stop
            (LifecycleState::Stopping, LifecycleState::Shutdown) => true,
            // Final transition
            (LifecycleState::Stopped, LifecycleState::Shutdown) => true,
            _ => false,
        };
        if valid {
            self.state = to;
            Ok(())
        } else {
            Err(AdapterError::InvalidTransition { from, to })
        }
    }
}
