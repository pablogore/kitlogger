# Implementation Plan: KIT-005 Logger API

**Branch**: `005-kit-005-logger` | **Date**: 2026-06-10 | **Spec**: [spec.md](./spec.md)

## Summary

Implement the stable, public logging API for KitLogger as a new workspace member crate `kit-logger`. The API defines provider-agnostic abstractions (Logger, LoggerFactory, LoggerContext, ContextLogger, NoopLogger, LoggerError) and convenience macros, reusing LogRecord, LogLevel, and Value from KIT-001. The crate must have zero dependencies on any logging backend, serialization framework, or observability pipeline.

## Technical Context

**Language/Version**: Rust 2024 edition  
**Primary Dependencies**: `kit-config` (reuse types), KIT-001 core types (LogRecord, LogLevel, Value from the existing crate)  
**Storage**: N/A — no persistence; in-memory log routing only  
**Testing**: `cargo test` with unit, integration, property, and compile-fail tests  
**Target Platform**: Linux/macOS (library, no platform-specific code)  
**Project Type**: Library — public API crate for structured logging  
**Performance Goals**: Zero-cost macro expansion when level disabled; minimal overhead for `enabled()` check; no allocations in disabled path  
**Constraints**: Provider-agnostic (zero backend deps); object-safe Logger trait; no dependency on tracing/slog/log/opentelemetry/serde_json  
**Scale/Scope**: Single public API crate; backends are separate crates (future KIT specs)

## Constitution Check

The constitution file is a template with no active gates. No violations to evaluate.

## Architecture Decisions

### ADR-001: New Workspace Crate for Public API

**Decision**: Extract the public logging API into a new workspace member `crates/kit-logger/` rather than mixing it into the existing monolithic crate.

**Rationale**: The provider-agnostic constraint (no backend, serialization, or observability deps) is impossible to enforce in the existing crate which already depends on serde_json, kit-config, and concrete backend code. A separate crate with a minimal dependency set acts as a compile-time contract boundary.

**Alternatives considered**: 
- Adding API modules to existing crate — rejected because Cargo.toml cannot selectively hide dependencies.
- Monorepo with feature flags — adds complexity without full isolation.

### ADR-002: log() Returns Result<(), LoggerError>

**Decision**: `Logger::log()` returns `Result<(), LoggerError>`. 

**Rationale**: Callers who care about failures (e.g., tests verifying flush success) can handle errors; callers who don't can use `let _ = logger.log(...)` or macros. Returning `Result` makes the fallibility explicit without forcing error handling on every invocation. The convenience methods (`info()`, `warn()`, etc.) internally call `log()` and discard the Result, matching the ergonomics of the existing code while providing the type-safe escape hatch.

**Alternatives considered**: Void return (silent discard) — loses error information permanently.

### ADR-003: with_context() Returns Arc<dyn Logger> for Object Safety

**Decision**: `with_context()` returns `Arc<dyn Logger>` (or a concrete ContextLogger that implements `dyn Logger`). This preserves object safety since the return type is a trait object, not `Self`.

**Rationale**: Returning `Self` (builder pattern) would break `dyn Logger`. A trait object return is the standard Rust pattern for object-safe chaining.

**Alternatives considered**: Boxing (`Box<dyn Logger>`) — works but `Arc` is more useful for shared ownership across threads.

### ADR-004: Convenience Methods Are Default Trait Methods

**Decision**: The `Logger` trait provides default implementations of `trace()`, `debug()`, `info()`, `warn()`, and `error()` that call through to `log()`.

**Rationale**: Implementors only need to implement `enabled()`, `log()`, `flush()`, and `with_context()`. The five per-level methods are provided once, reducing boilerplate and ensuring consistent short-circuit behavior.

**Alternatives considered**: Extension trait — works but default methods are more discoverable.

### ADR-005: Macros are a Separate Module, Not Trait Methods

**Decision**: Macros (`log_trace!`, `log_debug!`, etc.) are defined in a `macros.rs` module and exported at crate root. They accept a logger expression and a format-args message.

**Rationale**: Macros cannot be default trait methods (Rust doesn't allow macro definitions in traits). A macro module is the standard Rust pattern (c.f. `log` crate's `log!`, `tracing`'s `info!`).

**Alternatives considered**: Single `log!` macro with level parameter — less ergonomic than per-level macros.

### ADR-006: LoggerContext Uses BTreeMap for Deterministic Ordering

**Decision**: `LoggerContext` stores fields in a `BTreeMap<String, Value>` (where `Value` is from KIT-001).

**Rationale**: Deterministic field ordering is a spec requirement (FR-022). BTreeMap provides natural key ordering without external dependencies.

**Alternatives considered**: Vec of ordered pairs — faster insertion but O(n) lookup; HashMap — non-deterministic ordering.

### ADR-007: ContextLogger Stores Inner as Arc<dyn Logger>

**Decision**: The ContextLogger wrapper holds `inner: Arc<dyn Logger>` and `context: LoggerContext`.

**Rationale**: The inner logger could be any implementation (NoopLogger, backend logger, or another ContextLogger for nested contexts). `Arc<dyn Logger>` supports all cases while maintaining object safety.

## Project Structure

### Documentation (this feature)

```text
specs/005-kit-005-logger/
├── plan.md              # This file
├── research.md          # Research findings
├── data-model.md        # Data model definitions
├── quickstart.md        # Quickstart guide
├── contracts/           # Interface contracts
│   └── logger-api.md
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── spec.md              # Feature specification
```

### Source Code

```text
kitlogger/                          # Workspace root (Cargo workspace)
├── Cargo.toml                      # Workspace manifest
├── crates/
│   └── kit-logger/                 # NEW: Public API crate
│       ├── Cargo.toml              # Minimal dependencies
│       │   Dependencies:
│       │   - kit_config (types only, not backend config)
│       │   - KIT-001 crate (LogRecord, LogLevel, Value)
│       │   - thiserror (LoggerError derive)
│       │   - serde (Value derive only, optional)
│       │   - [NO: serde_json, tracing, slog, log, opentelemetry]
│       └── src/
│           ├── lib.rs              # Crate root, re-exports
│           ├── logger.rs           # Logger trait definition
│           ├── factory.rs          # LoggerFactory trait
│           ├── context.rs          # LoggerContext (immutable builder)
│           ├── context_logger.rs   # ContextLogger wrapper
│           ├── noop.rs             # NoopLogger implementation
│           ├── error.rs            # LoggerError type
│           ├── macros.rs           # log_trace!, log_debug!, etc.
│           └── extension.rs        # Convenience methods as defaults
└── src/                            # Existing backend crate
    ├── lib.rs
    ├── logger.rs                   # Existing concrete Logger struct
    ├── event.rs                    # Existing LogEvent (may be replaced by KIT-001's LogRecord)
    ├── provider.rs                 # LoggerProvider trait
    ├── formatter.rs
    ├── output.rs
    ├── buffering.rs
    ├── sampling.rs
    ├── rotation.rs
    └── redaction.rs
```

## Implementation Phases

### Phase 1: Workspace Setup & Crate Scaffold

Create the workspace root `Cargo.toml` and the new `crates/kit-logger/` crate with minimal dependencies. Define `lib.rs` with module declarations.

**Files to create**: `Cargo.toml` (workspace), `crates/kit-logger/Cargo.toml`, `crates/kit-logger/src/lib.rs`

**Dependencies**: thiserror (for LoggerError derive)

**No dependencies on**: serde_json, tracing, slog, log, opentelemetry, or any backend code

### Phase 2: Core Traits — Logger, LoggerFactory

Define the object-safe `Logger` trait and `LoggerFactory` trait.

**Files**: `crates/kit-logger/src/logger.rs`, `crates/kit-logger/src/factory.rs`

**Logger trait (pseudocode)**:
```rust
pub trait Logger: Send + Sync {
    fn enabled(&self, level: &LogLevel) -> bool;
    fn log(&self, record: &LogRecord) -> Result<(), LoggerError>;
    fn flush(&self) -> Result<(), LoggerError>;
    fn with_context(&self, context: LoggerContext) -> Arc<dyn Logger>;
    // Default methods for trace/debug/info/warn/error
}
```

**LoggerFactory trait**:
```rust
pub trait LoggerFactory: Send + Sync {
    fn create(&self) -> Result<Arc<dyn Logger>, LoggerError>;
}
```

### Phase 3: LoggerContext & ContextLogger

Define the immutable builder context and the wrapper logger.

**Files**: `crates/kit-logger/src/context.rs`, `crates/kit-logger/src/context_logger.rs`

**Context merge algorithm**:
1. ContextLogger receives `log(record)` call
2. Clone the LogRecord
3. For each field in `self.context.fields`:
   - If the record does NOT already have a field with that key, insert it
   - If the record already has a field with that key, skip (per-entry wins)
4. Delegate `log()` to `self.inner`

### Phase 4: NoopLogger & LoggerError

Define the no-op implementation and error type.

**Files**: `crates/kit-logger/src/noop.rs`, `crates/kit-logger/src/error.rs`

**LoggerError variants**:
```rust
#[derive(Error, Debug)]
pub enum LoggerError {
    #[error("Configuration error: {0}")]
    Configuration(String),       // Factory creation failures
    #[error("Backend error: {0}")]
    Backend(Box<dyn Error + Send + Sync>),  // Backend write failures
    #[error("Serialization error: {0}")]
    Serialization(Box<dyn Error + Send + Sync>),  // Format failures
    #[error(transparent)]
    Other(Box<dyn Error + Send + Sync>),  // Extensibility (non-exhaustive)
}
```

### Phase 5: Convenience Methods & Macros

Implement default per-level methods and macros.

**Files**: `crates/kit-logger/src/extension.rs`, `crates/kit-logger/src/macros.rs`

### Phase 6: Testing

Full test suite across all components.

**Test matrix**:
- Unit: each component in isolation
- Integration: end-to-end through LoggerFactory → Logger → mock backend
- Property: LogRecord field ordering determinism, LoggerContext merge determinism
- Compile: `Arc<dyn Logger>` compiles; `Box<dyn Logger>` compiles; macros accept logger expressions

### Phase 7: Documentation & Migration

Rustdoc on all public items. Migration guide from existing `Logger` struct to new `dyn Logger` API.

## Testing Strategy

### Unit Tests

| Module | Test | Verification |
|---|---|---|
| logger.rs | Logger trait default methods produce correct LogRecord levels | Each convenience method emits correct level |
| logger.rs | enabled() short-circuits disabled levels | LogRecord not created when level disabled |
| factory.rs | Factory returns concrete logger | `Arc<dyn Logger>` obtained from factory |
| factory.rs | Factory creation error propagation | Invalid config returns Err |
| context.rs | Builder produces immutable fields | Each `.with()` returns new instance |
| context.rs | Field ordering is deterministic | Same fields inserted in different order produce same iteration |
| context_logger.rs | Context merge adds context fields | Entry carries context fields |
| context_logger.rs | Per-entry field shadows context | Entry field wins over same-key context field |
| context_logger.rs | Original logger unchanged | Entry via original logger has no context fields |
| noop.rs | All methods accepted silently | No panic, no error on enabled/log/flush |
| error.rs | Display for all variants | Each variant produces readable message |
| extension.rs | Error from backend is discarded by convenience method | Convenience method doesn't return Result |
| macros.rs | Zero-cost when disabled | Side-effect probe not evaluated |

### Integration Tests

- End-to-end: `NoopLogger` created via factory, entries emitted through convenience methods, no errors
- Context flow: ContextLogger wraps a mock logger, entries checked for correct field merge
- Factory + Logger: Factory produces logger, entries go through all paths

### Property Tests

- Context merge: For any set of fields, merge with record fields deterministically produces same result regardless of insertion order
- LogRecord creation: `new()` always produces consistent ordering for same field set

### Compile Tests

- `let l: Arc<dyn Logger> = Arc::new(NoopLogger);` compiles
- `let l: Box<dyn Logger> = Box::new(NoopLogger);` compiles
- `let l: Arc<dyn Logger> = factory.create().unwrap();` compiles
- `let l2 = l1.with_context(ctx);` where both are `Arc<dyn Logger>` compiles
- Macros accept `&dyn Logger`, `Arc<dyn Logger>`, and concrete logger references

## Migration Strategy

1. **Add new crate** — The `crates/kit-logger/` crate is additive. Existing code in `src/` continues to work unchanged.
2. **Adapter module** (optional) — A bridge in the existing crate can implement the new `Logger` trait for the existing concrete `Logger` struct, allowing gradual migration.
3. **Deprecate direct use** — Existing convenience methods (`logger.info(...)`) on the concrete struct are deprecated in favor of the trait API.
4. **Remove legacy** — Future KIT spec removes the old concrete `Logger` struct after all consumers migrate.

## Task Breakdown

### Task 1: Create workspace Cargo.toml and kit-logger crate scaffold
**Files**: `Cargo.toml`, `crates/kit-logger/Cargo.toml`, `crates/kit-logger/src/lib.rs`
**Verification**: `cargo build` in workspace compiles; `cargo build -p kit-logger` compiles

### Task 2: Implement LoggerError
**Files**: `crates/kit-logger/src/error.rs`
**Dependencies**: thiserror
**Verification**: Unit tests pass for Display and Debug

### Task 3: Implement Logger trait with default convenience methods
**Files**: `crates/kit-logger/src/logger.rs`
**Dependencies**: LogLevel, LogRecord, Value (KIT-001 types)
**Verification**: Object safety tests compile; default methods work

### Task 4: Implement LoggerFactory trait
**Files**: `crates/kit-logger/src/factory.rs`
**Dependencies**: Logger trait
**Verification**: Factory create returns Arc<dyn Logger>

### Task 5: Implement LoggerContext (immutable builder)
**Files**: `crates/kit-logger/src/context.rs`
**Dependencies**: Value (KIT-001)
**Verification**: Builder tests pass; deterministic ordering confirmed

### Task 6: Implement ContextLogger wrapper
**Files**: `crates/kit-logger/src/context_logger.rs`
**Dependencies**: Logger trait, LoggerContext
**Verification**: Context merge tests pass; shadowing tests pass; original logger isolation confirmed

### Task 7: Implement NoopLogger
**Files**: `crates/kit-logger/src/noop.rs`
**Dependencies**: Logger trait
**Verification**: All methods accepted; no panics; no errors

### Task 8: Implement macros
**Files**: `crates/kit-logger/src/macros.rs`
**Dependencies**: Logger trait, LogLevel, LogRecord (KIT-001)
**Verification**: Zero-cost side-effect test; module_path capture test; all five macros work

### Task 9: Write tests
**Files**: `crates/kit-logger/tests/*.rs`, inline `#[cfg(test)] mod tests`
**Verification**: All unit, integration, property, and compile tests pass

### Task 10: Documentation and final review
**Files**: All `*.rs` with rustdoc
**Verification**: `cargo doc --no-deps` with no warnings; >85% public API documented; examples compile

## Acceptance Validation Plan

| Spec Requirement | Validation Method | Phase |
|---|---|---|
| FR-001: Provider-agnostic interface | Cargo.toml has zero backend deps; compile test for `dyn Logger` | 2 |
| FR-002: enabled() query | Unit test calls enabled before log | 3 |
| FR-003: log() accepts LogRecord | Integration test records and verifies | 3 |
| FR-004: flush() returns Result | Unit test verifies error propagation | 3 |
| FR-005: LoggerFactory | Factory test | 4 |
| FR-006: Factory returns logger or error | Integration test: success and failure paths | 4 |
| FR-007: Factory abstract | Factory trait has no backend types | 4 |
| FR-020: with_context() returns new logger | ContextLogger test | 6 |
| FR-021: Context immutable | Builder test | 5 |
| FR-023: Shadowing | ContextLogger test | 6 |
| FR-024: Convenience methods | Unit test for all five levels | 3 |
| FR-026: Short-circuit | Unit test with disabled level | 3 |
| FR-027: Macros for all levels | Compile test | 8 |
| FR-028: Zero-cost when disabled | Side-effect probe test | 8 |
| FR-031: LoggerError typed variants | Error test | 2 |
| FR-034: NoopLogger | Noop unit tests | 7 |
| FR-036: Object safety | `Arc<dyn Logger>` compile test | 3 |
| SC-009: Object safety compile | Separate compile test file | 10 |

## Out of Scope

- **Backend implementations**: No tracing, slog, OpenTelemetry, or Datadog adapters
- **Async pipeline**: No async-aware logger or await-based flushing
- **File sinks**: No file, stdout, stderr, or network output
- **Log rotation**: No rotation or retention
- **Configuration**: No LoggingConfig parsing or validation (kit-config handles this)
- **Dynamic level changes**: No runtime reconfiguration of logger levels
