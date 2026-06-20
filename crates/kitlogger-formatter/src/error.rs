//! Format error types for the kitlogger-formatter crate.

/// Errors that can occur during log record formatting.
///
/// Panics on format failure are PROHIBITED — all failures surface as `Err(FormatError)`.
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    /// A serde_json (or other) serialization step failed.
    #[error("serialization failed: {0}")]
    SerializationError(String),

    /// An attribute value could not be rendered to its target representation.
    #[error("value rendering failed: {0}")]
    RenderError(String),
}
