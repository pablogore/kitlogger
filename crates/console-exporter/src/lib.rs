//! Console exporter crate for KITLogger.

pub mod error;
pub mod exporter;
pub mod flush;
pub mod lifecycle;
pub mod stream_router;

pub use error::ExportError;
pub use exporter::{ConsoleExporter, ConsoleExporterImpl};
pub use flush::{BatchFlush, FlushStrategy, ImmediateFlush, OnShutdownFlush};
pub use lifecycle::LifecycleStateMachine;
pub use stream_router::{LevelStream, LevelStreamMapping, StreamRouter};

#[cfg(test)]
mod integration_test;
