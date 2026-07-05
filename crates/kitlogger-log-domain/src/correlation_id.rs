//! Correlation identifier for structured logs.

use std::fmt::{Display, Formatter, Result as FmtResult};

/// Opaque string identifier for cross-service correlation.
///
/// For wire-level, W3C Trace Context-compliant correlation identifiers, see
/// `context_propagation::correlation::CorrelationIdentifier` — that type is
/// intentionally a separate, richer concept for cross-process interop, not a
/// duplicate of this one (ADR-009 Amendment): this type exists only to tag a
/// log line, opaque and format-free by design.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CorrelationId(String);

impl CorrelationId {
    /// Creates a new correlation ID.
    pub fn new(id: String) -> Self {
        CorrelationId(id)
    }

    /// Returns a string slice of the inner identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for CorrelationId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for CorrelationId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for CorrelationId {
    fn from(s: String) -> Self {
        CorrelationId(s)
    }
}
