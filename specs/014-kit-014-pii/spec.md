# Feature Specification: KIT-014 PII Redaction

**Feature Branch**: `014-kit-014-pii`  
**Created**: 2026-06-10  
**Status**: Draft  
**Input**: User description: "KIT-014 PII Redaction — Automatic PII detection, masking, redaction and compliance controls for Kit Logger."

## Overview

This feature implements a deterministic, configurable, and high-performance PII (Personally Identifiable Information) redaction system for Kit Logger. The system allows applications to safely log operational data while preventing accidental exposure of sensitive information such as emails, phone numbers, national IDs, tax IDs, credit card numbers, API keys, access tokens, passwords, session identifiers, IP addresses, JWTs, and custom business-sensitive fields.

The redaction system operates as middleware within the logging pipeline, ensuring that sensitive data never reaches exporters. This provides automatic protection without requiring application developers to manually sanitize every log message.

## User Scenarios & Testing _(mandatory)_

### User Story 1 - Automatic PII Detection and Redaction (Priority: P1)

As a platform engineer, I want sensitive values automatically redacted from logs so that logs are safe by default without requiring manual intervention.

**Why this priority**: This is the core value proposition of the feature. Without automatic redaction, developers must manually sanitize every sensitive field, which is error-prone and easily forgotten.

**Independent Test**: Configure the logger with PII redaction enabled, emit a log message containing an email address, and verify the output contains the redacted version instead of the original email.

**Acceptance Scenarios**:

1. **Given** a configured logger with PII redaction enabled, **When** a log message contains "john.doe@example.com", **Then** the output must contain "[EMAIL_REDACTED]" or equivalent placeholder instead of the original email.
2. **Given** a log event with structured attributes containing PII, **When** the event is processed by the logger, **Then** sensitive fields are redacted before reaching any exporter.
3. **Given** the same input value appears in multiple log entries, **When** processed through the redaction system, **Then** the output is always identical (deterministic).

---

### User Story 2 - Deterministic Masking Policies (Priority: P1)

As a compliance officer, I want deterministic masking policies so that all environments behave consistently and log audits can reliably verify what data was protected.

**Why this priority**: Compliance requirements demand reproducibility. If redaction produces different outputs for the same input, it becomes impossible to verify protection coverage and analyze historical data.

**Independent Test**: Process the same input string through the redaction system 100 times and verify the output is identical in all 100 iterations.

**Acceptance Scenarios**:

1. **Given** the input string "user@example.com", **When** processed through the redaction system, **Then** the output is always the same redacted version.
2. **Given** identical log messages across different environments (dev, staging, production), **When** each environment uses the same PII configuration, **Then** the redaction output is byte-for-byte identical.
3. **Given** no random elements or seeds in the configuration, **When** redaction is deterministic, **Then** there is no entropy source or randomization in the masking algorithm.

---

### User Story 3 - Hashing Strategy for Correlation (Priority: P2)

As a security engineer, I want to hash selected sensitive values so that correlation remains possible between log entries without exposing the original sensitive data.

**Why this priority**: Complete redaction prevents analysts from tracking user behavior across sessions. Hashing preserves the ability to correlate related events while protecting the original value.

**Independent Test**: Configure email redaction to use SHA256 hashing, log an email address, and verify the output contains a hash prefix (e.g., "EMAIL_SHA256:") followed by a hexadecimal hash value that is consistent for the same input.

**Acceptance Scenarios**:

1. **Given** the input "user@example.com" with SHA256 hashing enabled, **When** processed, **Then** the output is "EMAIL_SHA256:<64-character-hex-hash>" where the hash is the SHA256 of the original value.
2. **Given** the same input hashed multiple times, **When** using deterministic hashing, **Then** the hash output is identical each time.
3. **Given** a sensitive value that requires correlation, **When** hashing is configured, **Then** the original value is never present in any output.

---

### User Story 4 - Field-Based Structured Log Redaction (Priority: P1)

As an application developer, I want field-based redaction for structured logs so that I can configure which JSON or structured attributes should be automatically protected.

**Why this priority**: Structured logging is the primary pattern in modern applications. Developers need to declare which fields are sensitive without manually processing each log line.

**Independent Test**: Configure the redaction system with field patterns (e.g., "password", "token", "api_key"), emit a structured log with these fields, and verify that only the sensitive fields are redacted while non-sensitive fields remain unchanged.

**Acceptance Scenarios**:

1. **Given** a structured log with fields `{"email": "john@example.com", "name": "John", "password": "secret123"}`, **When** processed, **Then** the output is `{"email": "[EMAIL_REDACTED]", "name": "John", "password": "[REDACTED]"}`.
2. **Given** a field pattern configured as "token", **When** a field named "access_token" or "refresh_token" appears in structured log data, **Then** it is automatically redacted.
3. **Given** nested object structures in logs, **When** nested fields contain sensitive data, **Then** the redaction system recursively processes all depth levels.

---

### User Story 5 - Custom Detection Rules (Priority: P2)

As a security engineer, I want to register custom detection rules so that organization-specific sensitive data patterns can be protected.

**Why this priority**: Built-in detectors cover common PII types, but organizations often have custom identifiers (employee IDs, account numbers, membership IDs) that require protection beyond standard types.

**Independent Test**: Register a custom regex pattern for employee IDs (e.g., "EMP-######"), emit a log containing such an ID, and verify it is redacted according to the custom rule.

**Acceptance Scenarios**:

1. **Given** a custom regex rule registered for "employee_id" with pattern "EMP-[0-9]{6}", **When** a log contains "EMP-123456", **Then** it is redacted to "[EMPLOYEE_ID]".
2. **Given** a custom field name rule for "customer_account_number", **When** log fields contain this key, **Then** the value is redacted regardless of content.
3. **Given** multiple custom rules, **When** processing a log message, **Then** all applicable rules are applied in registration order.

---

### User Story 6 - Nested Object Processing (Priority: P2)

As a developer, I want nested JSON objects in log fields to be recursively scanned and redacted so that deeply structured data is fully protected.

**Why this priority**: Modern APIs use complex nested structures. Simple top-level field matching would leave sensitive data unprotected in nested objects.

**Independent Test**: Configure redaction and emit a structured log with nested objects containing emails, verify all levels are redacted.

**Acceptance Scenarios**:

1. **Given** the input `{"user": {"profile": {"email": "john@example.com"}}}`, **When** redaction runs, **Then** the nested email is redacted: `{"user": {"profile": {"email": "[EMAIL_REDACTED]"}}}`.
2. **Given** arrays containing sensitive data, **When** processing, **Then** array elements are processed individually.
3. **Given** deeply nested objects (10+ levels), **When** processing, **Then** all levels are processed without stack overflow or performance degradation.

---

### User Story 7 - Plain Text Message Scanning (Priority: P2)

As a developer, I want plain text log messages scanned for PII so that unstructured log output is also protected.

**Why this priority**: Not all logging uses structured formats. Free-text messages are common and can accidentally contain sensitive information.

**Independent Test**: Emit a plain text log message containing "Payment received from john@example.com", verify the output shows "Payment received from [EMAIL_REDACTED]".

**Acceptance Scenarios**:

1. **Given** a log message "Contact john.doe@example.com for support", **When** plain text scanning is enabled, **Then** the email in the message is replaced with "[EMAIL_REDACTED]".
2. **Given** multiple PII types in a single message, **When** scanning, **Then** each detected type is replaced with its corresponding redaction marker.
3. **Given** a message without any PII, **When** scanned, **Then** the message remains unchanged.

---

### User Story 8 - Message Scanning with Full, Type, and Partial Redaction (Priority: P3)

As an operator, I want configurable redaction strategies so that different PII types can use different protection levels based on operational needs.

**Why this priority**: Different data types have different sensitivity levels and operational needs. Emails may need type-specific redaction while internal employee IDs might need complete removal.

**Independent Test**: Configure different strategies for different PII types and verify each strategy produces its expected output format.

**Acceptance Scenarios**:

1. **Given** email redaction using type redaction strategy, **When** input is "user@example.com", **Then** output is "[EMAIL_REDACTED]".
2. **Given** phone redaction using partial masking strategy, **When** input is "555-123-4567", **Then** output is "555-***-4567" or similar partial mask.
3. **Given** a password field using full redaction strategy, **When** any value appears, **Then** output is "[REDACTED]".

---

### Edge Cases

- **Empty or null values**: When redaction encounters empty strings or null values, the system must return them unchanged without error.
- **Malformed input**: When regex patterns cannot match or input is not valid UTF-8, the system must not panic and must return the original value.
- **Detector failure**: If any detector encounters an error, the system must continue processing with other detectors and must not expose raw values.
- **Boundary conditions**: Extremely long strings (over 1MB) must either be fully processed or gracefully truncated with a safe default.
- **Mixed content**: Strings containing both sensitive and non-sensitive content should have only the sensitive portions replaced.
- **Configuration conflicts**: When multiple rules could match the same field, the most specific rule should take precedence.
- **Performance under load**: When processing thousands of logs per second, the redaction system must not become a bottleneck.

## Requirements _(mandatory)_

### Functional Requirements

#### Detection Engine

- **FR-001**: The system MUST provide built-in detectors for: Email (RFC 5322 patterns), Phone (international formats), IPv4, IPv6, JWT (RFC 7519), Credit Card (Luhn-valid), Password, API Key, Access Token, National ID patterns, Tax ID patterns.
- **FR-002**: The detector system MUST be extensible, allowing users to register custom detectors.
- **FR-003**: Each detector MUST have a unique identifier (e.g., "email", "phone", "ipv4", "jwt").

#### Detection Modes

- **FR-004**: The system MUST support pattern-based detection using regular expressions for unstructured text content.
- **FR-005**: The system MUST support field-based detection using key name matching for structured log attributes.
- **FR-006**: Field detection MUST support pattern matching including exact match, case-insensitive match, and suffix prefix patterns.

#### Redaction Strategies

- **FR-007**: The system MUST support full redaction where the entire value is replaced with "[REDACTED]".
- **FR-008**: The system MUST support type-specific redaction where each PII type has its own marker (e.g., "[EMAIL_REDACTED]", "[PHONE_REDACTED]").
- **FR-009**: The system MUST support partial masking where portions of the value are hidden (e.g., first characters visible, last characters masked).
- **FR-010**: The system MUST support hashing strategy using SHA-256 with a type prefix (e.g., "EMAIL_SHA256:abc123...").
- **FR-011**: The hashing strategy MUST be deterministic — the same input always produces the same hash output.

#### Structured Logging Protection

- **FR-012**: PII detection MUST work on structured log attributes (key-value pairs attached to log events).
- **FR-013**: PII detection MUST work on metadata attached to log events.
- **FR-014**: PII detection MUST work on context values propagated with log events.

#### Message Scanning

- **FR-015**: The system MUST scan plain text log messages for PII patterns.
- **FR-016**: Plain text scanning MUST replace detected PII with the configured redaction marker.

#### Nested Object Support

- **FR-017**: The system MUST recursively traverse nested JSON objects in structured log data.
- **FR-018**: The system MUST process array elements individually for PII detection.
- **FR-019**: Nested processing MUST handle at least 10 levels of depth without performance degradation.

#### Custom Rules

- **FR-020**: Users MUST be able to register custom regex-based detection rules.
- **FR-021**: Users MUST be able to register custom field name matching rules.
- **FR-022**: Custom rules MUST be evaluated alongside built-in rules.
- **FR-023**: Custom rules MUST support custom replacement patterns.

#### Pipeline Integration

- **FR-024**: PII redaction MUST be implemented as middleware in the logging pipeline.
- **FR-025**: The redaction pipeline order MUST be: Application → Logger → PII Middleware → Formatter → Exporter.
- **FR-026**: Exporters MUST never receive raw PII — all redaction MUST occur before formatting.

#### Fail-Safe Behavior

- **FR-027**: The redaction pipeline MUST remain operational when detector failures occur. Built-in detectors MUST return `Result<T, DetectorError>` instead of relying on panic-based control flow. Detector errors MUST be handled as ordinary failures and converted into fail-safe redaction behavior. Detection failures MUST NOT expose raw sensitive values and MUST NOT terminate the logging pipeline.
- **FR-027.1**: All built-in detectors MUST use Result-based error handling. `panic!` MUST NOT be used for expected detection failures. Examples of conditions that MUST return `DetectorError` values: malformed input, invalid UTF-8, parsing failures, unsupported formats.
- **FR-027.2**: Custom detector implementations MAY fail independently. The implementation MUST isolate custom detector failures from the rest of the pipeline. Failures in one detector MUST NOT affect: other detectors, middleware execution, formatter execution, or exporter execution.
- **FR-027.3**: When detector execution fails: 1) raw values MUST NOT be exposed, 2) exporters MUST remain protected, 3) fail-safe redaction MUST be applied. Preferred output is `[REDACTED]` rather than leaking the original value.
- **FR-027.4**: The implementation SHOULD use Result-based error handling as the primary mechanism. `catch_unwind` MAY be used only at plugin boundaries or third-party detector boundaries where panic isolation is required. `catch_unwind` MUST NOT be part of the normal detector execution path.
- **FR-028**: When redaction fails, the system MUST default to full redaction "[REDACTED]" rather than exposing raw values.
- **FR-029**: The logging system MUST continue functioning even when PII detection encounters errors.

#### Rule Precedence

- **FR-030**: The redaction engine MUST define deterministic rule precedence.
- **FR-030.1**: When multiple rules match the same value, the following order MUST be applied:
  1. Explicit Field Rule
  2. Custom Detector Rule
  3. Built-in Detector Rule
  4. Global Default Rule
- **FR-030.2**: The most specific rule MUST always win.
- **FR-030.3**: Example: Given an email field configured with SHA256 hashing AND built-in email detector configured with type redaction, SHA256 hashing MUST be applied.
- **FR-030.4**: The precedence model MUST be deterministic and fully documented.

#### Fast Path Detection

- **FR-031**: The redaction engine MUST optimize detection for known sensitive field names.
- **FR-031.1**: Before executing regex-based detection, the engine MUST evaluate field-name rules.
- **FR-031.2**: Examples of fast-path fields that MUST be redacted immediately without regex evaluation: password, passwd, secret, token, access_token, refresh_token, api_key, authorization, bearer_token.
- **FR-031.3**: This optimization MUST reduce CPU overhead for structured logging workloads.
- **FR-031.4**: Regex scanning should only be executed when field-based detection does not apply.

#### Allowlist Mode

- **FR-032**: The system MUST support two protection modes:
- **FR-032.1**: **Blacklist Mode (Default)**: Only configured sensitive fields and detectors are redacted.
- **FR-032.2**: **Allowlist Mode**: All fields are considered sensitive unless explicitly allowed.
- **FR-032.3**: In Allowlist Mode, only fields explicitly listed in `allowed_fields` (e.g., user_id, request_id, status) are preserved; all other fields MUST be redacted.
- **FR-032.4**: Allowlist mode MUST be deterministic and compatible with structured logging.
- **FR-032.5**: The selected mode MUST be configurable at startup.

#### Secret Detection

- **FR-033**: The system MUST distinguish between PII, Secrets, and Credentials.
- **FR-033.1**: Built-in secret detectors MUST include: AWS Access Keys, AWS Secret Keys, GitHub Tokens, GitLab Tokens, OpenAI API Keys, Anthropic API Keys, Google API Keys, Bearer Tokens, OAuth Tokens, JWT Tokens.
- **FR-033.2**: Secret detection MUST execute before export.
- **FR-033.3**: Secrets MUST never appear in any exporter output.
- **FR-033.4**: Secret detection MUST support custom extensions.

#### Redaction Metrics

- **FR-034**: The system MUST expose counters for detected and redacted sensitive data.
- **FR-034.1**: Counters MUST include: email_detected, phone_detected, jwt_detected, api_key_detected, password_detected, and corresponding redacted variants.
- **FR-034.2**: Counter values MUST be aggregated by DetectionCategory (PII, SECRET, CREDENTIAL).
- **FR-034.3**: The counters MUST be exportable through the existing metrics infrastructure.
- **FR-034.4**: The system MAY emit internal security events when sensitive data is detected, but this is optional.
- **FR-034.5**: If events are emitted, they MUST NOT contain the original sensitive value.

#### Large Payload Processing (Future Consideration)

- **FR-035**: The redaction engine SHOULD support large payload processing as a future enhancement.
- **FR-035.1**: Future implementation SHOULD avoid loading entire payloads into memory when possible.
- **FR-035.2**: Future implementation SHOULD handle payloads larger than 1 MB without memory amplification attacks.
- **FR-035.3**: Large payload processing MUST preserve deterministic redaction behavior.
- **FR-035.4**: This requirement depends on Kit Logger adding payload/body logging capabilities (future work).

#### Redaction Cache

- **FR-036**: The redaction engine SHOULD cache deterministic redaction results.
- **FR-036.1**: The cache MUST preserve deterministic behavior — the same input always produces the same cached output.
- **FR-036.2**: The cache MUST support bounded memory usage with eviction policies (LRU, TTL, or size-based).
- **FR-036.3**: First occurrence of a value triggers regex evaluation and hashing.
- **FR-036.4**: Subsequent occurrences of the same value use cache lookup instead of recomputing.
- **FR-036.5**: Cache hit rate MUST be measurable and exposed via metrics.
- **FR-036.6**: The cache optimization is particularly beneficial for structured logging with repeated values (same user, same token, same email).
- **FR-036.7**: Cache implementations MUST be pluggable. The default implementation SHOULD be memory-based. Alternative implementations MAY include: moka, dashmap-based, or custom lock-free caches. The redaction engine MUST depend on traits rather than concrete cache implementations to allow for dependency injection and testing.

#### Detection Categories

- **FR-037**: The system MUST define DetectionCategory as a first-class concept.
- **FR-037.1**: Categories MUST include: PII, SECRET, CREDENTIAL, CUSTOM.
- **FR-037.2**: PII category: Email, Phone, IPv4, IPv6, National ID, Tax ID, Credit Card.
- **FR-037.3**: SECRET category: JWT, Bearer Tokens, OAuth Tokens, API Keys.
- **FR-037.4**: CREDENTIAL category: Passwords, Access Tokens, AWS Keys, GitHub/GitLab Tokens, OpenAI/Anthropic/Google API Keys.
- **FR-037.5**: CUSTOM category: User-defined detectors with custom category assignment.
- **FR-037.6**: DetectionCategory MUST be used to organize: metrics, audit logs, policy rules, and exporter filtering.

### Non-Functional Requirements

#### NFR-001 Rust Error Handling Model

The implementation MUST follow idiomatic Rust error handling principles.

**Error Hierarchy**:

- **Expected Failure** → `Result<T, DetectorError>`
- **Unexpected Internal Bug** → `panic!`
- **Third-Party Plugin Failure** → Optional `catch_unwind` boundary

`panic!` MUST NOT be used as a normal control-flow mechanism.

#### NFR-002 Detector Trait Design

Detector interfaces SHOULD be designed around Result-based execution.

Example:

```
pub trait Detector {
    fn detect(
        &self,
        value: &str,
    ) -> Result<Vec<Detection>, DetectorError>;
}
```

The trait design MAY evolve, but detector execution MUST remain deterministic and error-aware.

#### NFR-003 Panic-Free Hot Path

The logging hot path SHOULD avoid panic recovery mechanisms.

- Built-in detectors SHOULD execute without `catch_unwind` overhead.
- Panic isolation SHOULD only exist at extension/plugin boundaries.

The implementation SHOULD optimize for: low allocation, predictable latency, deterministic execution, and zero panic overhead in normal operation.

#### NFR-004 Deterministic Iteration

Detector execution order MUST be deterministic.

Rule evaluation order MUST be deterministic.

Cache behavior MUST NOT affect redaction output.

Given identical inputs and configuration:

- detection order
- strategy selection
- redaction output

MUST be reproducible.

HashMap iteration order MUST NOT affect behavior. The implementation MUST use ordered collections (e.g., `BTreeMap`, `IndexMap`) or explicit sorting when iteration order matters. Hash randomization (enabled by default in Rust's standard library for security) MUST NOT affect detection or redaction results.

#### NFR-005 Regex Engine Selection

The implementation SHOULD use Rust's `regex` crate.

Regex implementations with catastrophic backtracking characteristics MUST be avoided.

Detection complexity SHOULD remain linear relative to input size.

Custom user-provided regex patterns SHOULD be validated for complexity before compilation to prevent denial-of-service attacks.

### Key Entities

- **PIIDetector**: A trait that defines detection logic for a specific PII type. Contains a unique identifier, optional regex pattern, and detection method. Returns `Result<Vec<Detection>, DetectorError>`.
- **DetectorError**: Represents deterministic detector failures. Variants include: `InvalidInput` (malformed input), `ParseError` (parsing failure), `UnsupportedFormat` (unsupported format), `DetectorMisconfigured` (misconfiguration), and `InternalError` (unexpected internal error). All variants MUST be handled via `Result<T, DetectorError>` rather than `panic!`.
- **RedactionStrategy**: Defines how detected PII is protected. Types include: Full, TypeSpecific, PartialMask, Hash.
- **PIIRule**: User-defined or built-in rule combining a detector with a strategy and configuration.
- **PIIConfig**: Configuration container specifying which detectors are enabled, what strategies to use per type, and any custom rules.
- **RedactionMiddleware**: Middleware component that intercepts log events and applies redaction before passing to the next pipeline stage.
- **SecretDetector**: Specialized detector for secret/credential patterns (AWS keys, GitHub tokens, API keys). Implements the Detector trait.
- **ComplianceEvidence**: Data structure exposing redaction configuration and operational metrics for audit purposes.
- **DetectionCategory**: Enum categorizing detectors: PII, SECRET, CREDENTIAL, CUSTOM.
- **RedactionCache**: Trait for caching deterministic redaction results. Default implementation is memory-based with LRU/TTL eviction. Pluggable implementations MAY include moka, dashmap-based, or custom lock-free caches.
- **CacheConfig**: Configuration for cache behavior including max size, TTL, and eviction policy.

#### Architecture Components

The implementation MUST be organized as a standalone crate at `crates/pii/`:

```
crates/pii/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Crate root, public exports
│   ├── error.rs         # DetectorError enum and error types
│   ├── detector.rs      # Detector trait and built-in implementations
│   ├── registry.rs      # Detector registration and lookup
│   ├── redactor.rs      # Core redaction execution
│   ├── strategy.rs     # Redaction strategy implementations
│   ├── scanner.rs      # Text and structured data scanning
│   ├── middleware.rs    # Logger middleware integration
│   ├── config.rs       # Configuration management
│   ├── fastpath.rs     # Field-name optimization for known fields
│   ├── secrets.rs      # Secret detection engine
│   ├── policy.rs       # Rule precedence resolution
│   ├── evidence.rs     # Compliance evidence generation
│   ├── cache.rs        # Cache trait and implementations
│   └── categories.rs   # DetectionCategory enum
```

**Module Responsibilities**:

| Module | Responsibility |
|--------|---------------|
| `error.rs` | `DetectorError` enum and error handling traits |
| `detector.rs` | `Detector` trait and built-in PII detectors |
| `registry.rs` | Detector registration, lookup, and management |
| `redactor.rs` | Core redaction execution logic |
| `strategy.rs` | Strategy trait and implementations (Full, Type, Partial, Hash) |
| `scanner.rs` | Text scanning, pattern matching, structured data traversal |
| `middleware.rs` | Logger middleware integration (`PIIMiddleware`) |
| `config.rs` | Configuration structs and deserialization |
| `fastpath.rs` | Fast-path field-name matching for known sensitive fields |
| `secrets.rs` | Secret and credential detection (AWS, GitHub, API keys) |
| `policy.rs` | Rule precedence resolution and conflict handling |
| `evidence.rs` | Compliance evidence and metrics generation |
| `cache.rs` | Cache trait and default LRU/TTL implementation |
| `categories.rs` | `DetectionCategory` enum and categorization logic |

**Architecture Note**: Built-in detectors are trusted components and MUST use Result-based error handling. Custom detectors are untrusted extensions and MAY be isolated through dedicated execution boundaries. The core redaction engine MUST remain panic-free under normal operation.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: When PII redaction is enabled, 100% of detected sensitive data (emails, phone numbers, credit cards, passwords, tokens) MUST be redacted before reaching any exporter. Verified by automated tests that verify no sensitive patterns survive redaction.
- **SC-002**: Given identical input strings processed multiple times, the redaction output MUST be byte-for-byte identical across all iterations. Verified by 1000-iteration deterministic test with hash comparison.
- **SC-003**: When hashing strategy is configured for emails, the output MUST contain "EMAIL_SHA256:" prefix followed by a 64-character hexadecimal SHA-256 hash that corresponds to the input value. Verified by test comparing hash output to known hash of test value.
- **SC-004**: Structured log attributes containing sensitive field names (password, token, api_key, secret) MUST be redacted regardless of value content. Verified by tests with various field patterns.
- **SC-005**: Nested JSON objects at 10+ levels of depth MUST be fully processed without stack overflow or timeout. Verified by stress test with deeply nested structures.
- **SC-006**: Plain text messages containing PII MUST have the sensitive portions replaced. Verified by tests with mixed content strings.
- **SC-007**: Custom regex rules MUST be evaluated and applied correctly. Verified by tests registering custom patterns and confirming detection.
- **SC-008**: When any detector returns `Result::Err` (error), the system MUST NOT expose raw values and MUST continue processing. Verified by tests that verify detector error handling continues logging and applies fail-safe redaction.
- **SC-009**: PII redaction processing overhead MUST be less than 10% at P95 compared to logger without redaction, under load of 10,000 logs/second. Verified by benchmark tests.

### Measurable Outcomes (continued)

- **SC-010**: The system MUST expose evidence that redaction is active and functioning.
- **SC-010.1**: Evidence MUST include: enabled detectors, enabled strategies, policy version, configuration checksum, redaction counters.
- **SC-010.2**: Example metrics: pii_redacted_total, secret_redacted_total, hash_operations_total, policy_version.
- **SC-010.3**: The evidence MUST be usable for SOC2 audits, ISO 27001 audits, HIPAA reviews, and PCI-DSS assessments.
- **SC-010.4**: The evidence MUST NOT expose any sensitive values.

### SC-011 Rust Error Model Compliance

All built-in detectors MUST compile and operate without panic-based error handling.

**Verification criteria**:

- Detector failures return `Result::Err`
- Logging continues after detector errors
- Sensitive values remain protected
- Exporters never receive unredacted data

### Security Verification Criteria

- **SVC-001**: Passwords MUST NEVER appear in any log output. Verified by automated tests attempting to log known password values.
- **SVC-002**: API keys and tokens MUST NEVER appear in any log output. Verified by automated tests with various token formats.
- **SVC-003**: JWT payloads MUST NEVER appear in any log output. Verified by tests logging JWT strings.
- **SVC-004**: Credit card numbers MUST be masked or redacted. Verified by tests with various card number formats.
- **SVC-005**: AWS credentials MUST NEVER appear in exported logs. Verified by tests with AWS access key and secret key patterns.
- **SVC-006**: GitHub, GitLab and OpenAI API keys MUST NEVER appear in exported logs. Verified by tests with specific token format patterns.
- **SVC-007**: OAuth access tokens MUST NEVER appear in exported logs. Verified by tests with OAuth token patterns.
- **SVC-008**: Bearer tokens MUST NEVER appear in exported logs. Verified by tests with Bearer token header patterns.
- **SVC-009**: Custom secret detectors MUST provide the same protection guarantees as built-in detectors. Verified by tests registering custom secret patterns.

---

[Return to specification quality checklist for validation]
