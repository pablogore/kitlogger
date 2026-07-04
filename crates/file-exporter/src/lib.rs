//! File-based output implementation for the Output Port defined by
//! `output-adapter-contracts`, including file rotation as an internal
//! module (not a separate crate — see design.md Q3).

mod exporter;
mod rotation;

pub use exporter::FileExporter;
pub use rotation::RotationManager;
