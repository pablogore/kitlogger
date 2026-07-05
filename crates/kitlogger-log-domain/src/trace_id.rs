//! Trace identifier for structured logs.

use std::fmt::{Display, Formatter, Result as FmtResult};

/// Opaque string identifier for trace correlation.
///
/// For wire-level, W3C Trace Context-compliant trace identifiers, see
/// `context_propagation::trace_context::TraceId` — that type is intentionally
/// a separate, richer concept for cross-process interop, not a duplicate of
/// this one (ADR-009 Amendment): this type exists only to tag a log line,
/// opaque and format-free by design.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraceId(String);

impl TraceId {
    /// Creates a new trace ID.
    pub fn new(id: String) -> Self {
        TraceId(id)
    }

    /// Returns a string slice of the inner identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for TraceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for TraceId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for TraceId {
    fn from(s: String) -> Self {
        TraceId(s)
    }
}
