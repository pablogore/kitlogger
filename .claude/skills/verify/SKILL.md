---
name: verify
description: Project-specific runtime verification recipe for the kitlogger workspace (pure Rust library workspace, no binaries).
---

# Verifying kitlogger changes

This workspace has no CLI, server, or GUI — every crate (`kitlogger`, `kitlogger-log-domain`, `console-exporter`, etc.) is a library. There is no `src/bin/` entry point in use (an empty `src/bin/` dir exists at the repo root but is not a workspace member). The runtime surface is the **package boundary**: a consumer program that imports a crate's public exports, exactly as an external user of the crate would.

## Recipe

1. Identify the touched crate and its new/changed public API (read the diff, not just the tests).
2. Create a throwaway example: `crates/<crate>/examples/verify_<thing>.rs`, importing the crate the way an external consumer would (`use kitlogger::KITLogger;`, not `use crate::...` or anything internal).
3. Run it: `cargo run --example verify_<thing> -p <crate>`.
4. Read the actual printed output as your evidence — not the exit code, not whether it compiled.
5. Delete the throwaway example when done (this repo has no `examples/` convention yet — don't leave one-off verification scripts behind unless the user asks for a permanent example).

## Gotchas found so far

- `kit_config` types (`LoggingConfig`, `SamplingConfig`, etc.) live in the sibling repo at `../kit-config` (path dependency). Their `Default` impls are hand-written, not `#[derive(Default)]` — check the actual `impl Default` block before assuming field defaults (e.g. `LoggingConfig::default().enabled == true`, `SamplingConfig::default().enabled == false`).
- `kit_config::ValidationReport`/`ValidationError` derive `Debug` only, no `Display` — format with `{:?}`, not `{}`.
- `LoggingConfig::validate()`'s sub-checks (e.g. `sampling.rate` range) key off the sub-config's own `enabled` flag (`self.sampling.enabled`), not the top-level `LoggingConfig.enabled` — a disabled top-level config does NOT skip sub-validation. Confirmed by direct observation, not just reading the source.
- `ValidationReport` aggregates every violated rule into `domain_errors`, not just the first — confirmed by triggering two violations at once (`sampling.rate` + `retention.days`) and observing both in the same report.

## Example verified so far

- `KITLogger::from_logging_config(LoggingConfig) -> Result<Self, ValidationReport>` (added in `openspec/changes/012-logging-pipeline-consolidation`): valid config constructs; out-of-range `Probabilistic` rate rejects with a specific `domain_errors` entry; boundary values `0.0`/`1.0` accepted (inclusive range); multiple simultaneous violations all appear in one report.
