# Tasks: Telemetry Configuration Semantics

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 280–360 |
| 400-line budget risk | Medium |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | single-pr |
| Chain strategy | size-exception |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: size-exception
400-line budget risk: Medium

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | All work in one PR | PR 1 | ~320 lines; within budget; no split required |

---

## Phase 1: Foundation — New Types + RED Tests

- [x] 1.1 Create `crates/telemetry-config-semantics/src/capability_state.rs` — define `enum CapabilityState { Enabled, Disabled }` with `Clone, Debug, PartialEq, Eq, Serialize, Deserialize` derives.
- [x] 1.2 Create `crates/telemetry-config-semantics/src/effective_state.rs` — define `enum EffectiveTelemetryState { Enabled, Disabled, Partial, Fallback }` with same derives.
- [x] 1.3 Create `crates/telemetry-config-semantics/src/config_error.rs` — define `enum ConfigError { InvalidSamplingRate(f64) }` with manual `std::fmt::Display` and `std::error::Error` impls (no `thiserror`).
- [x] 1.4 Update `crates/telemetry-config-semantics/src/lib.rs` — add `pub mod capability_state; pub mod effective_state; pub mod config_error;` and corresponding `pub use` re-exports.
- [x] 1.5 RED: In `crates/telemetry-config-semantics/tests/config_test.rs`, add failing tests for serde round-trip of `CapabilityState` and `EffectiveTelemetryState` (both variants). Verify `cargo test` compiles but these tests fail or the new imports don't resolve yet.

## Phase 2: TelemetryConfig Migration — Rename + Capability Flags + RED Tests

- [x] 2.1 Update `crates/telemetry-config-semantics/src/telemetry_config.rs` — rename field `enabled` → `telemetry_enabled`, add `#[serde(alias = "enabled")]`.
- [x] 2.2 Same file — add `fn default_true() -> bool { true }` and four new fields `tracing_enabled`, `metrics_enabled`, `correlation_enabled`, `propagation_enabled`, each `bool` with `#[serde(default = "default_true")]`.
- [x] 2.3 Update `impl Default for TelemetryConfig` — replace `enabled: true` with `telemetry_enabled: true`; new flags already default via `default_true`.
- [x] 2.4 Update existing tests in `crates/telemetry-config-semantics/tests/config_test.rs` — replace all `config.enabled` references with `config.telemetry_enabled` and fix the struct-literal in `test_telemetry_config_disabled` accordingly.
- [x] 2.5 RED: Add failing tests for the four new capability fields and for serde backward-compat (payload with `"enabled": true`, no capability keys → all five flags `true`).
- [x] 2.6 Verify `cargo test --workspace` compiles and existing tests pass before proceeding.

## Phase 3: SamplingPolicy Validation — RED then GREEN

- [x] 3.1 RED: In `crates/telemetry-config-semantics/tests/config_test.rs`, add failing `#[test]` cases for `SamplingPolicy::validate()`: `sampling_rate = -0.1` → `Err`, `sampling_rate = 1.5` → `Err`, `sampling_rate = 0.0` → `Ok`, `sampling_rate = 1.0` → `Ok`.
- [x] 3.2 GREEN: In `crates/telemetry-config-semantics/src/sampling_policy.rs`, implement `pub fn validate(&self) -> Result<(), ConfigError>` — return `Err(ConfigError::InvalidSamplingRate(self.sampling_rate))` when `!(0.0..=1.0).contains(&self.sampling_rate)`.
- [x] 3.3 Run `cargo test --workspace` — all validation tests must pass.

## Phase 4: effective_state() — RED then GREEN

- [x] 4.1 RED: Add failing tests in `config_test.rs` for all four `effective_state()` branches: Disabled (`telemetry_enabled=false`, valid sampling), Enabled (all flags `true`, valid sampling), Partial (one flag `false`, valid sampling), Fallback (`sampling_rate=-0.5`, any flags).
- [x] 4.2 RED: Add edge-case test — `telemetry_enabled=false` AND `sampling_rate=-0.5` → `Fallback` (Fallback checked first, FR-007).
- [x] 4.3 RED: Add test for FR-009 — call `effective_state()` three times on unchanged config, assert all three results equal.
- [x] 4.4 RED: Add test for FR-010 — `TelemetryConfig::default().effective_state() == EffectiveTelemetryState::Enabled`.
- [x] 4.5 GREEN: In `crates/telemetry-config-semantics/src/telemetry_config.rs`, implement `pub fn effective_state(&self) -> EffectiveTelemetryState` using the Fallback-first order defined in the design.
- [x] 4.6 Run `cargo test --workspace` — all effective_state tests must pass.

## Phase 5: Serde Round-Trip Tests

- [x] 5.1 Add round-trip tests in `config_test.rs` for `TelemetryConfig` with mixed capability flags (serialize → JSON → deserialize → assert eq original).
- [x] 5.2 Add round-trip tests for `SamplingPolicy`, `SamplingPolicyType`, `ExporterConfig`, `CompressionType`, `ResourceConfig`, `VerbosityPolicy`, `SchemaVersion` (all 8 existing types). Use `serde_json::to_string` / `from_str`.
- [x] 5.3 Add backward-compat deserialization test — legacy JSON with `"enabled": true` and no capability keys → `telemetry_enabled = true`, all four capability flags `true`, `effective_state() == Enabled`.
- [x] 5.4 Run `cargo test --workspace` — all serde round-trip tests must pass.

## Phase 6: KITLogger::with_config Integration — RED then GREEN

- [x] 6.1 Update `crates/kitlogger/Cargo.toml` — add `telemetry-config-semantics = { path = "../telemetry-config-semantics" }` under `[dependencies]`.
- [x] 6.2 RED: Add failing integration test in `crates/kitlogger/` (new file `tests/with_config_test.rs` or inline `#[cfg(test)]`) — assert `KITLogger::with_config(TelemetryConfig::default())` returns an instance and `logger.effective_state() == EffectiveTelemetryState::Enabled`.
- [x] 6.3 RED: Add test asserting that `KITLogger::new()` still constructs successfully and `KITLogger::with_format(LogFormat::Json)` still constructs successfully (FR-011 non-regression).
- [x] 6.4 GREEN: In `crates/kitlogger/src/lib.rs` — add `effective_state: EffectiveTelemetryState` field to `KITLogger` struct. Initialise to `EffectiveTelemetryState::Enabled` in `new()` and `with_format()` (struct literal must stay exhaustive). Implement `pub fn with_config(config: TelemetryConfig) -> Self` that calls `config.effective_state()`, stores it, and wires exporter+formatter same as `new()`.
- [x] 6.5 Implement `pub fn effective_state(&self) -> EffectiveTelemetryState` accessor on `KITLogger` returning `self.effective_state.clone()`.
- [x] 6.6 Run `cargo test --workspace` — all integration tests must pass, existing tests unbroken.

## Phase 7: Cleanup and Verification

- [x] 7.1 Run `cargo clippy --workspace -- -D warnings` — fix any lint warnings (especially `new_without_default` if applicable to `KITLogger`).
- [x] 7.2 Run `cargo fmt --all -- --check` — apply formatting if needed.
- [x] 7.3 Final `cargo test --workspace` — full green across all crates.
- [x] 7.4 Verify no vendor telemetry deps introduced: inspect `Cargo.lock` diff — no `opentelemetry`, `otlp`, `jaeger`, `zipkin`, or `prometheus` entries added.
