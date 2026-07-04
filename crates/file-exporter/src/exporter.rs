//! File-based output implementation, conforming to the Output Port.

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use kit_config::RotationConfig;
use kitlogger_log_domain::Severity;
use output_adapter_contracts::{Output, OutputError};

use crate::rotation::RotationManager;

/// Writes dispatched records to a file, rotating per `RotationConfig` when
/// the configured size boundary would otherwise be exceeded.
///
/// `FileExporter` assumes exclusive ownership of the target log file: it
/// tracks the file's size in-memory (`current_size`) rather than re-reading
/// `metadata()` on every write, and that tracked size is only ever correct
/// if nothing else writes to the file concurrently. External writers are
/// unsupported — if another process or thread modifies the file, rotation
/// decisions will no longer reflect the file's actual size on disk.
///
/// `current_size` and `file` are independent mutexes. `write_line` acquires
/// them in the order `current_size` -> `file`, and holds `current_size` for
/// the whole rotate-then-write sequence so no other writer can interleave a
/// write between rotation and the size reset. Any future method that needs
/// both MUST acquire them in this same order to avoid a deadlock.
pub struct FileExporter {
    path: PathBuf,
    file: Mutex<std::fs::File>,
    current_size: Mutex<u64>,
    rotation: RotationManager,
}

impl FileExporter {
    /// Opens (creating if necessary) the file at `path` in append mode,
    /// configuring rotation per `rotation_config`.
    pub fn new(path: impl AsRef<Path>, rotation_config: RotationConfig) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        let current_size = file.metadata()?.len();

        Ok(Self {
            path,
            file: Mutex::new(file),
            current_size: Mutex::new(current_size),
            rotation: RotationManager::new(rotation_config),
        })
    }

    /// Appends `line` to the file, rotating first if writing it would
    /// exceed the configured size boundary (FR-002).
    fn write_line(&self, line: &str) -> io::Result<()> {
        let bytes_to_write = line.len() as u64 + 1; // +1 for the trailing newline
        let mut current_size = self.current_size.lock().unwrap();
        let prospective_size = *current_size + bytes_to_write;

        if self.rotation.should_rotate(prospective_size) {
            {
                let mut file = self.file.lock().unwrap();
                file.flush()?;
            }
            self.rotation.rotate(&self.path)?;
            let new_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)?;
            *self.file.lock().unwrap() = new_file;
            *current_size = 0;
        }

        let mut file = self.file.lock().unwrap();
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
        *current_size += bytes_to_write;

        Ok(())
    }
}

impl Output for FileExporter {
    fn dispatch(&self, formatted: &str, _severity: Severity) -> Result<(), OutputError> {
        self.write_line(formatted)
            .map_err(|e| OutputError::new(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use output_adapter_contracts::{OutputId, Registry};
    use std::fs;

    fn rotation_config(enabled: bool, max_size_mb: u32, max_backups: u32) -> RotationConfig {
        RotationConfig {
            enabled,
            max_size_mb,
            max_backups,
            max_age_days: 30,
        }
    }

    fn disabled_rotation() -> RotationConfig {
        rotation_config(false, 100, 10)
    }

    #[test]
    fn dispatched_record_appended_to_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.log");
        let exporter = FileExporter::new(&path, disabled_rotation()).unwrap();

        exporter.dispatch("first record", Severity::Info).unwrap();
        exporter.dispatch("second record", Severity::Info).unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "first record\nsecond record\n");
    }

    #[test]
    fn rotation_triggers_at_size_boundary() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.log");

        // Pre-populate the file to just under the 1 MB boundary.
        let one_mb = 1024 * 1024usize;
        let padding = "a".repeat(one_mb - 10);
        fs::write(&path, padding.as_bytes()).unwrap();

        let exporter = FileExporter::new(&path, rotation_config(true, 1, 5)).unwrap();

        // This record pushes the file over the 1 MB boundary.
        exporter
            .dispatch("this record triggers rotation", Severity::Info)
            .unwrap();

        let backup = path.with_extension("log.1");
        assert!(backup.exists(), "rotation should have produced a backup");

        let new_file_size = fs::metadata(&path).unwrap().len();
        assert!(
            new_file_size <= one_mb as u64,
            "post-rotation file size {new_file_size} should not exceed the configured boundary"
        );
    }

    #[test]
    fn backups_beyond_max_are_discarded() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.log");

        // A tiny max_size so every dispatched record triggers rotation.
        // max_size_mb = 0 is only valid here because the test constructs
        // `RotationConfig` directly; in production this value would be
        // rejected by kit-config's own validation before reaching this
        // crate — it is not a supported value of the domain.
        let exporter = FileExporter::new(&path, rotation_config(true, 0, 2)).unwrap();

        // max_size_mb = 0 means any non-empty write exceeds the boundary,
        // so every dispatch below rotates before writing (including the
        // first, since the file already exists empty at construction).
        exporter.dispatch("record-1", Severity::Info).unwrap();
        exporter.dispatch("record-2", Severity::Info).unwrap();
        exporter.dispatch("record-3", Severity::Info).unwrap();

        let backup_1 = path.with_extension("log.1");
        let backup_2 = path.with_extension("log.2");
        let backup_3 = path.with_extension("log.3");

        assert!(backup_1.exists());
        assert!(backup_2.exists());
        assert!(!backup_3.exists());
    }

    #[test]
    fn disabled_rotation_grows_file_unbounded() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.log");
        // max_size_mb = 0 would trigger rotation on every write if enabled;
        // with rotation disabled, no rotation must occur regardless. As in
        // `backups_beyond_max_are_discarded` above, this value is only
        // valid because the test builds `RotationConfig` directly — it is
        // not a value production would ever see past kit-config validation.
        let exporter = FileExporter::new(&path, rotation_config(false, 0, 5)).unwrap();

        for i in 0..50 {
            exporter
                .dispatch(&format!("record-{i}"), Severity::Info)
                .unwrap();
        }

        let backup = path.with_extension("log.1");
        assert!(!backup.exists(), "no rotation should occur when disabled");

        let contents = fs::read_to_string(&path).unwrap();
        assert_eq!(contents.lines().count(), 50);
    }

    #[test]
    fn file_exporter_conforms_to_output_port() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.log");
        let exporter = FileExporter::new(&path, disabled_rotation()).unwrap();

        struct FakeOutput;
        impl Output for FakeOutput {
            fn dispatch(&self, _formatted: &str, _severity: Severity) -> Result<(), OutputError> {
                Ok(())
            }
        }

        let mut registry = Registry::new();
        registry
            .register(OutputId::new("file"), Box::new(exporter))
            .unwrap();
        registry
            .register(OutputId::new("fake"), Box::new(FakeOutput))
            .unwrap();

        registry.dispatch("registered via the Port", Severity::Info);

        let contents = fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "registered via the Port\n");
    }
}
