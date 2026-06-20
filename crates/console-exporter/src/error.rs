//! Error types for the console exporter.

use std::io;

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("lifecycle error: {0}")]
    Lifecycle(String),
    #[error("flush error: {0}")]
    Flush(String),
}
