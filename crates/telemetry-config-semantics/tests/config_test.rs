#[cfg(test)]
mod tests {
    use telemetry_config_semantics::{
        CompressionType, ExporterConfig, ResourceConfig, SamplingPolicy, SamplingPolicyType,
        SchemaVersion, TelemetryConfig, VerbosityLevel, VerbosityPolicy,
    };

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert!(config.enabled);
        assert!(config.sampling.is_some());
        assert!(config.exporters.is_some());
        assert!(config.resources.is_some());
        assert!(config.verbosity.is_some());
        assert_eq!(config.schema_version.version, "1.0.0".to_string());
    }

    #[test]
    fn test_telemetry_config_disabled() {
        let config = TelemetryConfig {
            enabled: false,
            ..Default::default()
        };
        assert!(!config.enabled);
    }

    #[test]
    fn test_sampling_policy_default() {
        let policy = SamplingPolicy::default();
        assert_eq!(policy.policy_type, SamplingPolicyType::AlwaysOn);
        assert_eq!(policy.sampling_rate, 1.0);
    }

    #[test]
    fn test_exporter_config_default() {
        let exporter = ExporterConfig::default();
        assert_eq!(exporter.exporter_type, "console".to_string());
        assert_eq!(exporter.compression, CompressionType::None);
        assert_eq!(exporter.timeout_secs, 30);
    }

    #[test]
    fn test_resource_config_default() {
        let resource = ResourceConfig::default();
        assert_eq!(resource.service_version, "unknown".to_string());
        assert_eq!(resource.deployment_environment, "development".to_string());
    }

    #[test]
    fn test_verbosity_policy_default() {
        let verbosity = VerbosityPolicy::default();
        assert_eq!(verbosity.trace_level, VerbosityLevel::Info);
        assert_eq!(verbosity.metric_level, VerbosityLevel::Info);
        assert_eq!(verbosity.log_level, VerbosityLevel::Info);
    }

    #[test]
    fn test_schema_version_default() {
        let schema = SchemaVersion::default();
        assert_eq!(schema.version, "1.0.0".to_string());
    }
}
