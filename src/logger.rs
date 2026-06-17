use std::sync::Arc;

use kit_config::{Validation, LoggingConfig};

use crate::buffering::Buffer;
use crate::event::LogEvent;
use crate::formatter::{formatter_from_config, Formatter};
use crate::output::{output_from_target, Output};
use crate::provider::LoggerProvider;
use crate::redaction::Redactor;
use crate::sampling::Sampler;

pub struct Logger {
    provider: Arc<dyn LoggerProvider>,
    config: LoggingConfig,
    sampler: Sampler,
    redactor: Redactor,
}

impl Logger {
    pub fn new(config: LoggingConfig, provider: Arc<dyn LoggerProvider>) -> Self {
        let sampler = Sampler::new(config.sampling.clone());
        let redactor = Redactor::new(config.redact.clone());

        Self {
            provider,
            config,
            sampler,
            redactor,
        }
    }

    pub fn from_config(config: LoggingConfig) -> Result<Self, Box<dyn std::error::Error>> {
        config.validate().map_err(|report| -> Box<dyn std::error::Error> {
            format!("Configuration validation failed: {:?}", report).into()
        })?;

        let formatter: Box<dyn Formatter> = formatter_from_config(&config.format);
        let mut outputs: Vec<Box<dyn Output>> = Vec::new();

        for target in &config.output.targets {
            outputs.push(output_from_target(target));
        }

        let provider = Arc::new(DefaultProvider::new(outputs, formatter, config.buffering.clone()));

        Ok(Self::new(config, provider))
    }

    pub fn log(&self, event: LogEvent) {
        if !self.config.enabled {
            return;
        }

        if event.level as u8 > self.config.level as u8 {
            return;
        }

        if !self.sampler.should_sample() {
            return;
        }

        let mut event = event;
        if self.redactor.is_enabled() {
            for (key, value) in event.fields.iter_mut() {
                if value.as_str().is_some() && self.redactor.is_sensitive(key) {
                    *value = serde_json::Value::String("**REDACTED**".to_string());
                }
            }
        }

        self.provider.log(event);
    }

    pub fn trace(&self, message: impl Into<String>) {
        self.log(LogEvent::new(
            kit_config::LogLevel::Trace,
            message,
            "",
        ));
    }

    pub fn debug(&self, message: impl Into<String>) {
        self.log(LogEvent::new(
            kit_config::LogLevel::Debug,
            message,
            "",
        ));
    }

    pub fn info(&self, message: impl Into<String>) {
        self.log(LogEvent::new(
            kit_config::LogLevel::Info,
            message,
            "",
        ));
    }

    pub fn warn(&self, message: impl Into<String>) {
        self.log(LogEvent::new(
            kit_config::LogLevel::Warn,
            message,
            "",
        ));
    }

    pub fn error(&self, message: impl Into<String>) {
        self.log(LogEvent::new(
            kit_config::LogLevel::Error,
            message,
            "",
        ));
    }

    pub fn flush(&self) {
        self.provider.flush();
    }

    pub fn config(&self) -> &LoggingConfig {
        &self.config
    }

    pub fn into_provider(self) -> Arc<dyn LoggerProvider> {
        self.provider.clone()
    }
}

pub struct LoggerBuilder {
    config: Option<LoggingConfig>,
    provider: Option<Arc<dyn LoggerProvider>>,
}

impl LoggerBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            provider: None,
        }
    }

    pub fn with_config(mut self, config: LoggingConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_provider(mut self, provider: Arc<dyn LoggerProvider>) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn build(self) -> Result<Logger, Box<dyn std::error::Error>> {
        let config = self.config.unwrap_or_default();
        config.validate().map_err(|report| -> Box<dyn std::error::Error> {
            format!("Configuration validation failed: {:?}", report).into()
        })?;

        if let Some(provider) = self.provider {
            Ok(Logger::new(config, provider))
        } else {
            Logger::from_config(config)
        }
    }
}

impl Default for LoggerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

enum DefaultProviderInner {
    Direct {
        outputs: Vec<Box<dyn Output>>,
        formatter: Box<dyn Formatter>,
    },
    Buffered(Buffer),
}

struct DefaultProvider {
    inner: DefaultProviderInner,
}

impl DefaultProvider {
    fn new(
        outputs: Vec<Box<dyn Output>>,
        formatter: Box<dyn Formatter>,
        buffering_config: kit_config::BufferingConfig,
    ) -> Self {
        let inner = if buffering_config.enabled {
            DefaultProviderInner::Buffered(Buffer::new(buffering_config, outputs, formatter))
        } else {
            DefaultProviderInner::Direct { outputs, formatter }
        };

        Self { inner }
    }
}

impl LoggerProvider for DefaultProvider {
    fn log(&self, event: LogEvent) {
        match &self.inner {
            DefaultProviderInner::Buffered(buffer) => {
                buffer.send(event);
            }
            DefaultProviderInner::Direct { outputs, formatter } => {
                for output in outputs {
                    let _ = output.write(&event, &**formatter);
                }
            }
        }
    }

    fn flush(&self) {
        match &self.inner {
            DefaultProviderInner::Buffered(buffer) => {
                buffer.flush();
            }
            DefaultProviderInner::Direct { outputs, .. } => {
                for output in outputs {
                    let _ = output.flush();
                }
            }
        }
    }
}
