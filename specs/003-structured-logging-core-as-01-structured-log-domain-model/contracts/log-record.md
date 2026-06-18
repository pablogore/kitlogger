# LogRecord Contract

**SPEC_ID**: `003-structured-logging-core-as-01-structured-log-domain-model`

---

## Constructor

```rust
impl LogRecord {
    pub fn new(
        timestamp: SystemTime,
        severity: Severity,
        message: String,
        attributes: Vec<LogAttribute>,
    ) -> Result<Self, ValidationError>;
}
```

**Preconditions**:
- `message` must be non-empty → `ValidationError::EmptyMessage`
- Each `LogAttribute.name` must match `[a-z][a-z0-9._]{0,63}` → `ValidationError::InvalidAttributeName`
- Each `LogAttribute.name` must not be a reserved field name → `ValidationError::InvalidAttributeName`
- Each `LogAttributeValue` must be flat (no nested objects) → `ValidationError::InvalidAttributeValue`
- Array `LogAttributeValue`s must be homogeneous → `ValidationError::InvalidAttributeValue`

**Postconditions**:
- All fields are accessible via accessor methods
- No mutation methods exist on the constructed instance

## Accessors

```rust
impl LogRecord {
    pub fn timestamp(&self) -> &SystemTime;
    pub fn severity(&self) -> &Severity;
    pub fn message(&self) -> &str;
    pub fn attributes(&self) -> &[LogAttribute];
}
```

## Reserved Field Names

The following names are reserved and cannot be used as attribute names:
- `timestamp`
- `severity`
- `message`
- `attributes`
