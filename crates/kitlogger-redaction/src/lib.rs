//! Sensitive-attribute redaction over `LogRecord`, driven by
//! `kit_config::RedactionConfig`.

mod redactor;

pub use redactor::Redactor;
