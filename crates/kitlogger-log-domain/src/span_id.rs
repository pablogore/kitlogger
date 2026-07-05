//! Span identifier for structured logs.

use std::fmt::{Display, Formatter, Result as FmtResult};

/// Opaque string identifier for span correlation.
///
/// For wire-level, W3C Trace Context-compliant span identifiers, see
/// `context_propagation::trace_context::SpanId` — that type is intentionally
/// a separate, richer concept for cross-process interop, not a duplicate of
/// this one (ADR-009 Amendment): this type exists only to tag a log line,
/// opaque and format-free by design.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpanId(String);

impl SpanId {
    /// Creates a new span ID.
    pub fn new(id: String) -> Self {
        SpanId(id)
    }

    /// Returns a string slice of the inner identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for SpanId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for SpanId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for SpanId {
    fn from(s: String) -> Self {
        SpanId(s)
    }
}
