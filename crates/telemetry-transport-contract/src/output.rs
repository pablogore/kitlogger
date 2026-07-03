use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::Mutex;

use kit_config::{OutputTarget, RotationConfig};

use crate::event::LogEvent;
use crate::formatter::Formatter;

pub trait Output: Send + Sync {
    fn write(&self, event: &LogEvent, formatter: &dyn Formatter) -> io::Result<()>;
    fn flush(&self) -> io::Result<()>;
}

pub struct ConsoleOutput {
    target: OutputTarget,
}

impl ConsoleOutput {
    pub fn new(target: OutputTarget) -> Self {
        Self { target }
    }
}

impl Output for ConsoleOutput {
    fn write(&self, event: &LogEvent, formatter: &dyn Formatter) -> io::Result<()> {
        let line = formatter.format(event);
        match self.target {
            OutputTarget::Console | OutputTarget::Stdout => {
                writeln!(io::stdout(), "{}", line)
            }
            OutputTarget::Stderr => {
                writeln!(io::stderr(), "{}", line)
            }
        }
    }

    fn flush(&self) -> io::Result<()> {
        match self.target {
            OutputTarget::Console | OutputTarget::Stdout => io::stdout().flush(),
            OutputTarget::Stderr => io::stderr().flush(),
        }
    }
}

pub struct FileOutput {
    file: Mutex<File>,
    path: String,
    rotation: Option<RotationConfig>,
    current_size: Mutex<u64>,
}

impl FileOutput {
    pub fn new(path: impl Into<String>, rotation: Option<RotationConfig>) -> io::Result<Self> {
        let path_str = path.into();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path_str)?;
        let metadata = file.metadata()?;
        let size = metadata.len();

        Ok(Self {
            file: Mutex::new(file),
            path: path_str,
            rotation,
            current_size: Mutex::new(size),
        })
    }
}

impl Output for FileOutput {
    fn write(&self, event: &LogEvent, formatter: &dyn Formatter) -> io::Result<()> {
        let line = formatter.format(event);
        let mut file = self.file.lock().unwrap();
        let line_bytes = line.as_bytes();
        let line_len = line_bytes.len() as u64;

        if let Some(ref rotation) = self.rotation {
            if rotation.enabled {
                let mut current_size = self.current_size.lock().unwrap();
                let max_size = (rotation.max_size_mb as u64) * 1024 * 1024;
                if *current_size + line_len > max_size {
                    drop(file);
                    self.rotate()?;
                    file = self.file.lock().unwrap();
                    *current_size = 0;
                }
                *current_size += line_len;
            }
        }

        file.write_all(line_bytes)?;
        file.write_all(b"\n")?;
        Ok(())
    }

    fn flush(&self) -> io::Result<()> {
        self.file.lock().unwrap().flush()
    }
}

impl FileOutput {
    fn rotate(&self) -> io::Result<()> {
        let path = Path::new(&self.path);
        if path.exists() {
            let backup = path.with_extension("log.1");
            if backup.exists() {
                fs::remove_file(&backup)?;
            }
            fs::rename(path, &backup)?;
        }

        let new_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        *self.file.lock().unwrap() = new_file;
        Ok(())
    }
}

pub fn output_from_target(target: &OutputTarget) -> Box<dyn Output> {
    Box::new(ConsoleOutput::new(*target))
}
