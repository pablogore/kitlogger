# Data Model: Structured Log Domain Model

**SPEC_ID**: `003-structured-logging-core-as-01-structured-log-domain-model`

**Status**: Draft

---

## Entities

### LogRecord

Canonical log entry. Immutable after construction.

| Field | Type | Validation | Description |
|-------|------|------------|-------------|
| `timestamp` | `SystemTime` | Must be present | UTC-referenced point-in-time |
| `severity` | `Severity` | Must be canonical level | Log severity classification |
| `message` | `String` | Must be non-empty | Log message text |
| `attributes` | `Vec<LogAttribute>` | Trusts pre-validated LogAttribute instances | Structured key-value data |

**Constraints**:
- No public mutation methods
- All fields set at construction
- Construction returns `Result<LogRecord, ValidationError>`

### Severity

Enumeration of six canonical severity levels.

| Variant | Order |
|---------|-------|
| `Trace` | 0 (least severe) |
| `Debug` | 1 |
| `Info` | 2 |
| `Warn` | 3 |
| `Error` | 4 |
| `Fatal` | 5 (most severe) |

**Constraints**:
- Implements `PartialOrd` for severity comparison
- Only six variants; no non-canonical variants
- Implements `Display` and `FromStr`

### LogAttribute

A named key-value pair of structured data.

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Attribute name matching `[a-z][a-z0-9._]{0,63}` |
| `value` | `LogAttributeValue` | Strongly typed attribute value |

**Constraints**:
- Name must not conflict with reserved LogRecord field names
- Name max 64 characters
- Name pattern: `^[a-z][a-z0-9._]{0,63}$`

### LogAttributeValue

Strongly typed wrapper for attribute values.

| Variant | Inner Type | Description |
|---------|------------|-------------|
| `String` | `String` | UTF-8 string value |
| `Integer` | `i64` | Signed 64-bit integer |
| `Float` | `f64` | 64-bit floating point |
| `Boolean` | `bool` | Boolean value |
| `Timestamp` | `SystemTime` | Point in time (UTC) |
| `Array` | `Vec<LogAttributeValue>` | Homogeneous array of values |

**Constraints**:
- No nested object types (only flat scalar types)
- Array variant enforces homogeneous element types at construction
- Total of 6 supported value types

### CorrelationId

Opaque string identifier for cross-service correlation.

| Method | Signature | Description |
|--------|-----------|-------------|
| Constructor | `fn new(id: String) -> Self` | Wrap a string identifier |
| `as_str` | `fn as_str(&self) -> &str` | Borrow the inner string |
| `Display` | `fn fmt(&self, f: &mut Formatter) -> fmt::Result` | Format the identifier |

### TraceId

Opaque string identifier for distributed trace association.

Same interface pattern as `CorrelationId`.

### SpanId

Opaque string identifier for span-level identification within a trace.

Same interface pattern as `CorrelationId`.

### ValidationError

Enumeration of all domain validation failure modes.

| Variant | Description |
|---------|-------------|
| `EmptyMessage` | Message string is empty |
| `InvalidSeverity` | Severity level not recognized (applicable via `Severity::from_str` parsing) |
| `InvalidAttributeName(String)` | Attribute name violates naming constraints |
| `InvalidAttributeValue(String)` | Attribute value violates type constraints |

## Relationships

```
LogRecord
├── 1 timestamp: SystemTime
├── 1 severity: Severity
├── 1 message: String
└── 0..* attributes: LogAttribute
                  └── 1 value: LogAttributeValue (String | Integer | Float | Boolean | Timestamp | Array)

CorrelationId (standalone, referenced by AS-02 LogContext)
TraceId (standalone, referenced by AS-02 LogContext)
SpanId (standalone, referenced by AS-02 LogContext)
```

## State Transitions

LogRecord has no state transitions — it is immutable after construction. The only transition is:

```
(no record) ──► LogRecord (constructed via LogRecord::new)
```

All validation occurs at construction time. After construction, all fields are read-only.

## Validation Rules

| Rule | Source | Condition | Error |
|------|--------|-----------|-------|
| Message non-empty | `LogRecord::new` | `message.is_empty()` | `ValidationError::EmptyMessage` |
| Attribute name pattern | `LogAttribute::new` | `!name_matches_pattern(name)` | `ValidationError::InvalidAttributeName(name)` |
| Attribute name reserved | `LogAttribute::new` | `reserved_fields.contains(name)` | `ValidationError::InvalidAttributeName(name)` |
| Attribute value flat | `LogAttributeValue` constructors | nested object detected | `ValidationError::InvalidAttributeValue` |
| Array homogeneous | `LogAttributeValue::array` | mixed element types | `ValidationError::InvalidAttributeValue` |
| Invalid severity | `Severity::from_str` | unrecognized severity string | `ValidationError::InvalidSeverity` |
