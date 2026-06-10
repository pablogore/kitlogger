use std::collections::HashMap;
use std::time::SystemTime;

use kit_config::LogLevel;

#[derive(Debug, Clone)]
pub struct LogEvent {
    pub level: LogLevel,
    pub message: String,
    pub target: String,
    pub timestamp: SystemTime,
    pub module: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub fields: HashMap<String, serde_json::Value>,
    pub correlation_id: Option<String>,
}

impl LogEvent {
    pub fn new(
        level: LogLevel,
        message: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self {
            level,
            message: message.into(),
            target: target.into(),
            timestamp: SystemTime::now(),
            module: String::new(),
            file: None,
            line: None,
            fields: HashMap::new(),
            correlation_id: None,
        }
    }

    pub fn with_module(mut self, module: impl Into<String>) -> Self {
        self.module = module.into();
        self
    }

    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    pub fn with_line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }

    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }
}
