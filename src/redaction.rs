use kit_config::RedactionConfig;

pub struct Redactor {
    config: RedactionConfig,
}

impl Redactor {
    pub fn new(config: RedactionConfig) -> Self {
        Self { config }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn redact(&self, key: &str, value: &str) -> String {
        if !self.config.enabled {
            return value.to_string();
        }

        for field in &self.config.fields {
            if key.eq_ignore_ascii_case(field) || key.to_lowercase().contains(&field.to_lowercase()) {
                return "**REDACTED**".to_string();
            }
        }

        value.to_string()
    }

    pub fn is_sensitive(&self, key: &str) -> bool {
        if !self.config.enabled {
            return false;
        }

        self.config.fields.iter().any(|f| {
            key.eq_ignore_ascii_case(f) || key.to_lowercase().contains(&f.to_lowercase())
        })
    }
}
