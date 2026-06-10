use std::fs;
use std::path::Path;

use kit_config::RotationConfig;

pub struct RotationManager {
    config: RotationConfig,
}

impl RotationManager {
    pub fn new(config: RotationConfig) -> Self {
        Self { config }
    }

    pub fn should_rotate(&self, current_size: u64) -> bool {
        self.config.enabled && current_size >= (self.config.max_size_mb as u64) * 1024 * 1024
    }

    pub fn rotate(&self, log_path: &Path) -> std::io::Result<()> {
        if !log_path.exists() {
            return Ok(());
        }

        let max_backups = self.config.max_backups as usize;

        for i in (1..max_backups).rev() {
            let old = log_path.with_extension(format!("log.{}", i));
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
