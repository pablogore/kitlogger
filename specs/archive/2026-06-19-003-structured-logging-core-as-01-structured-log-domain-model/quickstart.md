# Quickstart: Structured Log Domain Model

**SPEC_ID**: `003-structured-logging-core-as-01-structured-log-domain-model`

---

## Prerequisites

- Rust toolchain (stable)
- `cargo test` for validation

## Validation Scenarios

### Scenario 1: Create a valid LogRecord

```rust
use std::time::SystemTime;
use kitlogger_log_domain::{LogRecord, Severity, LogAttribute, LogAttributeValue};

let record = LogRecord::new(
    SystemTime::now(),
    Severity::Info,
    "User login successful".to_string(),
    vec![
        LogAttribute::new(
            "user_id".to_string(),
            LogAttributeValue::String("abc123".to_string()),
        ).unwrap(),
        LogAttribute::new(
            "amount".to_string(),
            LogAttributeValue::Float(99.95),
        ).unwrap(),
    ],
).unwrap();

assert_eq!(record.severity(), &Severity::Info);
assert_eq!(record.message(), "User login successful");
assert_eq!(record.attributes().len(), 2);
```

**Expected**: Construction succeeds. All fields readable via accessors.

### Scenario 2: Reject empty message

```rust
let result = LogRecord::new(
    SystemTime::now(),
    Severity::Info,
    "".to_string(),
    vec![],
);

assert!(result.is_err());
// matches! result, Err(ValidationError::EmptyMessage)
```

**Expected**: `ValidationError::EmptyMessage`.

### Scenario 3: Reject invalid attribute name

```rust
let result = LogAttribute::new(
    "UPPERCASE_NAME".to_string(),
    LogAttributeValue::Boolean(true),
);

assert!(result.is_err());
// matches! result, Err(ValidationError::InvalidAttributeName(_))
```

**Expected**: `ValidationError::InvalidAttributeName`.

### Scenario 4: Severity ordering

```rust
use kitlogger_log_domain::Severity;

assert!(Severity::Trace < Severity::Debug);
assert!(Severity::Debug < Severity::Info);
assert!(Severity::Info < Severity::Warn);
assert!(Severity::Warn < Severity::Error);
assert!(Severity::Error < Severity::Fatal);
```

**Expected**: All ordering assertions pass.

### Scenario 5: LogRecord immutability

```rust
let record = LogRecord::new(
    SystemTime::now(),
    Severity::Error,
    "Disk full".to_string(),
    vec![],
).unwrap();

// Verify no mutation methods exist (compile-time check)
// The following would fail to compile:
// record.message = "new message"; // error: field is private
// record.set_severity(...);       // error: no such method
```

**Expected**: Compile-time immutability. No public fields or setter methods.

### Scenario 6: Identifier creation and display

```rust
use kitlogger_log_domain::CorrelationId;

let cid = CorrelationId::new("req-42".to_string());
assert_eq!(cid.as_str(), "req-42");
assert_eq!(cid.to_string(), "req-42");
```

**Expected**: Identifiers wrap strings and display correctly.

## Running Tests

```bash
cargo test -p kitlogger-log-domain
```

## Contracts Reference

- [LogRecord](contracts/log-record.md)
- [Severity](contracts/severity.md)
- [LogAttribute](contracts/log-attribute.md)
- [Identifiers](contracts/identifiers.md)

## Data Model

- [Data Model](../data-model.md)
