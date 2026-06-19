# Research: Structured Log Domain Model

**SPEC_ID**: `003-structured-logging-core-as-01-structured-log-domain-model`

**Status**: Draft

---

## Resolved Decisions

### D-1: Severity representation

**Decision**: Use a plain Rust enum with six canonical variants (`Trace`, `Debug`, `Info`, `Warn`, `Error`, `Fatal`) implementing `PartialOrd` for severity ordering.

**Rationale**: A plain enum guarantees exhaustiveness checking, provides natural ordering via `PartialOrd` derive, and requires zero external dependencies. No non-canonical severity values need to be representable.

**Alternatives considered**: N/A (domain model constraint)

### D-2: Identifier types (CorrelationId, TraceId, SpanId)

**Decision**: Use newtype wrappers over `String` with `Display`, `AsRef<str>`, and `From<String>` implementations.

**Rationale**: Newtypes provide type safety while remaining zero-cost abstractions. String inner allows variable-length opaque identifiers.

**Alternatives considered**: UUID-specific types — rejected because identifiers are opaque strings with no required internal structure per spec assumption.

### D-3: LogAttributeValue representation

**Decision**: Use an enum with variants: `String(String)`, `Integer(i64)`, `Float(f64)`, `Boolean(bool)`, `Timestamp(SystemTime)`, `Array(Vec<LogAttributeValue>)`.

**Rationale**: Enum variants match the spec's supported value types exactly. Homogeneous arrays are enforced via runtime validation in the `Array` variant constructor.

**Alternatives considered**: 
- `serde_json::Value` — rejected: introduces unnecessary dependency, allows nested objects which are prohibited.
- Traits — rejected: needs object safety and this is a pure data model, not a behavioral hierarchy.

### D-4: Timestamp representation

**Decision**: Use `std::time::SystemTime` for timestamp fields.

**Rationale**: `SystemTime` is a standard library type, requiring zero external dependencies. The spec states "Timestamps use UTC as the reference timezone; formatting for display is a downstream concern" — the core domain model only stores the timestamp, it does not format it. UTC interpretation is documented, not enforced at the type level.

**Alternatives considered**: 
- `chrono::DateTime<Utc>` — rejected: introduces an undeclared dependency; formatting is a downstream concern per spec.
- `time` crate — rejected: unnecessary dependency for a pure domain model.

### D-5: Attribute naming validation

**Decision**: Implement naming validation as a function `validate_attribute_name(name: &str) -> Result<(), ValidationError>` that checks the regex pattern `^[a-z][a-z0-9._]{0,63}$` and reserved field name exclusion.

**Rationale**: Independent validation function enables testing in isolation and reuse across multiple construction paths.

**Alternatives considered**: N/A (mapped directly from spec requirement)

### D-6: LogRecord construction

**Decision**: Use a `TryFrom`-based builder pattern or named constructor `LogRecord::new(...)` that returns `Result<LogRecord, ValidationError>`.

**Rationale**: Construction-time validation requires fallible construction. A named constructor is simpler than a full builder pattern since LogRecord has a fixed set of required fields.

**Alternatives considered**: Builder pattern — rejected: LogRecord has no optional fields (only required: timestamp, severity, message). Builder adds complexity without benefit.

### D-7: Immutability strategy

**Decision**: Store all fields as private with public accessor methods returning references (`&str`, `&Severity`, `&[LogAttribute]`).

**Rationale**: Standard Rust immutability pattern. No mutation methods are exposed, enforcing compile-time immutability.

**Alternatives considered**: N/A (standard practice)

### D-8: ValidationError type

**Decision**: Define a `ValidationError` enum with variants covering all validation failure modes: `EmptyMessage`, `InvalidSeverity`, `InvalidAttributeName(String)`, `InvalidAttributeValue(String)`.

**Rationale**: A dedicated error type enables callers to handle specific validation failures. Using `thiserror` for derive is avoided (undeclared dependency); manual `Display` and `Error` implementations are used instead.

**Alternatives considered**: 
- `String` error messages — rejected: loses type information.
- `thiserror` — rejected: undeclared dependency per tech-stack.

## Technology Compliance

All technologies referenced in this document are declared in `tech-stack.yaml` or are std library features. No undeclared technology violations.
