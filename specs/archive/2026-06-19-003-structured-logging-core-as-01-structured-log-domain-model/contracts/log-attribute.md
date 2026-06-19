# LogAttribute Contract

**SPEC_ID**: `003-structured-logging-core-as-01-structured-log-domain-model`

---

## LogAttribute

```rust
#[derive(Clone)]
pub struct LogAttribute {
    name: String,
    value: LogAttributeValue,
}

impl LogAttribute {
    pub fn new(name: String, value: LogAttributeValue) -> Result<Self, ValidationError>;
    pub fn name(&self) -> &str;
    pub fn value(&self) -> &LogAttributeValue;
}
```

## LogAttributeValue

```rust
#[derive(Clone)]
pub enum LogAttributeValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Timestamp(SystemTime),
    Array(Vec<LogAttributeValue>),
}
```

## Attribute Naming

- Pattern: `^[a-z][a-z0-9._]{0,63}$`
- Max length: 64 characters
- Must not conflict with reserved LogRecord field names
- Rejected names return `ValidationError::InvalidAttributeName`
