# Feature Specification: Error Logging

**Feature Branch**: `010-kit-010-error`  
**Created**: 2026-06-10  
**Status**: Draft  
**Input**: User description: "KIT-010 Error Logging - Provide structured error logging APIs, support error chains and root cause extraction, contextual metadata, error classification, correlation IDs, stack trace capture, remain provider-agnostic with zero required external dependencies."

## User Scenarios & Testing _(mandatory)_

### User Story 1 - Structured Error Logging (Priority: P1)

As a developer, I want to log an error with structured fields so that failures are searchable.

**Why this priority**: Structured error logging is the core value proposition of this feature. Without it, none of the other error handling capabilities matter. This enables operators to search, filter, and analyze errors in observability platforms.

**Independent Test**: Can be fully tested by calling the error logging API with an error object and verifying that a structured event is produced with all required fields (level, message, error, timestamp).

**Acceptance Scenarios**:

1. **Given** a standard error object, **When** the developer calls `logger.error(err)`, **Then** a structured event is produced with level="error", the error message, an error identifier, and timestamp.
2. **Given** an error with additional fields, **When** the developer calls `logger.error_with_fields(err, fields)`, **Then** those fields are included in the output without modifying the original error.
3. **Given** an error that needs contextual information, **When** the developer calls `logger.error_with_context(err, context)`, **Then** the context is attached to the error event for correlation.

---

### User Story 2 - Error Chain Preservation (Priority: P1)

As a developer, I want error chains preserved so that root causes remain visible.

**Why this priority**: Error chains (wrapped errors) are common in Go and other languages. Without chain preservation, the root cause of failures can be obscured, making debugging significantly harder.

**Independent Test**: Can be tested by creating a chain of errors (e.g., DatabaseError → ConnectionError → TimeoutError) and verifying the complete chain is captured with root cause identified.

**Acceptance Scenarios**:

1. **Given** an error with a chain of causes, **When** logging the error, **Then** all errors in the chain are preserved in the output.
2. **Given** an error chain of any depth, **When** logging, **Then** the root cause is extracted and exposed as a dedicated field.
3. **Given** a chain containing a cycle, **When** logging, **Then** the cycle is detected and safely terminated without infinite recursion.

---

### User Story 3 - Consistent Metadata Across Exporters (Priority: P1)

As an operator, I want consistent error metadata across exporters.

**Why this priority**: Different exporters (console, JSON, file, gRPC) must produce semantically equivalent output so that operators can switch between view modes without losing information.

**Independent Test**: Can be tested by configuring multiple exporters and verifying each receives and renders the same normalized error structure.

**Acceptance Scenarios**:

1. **Given** a configured console exporter, **When** an error is logged, **Then** it renders in human-readable form.
2. **Given** a configured JSON exporter, **When** an error is logged, **Then** all structured fields are preserved.
3. **Given** any configured exporter, **When** an error is logged, **Then** it receives the complete normalized error structure.

---

### User Story 4 - Automatic Correlation ID Propagation (Priority: P2)

As a developer, I want correlation IDs attached automatically when available.

**Why this priority**: Correlation IDs (request_id, trace_id, correlation_id) are essential for tracing requests across service boundaries. Manual attachment is error-prone.

**Independent Test**: Can be tested by setting correlation values in context and verifying they automatically appear in error logs.

**Acceptance Scenarios**:

1. **Given** a request_id in the context, **When** logging an error, **Then** the request_id is automatically included in the error event.
2. **Given** a trace_id in the context, **When** logging an error, **Then** the trace_id is automatically included in the error event.
3. **Given** no correlation IDs in context, **When** logging an error, **Then** the error is logged without correlation fields (no errors).

---

### User Story 5 - Error Classification (Priority: P2)

As an operator, I want machine-readable error classifications.

**Why this priority**: Error classifications enable automated response workflows, alerting rules, and dashboarding without manual parsing of error messages.

**Independent Test**: Can be tested by logging errors with different classifications and verifying the classification field is present and searchable.

**Acceptance Scenarios**:

1. **Given** an error with classification "database", **When** logged, **Then** classification="database" appears in the output.
2. **Given** a custom classification "MY_CUSTOM_ERROR", **When** logged, **Then** classification="MY_CUSTOM_ERROR" appears in the output.
3. **Given** an error logged without classification, **Then** the classification field is optional and may be omitted.

---

### User Story 6 - Configurable Stack Traces (Priority: P2)

As a developer, I want stack traces captured when configured.

**Why this priority**: Stack traces are essential for debugging but have performance overhead. They should only be captured when explicitly needed.

**Independent Test**: Can be tested by enabling stack trace capture, logging an error, and verifying stack trace appears in output. Then disable and verify no stack traces are captured.

**Acceptance Scenarios**:

1. **Given** stack trace capture is disabled (default), **When** logging an error, **Then** no stack trace is allocated or stored.
2. **Given** stack trace capture is enabled, **When** logging an error, **Then** the stack trace is captured as structured data in the output.
3. **Given** stack trace extraction fails, **When** logging continues normally, **Then** the error is still logged without the stack trace (no panic).

---

### User Story 7 - Sensitive Value Redaction (Priority: P1)

As a security reviewer, I want sensitive values redacted before being logged.

**Why this priority**: Errors often contain sensitive data (passwords, tokens, PII). Logging such data creates security and compliance violations.

**Independent Test**: Can be tested by including sensitive-looking values in error fields and verifying they are redacted in the output.

**Acceptance Scenarios**:

1. **Given** a field containing "password" in the key, **When** the error is logged, **Then** the value is redacted before export.
2. **Given** a field containing "token" in the key, **When** the error is logged, **Then** the value is redacted before export.
3. **Given** the redaction pipeline fails, **When** logging continues, **Then** the error is still logged (fail-open for availability).

---

### Edge Cases

- What happens when error chain depth exceeds the configured maximum (default 64)?
  - **Answer**: Traversal stops at max depth with a warning indicator.
- How does system handle errors that cannot be stringified?
  - **Answer**: Falls back to a generic error representation with error type information.
- How does system handle context propagation failures?
  - **Answer**: Error logging continues without correlation IDs; no error is raised.
- How does system behave when all exporters fail?
  - **Answer**: Exporter failures are logged internally; error logging call completes successfully.
- What happens with nil errors?
  - **Answer**: Nil errors are handled gracefully with appropriate messaging.

## Requirements _(mandatory)_

### Functional Requirements

#### Core Error Logging

- **FR-001**: The logger SHALL expose dedicated error logging APIs (logger.error(err), logger.error_with_fields(err, fields), logger.error_with_context(err, context)).
- **FR-002**: Every error log SHALL produce a structured event with minimum fields: level, message, error, timestamp.
- **FR-003**: The logger SHALL extract the error message from any error interface.
- **FR-004**: The logger SHALL support error chains (error wrapping via fmt.Errorf %w, or custom chain implementations).
- **FR-005**: When an error chain exists, the logger SHALL record the complete chain as an ordered list.
- **FR-006**: The logger SHALL identify the root cause and expose it in the root_cause field.

#### Error Classification

- **FR-007**: The logger SHALL support error classification with standard types: validation, authentication, authorization, timeout, network, database, io, serialization, configuration, internal, external.
- **FR-008**: Classification SHALL be optional.
- **FR-009**: The logger SHALL support custom classifications as user-defined strings.

#### Structured Fields and Correlation

- **FR-010**: Error logs SHALL support arbitrary structured fields in addition to error-specific data.
- **FR-011**: Error logs SHALL support correlation identifiers: request_id, correlation_id, trace_id.
- **FR-012**: Correlation fields SHALL be automatically included when present in context.
- **FR-013**: Error logs SHALL support service metadata: service, version, environment, instance_id.

#### Stack Traces

- **FR-014**: The logger SHALL support capturing stack traces when the Go error implements stack capture.
- **FR-015**: Stack trace capture SHALL be configurable via configuration (enabled/disabled).
- **FR-016**: Stack traces SHALL be disabled by default to minimize performance impact.
- **FR-017**: When enabled, stack traces SHALL be exported as structured fields.
- **FR-018**: The logger SHALL support exporter-independent stack trace representation.

#### Redaction and Security

- **FR-019**: Error logs SHALL pass through the existing redaction pipeline.
- **FR-020**: Sensitive values SHALL be removed before export (passwords, tokens, API keys, etc.).

#### Source Location

- **FR-021**: Error logs SHALL support source location metadata: file, line, module, function.
- **FR-022**: Source metadata SHALL be optional (disabled by default).

#### Error Codes and Severity

- **FR-023**: The logger SHALL support attaching error codes (e.g., USER_NOT_FOUND, DB_TIMEOUT, INVALID_REQUEST).
- **FR-024**: Error codes SHALL be searchable fields in the output.
- **FR-025**: The logger SHALL support severity metadata independent of log level: low, medium, high, critical.
- **FR-026**: Severity SHALL be optional.

#### Exporter Compatibility

- **FR-027**: All exporters SHALL receive the same normalized error structure.
- **FR-028**: Console exporter SHALL render errors in human-readable form.
- **FR-029**: JSON exporters SHALL preserve the complete structured representation.
- **FR-030**: The logger SHALL support nested causes up to the configured max_chain_depth (default: 64).

#### Safety and Reliability

- **FR-031**: Cycles in error chains SHALL be detected and safely terminated.
- **FR-032**: The logger SHALL never panic while processing an error.
- **FR-033**: Error logging SHALL remain functional even if stack trace extraction fails.
- **FR-034**: Exporter failures SHALL NOT prevent error logging from completing.

#### Middleware Integration

- **FR-035**: The logger SHALL support error logging from middleware layers.
- **FR-036**: HTTP middleware SHALL be able to emit structured error logs.
- **FR-037**: gRPC middleware SHALL be able to emit structured error logs.

#### Subsystem Integration

- **FR-038**: Error events SHALL support sampling using the existing sampling subsystem.
- **FR-039**: Error events SHALL support formatting through the existing formatting subsystem.
- **FR-040**: Error events SHALL be compatible with all configured exporters.

#### Error Kind

- **FR-041**: The logger SHALL support an error "kind" field for stable categorization independent of classification.
- **FR-041.1**: Standard kinds SHALL include: business, system, security, validation, infrastructure.
- **FR-041.2**: Kind SHALL be optional and distinct from classification.
- **FR-041.3**: Kind provides stable categorization for dashboards and alerting regardless of error message content.

#### Panic Compatibility

- **FR-042**: The logger SHALL support panic-to-error conversion when provided by external middleware.
- **FR-042.1**: Error logging APIs SHALL handle recovered panic values gracefully.
- **FR-042.2**: Panic values (strings, error interfaces, or unknown types) SHALL be converted to structured errors.
- **FR-042.3**: This enables future recovery middleware without coupling to specific implementations.

#### Error Metrics Hook

- **FR-043**: The logger SHALL expose a provider-agnostic hook interface for error metrics collection.
- **FR-043.1**: Hook interface SHALL be opt-in and not require external dependencies.
- **FR-043.2**: Hooks SHALL receive: error count, error by classification, error by code.
- **FR-043.3**: Implementation SHALL allow custom metric backends (Prometheus, OpenTelemetry, custom).
- **FR-043.4**: Metrics hooks SHALL not block error logging completion (fire-and-forget).

#### Determinism

- **FR-044**: The normalized ErrorRecord generated from the same input error, configuration, and context SHALL be deterministic.
- **FR-044.1**: Exporter-specific rendering SHALL NOT modify the normalized ErrorRecord.
- **FR-044.2**: Identical error input, configuration, and context SHALL produce byte-equivalent ErrorRecords across all export paths (Console, JSON, File, HTTP, gRPC).
- **FR-044.3**: Deterministic ErrorRecord generation enables reliable test assertions without exporter-specific comparison logic.

## Key Entities

- **ErrorRecord**: The primary structured representation of a logged error containing:
  - message: The human-readable error message
  - error: The original error interface
  - root_cause: The identified root cause of the error chain
  - kind: Error kind (business, system, security, validation, infrastructure)
  - classification: Error category (validation, authentication, etc.)
  - code: Machine-readable error code
  - severity: Error severity level (low, medium, high, critical)
  - chain[]: Complete ordered list of errors in the chain (flat array)
  - stacktrace[]: Captured stack trace entries
  - stack_truncated: Boolean indicating if stack trace was truncated
  - request_id: Request correlation identifier
  - correlation_id: General correlation identifier
  - trace_id: Distributed trace identifier
  - timestamp: When the error was logged
  - fields{}: Additional structured fields
  - source_location: Where the error was logged (file, line, module, function)
  - service_metadata: Service identification (name, version, environment)

- **ErrorLoggingConfiguration**: Feature toggle and behavior configuration:
  - enabled: Master switch for error logging feature
  - capture_stacktrace: Whether to capture stack traces (default: false)
  - max_stack_depth: Maximum stack trace depth before truncation (default: 64)
  - include_source_location: Whether to include source metadata (default: false)
  - include_error_chain: Whether to capture full error chains (default: true)
  - include_root_cause: Whether to extract and include root cause (default: true)
  - max_chain_depth: Maximum chain traversal depth (default: 64)
  - classification_enabled: Whether to process classifications (default: true)
  - classification_mode: Classification mode - explicit, heuristic, or hybrid (default: explicit)
  - code_enabled: Whether to process error codes (default: true)
  - severity_enabled: Whether to process severity levels (default: true)
  - metrics_enabled: Whether to enable metrics hooks (default: false)

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: A simple error generates a structured error event with level, message, error type, and timestamp within the existing logger latency budget.
- **SC-002**: Error chains of up to 64 levels are fully preserved with root cause correctly identified.
- **SC-003**: Root cause extraction correctly identifies the deepest error in any chain.
- **SC-004**: Structured fields attached to errors remain available in all exporter outputs.
- **SC-005**: Correlation identifiers present in context are automatically propagated to error logs.
- **SC-006**: Stack traces are emitted only when explicitly enabled in configuration.
- **SC-007**: Sensitive field values (password, token, key, secret) are redacted before any export.
- **SC-008**: JSON exporters preserve all 15+ structured fields of the ErrorRecord.
- **SC-009**: Console exporter renders readable error output with key information visible.
- **SC-010**: Custom classifications work correctly and appear in the classification field.
- **SC-011**: Custom error codes work correctly and appear in the code field.
- **SC-012**: Cycles in error chains do not cause infinite recursion; detection terminates safely.
- **SC-013**: Exporter failures do not crash the application; error logging completes successfully.
- **SC-014**: HTTP and gRPC middleware-generated errors use the same ErrorRecord schema.
- **SC-015**: Sampling behavior for errors remains consistent with existing logger sampling.
- **SC-016**: Given identical error input and configuration, the generated ErrorRecord is byte-equivalent before exporter rendering.

### Open Questions Resolved

The following questions were answered during clarification:

- **Q1**: Should stack traces be captured automatically for all errors when enabled, or only for errors explicitly created through Kit Logger error helpers?
  - **Answer**: **Option A** - Capture stack traces for all errors when `capture_stacktrace=true`.
  - **Rationale**: More simple, deterministic, compatible with external errors, no required custom helpers.

- **Q2**: Should error classification be manually assigned only, or should the framework provide optional automatic classification heuristics?
  - **Answer**: **Option B** - Manual classification + optional heuristics (hybrid mode).
  - **Rationale**: Avoids coupling to magic rules but allows improved UX.
  - **Configuration**: `error_logging: classification: mode: hybrid` with order: explicit → heuristic → none.

- **Q3**: Should error chains be exported as a flat array or as a nested tree structure?
  - **Answer**: **Option A** - Flat array.
  - **Rationale**: Easier to export, index, and query in Elastic/Loki/OpenSearch.
  - **Example**: `{ "chain": ["DatabaseError", "ConnectionError", "TimeoutError"] }`

- **Q4**: Should source location capture require explicit macros/helpers, or attempt automatic caller detection when supported by the language/runtime?
  - **Answer**: **Option B** - Automatic caller detection when enabled.
  - **Rationale**: Only when `include_source_location: true`, no explicit macros required.

- **Q5**: What is the maximum allowed exported stack trace depth before truncation?
  - **Answer**: **64 frames** (aligned with `max_chain_depth: 64`).
  - **Additional**: When truncated, include `{ "stack_truncated": true }` indicator.
