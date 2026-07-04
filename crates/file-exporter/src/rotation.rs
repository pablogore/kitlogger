//! File rotation: a numbered-backup-chain algorithm ported from the
//! orphaned `telemetry_transport_contract::rotation::RotationManager` — the
//! canonical rotation algorithm retained from the legacy implementation
//! (NOT `output::FileOutput::rotate()`'s divergent, less-complete inline
//! single-backup version). See design.md Q3.

use std::fs;
use std::io;
use std::path::Path;

use kit_config::RotationConfig;

/// Owns the rotation policy for a single file exporter. Internal to
/// `file-exporter` — rotation has exactly one consumer and no reuse case
/// among the current or planned output destinations (design.md Q3).
pub struct RotationManager {
    config: RotationConfig,
}

impl RotationManager {
    /// Creates a new `RotationManager` from the given `RotationConfig`.
    pub fn new(config: RotationConfig) -> Self {
        Self { config }
    }

    /// Returns whether writing a record that would bring the file to
    /// `prospective_size` bytes must rotate first. Always `false` when
    /// rotation is disabled (FR-004).
    pub fn should_rotate(&self, prospective_size: u64) -> bool {
        self.config.enabled && prospective_size > (self.config.max_size_mb as u64) * 1024 * 1024
    }

    /// Shifts existing numbered backups (`.log.1`, `.log.2`, ...) up by one
    /// slot and moves the current file into `.log.1`, discarding whatever
    /// falls beyond `RotationConfig.max_backups` (FR-003).
    pub fn rotate(&self, log_path: &Path) -> io::Result<()> {
        if !log_path.exists() {
            return Ok(());
        }

        let max_backups = self.config.max_backups as usize;

        for i in (1..max_backups).rev() {
            let old = log_path.with_extension(format!("log.{i}"));
            let new = log_path.with_extension(format!("log.{}", i + 1));
            if old.exists() {
                if new.exists() {
                    fs::remove_file(&new)?;
                }
                fs::rename(&old, &new)?;
            }
        }

        let first = log_path.with_extension("log.1");
        if first.exists() {
            fs::remove_file(&first)?;
        }
        fs::rename(log_path, &first)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn config(enabled: bool, max_size_mb: u32, max_backups: u32) -> RotationConfig {
        RotationConfig {
            enabled,
            max_size_mb,
            max_backups,
            max_age_days: 30,
        }
    }

    #[test]
    fn should_rotate_when_prospective_size_exceeds_max() {
        let manager = RotationManager::new(config(true, 1, 5));
        let one_mb = 1024 * 1024;

        assert!(!manager.should_rotate(one_mb));
        assert!(manager.should_rotate(one_mb + 1));
    }

    #[test]
    fn should_rotate_disabled_never_rotates() {
        let manager = RotationManager::new(config(false, 1, 5));
        assert!(!manager.should_rotate(u64::MAX));
    }

    #[test]
    fn backups_beyond_max_are_discarded() {
        let dir = tempfile::tempdir().unwrap();
        let log_path = dir.path().join("app.log");
        let manager = RotationManager::new(config(true, 1, 2));

        fs::write(&log_path, b"content-A").unwrap();
        manager.rotate(&log_path).unwrap();
        fs::write(&log_path, b"content-B").unwrap();
        manager.rotate(&log_path).unwrap();
        fs::write(&log_path, b"content-C").unwrap();
        manager.rotate(&log_path).unwrap();

        let backup_1 = log_path.with_extension("log.1");
        let backup_2 = log_path.with_extension("log.2");
        let backup_3 = log_path.with_extension("log.3");

        assert!(backup_1.exists());
        assert!(backup_2.exists());
        assert!(!backup_3.exists());

        assert_eq!(fs::read_to_string(&backup_1).unwrap(), "content-C");
        assert_eq!(fs::read_to_string(&backup_2).unwrap(), "content-B");
    }
}
