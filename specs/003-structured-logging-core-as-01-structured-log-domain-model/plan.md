# Implementation Plan: Structured Log Domain Model

**Branch**: `develop` | **Date**: 2026-06-18 | **Spec**: `003-structured-logging-core-as-01-structured-log-domain-model`

**Input**: Feature specification from `specs/003-structured-logging-core-as-01-structured-log-domain-model/spec.md`

**Note**: This template is filled in by the plan command.

**Technology gate**: Every technology named in this plan and its generated
artifacts MUST be declared in this specification's `tech-stack.yaml`. Missing
or undeclared technology is blocking; do not infer a replacement.

## Summary

Define the canonical structured logging domain model — LogRecord, Severity, LogAttribute, LogAttributeValue, CorrelationId, TraceId, SpanId — with construction-time validation, immutability, and attribute naming constraints. This is the foundational data layer for all KitLogger logging.

## Technical Context

**Language/Version**: Rust (stable)

**Primary Dependencies**: None (pure domain model — no runtime, no async, no external crates required for core entities)

**Storage**: N/A

**Testing**: cargo test

**Target Platform**: Library crate within KitLogger workspace

**Project Type**: Canonical domain model (pure data types + validation)

**Performance Goals**: Minimal allocation overhead for LogRecord construction; no heap allocation for attribute values where possible

**Constraints**: Compile on stable Rust; no unsafe code; no runtime dependencies; no async dependencies; no proc macros unless strictly necessary

**Scale/Scope**: Foundational crate consumed by all KitLogger logging components (AS-02 through AS-05)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| Atomic Specification | PASS | Single responsibility: LogRecord entity and validation rules only |
| Clear Boundaries | PASS | Scope/non-scope well-defined; explicitly excludes Logger, LogContext, serialization, configuration |
| Dependency Management | PASS | Depends only on KIT-002 (shared domain primitives) |
| Testability | PASS | Acceptance criteria define measurable outcomes; scenarios cover construction, validation, and immutability |
| Extensibility | PASS | Immutable record; attribute system supports future value types without breaking changes |

## Project Structure

### Documentation (this feature)

```text
specs/003-structured-logging-core-as-01-structured-log-domain-model/
├── plan.md              # This file
├── tech-stack.yaml      # Technology declarations
├── spec.md              # Feature specification
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
crates/kitlogger-log-domain/
├── src/
│   ├── lib.rs
│   ├── log_record.rs
│   ├── severity.rs
│   ├── log_attribute.rs
│   ├── log_attribute_value.rs
│   ├── correlation_id.rs
│   ├── trace_id.rs
│   ├── span_id.rs
│   └── validation.rs
└── tests/
    ├── log_record_tests.rs
    ├── severity_tests.rs
    └── attribute_tests.rs
```

**Structure Decision**: Single library crate (`kitlogger-log-domain`) in a workspace. Pure domain model with no runtime dependencies, matching the "runtime: none" declaration.

## Complexity Tracking

No constitution violations.
