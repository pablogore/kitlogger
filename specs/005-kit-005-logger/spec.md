# Feature Specification: KIT-005 Logger API

**Feature Branch**: `005-kit-005-logger`
**Created**: 2026-06-10
**Status**: Draft
**Input**: Implement KIT-005 Logger API following the project constitution and architecture principles.

## Overview

This feature defines the stable, public logging API for KitLogger. KIT-001 established the foundational observability data model including **LogRecord**, **LogLevel**, and **Value** — KIT-005 reuses those types as the canonical source of truth. This feature introduces only:

- **Logger** — the primary logging interface
- **LoggerFactory** — logger construction
- **LoggerContext** — contextual metadata via `with_context()`, returning a new logger instance
- **NoopLogger** — default no-op implementation
- **LoggerError** — typed error handling
- **Convenience methods** — per-level methods (`trace`, `debug`, `info`, `warn`, `error`)
- **Logging macros** — compile-time ergonomic logging

The API must be provider-agnostic — it must not depend on any specific logging backend, serialization framework, or observability pipeline.

The Logger API establishes a clean separation between the logging contract (interfaces and data structures) and the mechanics of writing, formatting, buffering, and transporting log data. Application code imports this API only; backends and providers implement it.

## Clarifications

### Session 2026-06-10

- Q: Should KIT-005 reuse the existing KIT-001 observability model as the canonical source of LogRecord, LogLevel, and Value, or should it introduce independent API-specific versions? → A: Reuse KIT-001 types. KIT-005 MUST NOT define its own LogRecord, LogLevel, or Value. It introduces only Logger, LoggerFactory, LoggerContext, NoopLogger, LoggerError, convenience methods, and macros.
- Q: Should LoggerContext be implemented as a standalone context object attached at call time, or as a context-aware logger wrapper returning a new logger instance? → A: Context-aware logger wrapper. `logger.with_context(ctx)` returns a new logger instance carrying the context; subsequent `logger.info(...)` calls implicitly include the context fields.
- Q: How should contextual loggers be represented internally — as a wrapper layer or by storing context inside every logger implementation? → A: Context Logger Wrapper. `with_context()` returns a `ContextLogger` that wraps the original logger. The wrapper merges context fields into each log entry before delegating to the inner logger. No backend changes required; context logic lives in one place.
- Q: Should Logger be required to be object-safe? → A: Yes, Logger MUST be object-safe. `Arc<dyn Logger>` and `Box<dyn Logger>` must compile. The trait MUST NOT require generic methods, associated generic types, or any construct that prevents dynamic dispatch.

## User Scenarios & Testing _(mandatory)_

### User Story 1 — Emit Structured Log Entries Through a Stable Interface (Priority: P1)

As a developer using KitLogger, I want to emit structured log entries (messages with severity levels and key-value fields) through a stable interface that does not change when the logging backend changes, so that my application code is decoupled from infrastructure details.

**Why this priority**: The core logging contract is the primary value this feature delivers. Without it, all application code is coupled to implementation details.

**Independent Test**: A test writes a log entry at each severity level (trace, debug, info, warn, error) with a message and structured fields, and verifies the entry is recorded through the public API without coupling to any specific backend.

**Acceptance Scenarios**:

1. **Given** a logger instance obtained through the public API, **When** a log entry with message "hello world" and severity Info is emitted, **Then** the entry is accepted and processed without error.
2. **Given** a logger instance, **When** log entries are emitted at each severity level (trace, debug, info, warn, error), **Then** each entry carries the correct severity classification.
3. **Given** a logger instance, **When** a log entry is emitted with structured fields (e.g., `user_id`, `duration_ms`, `success`), **Then** all fields are preserved verbatim in the recorded entry.
4. **Given** a logger instance configured with a minimum severity level, **When** an entry below that threshold is emitted, **Then** the entry is silently discarded (not recorded).

---

### User Story 2 — Configure and Create Logger Instances (Priority: P1)

As an application developer or platform engineer, I want to create logger instances through a factory mechanism, so that the construction details (backend selection, configuration, initialization) are separated from consumption.

**Why this priority**: Separating construction from consumption enables backend swapping without application code changes and supports dependency injection patterns.

**Independent Test**: A test creates a logger instance via the factory API, verifies it returns a valid logger, and confirms the logger can emit entries — without the caller knowing or specifying which backend is in use.

**Acceptance Scenarios**:

1. **Given** a logger factory, **When** `create()` is called, **Then** a valid logger instance is returned.
2. **Given** a logger factory configured with a specific backend, **When** a logger is created and an entry is emitted, **Then** the entry reaches the configured backend.
3. **Given** a logger factory, **When** multiple logger instances are created, **Then** each is independent and can be used concurrently from different threads or tasks.

---

### User Story 3 — Determine Log Severity with Ordering and Parsing (Priority: P2)

As a developer or operator, I want to compare, order, parse, and display log severity levels so that I can filter, sort, and configure logging behaviour programmatically and through configuration files.

**Why this priority**: Log level manipulation is a cross-cutting concern. Without proper semantics, filtering and configuration become fragile.

**Independent Test**: A test verifies that severity levels have a total ordering (Trace < Debug < Info < Warn < Error), can be parsed from their string representations case-insensitively, and display correctly.

**Acceptance Scenarios**:

1. **Given** two severity levels, **When** compared, **Then** the ordering follows Trace < Debug < Info < Warn < Error.
2. **Given** a string `"trace"`, `"TRACE"`, or `"Trace"`, **When** parsed as a severity level, **Then** the result is `Trace`.
3. **Given** a severity level `Info`, **When** displayed, **Then** the output is `"Info"`.
4. **Given** a severity level, **When** serialized and deserialized, **Then** the roundtrip preserves the original value.

---

### User Story 4 — Attach Contextual Metadata to Log Entries via Logger Wrapper (Priority: P2)

As a developer debugging a multi-tenant or multi-request system, I want to attach contextual metadata (tenant ID, request ID, correlation ID) by wrapping a logger with context via `with_context()`, so that all downstream log entries automatically carry the context without requiring manual field injection on each call.

**Why this priority**: Contextual logging is essential for operability in distributed and multi-tenant systems. A wrapper pattern (`logger.with_context(ctx).info(...)`) provides immutability and prevents accidental state mutation across threads.

**Independent Test**: A test creates a context with two fields, obtains a new logger via `with_context()`, emits three log entries through the wrapped logger, and verifies all three entries carry the context fields in addition to any per-entry fields.

**Acceptance Scenarios**:

1. **Given** a context with fields `{tenant_id: "acme", request_id: "req-123"}`, **When** `logger.with_context(ctx).info("hello")` is called, **Then** the emitted entry carries both context fields.
2. **Given** a context with fields and a log entry with an overlapping field key, **When** the entry is emitted through the wrapped logger, **Then** the per-entry field value takes precedence (context field is shadowed).
3. **Given** a context attached to a logger via `with_context()`, **When** multiple log entries are emitted through the wrapped logger, **Then** each entry carries the context fields independently (no mutation between entries).
4. **Given** a context built with no fields, **When** attached via `with_context()` and an entry is emitted, **Then** the entry has no additional fields beyond what was explicitly provided.
5. **Given** an original logger and a wrapped logger (`let wrapped = logger.with_context(ctx)`), **When** an entry is emitted through the original logger, **Then** the entry does NOT carry the context fields (original logger is unchanged).

---

### User Story 5 — Ergonomic Per-Level Logging (Priority: P2)

As a developer writing instrumentation code, I want to call `logger.info(...)`, `logger.error(...)`, etc., without manually specifying the severity level on each call, so that code is more readable and less error-prone.

**Why this priority**: Per-level convenience methods are the primary developer interaction point. They reduce boilerplate and prevent mismatched levels.

**Independent Test**: A test calls each per-level method (trace, debug, info, warn, error), and verifies that each emitted entry carries the correct corresponding severity level.

**Acceptance Scenarios**:

1. **Given** a logger instance, **When** `logger.info("message")` is called, **Then** the emitted log entry has severity Info and message "message".
2. **Given** a logger instance, **When** each per-level method is called (trace, debug, info, warn, error), **Then** the emitted entry carries the corresponding severity level.
3. **Given** a logger where `enabled(Info)` returns false, **When** `logger.info("message")` is called, **Then** no log entry is recorded (short-circuit behaviour).

---

### User Story 6 — Compile-Time Efficient Logging Macros (Priority: P3)

As a developer writing high-performance or hot-path code, I want to use logging macros (`log_info!`, `log_error!`, etc.) that evaluate to zero cost when the log level is disabled at compile time or runtime, so that verbose debug logging does not impact production performance.

**Why this priority**: Macros provide the most ergonomic and performant logging surface. However, the convenience API (User Story 5) covers the common case, so macros are lower priority.

**Independent Test**: A test verifies that a macro invocation at a disabled level does not evaluate its message expression (proven by wrapping the expression in a side-effect function and confirming the side effect does not fire).

**Acceptance Scenarios**:

1. **Given** a macro invocation at a disabled log level, **When** the macro is expanded and the level check evaluates to false, **Then** the message argument is not evaluated (zero-cost when disabled).
2. **Given** a macro invocation `log_info!(logger, "hello {}", name)`, **When** expanded, **Then** the emitted entry carries severity Info and the formatted message.
3. **Given** a macro invocation within a module, **When** expanded, **Then** the emitted entry carries the module's path as the target (compile-time target capture).

---

### User Story 7 — Handle Logging Errors Gracefully (Priority: P3)

As a developer integrating logging into a production system, I want the logging API to report failures (backed-up buffers, write errors, configuration errors) through a typed error mechanism, so that I can decide how to handle failures without crash or silent data loss.

**Why this priority**: Error handling is important for production reliability, but the common case (successful log emission) does not require explicit error handling. Factory creation errors are more critical than per-entry errors.

**Independent Test**: A test simulates a configuration error during logger construction and a write failure during emission, and verifies both produce distinguishable, typed errors.

**Acceptance Scenarios**:

1. **Given** an invalid configuration, **When** a logger factory attempts to create a logger, **Then** an error describing the configuration failure is returned.
2. **Given** a backend that fails to write, **When** a log entry is emitted, **Then** the failure is reported via the error type without panicking.
3. **Given** multiple distinct failure modes (configuration, backend, serialization), **When** each occurs, **Then** the error type distinguishes between them.

### Edge Cases

- **Concurrent logger access**: Multiple threads calling `log()` on the same logger concurrently must not corrupt state or panic.
- **Empty message**: An empty string message must be accepted and recorded without error (not treated as a special case or failure).
- **Large field values**: Very large field values (multiline strings, large payloads) must be accepted without truncation at the API level (truncation, if any, is a transport concern).
- **High-cardinality fields**: The API must not reject fields with duplicate keys; the last writer wins for field values on the same entry.
- **Disabled logger**: When all logging is disabled, all methods (including `enabled()`, `log()`, and `flush()`) must complete without error and without observable side effects.
- **No-op logger**: A default no-op logger must be available that accepts all API calls and silently discards all data — safe for production use in tests or when no backend is configured.
- **Logger factory with no backend**: Creating a logger through the factory without a configured backend must return a no-op logger, not an error.

## Requirements _(mandatory)_

### Functional Requirements

#### Logger Interface

- **FR-001**: The system MUST provide a stable, provider-agnostic interface for emitting log entries. This interface must be safe to use from multiple concurrent threads without data corruption and must not expose any backend-specific types or methods.
- **FR-002**: The interface MUST support querying whether a given severity level is enabled, allowing callers to avoid expensive work for disabled log levels.
- **FR-003**: The interface MUST accept a structured log entry and record it through whatever backend is configured.
- **FR-004**: The interface MUST support flushing buffered or pending log entries, with failures reported through the error type.

#### Logger Factory

- **FR-005**: The system MUST provide a factory mechanism for constructing logger instances. The factory must be safe to use from multiple concurrent threads.
- **FR-006**: Requesting a logger from a factory MUST return a valid logger instance, or an error if construction fails.
- **FR-007**: The factory interface MUST remain abstract — it must not hardcode which backend, provider, or configuration mechanism is used.

#### Log Severity Levels (from KIT-001)

KIT-005 reuses the `LogLevel` type defined by KIT-001. The following requirements are satisfied by that type and are listed here for reference:

- **FR-008**: LogLevel MUST define five severity levels with total ordering: Trace, Debug, Info, Warn, Error (from lowest to highest severity).
- **FR-009**: LogLevel MUST support equality and ordering comparisons (e.g., Trace < Debug < Info < Warn < Error).
- **FR-010**: LogLevel MUST support serialization and deserialization without data loss (roundtrip stable).
- **FR-011**: LogLevel MUST support display formatting (human-readable output).
- **FR-012**: LogLevel MUST support case-insensitive parsing from text input (e.g., "trace", "TRACE", "Trace" all parse to Trace).
- **FR-013**: When parsing an unrecognized severity string, LogLevel parsing MUST fail with a clear error rather than defaulting to a level.

#### Structured Log Records (from KIT-001)

KIT-005 reuses the `LogRecord` type defined by KIT-001. The following requirements are satisfied by that type and are listed here for reference:

- **FR-014**: LogRecord MUST be immutable once created — all fields are set at construction and cannot be modified after.
- **FR-015**: LogRecord MUST carry: severity level, target (source component identifier), message text, and a timestamp.
- **FR-016**: LogRecord MUST carry structured fields as key-value pairs with deterministic ordering (e.g., sorted by key).

KIT-005 additionally reuses the `Value` type defined by KIT-001 as the field value type carried by LogRecord:

- **FR-017**: Field values MUST support at minimum: strings, booleans, signed and unsigned integers, and floating-point numbers.
- **FR-018**: The field value types MUST be self-contained — they must not require external serialization frameworks to construct or inspect.
- **FR-019**: Field values MUST support duplication, serialization, deserialization, and display.

#### Contextual Logging

- **FR-020**: The Logger interface MUST provide a `with_context()` method that accepts a LoggerContext and returns a new logger instance carrying the context. The original logger MUST remain unchanged.
- **FR-021**: LoggerContext MUST be immutable — each operation (adding a field) MUST produce a new context without modifying the original, enabling safe sharing across threads.
- **FR-022**: LoggerContext fields MUST have deterministic ordering (e.g., sorted by key).
- **FR-023**: When a log entry is emitted through a context-wrapped logger and carries a field with the same key as a context field, the per-entry field value MUST take precedence (context field is shadowed for that entry only).

#### Per-Level Convenience Methods

- **FR-024**: The Logger interface MUST provide convenience methods for each severity level: `trace(…)`, `debug(…)`, `info(…)`, `warn(…)`, `error(…)`.
- **FR-025**: Each convenience method MUST internally build a LogRecord with the correct severity level and pass it to the `log()` method.
- **FR-026**: Each convenience method MUST check if the level is enabled before constructing the log record (short-circuit when disabled).

#### Logging Macros

- **FR-027**: The system MUST provide macros for each severity level: `log_trace!`, `log_debug!`, `log_info!`, `log_warn!`, `log_error!`.
- **FR-028**: Macros MUST evaluate to zero cost when the corresponding level is disabled — the message expression must not be evaluated.
- **FR-029**: Macros MUST capture the compile-time source module path as the target.
- **FR-030**: Macros MUST work with any logger implementing the Logger interface.

#### Error Handling

- **FR-031**: The system MUST provide a typed error for logging failures. The error type must distinguish between at minimum: configuration errors, backend errors, and serialization errors.
- **FR-032**: The error type MUST be non-exhaustive (extensible without breaking changes).
- **FR-033**: The error type MUST support human-readable display formatting.

#### No-Op Default

- **FR-034**: A no-op logger implementation MUST be available that implements the Logger interface, accepts all API calls, and silently discards all data without error.
- **FR-035**: The no-op logger MUST be safe for production use (no panics, minimal overhead).

#### Object Safety

- **FR-036**: The Logger interface MUST be object-safe. The following MUST compile: `Arc<dyn Logger>`, `Box<dyn Logger>`. The trait MUST NOT require generic methods, associated generic types, or any construct that prevents dynamic dispatch.

### Key Entities

- **Logger** (KIT-005): The primary logging interface. Thread-safe, provider-agnostic, and object-safe (supports `Arc<dyn Logger>`, `Box<dyn Logger>`). Accepts structured log entries, supports level-based querying, flushing, and `with_context()` wrapping. KIT-005 introduces this interface.
- **LoggerFactory** (KIT-005): Constructs logger instances. Separates creation from consumption, enabling backend swapping and dependency injection without application code changes. KIT-005 introduces this interface.
- **LogLevel** (from KIT-001): Determines severity of a log entry. Supports total ordering (Trace < Debug < Info < Warn < Error), serialization, display, and case-insensitive parsing. Reused from KIT-001, not redefined.
- **LogRecord** (from KIT-001): An immutable structured log entry. Carries severity level, target, message, timestamp, and deterministic ordered key-value fields. Reused from KIT-001, not redefined.
- **Value** (from KIT-001): Represents a structured field value in a log entry. Supports string, boolean, integer (signed and unsigned), and floating-point variants. Self-contained — no external serialization dependency required. Reused from KIT-001, not redefined.
- **LoggerError** (KIT-005): Typed error for logging failures. Distinguishes between configuration, backend, and serialization failures. Non-exhaustive for future extensibility. KIT-005 introduces this type.
- **LoggerContext** (KIT-005): Immutable builder-style container for contextual metadata. Passed to `Logger::with_context()`, which returns a ContextLogger wrapper that merges context fields into each log entry before delegating to the inner logger. Per-entry fields can shadow context fields. KIT-005 introduces this type.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: A developer can write application code that emits structured log entries using only the public Logger API, without importing or referencing any backend, provider, or serialization types. Verified by a test that compiles and passes with no backend-specific dependencies.
- **SC-002**: A developer can create a logger through the factory API, emit entries at all five severity levels, and observe the entries with correct level, message, fields, and timestamp — without knowing which backend is in use. Verified by integration test.
- **SC-003**: Severity levels can be created from string input case-insensitively, compared for ordering, serialized and deserialized without data loss. Verified by property-based tests covering all five levels and typical malformed inputs.
- **SC-004**: A context with multiple fields can be attached to a logger, and all subsequent entries carry those fields (with per-entry field shadowing). Verified by a test that emits entries before and after context attachment and inspects field presence.
- **SC-005**: Each per-level convenience method (trace, debug, info, warn, error) produces a LogRecord with the correct severity level. Verified by a test exercising all five methods and asserting the recorded level.
- **SC-006**: A macro invocation at a disabled level evaluates to zero cost (message expression not evaluated). Verified by a test using a side-effect probe as the message argument and confirming the side effect does not fire when the level is disabled.
- **SC-007**: The no-op logger accepts all API calls (enabled, log, flush, convenience methods) without panic or error. Verified by a test exercising every public method on a no-op logger.
- **SC-008**: 85%+ of public API items have documentation with runnable usage examples. Verified by executing the documentation examples and confirming they produce correct output.
- **SC-009**: Logger is object-safe: `Arc<dyn Logger>` and `Box<dyn Logger>` compile. Verified by a separate compilation test.

## Assumptions

- KIT-005 reuses LogRecord, LogLevel, and Value from KIT-001. If those types are not yet implemented in KIT-001, KIT-005 depends on KIT-001 completing its implementation first. KIT-005 MUST NOT define its own versions of these types.
- The feature builds on the existing KitLogger codebase, which already has concrete backend mechanics (outputs, formatters, buffers, samplers, redactors). The new public API layer wraps or replaces the existing concrete Logger with an abstract interface.
- Logging macros are included in this feature. If macro implementation proves complex (e.g., hygiene issues), they may be deferred to a follow-up feature, but scoping is part of this feature's work.
- The provider-agnostic constraint means the public API must not depend on any serialization or observability framework. Backend implementations may use whatever they need.
- Thread safety of the Logger interface is a design requirement, not an implementation detail — the interface contract guarantees safe concurrent use.
- `LoggerContext` is always applied via `with_context()` returning a new ContextLogger wrapper instance that delegates to the inner logger after merging context fields into each log entry. The original logger remains unchanged. No `info_with_context()` or similar per-call context API is provided.
- The Logger interface is object-safe. This enables dynamic dispatch (`dyn Logger`), which is required for `with_context()` wrapping, NoopLogger, LoggerFactory return types, and provider-agnostic design.

## Dependencies

- **KIT-001 Foundational Observability Abstractions**: Provides the canonical definitions of LogRecord, LogLevel, and Value. KIT-005 reuses these types directly; they are the single source of truth. If KIT-001 has not yet implemented these types, KIT-005 must await or coordinate with KIT-001's implementation.
- **kit-config**: External configuration types (LoggingConfig, etc.) are used by backend implementations but must not appear in the public Logger interface.
