//! Stream router for console exporter.

use std::io::{self, Write};

use crate::error::ExportError;
use kitlogger_log_domain::Severity;

/// Maps log levels to output streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LevelStream {
    Stdout,
    Stderr,
}

/// Maps severity levels to output streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LevelStreamMapping {
    pub debug: LevelStream,
    pub info: LevelStream,
    pub warn: LevelStream,
    pub error: LevelStream,
    pub fatal: LevelStream,
}

impl Default for LevelStreamMapping {
    fn default() -> Self {
        Self {
            debug: LevelStream::Stdout,
            info: LevelStream::Stdout,
            warn: LevelStream::Stderr,
            error: LevelStream::Stderr,
            fatal: LevelStream::Stderr,
        }
    }
}

/// Stream router that writes log messages to the appropriate stream based on severity.
pub struct StreamRouter {
    mapping: LevelStreamMapping,
    stdout: Box<dyn Write + Send>,
    stderr: Box<dyn Write + Send>,
}

impl Default for StreamRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamRouter {
    /// Creates a new StreamRouter with default mappings and stdout/stderr writers.
    pub fn new() -> Self {
        Self::with_writers(LevelStreamMapping::default(), Box::new(io::stdout()), Box::new(io::stderr()))
    }

    /// Creates a new StreamRouter with custom mapping and writers.
    pub fn with_writers(
        mapping: LevelStreamMapping,
        stdout: Box<dyn Write + Send>,
        stderr: Box<dyn Write + Send>,
    ) -> Self {
        Self {
            mapping,
            stdout,
            stderr,
        }
    }

    /// Writes a message to the appropriate stream based on severity.
    pub fn write(&mut self, msg: &str, severity: Severity) -> Result<(), ExportError> {
        let stream = match severity {
            Severity::Trace => self.mapping.debug,
            Severity::Debug => self.mapping.debug,
            Severity::Info => self.mapping.info,
            Severity::Warn => self.mapping.warn,
            Severity::Error => self.mapping.error,
            Severity::Fatal => self.mapping.fatal,
        };

        match stream {
            LevelStream::Stdout => {
                writeln!(self.stdout, "{}", msg)?;
                self.stdout.flush()?;
            }
            LevelStream::Stderr => {
                writeln!(self.stderr, "{}", msg)?;
                self.stderr.flush()?;
            }
        }

        Ok(())
    }

    /// Returns a reference to the current mapping.
    pub fn mapping(&self) -> &LevelStreamMapping {
        &self.mapping
    }

    /// Sets a new level-to-stream mapping.
    pub fn set_mapping(&mut self, mapping: LevelStreamMapping) {
        self.mapping = mapping;
    }

    /// Sets new writers for stdout and stderr.
    pub fn set_writers(&mut self, stdout: Box<dyn Write + Send>, stderr: Box<dyn Write + Send>) {
        self.stdout = stdout;
        self.stderr = stderr;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kitlogger_log_domain::Severity;
    use std::io::{Error, ErrorKind};

    /// A writer that records all written data and makes it observable for assertions.
    #[derive(Clone)]
    struct TestWriter {
        buf: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    }

    impl TestWriter {
        fn new() -> Self {
            Self {
                buf: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }

        fn contents(&self) -> String {
            let guard = self.buf.lock().unwrap();
            String::from_utf8(guard.clone()).unwrap()
        }

        fn clear(&self) {
            let mut guard = self.buf.lock().unwrap();
            guard.clear();
        }
    }

    impl Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let mut guard = self.buf.lock().unwrap();
            guard.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl std::panic::UnwindSafe for TestWriter {}

    fn make_router() -> (StreamRouter, TestWriter, TestWriter) {
        let stdout_writer = TestWriter::new();
        let stderr_writer = TestWriter::new();
        let router = StreamRouter::with_writers(
            LevelStreamMapping::default(),
            Box::new(stdout_writer.clone()),
            Box::new(stderr_writer.clone()),
        );
        (router, stdout_writer, stderr_writer)
    }

    fn make_router_with_mapping(mapping: LevelStreamMapping) -> (StreamRouter, TestWriter, TestWriter) {
        let stdout_writer = TestWriter::new();
        let stderr_writer = TestWriter::new();
        let router = StreamRouter::with_writers(
            mapping,
            Box::new(stdout_writer.clone()),
            Box::new(stderr_writer.clone()),
        );
        (router, stdout_writer, stderr_writer)
    }

    #[test]
    fn info_routes_to_individual_info_field_not_debug() {
        let mapping = LevelStreamMapping {
            debug: LevelStream::Stdout,
            info: LevelStream::Stderr,
            warn: LevelStream::Stderr,
            error: LevelStream::Stderr,
            fatal: LevelStream::Stderr,
        };
        let (mut router, stdout, stderr) = make_router_with_mapping(mapping);

        router.write("info msg", Severity::Info).unwrap();

        // BUG: current code routes Info using the debug field (Stdout).
        // With the fix it should route using the info field (Stderr).
        assert!(
            stdout.contents().is_empty(),
            "Info should NOT go to stdout when info field is Stderr. Got stdout: {:?}",
            stdout.contents()
        );
        assert!(
            stderr.contents().contains("info msg"),
            "Info should go to stderr when info field is Stderr. Got stderr: {:?}",
            stderr.contents()
        );
    }

    #[test]
    fn error_routes_to_individual_error_field_not_warn() {
        let mapping = LevelStreamMapping {
            debug: LevelStream::Stdout,
            info: LevelStream::Stdout,
            warn: LevelStream::Stderr,
            error: LevelStream::Stdout,
            fatal: LevelStream::Stderr,
        };
        let (mut router, stdout, stderr) = make_router_with_mapping(mapping);

        router.write("error msg", Severity::Error).unwrap();

        // BUG: current code routes Error using the warn field (Stderr).
        // With the fix it should route using the error field (Stdout).
        assert!(
            stderr.contents().is_empty(),
            "Error should NOT go to stderr when error field is Stdout. Got stderr: {:?}",
            stderr.contents()
        );
        assert!(
            stdout.contents().contains("error msg"),
            "Error should go to stdout when error field is Stdout. Got stdout: {:?}",
            stdout.contents()
        );
    }

    #[test]
    fn default_mapping_routes_correctly() {
        let (mut router, stdout, stderr) = make_router();

        router.write("debug msg", Severity::Debug).unwrap();
        assert!(stdout.contents().contains("debug msg"), "Debug should go to stdout");
        assert!(stderr.contents().is_empty(), "Debug should NOT go to stderr");

        stdout.clear();
        stderr.clear();

        router.write("error msg", Severity::Error).unwrap();
        assert!(stderr.contents().contains("error msg"), "Error should go to stderr");
        assert!(stdout.contents().is_empty(), "Error should NOT go to stdout");

        stderr.clear();

        router.write("fatal msg", Severity::Fatal).unwrap();
        assert!(stderr.contents().contains("fatal msg"), "Fatal should go to stderr");
        assert!(stdout.contents().is_empty(), "Fatal should NOT go to stdout");
    }

    #[test]
    fn trace_routes_to_debug_field() {
        let mapping = LevelStreamMapping {
            debug: LevelStream::Stderr,
            info: LevelStream::Stdout,
            warn: LevelStream::Stdout,
            error: LevelStream::Stdout,
            fatal: LevelStream::Stdout,
        };
        let (mut router, stdout, stderr) = make_router_with_mapping(mapping);

        router.write("trace msg", Severity::Trace).unwrap();
        assert!(
            stderr.contents().contains("trace msg"),
            "Trace should route to the debug field (Stderr). Got stderr: {:?}",
            stderr.contents()
        );
        assert!(
            stdout.contents().is_empty(),
            "Trace should NOT go to stdout when debug field is Stderr"
        );
    }

    #[test]
    fn custom_mapping_warn_to_stdout() {
        // Spec scenario: WARN → stdout with custom mapping
        let mapping = LevelStreamMapping {
            debug: LevelStream::Stdout,
            info: LevelStream::Stdout,
            warn: LevelStream::Stdout, // WARN goes to stdout instead of stderr
            error: LevelStream::Stderr,
            fatal: LevelStream::Stderr,
        };
        let (mut router, stdout, stderr) = make_router_with_mapping(mapping);

        router.write("warn msg", Severity::Warn).unwrap();
        assert!(
            stdout.contents().contains("warn msg"),
            "Warn should go to stdout when mapping.warn is Stdout"
        );
        assert!(
            stderr.contents().is_empty(),
            "Warn should NOT go to stderr when mapping.warn is Stdout"
        );
    }

    #[test]
    fn custom_mapping_fatal_to_stdout() {
        let mapping = LevelStreamMapping {
            debug: LevelStream::Stdout,
            info: LevelStream::Stdout,
            warn: LevelStream::Stderr,
            error: LevelStream::Stderr,
            fatal: LevelStream::Stdout, // Fatal goes to stdout
        };
        let (mut router, stdout, stderr) = make_router_with_mapping(mapping);

        router.write("fatal msg", Severity::Fatal).unwrap();
        assert!(
            stdout.contents().contains("fatal msg"),
            "Fatal should go to stdout when mapping.fatal is Stdout"
        );
        assert!(
            stderr.contents().is_empty(),
            "Fatal should NOT go to stderr when mapping.fatal is Stdout"
        );
    }

    #[test]
    fn reversed_mapping_all_routes_correct() {
        // Completely reversed mapping
        let mapping = LevelStreamMapping {
            debug: LevelStream::Stderr,
            info: LevelStream::Stderr,
            warn: LevelStream::Stdout,
            error: LevelStream::Stdout,
            fatal: LevelStream::Stdout,
        };
        let (mut router, stdout, stderr) = make_router_with_mapping(mapping);

        router.write("debug on stderr", Severity::Debug).unwrap();
        assert!(stderr.contents().contains("debug on stderr"));

        stderr.clear();
        router.write("info on stderr", Severity::Info).unwrap();
        assert!(stderr.contents().contains("info on stderr"));

        stderr.clear();
        router.write("warn on stdout", Severity::Warn).unwrap();
        assert!(stdout.contents().contains("warn on stdout"));
    }

    #[test]
    fn write_error_returns_err_without_panic() {
        // Spec: write failure returns error without panic
        struct FailingWriter;
        impl Write for FailingWriter {
            fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
                Err(Error::new(ErrorKind::BrokenPipe, "broken pipe"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let mut router = StreamRouter::with_writers(
            LevelStreamMapping::default(),
            Box::new(FailingWriter),
            Box::new(FailingWriter),
        );

        let result = router.write("test", Severity::Info);
        assert!(result.is_err(), "Write failure should return an error");
        match result {
            Err(ExportError::Io(_)) => {} // Expected
            _ => panic!("Expected ExportError::Io, got: {:?}", result),
        }
    }

    #[test]
    fn set_mapping_updates_routing() {
        let (mut router, stdout, stderr) = make_router();

        // Default: info goes to stdout
        router.write("first", Severity::Info).unwrap();
        assert!(stdout.contents().contains("first"));
        stdout.clear();

        // Change mapping: info → stderr
        let new_mapping = LevelStreamMapping {
            debug: LevelStream::Stdout,
            info: LevelStream::Stderr,
            warn: LevelStream::Stderr,
            error: LevelStream::Stderr,
            fatal: LevelStream::Stderr,
        };
        router.set_mapping(new_mapping);

        router.write("second", Severity::Info).unwrap();
        assert!(stderr.contents().contains("second"), "After set_mapping, info should go to stderr");
    }

    #[test]
    fn set_writers_replaces_output() {
        let mapping = LevelStreamMapping::default();
        let stdout1 = TestWriter::new();
        let stderr1 = TestWriter::new();
        let mut router = StreamRouter::with_writers(
            mapping,
            Box::new(stdout1.clone()),
            Box::new(stderr1.clone()),
        );

        router.write("before", Severity::Info).unwrap();
        assert!(stdout1.contents().contains("before"));

        // Replace writers
        let stdout2 = TestWriter::new();
        let stderr2 = TestWriter::new();
        router.set_writers(Box::new(stdout2.clone()), Box::new(stderr2.clone()));

        router.write("after", Severity::Info).unwrap();
        // Old writer should NOT have the new content
        assert!(!stdout1.contents().contains("after"), "Old writer should not receive new writes");
        // New writer should have the content
        assert!(stdout2.contents().contains("after"), "New writer should receive writes");
    }
}