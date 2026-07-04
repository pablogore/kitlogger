#[cfg(test)]
mod tests {
    use telemetry_config_semantics::{
        CapabilityState, CompressionType, ExporterConfig, ResourceConfig, SchemaVersion,
        TelemetryConfig, VerbosityLevel, VerbosityPolicy,
    };

    // -----------------------------------------------------------------------
    // Existing tests — updated to use `telemetry_enabled`
    // -----------------------------------------------------------------------

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert!(config.telemetry_enabled);
        assert!(config.exporters.is_some());
        assert!(config.resources.is_some());
        assert!(config.verbosity.is_some());
        assert_eq!(config.schema_version.version, "1.0.0".to_string());
    }

    #[test]
    fn test_telemetry_config_disabled() {
        let config = TelemetryConfig {
            telemetry_enabled: false,
            ..Default::default()
        };
        assert!(!config.telemetry_enabled);
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

    // -----------------------------------------------------------------------
    // CapabilityState serde round-trips
    // -----------------------------------------------------------------------

    #[test]
    fn capability_state_enabled_serde_round_trip() {
        let original = CapabilityState::Enabled;
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: CapabilityState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, original);
    }

    #[test]
    fn capability_state_disabled_serde_round_trip() {
        let original = CapabilityState::Disabled;
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: CapabilityState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, original);
    }

    // -----------------------------------------------------------------------
    // FR-001: exactly four capability flags, no `correlation_enabled`
    // -----------------------------------------------------------------------

    /// Compile-fail-style assertion (design.md Testing Strategy): a struct literal
    /// naming exactly the four surviving fields plus the four Option/struct fields,
    /// with no `..Default::default()`, only compiles if `TelemetryConfig` has no
    /// other required fields (i.e. `correlation_enabled` and `sampling` are gone).
    fn four_flag_config() -> TelemetryConfig {
        TelemetryConfig {
            telemetry_enabled: true,
            tracing_enabled: true,
            metrics_enabled: true,
            propagation_enabled: true,
            exporters: None,
            resources: None,
            verbosity: None,
            schema_version: SchemaVersion::default(),
        }
    }

    #[test]
    fn telemetry_config_has_exactly_four_capability_flags() {
        let config = four_flag_config();
        assert!(config.telemetry_enabled);
        assert!(config.tracing_enabled);
        assert!(config.metrics_enabled);
        assert!(config.propagation_enabled);
    }

    #[test]
    fn telemetry_config_capability_flags_all_default_true() {
        let config = TelemetryConfig::default();
        assert!(config.tracing_enabled);
        assert!(config.metrics_enabled);
        assert!(config.propagation_enabled);
    }

    #[test]
    fn telemetry_config_capability_flags_can_be_set_false() {
        let config = TelemetryConfig {
            tracing_enabled: false,
            metrics_enabled: false,
            ..Default::default()
        };
        assert!(!config.tracing_enabled);
        assert!(!config.metrics_enabled);
        assert!(config.propagation_enabled);
    }

    /// serde backward-compat: old payload with `"enabled": true` and no capability keys.
    #[test]
    fn telemetry_config_serde_backward_compat_enabled_alias() {
        let json = r#"{
            "enabled": true,
            "exporters": null,
            "resources": null,
            "verbosity": null,
            "schema_version": { "version": "1.0.0", "description": null }
        }"#;
        let config: TelemetryConfig = serde_json::from_str(json).unwrap();
        assert!(
            config.telemetry_enabled,
            "telemetry_enabled must be true via 'enabled' alias"
        );
        assert!(
            config.tracing_enabled,
            "tracing_enabled must default to true"
        );
        assert!(
            config.metrics_enabled,
            "metrics_enabled must default to true"
        );
        assert!(
            config.propagation_enabled,
            "propagation_enabled must default to true"
        );
    }

    // -----------------------------------------------------------------------
    // FR-012: Serde round-trip tests for all eight remaining types
    // -----------------------------------------------------------------------

    #[test]
    fn telemetry_config_serde_round_trip_mixed_capability_flags() {
        let original = TelemetryConfig {
            telemetry_enabled: true,
            tracing_enabled: false,
            metrics_enabled: true,
            propagation_enabled: true,
            exporters: Some(vec![ExporterConfig::default()]),
            resources: Some(ResourceConfig::default()),
            verbosity: Some(VerbosityPolicy::default()),
            schema_version: SchemaVersion::default(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: TelemetryConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, original);
    }

    #[test]
    fn exporter_config_serde_round_trip() {
        let original = ExporterConfig::default();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ExporterConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, original);
    }

    #[test]
    fn compression_type_serde_round_trip() {
        for variant in [CompressionType::None, CompressionType::Gzip] {
            let json = serde_json::to_string(&variant).unwrap();
            let deserialized: CompressionType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, variant);
        }
    }

    #[test]
    fn resource_config_serde_round_trip() {
        let original = ResourceConfig::default();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ResourceConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, original);
    }

    #[test]
    fn verbosity_policy_serde_round_trip() {
        let original = VerbosityPolicy::default();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: VerbosityPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, original);
    }

    #[test]
    fn schema_version_serde_round_trip() {
        let original = SchemaVersion::default();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: SchemaVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, original);
    }

    /// FR-012: Backward-compatible deserialization — legacy JSON → defaults to all flags true.
    #[test]
    fn telemetry_config_legacy_json_backward_compat() {
        let json = r#"{
            "enabled": true,
            "exporters": null,
            "resources": null,
            "verbosity": null,
            "schema_version": { "version": "1.0.0", "description": null }
        }"#;
        let config: TelemetryConfig = serde_json::from_str(json).unwrap();
        assert!(config.telemetry_enabled);
        assert!(config.tracing_enabled);
        assert!(config.metrics_enabled);
        assert!(config.propagation_enabled);
    }
}
