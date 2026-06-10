# Feature Specification: KIT-006 Formatting Pipeline

**Feature Branch**: `006-kit-006-formatting`
**Created**: 2026-06-10
**Status**: Draft
**Input**: Create the formatting subsystem for KitLogger — a provider-agnostic formatting layer that transforms LogRecord instances into serialized representations.

## Overview

This feature defines a reusable formatting layer for KitLogger that sits between logging and export. KIT-001 established the foundational observability data model (LogRecord, LogLevel, Value), and KIT-005 defined the public Logger API (Logger, LoggerFactory, etc.). KIT-006 builds on both to provide a dedicated formatting pipeline that transforms LogRecord instances into serialized output.

Without a dedicated formatting layer, every exporter (console, file, OTLP, etc.) would need to implement its own formatting logic, leading to code duplication, inconsistent output formats, and maintenance overhead. KIT-006 solves this by providing:

- A **Formatter trait** — an object-safe abstraction for transforming LogRecords into formatted output
- **Human formatter** — stable, readable log-line output with single-line and multi-line modes
- **JSON formatter** — valid, deterministic JSON output with compact and pretty modes
- A **Formatter registry** — runtime selection of formatters by enum (built-ins) or string (custom)
- **Formatting errors** — typed error handling for formatting failures
- **Configuration integration** — compatibility with kit-config for format selection

The formatting pipeline operates exclusively on LogRecord types from KIT-001. It does not concern itself with transport, buffering, or exporter concerns.

## Clarifications

### Session 2026-06-10

- Q: Should FormatterRegistry be pre-populated, start empty, or support both modes? → A: Support both modes via configuration. Pre-populated by default for ergonomics, but configurable to start empty for minimal deployments.
- Q: Should FormatError include output-buffer-related failures? → A: No. Formatter only transforms LogRecord → FormattedRecord. Buffer exhaustion belongs to exporters and transports.
- Q: When structured fields are present, should HumanFormatter always produce single-line output or support both single-line and multi-line via configuration? → A: Support both modes via configuration. Single-line for production, multi-line for development.
- Q: How should formatters be resolved — string names only or enum for built-ins with string extension mechanism? → A: Enum for built-in formatters (FormatKind) with string extension mechanism for custom formatters.
- Q: Should JsonFormatter support compact only or both compact and pretty? → A: Both compact and pretty via configuration. Compact for production, pretty for debugging/tests.

## User Scenarios & Testing _(mandatory)_

### User Story 1 — Export LogRecords as Human-Readable Lines (Priority: P1)

As a developer running an application locally, I want to format log entries as human-readable lines with timestamp, severity, and message, so that I can quickly read and understand application behaviour during development and debugging.

**Why this priority**: Human-readable output is the most common format for development and ad-hoc operations. Without it, developers must parse structured output to understand application behaviour.

**Independent Test**: A test creates a LogRecord with known fields, formats it through the human formatter, and verifies the output contains the expected timestamp, severity level, target, and message in a stable, readable layout.

**Acceptance Scenarios**:

1. **Given** a LogRecord at Info level with message "Server started", **When** formatted with the human formatter, **Then** the output contains "INFO", "Server started", and the timestamp in a consistent positional layout.
2. **Given** a LogRecord with trace-level severity, **When** formatted with the human formatter, **Then** the level is displayed as "TRACE".
3. **Given** a LogRecord with error-level severity, **When** formatted with the human formatter, **Then** the level is displayed as "ERROR".
4. **Given** a LogRecord with structured fields, **When** formatted with the human formatter, **Then** the fields are appended to the output line in deterministic order.

---

### User Story 2 — Export LogRecords as Valid JSON (Priority: P1)

As a platform engineer shipping logs to a centralized logging system, I want to format log entries as well-formed JSON, so that my log aggregation pipeline can parse, index, and query log data reliably.

**Why this priority**: JSON is the universal interchange format for log aggregation systems. Valid JSON output is a hard requirement for production observability pipelines.

**Independent Test**: A test creates a LogRecord with mixed field types (string, integer, boolean), formats it through the JSON formatter, and verifies the output is valid JSON with all fields present and correctly typed.

**Acceptance Scenarios**:

1. **Given** a LogRecord with a message and a severity level, **When** formatted with the JSON formatter, **Then** the output is valid JSON containing "message", "level", "timestamp", and "target" keys.
2. **Given** a LogRecord with fields `{user_id: "abc123", count: 42, active: true}`, **When** formatted with the JSON formatter, **Then** the output contains the string `"abc123"`, the number `42`, and the boolean `true`.
3. **Given** a LogRecord with nested field values, **When** formatted with the JSON formatter, **Then** the output is valid JSON with the nested structure preserved.
4. **Given** a LogRecord with empty fields, **When** formatted with the JSON formatter, **Then** the output is valid JSON with an empty fields object.

---

### User Story 3 — Select a Formatter at Runtime (Priority: P2)

As an application operator, I want to change the log output format at startup or during configuration without recompiling the application, so that I can adapt to different environments (development, staging, production) from a configuration file.

**Why this priority**: Runtime formatter selection decouples format choice from code. It enables different environments to use different formats without application changes.

**Independent Test**: A test registers two formatters, requests each through the registry (by enum for built-ins, by string for custom), formats identical LogRecords through each, and verifies the outputs differ in format while preserving the same data.

**Acceptance Scenarios**:

1. **Given** a registry pre-populated with formatters, **When** a formatter is requested via `FormatKind::Json`, **Then** a JSON formatter is returned.
2. **Given** a registry, **When** a formatter is requested via an unregistered `FormatKind` or string name, **Then** an error is returned (not a fallback default).
3. **Given** a registry, **When** a custom formatter is registered by a string name and then retrieved by that name, **Then** the custom formatter is returned.
4. **Given** a LogRecord formatted through different registry formatters, **When** compared, **Then** the data integrity is preserved (same level, message, target, fields across formats).
5. **Given** a registry configured to start empty, **When** no formatters have been registered, **Then** any retrieval attempt returns an error.

---

### User Story 4 — Extend Formatting with Custom Formatters (Priority: P2)

As a library author integrating KitLogger with a custom observability system, I want to implement the Formatter trait to produce a custom log format, so that I can emit log data in the format my observability system expects without modifying KitLogger's core.

**Why this priority**: Extensibility is a core goal. Without a public Formatter trait, custom formatting would require forking or modifying internal code.

**Independent Test**: A test implements a custom formatter that produces a simple key=value format, registers it via the registry, formats a LogRecord through it, and verifies the output matches the expected key=value pattern.

**Acceptance Scenarios**:

1. **Given** a custom formatter implementing the Formatter trait, **When** a LogRecord is formatted, **Then** the output follows the custom format rules.
2. **Given** a custom formatter, **When** registered in the registry under "custom", **Then** it is retrievable by that name and interchangeable with built-in formatters.
3. **Given** a custom formatter, **When** used through an exporter that consumes Formatter abstractions, **Then** no exporter changes are required.

---

### User Story 5 — Handle Formatting Errors Gracefully (Priority: P3)

As a developer integrating formatting into a production system, I want formatting failures (invalid data, serialization errors) to produce typed errors without panicking, so that a failed format operation does not crash the application or lose the log entry.

**Why this priority**: Formatting errors should not be fatal. The logging system should continue operating even when a single entry cannot be formatted.

**Independent Test**: A test creates a formatter that fails on specific input (e.g., extremely large message), formats through it, and verifies the error is returned as a typed FormatError without panic.

**Acceptance Scenarios**:

1. **Given** a formatter that encounters a serialization error, **When** `format()` is called, **Then** a typed `FormatError` is returned via `Result`.
2. **Given** a `FormatError`, **When** displayed, **Then** the error message describes the nature of the failure (serialization, invalid input, etc.).
3. **Given** a `FormatError`, **When** an exporter receives it, **Then** the exporter can choose to retry, skip, or propagate the error without crashing.
4. **Given** a formatter that exhausts an output buffer, **When** `format()` is called, **Then** the formatter does not attempt to handle buffer exhaustion — it either produces output or returns a serialization/invalid-data error. Buffer management is the exporter's responsibility.

### Edge Cases

- **Empty LogRecord**: A LogRecord with an empty message and no fields must produce valid output (empty string or minimal valid JSON).
- **Very large fields**: Extremely large field values (multiline strings, large payloads) must not cause the formatter to panic or produce invalid output. Truncation at a configurable limit is acceptable.
- **Special characters**: String values containing special characters (quotes, newlines, Unicode, null bytes) must be properly escaped in JSON output and handled gracefully in human output.
- **Non-UTF-8 data**: The formatter must produce valid UTF-8 output; non-UTF-8 field values should be replaced with a placeholder or hex escaped rather than producing invalid output.
- **Concurrent formatting**: Multiple threads calling `format()` on the same formatter instance concurrently must not corrupt state or produce interleaved output.
- **Unregistered formatter**: Requesting a formatter by an unknown name from the registry must return an error, not silently fall back to a default format.

## Requirements _(mandatory)_

### Functional Requirements

#### Formatter Trait

- **FR-001**: The system MUST provide a `Formatter` abstraction that transforms a `LogRecord` into a formatted byte representation. The abstraction must support dynamic dispatch (allow use through a pointer or reference without knowing the concrete type).
- **FR-002**: The `Formatter` abstraction MUST accept a reference to a `LogRecord` (from KIT-001) and return either the formatted output or a `FormatError`.
- **FR-003**: The `Formatter` abstraction MUST be safe to use from multiple concurrent threads without data corruption.
- **FR-004**: The `Formatter` abstraction MUST NOT depend on any specific exporter, transport, or serialization framework in its public interface.

#### Formatted Output

- **FR-005**: The formatted output MUST be a byte buffer (ordered sequence of bytes).
- **FR-006**: The formatter MUST produce deterministic output: identical LogRecords formatted with the same formatter must produce identical byte sequences.
- **FR-007**: The formatted output MUST be valid UTF-8.

#### Human Formatter

- **FR-008**: A `HumanFormatter` MUST produce a text representation with timestamp, severity level, target, and message in that order. Additional structured fields MUST follow in deterministic order.
- **FR-009**: The human formatter MUST support two layout modes via configuration: (a) single-line — all output on one line; (b) multi-line — fields on subsequent lines for readability.
- **FR-010**: The human formatter MUST NOT colorize or ANSI-escape output (coloring is an exporter concern, out of scope).

#### JSON Formatter

- **FR-011**: A `JsonFormatter` MUST produce valid JSON output.
- **FR-012**: The JSON formatter MUST include at minimum these top-level keys: `timestamp`, `level`, `target`, `message`, `fields`.
- **FR-013**: The `fields` key MUST contain the LogRecord's structured key-value pairs, ordered by key.
- **FR-014**: The JSON formatter MUST properly escape all string values (quotes, backslashes, control characters, Unicode).
- **FR-015**: The JSON formatter MUST support null, boolean, number, string, and array value types as defined by KIT-001's `Value` type.
- **FR-016**: The JSON formatter MUST support two output modes via configuration: (a) compact — no extra whitespace for production ingestion; (b) pretty — indented for debugging and tests.

#### Formatter Registry

- **FR-017**: The system MUST provide a `FormatterRegistry` for runtime formatter selection.
- **FR-018**: The registry MUST provide an enum (`FormatKind`) for built-in formatters with variants `Human` and `Json`.
- **FR-019**: The registry MUST support registering custom formatters with string names (e.g., `"my-custom-format"`) alongside built-in enum entries.
- **FR-020**: The registry MUST support retrieving a formatter by either `FormatKind` or string name, returning either the formatter or a registry error.
- **FR-021**: The registry MUST return an error when an unregistered name or kind is requested (no silent fallback).
- **FR-022**: The registry MUST be safe for concurrent registration and lookup from multiple threads.
- **FR-023**: The registry MUST support both modes of initialization: (a) pre-populated by default with `Human` and `Json` formatters; (b) empty start via configuration for minimal deployments.

#### Format Errors

- **FR-024**: The system MUST provide a `FormatError` type for formatting failures.
- **FR-025**: `FormatError` MUST distinguish between at minimum: serialization failures and invalid record data. Buffer exhaustion is an exporter concern and MUST NOT appear in FormatError.
- **FR-026**: `FormatError` MUST be non-exhaustive (extensible without breaking changes).
- **FR-027**: `FormatError` MUST support human-readable display formatting and the standard error protocol.
- **FR-028**: A `RegistryError` MUST be provided for registry lookup failures, distinguishable from formatting errors.

#### Configuration Integration

- **FR-029**: The formatter selection MUST be configurable via kit-config's `LogFormat` enum (or equivalent), mapping format names to concrete formatter implementations.
- **FR-030**: The registry MUST support being populated from a configuration source (kit-config `LoggingConfig`), registering the formatter selected by the user's format setting.

#### Extensibility

- **FR-031**: External code MUST be able to implement the `Formatter` abstraction and register custom formatters in the registry via string name.
- **FR-032**: Adding a new formatter MUST NOT require modifying existing formatters, exporters, or the registry itself.

### Key Entities

- **LogRecord** (from KIT-001): The immutable structured log entry that formatters consume. Contains severity level, target, message, timestamp, and structured fields. Not redefined by KIT-006.
- **Formatter**: An abstraction for transforming `LogRecord` into formatted byte output. Supports dynamic dispatch. Thread-safe and provider-agnostic. KIT-006 introduces this abstraction.
- **HumanFormatter**: A Formatter implementation producing text output with timestamp, severity, target, message, and fields in deterministic order. Supports single-line and multi-line modes via configuration.
- **JsonFormatter**: A Formatter implementation producing valid JSON output with proper escaping and deterministic key ordering. Supports compact and pretty modes via configuration.
- **FormatKind**: An enum for addressing built-in formatters, with variants `Human` and `Json`. Provides type safety for built-in selection while preserving string-based extensibility for custom formatters.
- **FormatterRegistry**: A thread-safe registry for runtime formatter selection. Supports both `FormatKind` (built-ins) and string names (custom). Configurable to start pre-populated or empty. KIT-006 introduces this type.
- **FormattedRecord**: The output of a formatter — a byte buffer containing the serialized representation of a LogRecord, guaranteed to be valid UTF-8.
- **FormatError**: Typed error for formatting failures. Distinguishes between serialization and invalid record data. Buffer exhaustion is excluded (exporter concern). Non-exhaustive.
- **RegistryError**: Typed error for registry operations (unknown formatter name, duplicate registration, etc.).

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: A LogRecord formatted with the human formatter produces a single-line string containing timestamp, severity, target, and message in a stable layout. Verified by a test that formats a known LogRecord and asserts line structure.
- **SC-002**: A LogRecord formatted with the JSON formatter produces valid JSON that, when parsed, contains `timestamp`, `level`, `target`, `message`, and `fields` keys. Verified by parsing the output with a JSON parser.
- **SC-003**: Two identical LogRecords formatted with the same formatter produce identical byte output (determinism). Verified by a test that formats twice and compares byte equality.
- **SC-004**: A formatter can be registered with a name and retrieved from the registry. Verified by integration test covering register, lookup, and format roundtrip.
- **SC-005**: Requesting an unregistered formatter name from the registry returns an error (not a silent fallback). Verified by a test requesting "unknown_format".
- **SC-006**: A custom formatter implementing the Formatter abstraction can be registered alongside built-in formatters and used interchangeably. Verified by a test implementing, registering, and formatting through a custom formatter.
- **SC-007**: JSON output with special characters (quotes, backslashes, Unicode) is properly escaped and valid. Verified by a test formatting an entry with such characters and parsing the output.
- **SC-008**: The Formatter abstraction supports dynamic dispatch: it can be used through a pointer or reference without knowing the concrete type. Verified by a compile test.

## Assumptions

- The formatted output type is a byte buffer (ordered sequence of bytes), chosen to avoid implying UTF-8 validity at the abstraction level even though the actual output is valid UTF-8. Exporters that write to files, stdout, or network sockets operate on bytes.
- The human formatter default layout is: `[timestamp] [LEVEL] [target] message field1=val1 field2=val2` (single-line mode). Multi-line mode places fields on subsequent lines. No ANSI coloring or terminal-specific features are included.
- The JSON formatter uses a simple key-value JSON object with deterministic key ordering. Compact mode has no extra whitespace; pretty mode adds indentation. No JSON schema or versioning is included.
- The registry stores formatter instances that can be shared by multiple exporters. It supports both `FormatKind` enum (for built-ins) and string names (for custom formatters).
- The registry can be configured to start pre-populated (default) or empty. When pre-populated, it registers `Human` and `Json` formatters automatically.
- FormatError excludes buffer-related failures — those belong to the exporter or transport layer.
- KIT-006 lives in the existing crate alongside KIT-005's types, as a new module or sub-module. If crate isolation becomes necessary (e.g., dependency conflicts), it may be extracted to a separate workspace member in a future iteration.
- The existing `src/formatter.rs` code (which formats `LogEvent`, not `LogRecord`) will coexist during a transition period. KIT-006's formatters operate exclusively on `LogRecord`. A future integration phase may consolidate them.

## Dependencies

- **KIT-001 Foundational Observability Abstractions**: Provides LogRecord, LogLevel, and Value types that the formatters consume.
- **KIT-005 Logger API**: Provides the Logger, LoggerFactory, and LoggerContext types that may reference formatters in their configuration.
- **kit-config**: Provides LogFormat enum and LoggingConfig for formatter selection and configuration integration.
- **serde_json** (or equivalent): Required only by the JsonFormatter implementation. Must not leak into the Formatter abstraction or registry.
