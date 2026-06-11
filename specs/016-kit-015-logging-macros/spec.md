# Feature Specification: KIT-015 Logging Macros

**Feature Branch**: `016-kit-015-logging-macros`  
**Created**: 2025-06-10  
**Status**: Draft  
**Input**: User description: "KIT-015 Logging Macros - Implementar un sistema de macros ergonómicas para kit-logger que permita generar logs estructurados de forma simple, consistente y con costo cero o mínimo en runtime."

## User Scenarios & Testing

### User Story 1 - Basic Logging (Priority: P1)

**As a** developer, **I want** to log a simple message using a macro, **so that** I can record events without manual event construction.

**Why this priority**: This is the foundational use case - all users need to log basic messages.

**Independent Test**: Can be tested by calling `info!("hello");` and verifying a valid INFO-level event is generated with the correct message.

**Acceptance Scenarios**:
1. **Given** the logging system is configured, **When** I call `info!("hello")`, **Then** a valid INFO event is generated with message "hello".
2. **Given** the logging system is configured, **When** I call `debug!("debug message")`, **Then** a valid DEBUG event is generated if the level is enabled.

---

### User Story 2 - Structured Fields (Priority: P1)

**As a** developer, **I want** to include structured key-value fields with my log messages, **so that** I can create machine-parseable log entries.

**Why this priority**: Structured logging is a core requirement for modern observability and debugging.

**Independent Test**: Can be tested by calling `info!(user_id = 123, tenant_id = "abc", "user logged in")` and verifying the fields appear in the structured payload.

**Acceptance Scenarios**:
1. **Given** structured fields are provided, **When** calling the macro, **Then** all fields are included in the structured event payload.
2. **Given** multiple fields of different types, **When** logging, **Then** each type is serialized correctly.

---

### User Story 3 - Error Logging (Priority: P1)

**As a** developer, **I want** to log errors with automatic context capture, **so that** I can consistently track failures across the codebase.

**Why this priority**: Error tracking is critical for production debugging and monitoring.

**Independent Test**: Can be tested by calling `error!(err = some_error, "database operation failed")` and verifying the error is properly serialized in the event.

**Acceptance Scenarios**:
1. **Given** an error value, **When** using `error!(err = e, "message")`, **Then** the error is serialized correctly with its message and type.
2. **Given** no error context needed, **When** using plain `error!("message")`, **Then** it works like basic logging.

---

### User Story 4 - Context Propagation (Priority: P1)

**As a** developer, **I want** to include distributed tracing context in my logs, **so that** I can correlate logs across service boundaries.

**Why this priority**: Essential for debugging distributed systems and microservices.

**Independent Test**: Can be tested by setting correlation_id and request_id in context, then logging with those fields, and verifying they appear in the output.

**Acceptance Scenarios**:
1. **Given** correlation_id exists in context, **When** calling any logging macro, **Then** the correlation_id is automatically included.
2. **Given** request_id is set, **When** logging, **Then** request_id appears in the event metadata.

---

### User Story 5 - Audit Events (Priority: P2)

**As a** developer, **I want** to create audit events for security-sensitive operations, **so that** I can maintain compliance audit trails.

**Why this priority**: Required for regulatory compliance and security monitoring.

**Independent Test**: Can be tested by calling `audit!(action = "user.deleted", actor = "user123", target = "user456", "user removed")` and verifying the event is marked as AUDIT level.

**Acceptance Scenarios**:
1. **Given** audit macro is called, **Then** the event is marked with level AUDIT.
2. **Given** audit fields (actor, action, target), **Then** they are included in the structured event.
3. **Given** the system includes KIT-011 (Audit Logging), **Then** the audit event is fully compatible with it.

---

### User Story 6 - Security Events (Priority: P2)

**As a** developer, **I want** to log security-relevant events using a dedicated macro, **so that** security teams can monitor and alert on security incidents.

**Why this priority**: Critical for security monitoring and incident response.

**Independent Test**: Can be tested by calling `security!(event = "authentication.failure", username = "john", ip = "192.168.1.1", "login failed")` and verifying the event is marked as SECURITY level.

**Acceptance Scenarios**:
1. **Given** security macro is called, **Then** the event is marked with level SECURITY.
2. **Given** security fields (event, username, ip, etc.), **Then** they are included in the structured event.
3. **Given** the system includes KIT-013 (Security Logging), **Then** the security event is fully compatible with it.

---

### User Story 7 - Automatic PII Redaction (Priority: P2)

**As a** developer, **I want** to log personal data without worrying about compliance, **so that** PII is automatically protected.

**Why this priority**: GDPR/privacy compliance requirement; developer ergonomics.

**Independent Test**: Can be tested by passing email and phone fields to a log macro and verifying they are redacted according to KIT-014 rules.

**Acceptance Scenarios**:
1. **Given** PII fields are logged, **Then** they pass through the PII redaction pipeline.
2. **Given** KIT-014 (PII Redaction) is available, **Then** all fields respect the configured redaction rules.
3. **Given** PII detection rules, **Then** sensitive fields are automatically identified and redacted.

---

### User Story 8 - Lazy Evaluation (Priority: P2)

**As a** developer, **I want** to avoid expensive computations when logging is disabled, **so that** I can include debug information without runtime cost in production.

**Why this priority**: Performance requirement - prevents expensive debug code from running in production.

**Independent Test**: Can be tested by passing `expensive = compute_value()` to a disabled log level and verifying the function is never called.

**Acceptance Scenarios**:
1. **Given** a log level is disabled, **When** passing a lazy expression, **Then** the expression is not evaluated.
2. **Given** a log level is enabled, **When** passing a lazy expression, **Then** the expression is evaluated and included.

---

### User Story 9 - Compile-Time Filtering (Priority: P3)

**As a** developer, **I want** to completely remove certain log levels from production builds, **so that** I can eliminate all logging overhead in production.

**Why this priority**: Zero-overhead requirement for high-performance systems.

**Independent Test**: Can be tested by building with `features = ["max_level_info"]` and verifying TRACE and DEBUG calls are compiled out.

**Acceptance Scenarios**:
1. **Given** compile-time feature flags, **When** building with "max_level_info", **Then** TRACE and DEBUG macros are compile-time no-ops.
2. **Given** different feature flags, **Then** appropriate log levels are included/excluded at compile time.

---

### User Story 10 - Named Targets (Priority: P3)

**As a** developer, **I want** to categorize logs by target/service name, **so that** I can filter and route logs by source.

**Why this priority**: Multi-service architectures need log categorization.

**Independent Test**: Can be tested by calling `info!(target = "auth-service", "user authenticated")` and verifying the target appears in event metadata.

**Acceptance Scenarios**:
1. **Given** a target is specified, **Then** the target is persisted in the event metadata.
2. **Given** no target specified, **Then** a default or module-based target is used.

---

### Edge Cases

- What happens when circular references exist in the data being logged?
- How does the system handle extremely large field values (e.g., megabytes of data)?
- What happens when the logging sink is down or unavailable?
- How are panics in user-provided field closures handled?
- What happens when incompatible field types are mixed in the same macro call?

## Requirements

### Functional Requirements

- **FR-001**: The system MUST provide base logging macros: `trace!`, `debug!`, `info!`, `warn!`, `error!`, `fatal!`
- **FR-002**: The system MUST provide specialized macros: `audit!`, `security!`, and optionally `http!`
- **FR-003**: The system MUST support structured field syntax: `info!(key = value, "message")`
- **FR-004**: The system MUST support plain message syntax: `info!("message")`
- **FR-005**: Each macro MUST attach metadata: timestamp, level, target, file, line, module_path
- **FR-006**: The system MUST automatically include correlation context (correlation_id, trace_id, span_id, request_id) when available
- **FR-007**: The system MUST integrate with KIT-014 for automatic PII redaction of all logged fields
- **FR-008**: The system MUST support lazy evaluation of field expressions
- **FR-009**: The system MUST support compile-time log level filtering via feature flags
- **FR-010**: The system MUST support named targets via `target = "name"` syntax
- **FR-011**: Error macros MUST serialize errors using a shared trait approach
- **FR-012**: The system MUST be compatible with existing and planned kit-logger features (KIT-001 through KIT-014)
- **FR-013**: All macros MUST delegate to a common Event Builder to avoid code duplication in event construction logic
- **FR-014**: Structured fields MUST follow a two-stage lifecycle: (1) Construction: fields collected using `SmallVec<Field, 8>` or equivalent stack-optimized structure; (2) Serialization: fields emitted in deterministic order using BTreeMap, sorted slices, or equivalent - deterministic output is required, BTreeMap is not required as internal storage
- **FR-015**: Macros MUST be able to retrieve context from async task-local storage when available (e.g., tokio::task_local!)
- **FR-016**: Macros MUST be able to attach trace_id and span_id without changing public API (future OpenTelemetry compatibility)
- **FR-017**: The system MUST provide a `LogValue` (or `IntoFieldValue`) trait that supports: String, &str, bool, all integer types (i8-i64, u8-u64), floats (f32, f64), Uuid, DateTime, std::error::Error, Option<T>, and Vec<T>
- **FR-018**: Macros MUST include source location (file, line, module_path) only when the "source-location" feature is enabled
- **FR-019**: The system MUST include compile-fail tests (trybuild) to verify invalid macro usage produces clear compile errors
- **FR-020**: All macros MUST be hygienic and MUST NOT introduce variables into the user's scope that could cause conflicts
- **FR-021**: The system MUST support configurable limits for: maximum number of fields per event and maximum serialized size per value (to prevent memory exhaustion from large payloads)
- **FR-022**: The event schema produced by macros MUST remain stable and backward-compatible across minor versions (e.g., 0.1.x → 0.2.x) for compatibility with exporters, audit storage, SIEM, OpenSearch, Loki, Elasticsearch, ClickHouse, and other observability tools
- **FR-023**: The primary logging macros (trace!, debug!, info!, warn!, error!, fatal!, audit!, security!) MUST be implemented using `macro_rules!` whenever possible - no proc_macro is required for KIT-015. Future procedural macros like #[instrument], #[audit], #[security_event] are out of scope for this feature.
- **FR-024**: The internal event construction pipeline MUST follow a unified architecture: Macro → FieldBuilder → EventBuilder → Logger → Exporter to avoid duplication and separate concerns
- **FR-025**: Field storage during construction MUST use `SmallVec<Field, 8>` for events with 0-8 fields to avoid heap allocations, then convert to ordered structure for serialization
- **FR-026**: Field values MUST use a strongly-typed enum (`FieldValue`) instead of `dyn Any` or `serde_json::Value` as the internal representation
- **FR-027**: The fatal! macro MUST: (1) create a FATAL level event, (2) emit the event through the logging pipeline, (3) return control to the caller. The macro MUST NOT panic, abort, shutdown, or restart the process. Behavior after emission of a FATAL event is outside the scope of KIT-015.
- **FR-028**: The event schema and structured fields MUST be fully preserved when exporting to OpenTelemetry, Loki, OpenSearch, Elasticsearch, and ClickHouse. Correlation IDs, Audit markers, and Security markers must be maintained in the exported format
- **FR-029**: The system MUST include snapshot tests for public API stability to verify macro expansion, generated schema, and metadata for info!, warn!, error!, audit!, and security! macros. Breaking changes MUST fail these tests
- **FR-030**: The macro API MUST support injection of trace_id, span_id, and correlation_id without requiring public API changes, enabling seamless integration with KIT-016 (OpenTelemetry)

### Key Entities

- **Log Macro**: The primary interface - declarative macro that generates structured log events
- **Structured Field**: Key-value pairs attached to log events
- **Event Metadata**: Automatically attached information (timestamp, level, file, line, target)
- **Context Carriers**: correlation_id, trace_id, span_id, request_id from context propagation
- **Event Builder**: Common internal component that all macros delegate to for constructing log events (must NOT contain serialization or exporter logic)
- **Field Builder**: Intermediate component that collects and processes fields before passing to EventBuilder
- **FieldValue Enum**: Strongly-typed enum (Str, I64, U64, F64, Bool, Error, DateTime, Uuid, Array, Null) - no dyn Any, no serde_json::Value
- **Value Serializer Trait**: Trait (e.g., `LogValue` or `IntoFieldValue`) that defines how types are converted to FieldValue

## Success Criteria

### Measurable Outcomes

- **SC-001**: Developers can log a simple message with one line of code: `info!("started")`
- **SC-002**: Structured fields are included in the log output with correct serialization
- **SC-003**: Error information is properly serialized with type and message context
- **SC-004**: Correlation IDs are automatically propagated when present in context
- **SC-005**: Audit events are marked with AUDIT level and include actor/action/target fields
- **SC-006**: Security events are marked with SECURITY level and include appropriate security fields
- **SC-007**: PII fields are automatically redacted when KIT-014 integration is enabled
- **SC-008**: Disabled log levels do not evaluate their arguments (lazy evaluation)
- **SC-009**: Build feature flags can completely remove log levels from the binary
- **SC-010**: All existing kit-logger functionality continues to work without regression
- **SC-011**: Test coverage is at least 85%
- **SC-012**: Zero tests are ignored and all tests pass
- **SC-013**: `info!("hello")` MUST generate fewer than 1 heap allocation in the hot path
- **SC-014**: Disabled log levels (e.g., `debug!` when debug is disabled) MUST have near-zero runtime cost

### Technical Implementation Notes

The following notes provide context for implementers but are not user-facing requirements:

- **Architecture**: Create a separate `crates/macros` crate (not `logging-macros`) to centralize macro infrastructure for future extensibility (audit macros, metric macros, tracing macros, OTel macros). However, KIT-015 itself uses macro_rules! only - no proc_macro required.
- **Crate Structure**: The macros crate provides `kit_logger_macros` crate-local, and re-exports from the main `kit_logger` crate
- **Zero Allocation Goals**: Avoid unnecessary allocations, avoid boxing, prefer `Cow<'static, str>` when appropriate
- **Deterministic Ordering**: Use `BTreeMap` instead of `HashMap` to ensure consistent field ordering
- **Compatibility**: Full compatibility with existing features; for features not yet implemented, create compatible stub interfaces to allow progress
- **Performance**: Minimum overhead, lazy evaluation for disabled levels, compile-time filtering option
- **Safety**: No unsafe code unless explicitly justified and documented in code comments

### Implementation Guidance (Non-Normative)

These are implementation recommendations that help achieve the functional requirements but are not themselves mandatory. Implementation teams MAY choose equivalent alternatives if they achieve the same outcomes.

#### IMP-001: Prefer time crate over chrono

FieldValue::DateTime should abstract the concrete implementation. The public API should NOT expose chrono::DateTime or OffsetDateTime directly. This allows transparent migration from chrono to time in the future.

- Use `time` crate for DateTime handling
- Design abstract interface for serialization
- Maintain stable serialization format

#### IMP-002: UUID Abstraction Layer

Create an internal `UuidValue` abstraction that can be converted from `uuid::Uuid` and potentially other UUID implementations. FieldValue should not couple directly to external crates.

- Abstract uuid::Uuid behind internal type
- Keep public API stable
- Allow future implementation swapping

#### IMP-003: Criterion Benchmark Suite

Implement benchmarks using criterion to measure:

- `info!("hello")` latency
- `info!(field = value)` latency
- `error!(...)`, `audit!(...)`, `security!(...)` latency
- Disabled level overhead (debug!, trace!)

Compare against: tracing, log, slog

#### IMP-004: Compile-Time Benchmark Validation

 Measure macro_rules! impact on compilation time by comparing:

- Current implementation
- Experimental proc_macro alternative

Document results. Keep macro_rules! unless proc_macro proves significantly better.

#### IMP-005: Snapshot Golden Files

Generate golden snapshots for:

- info!, warn!, error!, audit!, security!

Validate: schema, metadata, field ordering, serialization. Incompatible changes must break snapshots.

#### IMP-006: Fuzz Testing

Use cargo-fuzz to test:

- Invalid syntax handling
- Extreme field values
- Large payloads
- Unexpected combinations

Goal: no panic, no UB, no crashes.

#### IMP-007: Exporter Compatibility Matrix

Add integration tests for:

- JSON Exporter
- Console Exporter  
- Audit Exporter
- Security Exporter
- Future: OpenTelemetry Adapter

All must preserve: fields, metadata, correlation_id, audit markers, security markers.

### Compile-Time Stability

The macros MUST work correctly in all standard Rust compilation scenarios:

- **External crates**: Macros must work when imported from external crates
- **Workspaces**: Macros must work in monorepo and multi-crate workspace setups
- **Re-exports**: Macros must be properly re-exportable without losing functionality
- **Full paths**: Support absolute paths like `::kit_logger::info!`

### Trybuild Compile-Fail Tests

The system MUST include compile-fail tests that verify invalid macro usage produces clear error messages:

- `info!(foo);` MUST fail to compile (literal without =)
- `info!(x =);` MUST fail to compile (incomplete field)
- `info!(= value);` MUST fail to compile (invalid field syntax)
- Other invalid patterns as defined by the macro grammar

## Assumptions

1. KIT-001 (Core Logger) provides the underlying event emission system
2. KIT-002 (Structured Events) provides the event structure format
3. KIT-011 (Audit Logging) will provide audit-specific fields and handling
4. KIT-013 (Security Logging) will provide security event handling
5. KIT-014 (PII Redaction) will provide the redaction rules and pipeline
6. For any KIT features not yet implemented, compatible stub interfaces will be created to allow progress on this feature
