//! Severity levels for structured logs.

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use crate::ValidationError;

/// Canonical severity levels for structured logs.
///
/// Implements `PartialOrd` for severity ordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Trace level - most verbose
    Trace,
    /// Debug level - development debugging
    Debug,
    /// Info level - general information
    Info,
    /// Warn level - warning conditions
    Warn,
    /// Error level - error conditions
    Error,
    /// Fatal level - critical errors
    Fatal,
}

impl Display for Severity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Severity::Trace => write!(f, "Trace"),
            Severity::Debug => write!(f, "Debug"),
            Severity::Info => write!(f, "Info"),
            Severity::Warn => write!(f, "Warn"),
            Severity::Error => write!(f, "Error"),
            Severity::Fatal => write!(f, "Fatal"),
        }
    }
}

impl FromStr for Severity {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(Severity::Trace),
            "debug" => Ok(Severity::Debug),
            "info" => Ok(Severity::Info),
            "warn" => Ok(Severity::Warn),
            "error" => Ok(Severity::Error),
            "fatal" => Ok(Severity::Fatal),
            _ => Err(ValidationError::InvalidSeverity),
        }
    }
}
