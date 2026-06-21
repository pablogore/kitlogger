# Design: Telemetry Configuration Semantics

## Technical Approach

Give the data-only `telemetry-config-semantics` crate behavioral meaning via a pure, deterministic
`effective_state()` evaluation plus `[0.0, 1.0]` sampling validation, and add an additive
`KITLogger::with_config` constructor that evaluates and stores the effective state without changing
runtime behavior. All new code is additive and technology-agnostic (no OpenTelemetry, no exporters).

### Reconciliation note (BLOCKING-LEVEL, resolved in design)

The brief's "confirmed struct" does not match the actual crate. Real fields today: `enabled`,
`sampling: Option<SamplingPolicy>`, `exporters: Option<Vec<ExporterConfig>>`, `resources: Option<ResourceConfig>`,
`verbosity: Option<VerbosityPolicy>`, `schema_version`. The crate depends only on `serde` (no `thiserror`).
This design follows the ACTUAL code and the SDD rule "follow existing patterns": rename `enabled`→`telemetry_enabled`
(with serde alias), add the four capability flags, and keep the existing optional container fields untouched.
The `Eq` derive requested for new enums is fine; `TelemetryConfig` keeps `PartialEq` only (it holds `f64`).

## Architecture Decisions

### Decision: Explicit capability flags vs. derived state
| Option | Tradeoff | Decision |
|--------|----------|----------|
| Explicit `tracing/metrics/correlation/propagation_enabled: bool` | More fields, but `Partial` is expressible and serde-stable | **Chosen** |
| Derive capability posture from exporters/sampling | No new fields, but `Partial` becomes ambiguous and couples to vendor config | Rejected |

Rationale: `Partial` requires a per-capability signal independent of exporters. Explicit binary flags keep the
model deterministic and decoupled from any vendor.

### Decision: Fallback priority order (validation first)
| Option | Tradeoff | Decision |
|--------|----------|----------|
| Check validation (`Fallback`) FIRST | An invalid config can never masquerade as `Disabled`/`Enabled`/`Partial` | **Chosen** |
| Check `telemetry_enabled` first | A disabled-but-invalid config hides the validation error | Rejected |

Rationale: invalid sampling is a config defect that must always surface, regardless of enablement.

### Decision: serde backward compatibility
| Option | Tradeoff | Decision |
|--------|----------|----------|
| `#[serde(alias = "enabled")]` on `telemetry_enabled` + `#[serde(default = "default_true")]` on new flags | Old `{"enabled": true}` payloads still deserialize; new flags optional | **Chosen** |
| Hard rename, break old payloads | Simpler, but breaks serialized configs | Rejected |

### Decision: error type — manual enum vs. thiserror
| Option | Tradeoff | Decision |
|--------|----------|----------|
| Hand-written `enum ConfigError` + manual `Display`/`Error` impls | Zero new deps; crate currently has only `serde` | **Chosen** |
| Add `thiserror` | Less boilerplate, but introduces a dependency the crate avoids today | Rejected |

### Decision: module layout
New types live in dedicated modules mirroring the existing one-type-per-file convention
(`capability_state.rs`, `effective_state.rs`, `config_error.rs`), re-exported from `lib.rs`.
`effective_state()`/`validate()` are `impl` blocks on existing types in their existing files.

## Data Flow

    TelemetryConfig ──effective_state()──► EffectiveTelemetryState
          │                                      ▲
          └── sampling.validate() ──Err──────────┘ (Fallback, checked FIRST)

    KITLogger::with_config(config) ──► config.effective_state() ──► stored EffectiveTelemetryState
                                                                     (no runtime behavior change)

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `crates/telemetry-config-semantics/src/telemetry_config.rs` | Modify | Rename `enabled`→`telemetry_enabled` (serde alias); add 4 capability flags with `default_true`; add `effective_state()` |
| `crates/telemetry-config-semantics/src/sampling_policy.rs` | Modify | Add `validate(&self) -> Result<(), ConfigError>` (range `[0.0,1.0]`) |
| `crates/telemetry-config-semantics/src/capability_state.rs` | Create | `enum CapabilityState { Enabled, Disabled }` |
| `crates/telemetry-config-semantics/src/effective_state.rs` | Create | `enum EffectiveTelemetryState { Enabled, Disabled, Partial, Fallback }` |
| `crates/telemetry-config-semantics/src/config_error.rs` | Create | `enum ConfigError { InvalidSamplingRate(f64) }` + manual `Display`/`Error` |
| `crates/telemetry-config-semantics/src/lib.rs` | Modify | `pub mod` + `pub use` for the 3 new modules |
| `crates/telemetry-config-semantics/tests/config_test.rs` | Modify | Serde round-trip (all 8 types) + 4 effective-state cases + serde alias back-compat |
| `crates/kitlogger/src/lib.rs` | Modify | Add `with_config`; store `effective_state: EffectiveTelemetryState` field |
| `crates/kitlogger/Cargo.toml` | Modify | Add `telemetry-config-semantics = { path = "../telemetry-config-semantics" }` |

## Interfaces / Contracts

```rust
// capability_state.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityState { Enabled, Disabled }

// effective_state.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectiveTelemetryState { Enabled, Disabled, Partial, Fallback }

// config_error.rs — no thiserror; manual impls
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError { InvalidSamplingRate(f64) }
// impl std::fmt::Display + impl std::error::Error for ConfigError

// telemetry_config.rs
fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelemetryConfig {
    #[serde(alias = "enabled")]
    pub telemetry_enabled: bool,
    #[serde(default = "default_true")] pub tracing_enabled: bool,
    #[serde(default = "default_true")] pub metrics_enabled: bool,
    #[serde(default = "default_true")] pub correlation_enabled: bool,
    #[serde(default = "default_true")] pub propagation_enabled: bool,
    pub sampling: Option<SamplingPolicy>,
    pub exporters: Option<Vec<ExporterConfig>>,
    pub resources: Option<ResourceConfig>,
    pub verbosity: Option<VerbosityPolicy>,
    pub schema_version: SchemaVersion,
}

impl TelemetryConfig {
    pub fn effective_state(&self) -> EffectiveTelemetryState {
        // Fallback FIRST: validate sampling only if present.
        if let Some(s) = &self.sampling {
            if s.validate().is_err() { return EffectiveTelemetryState::Fallback; }
        }
        if !self.telemetry_enabled { return EffectiveTelemetryState::Disabled; }
        let all = self.tracing_enabled && self.metrics_enabled
            && self.correlation_enabled && self.propagation_enabled;
        if all { EffectiveTelemetryState::Enabled } else { EffectiveTelemetryState::Partial }
    }
}

// sampling_policy.rs
impl SamplingPolicy {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !(0.0..=1.0).contains(&self.sampling_rate) {
            return Err(ConfigError::InvalidSamplingRate(self.sampling_rate));
        }
        Ok(())
    }
}

// kitlogger/src/lib.rs — additive, stores state, no behavior change
pub fn with_config(config: TelemetryConfig) -> Self { /* new()-equivalent wiring + store effective_state */ }
pub fn effective_state(&self) -> EffectiveTelemetryState { self.effective_state.clone() }
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|--------------|----------|
| Unit | `validate()` range guard, all 4 `effective_state` branches incl. Fallback-first | Table-driven asserts in crate tests |
| Integration | serde round-trip for all 8 types; `{"enabled":true}` alias deserializes; missing capability flags default true | `serde_json` round-trip in `config_test.rs` |
| Integration | `KITLogger::with_config` compiles, stores state, leaves `new()`/`with_format()` untouched | kitlogger crate test |

Note: `tests/config_test.rs` currently asserts `config.enabled` — those assertions MUST be updated to
`config.telemetry_enabled` as part of this change (rename touches existing tests).

## Migration / Rollout

No data migration. serde alias keeps old payloads deserializing. Rollback = revert the commit (purely additive).

## Open Questions

- [ ] None blocking. Confirmed-struct vs. actual-code mismatch resolved in favor of actual code (see Reconciliation note); `sampling` stays `Option<>`, so `effective_state` guards `None`.
