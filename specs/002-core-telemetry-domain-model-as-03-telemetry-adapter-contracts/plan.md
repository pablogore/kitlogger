# Implementation Plan: Telemetry Adapter Contracts

**Branch**: `main` | **Date**: 2026-06-17 | **Spec**: [Telemetry Adapter Contracts](spec.md)

**Input**: Feature specification from `specs/002-core-telemetry-domain-model-as-03-telemetry-adapter-contracts/spec.md`

**Technology gate**: Every technology named in this plan and its generated
artifacts MUST be declared in this specification's `tech-stack.yaml`. Missing
or undeclared technology is blocking; do not infer a replacement.

## Summary

Define the canonical adapter contract for telemetry provider abstraction. AS-03 owns ProviderAdapter and ExporterAdapter traits sharing three common bases: CommonAdapterBase (identity/health), LifecycleAdapter (flush/shutdown), TelemetryDelivery (deliver). AdapterRegistry with frozen-after-init semantics and thread-safe `Arc<dyn Adapter>` lookup. AdapterLifecycle state machine with explicit transition matrix (including startup failure transitions). Bidirectional entity-specific mapping contracts (Trace, Span, Metric, LogRecord, Resource). HealthReport struct (AdapterHealth + reason + timestamp). AdapterResult/AdapterError canonical failure model with typed lifecycle error hierarchy. All traits MUST be object-safe.

## Technical Context

**Language/Version**: Rust (edition 2021)

**Primary Dependencies**: serde (derive), async-trait, context-propagation (AS-01 for canonical domain types)

**Storage**: N/A (contract definitions only)

**Testing**: cargo test

**Target Platform**: Cross-platform (Rust library crate)

**Project Type**: Rust library crate providing adapter contract types and traits

**Performance Goals**: N/A — contract definitions only; concrete adapter implementations own performance targets

**Constraints**:
- No concrete adapter implementations; contracts only
- All adapter traits MUST be object-safe (registry uses `dyn Adapter`)
- Explicit lifecycle transition matrix with typed transition errors; Registered→Shutdown and Initialized→Shutdown allowed for startup failure
- LifecycleAdapter trait separate from CommonAdapterBase
- TelemetryDelivery trait separate with `&self` signature for Arc compatibility
- Registry stores `Arc<dyn Adapter>` after freeze; registration only during bootstrap phase
- Registry supports both ProviderAdapter and ExporterAdapter through common Adapter supertrait
- HealthReport struct (AdapterHealth + String reason + SystemTime timestamp) returned by health()
- Stopped retains resources; Shutdown releases resources and is terminal
- shutdown() implicitly invokes flush() before Stopped transition
- Bidirectional mapping contracts (Canonical ↔ OpenTelemetry)
- Adapter multiplexing with best-effort and aggregate failures

**Scale/Scope**: Library crate providing ProviderAdapter, ExporterAdapter, CommonAdapterBase, LifecycleAdapter, TelemetryDelivery, Adapter supertrait, AdapterRegistry, AdapterLifecycle, AdapterId, AdapterHealth, HealthReport, AdapterResult/AdapterError, and five mapping contract traits.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

1. **Atomic Specifications** - PASS: Single independently testable feature (telemetry adapter contracts)
2. **Clear Boundaries** - PASS: Scope/non-scope well-defined; concrete adapter implementations explicitly excluded as separate specs
3. **Dependency Management** - PASS: Depends only on parent capability canonical model; no circular dependencies
4. **Testability** - PASS: Success criteria defined with testable scenarios using mocks
5. **Extensibility** - PASS: New adapter implementations are always separate specs; mapping contracts are trait-based

**Pre-Design Verdict**: ALL GATES PASS - proceed to Phase 0

**Post-Design Verdict**: ALL GATES PASS — design artifacts consistent with constitution

## Project Structure

### Documentation (this feature)

```text
specs/002-core-telemetry-domain-model-as-03-telemetry-adapter-contracts/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
├── checklists/          # Phase 2 output
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
crates/telemetry-adapter-contracts/
├── Cargo.toml
├── src/
│   ├── lib.rs               # Crate root with re-exports
│   ├── adapter.rs           # CommonAdapterBase, LifecycleAdapter, TelemetryDelivery, Adapter supertrait, ProviderAdapter, ExporterAdapter
│   ├── registry.rs          # AdapterRegistry
│   ├── lifecycle.rs         # AdapterLifecycle state machine + transition matrix
│   ├── health.rs            # AdapterHealth status model
│   ├── error.rs             # AdapterResult, AdapterError, lifecycle error hierarchy
│   ├── mapping.rs           # Mapping contracts (TraceMappingContract, etc.)
│   └── id.rs                # AdapterId
└── tests/
    ├── adapter_test.rs      # Adapter contract validation tests
    ├── registry_test.rs     # Registry behavior tests
    ├── lifecycle_test.rs    # Lifecycle transition tests
    └── integration_tests.rs # End-to-end scenarios
```

**Structure Decision**: Single Rust library crate with flat module organization. Adapter traits, registry, lifecycle, health, error types, mapping contracts, and identity type each in dedicated modules. Tests validate abstract contracts via mocks.

## Complexity Tracking

*No constitution violations - not applicable.*
