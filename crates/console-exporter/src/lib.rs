//! Console exporter crate for KITLogger.

pub mod error;
pub mod exporter;
pub mod stream_router;
pub mod lifecycle;
pub mod flush;

pub use error::ExportError;
pub use exporter::{ConsoleExporter, ConsoleExporterImpl};
pub use stream_router::{LevelStream, LevelStreamMapping, StreamRouter};
pub use lifecycle::LifecycleStateMachine;
pub use flush::{FlushStrategy, ImmediateFlush, OnShutdownFlush, BatchFlush};

#[cfg(test)]
mod integration_test;