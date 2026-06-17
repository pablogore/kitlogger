/// Error types for telemetry adapter operations.
use std::fmt;
use std::fmt::{Display, Formatter};

use crate::id::AdapterId;
use crate::lifecycle::LifecycleState;

pub type AdapterResult<T> = Result<T, AdapterError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    InvalidTransition { from: LifecycleState, to: LifecycleState },
    AlreadyRegistered(AdapterId),
    Frozen,
    InitializationFailed(String),
    FlushFailed(String),
    ShutdownFailed(String),
    DeliveryFailed(Vec<(AdapterId, String)>),
    PartialDelivery(Vec<(AdapterId, String)>),
}

impl Display for AdapterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AdapterError::InvalidTransition { from, to } => {
                write!(f, "Invalid transition from {from:?} to {to:?}")
            }
            AdapterError::AlreadyRegistered(id) => {
                write!(f, "Adapter already registered: {id}")
            }
            AdapterError::Frozen => write!(f, "Registry is frozen"),
            AdapterError::InitializationFailed(reason) => {
                write!(f, "Initialization failed: {reason}")
            }
            AdapterError::FlushFailed(reason) => write!(f, "Flush failed: {reason}"),
            AdapterError::ShutdownFailed(reason) => write!(f, "Shutdown failed: {reason}"),
            AdapterError::DeliveryFailed(failures) => {
                write!(f, "All adapters failed: {failures:?}")
            }
            AdapterError::PartialDelivery(failures) => {
                write!(f, "Partial delivery failures: {failures:?}")
            }
        }
    }
}

impl std::error::Error for AdapterError {}
