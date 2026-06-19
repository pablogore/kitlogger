# Delta: Log Context & Enrichment

## ADDED Requirements

### Requirement: LogContext Construction

LogContext MUST support construction via both `new()` and `Default`, each producing an immutable empty context with zero attributes and no identifiers set.

#### Scenario: Empty context via new()

- GIVEN a `LogContext::new()` call
- WHEN the context is created
- THEN `attributes()` SHALL return an empty slice
- AND `correlation_id()`, `trace_id()`, `span_id()` SHALL all return `None`

#### Scenario: Empty context via Default

- GIVEN a `LogContext::default()` call
- WHEN the context is created
- THEN the result MUST be equivalent to `LogContext::new()`

### Requirement: Display Implementation

LogContext MUST implement `Display` for human-readable debugging, showing attributes and any set identifiers.

#### Scenario: Display empty context

- GIVEN a default `LogContext`
- WHEN `to_string()` is called
- THEN the output MUST include `LogContext` and SHOW no attributes

#### Scenario: Display enriched context

- GIVEN a LogContext with an attribute and a correlation_id
- WHEN `to_string()` is called
- THEN the output MUST contain the attribute key/value and the correlation_id

### Requirement: Attribute Enrichment

`with_attribute` MUST create a new LogContext with the given `LogAttribute` appended, or return `Err(ValidationError::EnrichmentError)` if the attribute name already exists.

#### Scenario: Adding attribute to context

- GIVEN an empty `LogContext`
- WHEN `with_attribute(LogAttribute::new("env", "prod"))` is called
- THEN the returned context SHALL have one attribute with name "env"
- AND the original context SHALL remain empty

#### Scenario: Duplicate attribute name rejected

- GIVEN a LogContext with an attribute named "env"
- WHEN `with_attribute(LogAttribute::new("env", "staging"))` is called
- THEN the result SHALL be `Err(ValidationError::EnrichmentError(..))`

### Requirement: Identifier Enrichment

`with_correlation_id`, `with_trace_id`, and `with_span_id` MUST each produce a new LogContext with the identifier set, returning `Result<Self, ValidationError>`. Setting an already-set identifier SHALL replace the previous value.

#### Scenario: Attach correlation identifier

- GIVEN an empty `LogContext`
- WHEN `with_correlation_id(CorrelationId::new("req-1"))` is called
- THEN the returned context SHALL have `correlation_id()` returning `Some("req-1")`

#### Scenario: Attach trace and span identifiers

- GIVEN a LogContext with a correlation_id
- WHEN `with_trace_id(TraceId::new("trace-abc"))` and `with_span_id(SpanId::new("span-42"))` are called
- THEN the resulting context SHALL have all three identifiers set

#### Scenario: Identifier idempotency (last-wins)

- GIVEN a LogContext with `correlation_id` set to "req-1"
- WHEN `with_correlation_id(CorrelationId::new("req-2"))` is called
- THEN the returned context SHALL have `correlation_id()` returning "req-2"

### Requirement: Enrichment Immutability

All enrichment methods MUST return a new `Self` and MUST NOT mutate the original LogContext.

#### Scenario: Original unchanged after enrichment

- GIVEN a LogContext `ctx` with one attribute
- WHEN `ctx.with_attribute(new_attr)` is called and discarded
- THEN `ctx.attributes()` SHALL still contain only the original attribute

### Requirement: EnrichmentError Variant

`ValidationError` MUST expose an `EnrichmentError(String)` variant for enrichment failures, with a Display output prefixed by `"Enrichment error: "`.

#### Scenario: EnrichmentError display

- GIVEN `ValidationError::EnrichmentError("duplicate name: env".into())`
- WHEN `to_string()` is called
- THEN the output SHALL be `"Enrichment error: duplicate name: env"`
