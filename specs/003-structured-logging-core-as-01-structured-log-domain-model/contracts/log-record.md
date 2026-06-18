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
- All `LogAttribute` instances passed in must have been validated at their own construction site

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
