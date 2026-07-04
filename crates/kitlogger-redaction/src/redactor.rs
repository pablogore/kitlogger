//! Redaction of sensitive `LogRecord` attribute values.

use kit_config::RedactionConfig;
use kitlogger_log_domain::{LogAttribute, LogAttributeValue, LogRecord};

/// Fixed value substituted for attribute values identified as sensitive.
pub const REDACTED_MARKER: &str = "**REDACTED**";

/// Decides which `LogRecord` attribute values are sensitive and replaces
/// them with [`REDACTED_MARKER`], per a `kit_config::RedactionConfig`.
pub struct Redactor {
    config: RedactionConfig,
}

impl Redactor {
    /// Creates a new `Redactor` from the given `RedactionConfig`.
    pub fn new(config: RedactionConfig) -> Self {
        Redactor { config }
    }

    /// Returns a new `LogRecord` with sensitive attribute values replaced by
    /// [`REDACTED_MARKER`]. Does not mutate `record`.
    ///
    /// An attribute is sensitive when its name case-insensitively contains
    /// one of the configured field identifiers. If redaction is disabled,
    /// returns a record equivalent to the input.
    pub fn redact(&self, record: &LogRecord) -> LogRecord {
        let attributes = record
            .attributes()
            .iter()
            .map(|attribute| self.redact_attribute(attribute))
            .collect();

        LogRecord::new(
            *record.timestamp(),
            *record.severity(),
            record.message().to_string(),
            attributes,
        )
        .expect("redaction preserves the original record's validity")
    }

    fn redact_attribute(&self, attribute: &LogAttribute) -> LogAttribute {
        if self.is_sensitive(attribute.name()) {
            LogAttribute::new(
                attribute.name().to_string(),
                LogAttributeValue::string(REDACTED_MARKER.to_string()),
            )
            .expect("attribute name is unchanged and was already valid")
        } else {
            attribute.clone()
        }
    }

    fn is_sensitive(&self, name: &str) -> bool {
        self.config.enabled
            && self
                .config
                .fields
                .iter()
                .any(|field| name.to_lowercase().contains(&field.to_lowercase()))
    }
}

#[cfg(test)]
mod tests {
    use kit_config::RedactionConfig;
    use kitlogger_log_domain::{LogAttribute, LogAttributeValue, LogRecord, Severity};
    use std::time::SystemTime;

    use crate::Redactor;

    fn record_with_attributes(attributes: Vec<LogAttribute>) -> LogRecord {
        LogRecord::new(
            SystemTime::now(),
            Severity::Info,
            "test message".to_string(),
            attributes,
        )
        .expect("valid record")
    }

    #[test]
    fn redacts_matching_field_case_insensitive() {
        // `LogAttribute` names are validated as lowercase-only
        // (`^[a-z][a-z0-9._]{0,63}$`), so mixed-case attribute names cannot
        // exist. Case-insensitivity is instead exercised via a mixed-case
        // configured field identifier matching a valid lowercase attribute
        // name — same FR-001 contract, compliant with the domain model.
        let config = RedactionConfig {
            enabled: true,
            fields: vec!["Password".to_string()],
        };
        let redactor = Redactor::new(config);
        let attribute = LogAttribute::new(
            "password".to_string(),
            LogAttributeValue::string("hunter2".to_string()),
        )
        .expect("valid attribute name");
        let record = record_with_attributes(vec![attribute]);

        let redacted = redactor.redact(&record);

        assert_eq!(redacted.attributes().len(), 1);
        assert_eq!(redacted.attributes()[0].name(), "password");
        assert_eq!(
            redacted.attributes()[0].value(),
            &LogAttributeValue::string("**REDACTED**".to_string())
        );
    }

    #[test]
    fn leaves_non_matching_fields_untouched() {
        let config = RedactionConfig {
            enabled: true,
            fields: vec!["password".to_string()],
        };
        let redactor = Redactor::new(config);
        let attribute = LogAttribute::new(
            "username".to_string(),
            LogAttributeValue::string("alice".to_string()),
        )
        .expect("valid attribute name");
        let record = record_with_attributes(vec![attribute]);

        let redacted = redactor.redact(&record);

        assert_eq!(redacted.attributes().len(), 1);
        assert_eq!(redacted.attributes()[0].name(), "username");
        assert_eq!(
            redacted.attributes()[0].value(),
            &LogAttributeValue::string("alice".to_string())
        );
    }

    #[test]
    fn does_not_mutate_input_record() {
        let config = RedactionConfig {
            enabled: true,
            fields: vec!["password".to_string()],
        };
        let redactor = Redactor::new(config);
        let attribute = LogAttribute::new(
            "password".to_string(),
            LogAttributeValue::string("hunter2".to_string()),
        )
        .expect("valid attribute name");
        let record = record_with_attributes(vec![attribute]);
        let original = record.clone();

        let _redacted = redactor.redact(&record);

        assert_eq!(record, original);
        assert_eq!(
            record.attributes()[0].value(),
            &LogAttributeValue::string("hunter2".to_string())
        );
    }

    #[test]
    fn disabled_config_returns_record_unchanged() {
        let config = RedactionConfig {
            enabled: false,
            fields: vec!["password".to_string()],
        };
        let redactor = Redactor::new(config);
        let attribute = LogAttribute::new(
            "password".to_string(),
            LogAttributeValue::string("hunter2".to_string()),
        )
        .expect("valid attribute name");
        let record = record_with_attributes(vec![attribute]);

        let redacted = redactor.redact(&record);

        assert_eq!(
            redacted.attributes()[0].value(),
            &LogAttributeValue::string("hunter2".to_string())
        );
    }
}
