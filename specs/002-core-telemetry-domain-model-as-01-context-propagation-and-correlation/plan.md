# Implementation Plan: Context Propagation and Correlation

**Branch**: `main` | **Date**: 2026-06-14 | **Spec**: [Context Propagation and Correlation](spec.md)

**Input**: Feature specification from `specs/002-core-telemetry-domain-model-as-01-context-propagation-and-correlation/spec.md`

**Technology gate**: Every technology named in this plan and its generated
artifacts MUST be declared in this specification's `tech-stack.yaml`. Missing
or undeclared technology is blocking; do not infer a replacement.

## Summary

Define context propagation (Trace Context, Correlation ID, Baggage) and cross-signal correlation across Traces, Metrics, and Logs. The implementation provides W3C Trace Context-compliant propagation, UUID-based correlation identifiers, W3C Baggage propagation, and a carrier abstraction for transport-agnostic context carriage.

## Technical Context

**Language/Version**: Rust (edition 2021)

**Primary Dependencies**: uuid (v7), serde (derive)

**Storage**: N/A (in-memory context model only)

**Testing**: cargo test

**Target Platform**: Cross-platform (Rust library crate)

**Project Type**: Rust library crate (`context-propagation`)

**Performance Goals**: <100µs per inject/extract operation, zero-copy where feasible

**Constraints**: W3C Trace Context spec compliance, thread-safe, no memory leaks

**Scale/Scope**: Library crate providing context types, propagator traits, and carrier abstraction

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

1. **Atomic Specifications** - PASS: Single independently testable feature (context propagation and correlation)
2. **Clear Boundaries** - PASS: Scope/non-scope well-defined; telemetry data model entities explicitly excluded
3. **Dependency Management** - PASS: Depends only on parent capability canonical model
4. **Testability** - PASS: Acceptance criteria defined with testable scenarios
5. **Extensibility** - PASS: Carrier abstraction and Propagator trait support future transport types

**Pre-Design Verdict**: ALL GATES PASS - proceed to Phase 0

## Project Structure

### Documentation (this feature)

```text
specs/002-core-telemetry-domain-model-as-01-context-propagation-and-correlation/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
├── spec.md              # Feature specification
└── tech-stack.yaml      # Technology declarations
```

### Source Code (repository root)

```text
src/
├── lib.rs               # Crate root with re-exports
├── trace_context.rs     # TraceId, SpanId, TraceFlags, TraceState, TraceContext
├── correlation.rs       # CorrelationIdentifier
├── baggage.rs           # Baggage, BaggageEntry, BaggageProperty
├── carrier.rs           # Injector, Extractor, Propagator traits, MapCarrier
├── propagation.rs       # TraceContextPropagator, CorrelationPropagator, BaggagePropagator
├── models/
│   └── mod.rs           # Context, Resource, InstrumentationScope, Span, LogRecord, Metric
├── traits/
│   └── mod.rs           # Logger, Tracer, Meter traits
├── api/
│   ├── mod.rs           # Builders (ContextBuilder, SpanBuilder, etc.)
│   └── validation.rs    # (re-exports from validation.rs)
├── validation.rs        # TelemetryError, validation functions
└── noop/
    └── mod.rs           # No-op implementations

tests/
├── trace_context_test.rs
├── correlation_test.rs
├── baggage_test.rs
└── propagation_test.rs
```

**Structure Decision**: Single Rust library crate (`context-propagation`) with flat module organization for core context types and nested modules for model entities. Tests follow integration test convention in `tests/` directory.

## Complexity Tracking

*No constitution violations - not applicable.*
