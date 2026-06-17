use crate::event::LogEvent;

pub trait LoggerProvider: Send + Sync {
    fn log(&self, event: LogEvent);
    fn flush(&self);
}
