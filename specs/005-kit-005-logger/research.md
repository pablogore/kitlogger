# Research: KIT-005 Logger API

## 1. Object-Safe Logger Design Patterns

**Decision**: Use a trait with only `&self` methods, no generic parameters, no associated types, and return `Arc<dyn Logger>` from `with_context()` rather than `Self`.

**Rationale**: Rust's object safety rules require that all trait methods are dispatchable through a vtable. Generic methods and `Self` return types violate this. The `log` crate's `Log` trait is the canonical example of an object-safe logger trait.

**Alternatives considered**:
- Enum dispatch (static dispatch via enum) — faster but requires all implementations known at compile time; defeats provider-agnostic goal.
- `Box<dyn Logger>` return — equivalent but `Arc` supports shared ownership across threads without cloning.

## 2. LoggerContext Immutable Builder Pattern

**Decision**: Use consuming builder pattern (`.with()` takes `self`, returns `Self`).
When storing for a ContextLogger, the `LoggerContext` is consumed once.

**Rationale**: Consuming builder is idiomatic Rust for immutable builders. Each `.with()` creates a new allocation for the new `BTreeMap` entry, but since context fields are typically set once at logger creation time, the one-time allocation cost is acceptable.

**Reference**: Standard Rust builder pattern (c.f. `std::process::Command`, `reqwest::Client::builder()`).

## 3. Macro Zero-Cost Semantics

**Decision**: Use `if enabled() { log() }` guard inside each macro, matching the approach used by the `log` crate.

**Rationale**: The compiler optimizes dead code inside `if false` blocks. When `enabled()` returns false at runtime, the `log()` call (and its argument expressions) are never evaluated. For compile-time dead code elimination, the compiler can inline `enabled()` and eliminate the branch entirely.

**References**: `log` crate's `log!` macro, `tracing` crate's `event!` macro.

## 4. Error Model

**Decision**: Use `thiserror` derive for LoggerError. Box errors for Backend and Serialization variants to keep the enum small (pointer-sized).

**Rationale**: `thiserror` is the standard for library error types. Boxing variant data keeps the enum at one pointer width, which is important for `Result<(), LoggerError>` used in hot paths.

**Alternatives considered**:
- `snafu` — more context but heavier; thiserror is simpler.
- Custom `Display` impl — more boilerplate with no benefit.

## 5. Crate Isolation Strategy

**Decision**: The `kit-logger` crate explicitly depends only on:
- `kit-config` (for LogLevel and Value type compatibility)
- `kit-core` or equivalent KIT-001 crate (for LogRecord, LogLevel, Value)
- `thiserror` (for derive)

It explicitly does NOT depend on: `serde_json`, `serde`, `tokio`, `tracing`, `slog`, `log`, `opentelemetry`.

**Rationale**: The dependency tree is the only enforceable contract boundary in Rust. If `serde_json` appears in `crates/kit-logger/Cargo.toml`, nothing prevents a backend author from using it in the public API accidentally. Excluding it at the manifest level makes the provider-agnostic constraint mechanically enforced.

## 6. Relationship with Existing Logger Struct

**Decision**: The existing `src/logger.rs` concrete `Logger` struct remains in the workspace root crate. The new API types live in `crates/kit-logger/`. An adapter can be provided later to wrap the concrete Logger in the new trait.

**Rationale**: The existing code has `LogEvent` (HashMap + serde_json::Value) while KIT-005 uses `LogRecord` (BTreeMap + KIT-001 Value). These are different types. Migration requires KIT-001 to define LogRecord first, then an adapter converts between them. This is deferred to a future integration phase.

## 7. LogRecord vs LogEvent Compatibility

**Decision**: KIT-005's `Logger::log()` accepts `&LogRecord` (KIT-001 type). The existing `LogEvent` in `src/event.rs` is not affected. Adapters will be defined when KIT-001 provides the concrete `LogRecord` implementation.

**Rationale**: The spec mandates reuse of KIT-001 types. Until KIT-001 provides them, the trait is written against those types but implementations remain pending.
