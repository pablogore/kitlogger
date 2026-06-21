#[cfg(test)]
mod tests {
    use telemetry_config_semantics::{
        CapabilityState, CompressionType, ConfigError, EffectiveTelemetryState, ExporterConfig,
        ResourceConfig, SamplingPolicy, SamplingPolicyType, SchemaVersion, TelemetryConfig,
        VerbosityLevel, VerbosityPolicy,
    };

    // -----------------------------------------------------------------------
    // Existing tests — updated to use `telemetry_enabled`
    // -----------------------------------------------------------------------

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert!(config.telemetry_enabled);
        assert!(config.sampling.is_some());
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

    // -----------------------------------------------------------------------
    // Phase 1.5: CapabilityState and EffectiveTelemetryState serde round-trips
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

    #[test]
    fn effective_telemetry_state_all_variants_serde_round_trip() {
        let variants = [
            EffectiveTelemetryState::Enabled,
            EffectiveTelemetryState::Disabled,
            EffectiveTelemetryState::Partial,
            EffectiveTelemetryState::Fallback,
        ];
        for variant in &variants {
            let serialized = serde_json::to_string(variant).unwrap();
            let deserialized: EffectiveTelemetryState = serde_json::from_str(&serialized).unwrap();
            assert_eq!(
                &deserialized, variant,
                "Round-trip failed for {:?}",
                variant
            );
        }
    }

    // -----------------------------------------------------------------------
    // Phase 2.5: Four new capability fields + defaults
    // -----------------------------------------------------------------------

    #[test]
    fn telemetry_config_capability_flags_all_default_true() {
        let config = TelemetryConfig::default();
        assert!(config.tracing_enabled);
        assert!(config.metrics_enabled);
        assert!(config.correlation_enabled);
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
        assert!(config.correlation_enabled);
        assert!(config.propagation_enabled);
    }

    /// serde backward-compat: old payload with `"enabled": true` and no capability keys.
    #[test]
    fn telemetry_config_serde_backward_compat_enabled_alias() {
        let json = r#"{
            "enabled": true,
            "sampling": null,
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
            config.correlation_enabled,
            "correlation_enabled must default to true"
        );
        assert!(
            config.propagation_enabled,
            "propagation_enabled must default to true"
        );
    }

    // -----------------------------------------------------------------------
    // Phase 3.1: SamplingPolicy::validate() tests
    // -----------------------------------------------------------------------

    #[test]
    fn sampling_validate_rate_below_zero_returns_err() {
        let policy = SamplingPolicy {
            policy_type: SamplingPolicyType::TraceIdRatio,
            sampling_rate: -0.1,
        };
        let result = policy.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ConfigError::InvalidSamplingRate(-0.1));
    }

    #[test]
    fn sampling_validate_rate_above_one_returns_err() {
        let policy = SamplingPolicy {
            policy_type: SamplingPolicyType::TraceIdRatio,
            sampling_rate: 1.5,
        };
        let result = policy.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ConfigError::InvalidSamplingRate(1.5));
    }

    #[test]
    fn sampling_validate_rate_zero_returns_ok() {
        let policy = SamplingPolicy {
            policy_type: SamplingPolicyType::TraceIdRatio,
            sampling_rate: 0.0,
        };
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn sampling_validate_rate_one_returns_ok() {
        let policy = SamplingPolicy {
            policy_type: SamplingPolicyType::TraceIdRatio,
            sampling_rate: 1.0,
        };
        assert!(policy.validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // Phase 4.1–4.4: effective_state() tests
    // -----------------------------------------------------------------------

    fn config_with_sampling_rate(rate: f64) -> TelemetryConfig {
        TelemetryConfig {
            sampling: Some(SamplingPolicy {
                policy_type: SamplingPolicyType::TraceIdRatio,
                sampling_rate: rate,
            }),
            ..Default::default()
        }
    }

    #[test]
    fn effective_state_disabled_when_telemetry_enabled_false() {
        let config = TelemetryConfig {
            telemetry_enabled: false,
            ..config_with_sampling_rate(1.0)
        };
        assert_eq!(config.effective_state(), EffectiveTelemetryState::Disabled);
    }

    #[test]
    fn effective_state_enabled_when_all_flags_true_and_valid_sampling() {
        let config = config_with_sampling_rate(0.5);
        assert_eq!(config.effective_state(), EffectiveTelemetryState::Enabled);
    }

    #[test]
    fn effective_state_partial_when_one_capability_flag_false() {
        let config = TelemetryConfig {
            tracing_enabled: false,
            ..config_with_sampling_rate(1.0)
        };
        assert_eq!(config.effective_state(), EffectiveTelemetryState::Partial);
    }

    #[test]
    fn effective_state_fallback_when_sampling_invalid() {
        let config = TelemetryConfig {
            sampling: Some(SamplingPolicy {
                policy_type: SamplingPolicyType::TraceIdRatio,
                sampling_rate: -0.5,
            }),
            ..Default::default()
        };
        assert_eq!(config.effective_state(), EffectiveTelemetryState::Fallback);
    }

    /// FR-007: Fallback supersedes Disabled — invalid sampling beats telemetry_enabled=false.
    #[test]
    fn effective_state_fallback_supersedes_disabled() {
        let config = TelemetryConfig {
            telemetry_enabled: false,
            sampling: Some(SamplingPolicy {
                policy_type: SamplingPolicyType::TraceIdRatio,
                sampling_rate: -0.5,
            }),
            ..Default::default()
        };
        assert_eq!(config.effective_state(), EffectiveTelemetryState::Fallback);
    }

    /// FR-009: effective_state() is pure — repeated calls return identical results.
    #[test]
    fn effective_state_is_deterministic() {
        let config = config_with_sampling_rate(0.5);
        let first = config.effective_state();
        let second = config.effective_state();
        let third = config.effective_state();
        assert_eq!(first, second);
        assert_eq!(second, third);
    }

    /// FR-010: TelemetryConfig::default() → Enabled.
    #[test]
    fn effective_state_default_config_is_enabled() {
        assert_eq!(
            TelemetryConfig::default().effective_state(),
            EffectiveTelemetryState::Enabled
        );
    }

    #[test]
    fn effective_state_partial_when_multiple_capability_flags_false() {
        let config = TelemetryConfig {
            telemetry_enabled: true,
            metrics_enabled: false,
            correlation_enabled: false,
            sampling: Some(SamplingPolicy {
                policy_type: SamplingPolicyType::TraceIdRatio,
                sampling_rate: 1.0,
            }),
            ..Default::default()
        };
        assert_eq!(config.effective_state(), EffectiveTelemetryState::Partial);
    }

    /// FR-007: Fallback supersedes Partial — invalid sampling beats a partially-enabled config.
    #[test]
    fn effective_state_fallback_supersedes_partial() {
        let config = TelemetryConfig {
            tracing_enabled: false,
            sampling: Some(SamplingPolicy {
                policy_type: SamplingPolicyType::TraceIdRatio,
                sampling_rate: 2.0,
            }),
            ..Default::default()
        };
        assert_eq!(config.effective_state(), EffectiveTelemetryState::Fallback);
    }

    // -----------------------------------------------------------------------
    // Phase 5: Serde round-trip tests for all types
    // -----------------------------------------------------------------------

    #[test]
    fn telemetry_config_serde_round_trip_mixed_capability_flags() {
        let original = TelemetryConfig {
            telemetry_enabled: true,
            tracing_enabled: false,
            metrics_enabled: true,
            correlation_enabled: false,
            propagation_enabled: true,
            sampling: Some(SamplingPolicy::default()),
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
    fn sampling_policy_serde_round_trip() {
        let original = SamplingPolicy {
            policy_type: SamplingPolicyType::TraceIdRatio,
            sampling_rate: 0.42,
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: SamplingPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, original);
    }

    #[test]
    fn sampling_policy_type_all_variants_serde_round_trip() {
        let variants = [
            SamplingPolicyType::AlwaysOn,
            SamplingPolicyType::AlwaysOff,
            SamplingPolicyType::TraceIdRatio,
            SamplingPolicyType::ParentBased,
            SamplingPolicyType::ConsistentProbability,
            SamplingPolicyType::Extension("custom".to_string()),
        ];
        for variant in &variants {
            let json = serde_json::to_string(variant).unwrap();
            let deserialized: SamplingPolicyType = serde_json::from_str(&json).unwrap();
            assert_eq!(
                &deserialized, variant,
                "Round-trip failed for {:?}",
                variant
            );
        }
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

    /// FR-012: Backward-compatible deserialization — legacy JSON → defaults to all flags true → Enabled.
    #[test]
    fn telemetry_config_legacy_json_backward_compat_effective_state() {
        let json = r#"{
            "enabled": true,
            "sampling": {
                "policy_type": "AlwaysOn",
                "sampling_rate": 1.0
            },
            "exporters": null,
            "resources": null,
            "verbosity": null,
            "schema_version": { "version": "1.0.0", "description": null }
        }"#;
        let config: TelemetryConfig = serde_json::from_str(json).unwrap();
        assert!(config.telemetry_enabled);
        assert!(config.tracing_enabled);
        assert!(config.metrics_enabled);
        assert!(config.correlation_enabled);
        assert!(config.propagation_enabled);
        assert_eq!(config.effective_state(), EffectiveTelemetryState::Enabled);
    }
}
