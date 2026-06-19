# Design: Logger Contracts

## Technical Approach

This design defines the canonical Logger and LoggerFactory contracts for the KitLogger ecosystem. The contracts are designed to be transport, exporter, and storage agnostic while integrating with existing domain components (LogRecord, LogContext, ValidationError). The approach follows the adapter pattern to decouple application code from concrete logging implementations, ensuring extensibility and maintainability.

## Architecture Decisions

### Decision: Contract Abstraction Level

**Choice**: Define Logger and LoggerFactory as pure trait abstractions without any concrete implementation details
**Alternatives considered**: Include base implementations or default behaviors in contracts
**Rationale**: Following the principle of pure abstractions ensures maximum flexibility for implementors. The contracts should only define what operations are available, not how they should be implemented. This allows for different logging backends (file, console, remote services) to implement the same interface without constraint.

### Decision: Integration with Domain Components

**Choice**: Logger contracts integrate with LogRecord and LogContext from the logging domain
**Alternatives considered**: Create separate internal representations for logging data
**Rationale**: Leveraging existing LogRecord and LogContext components ensures consistency with the broader logging ecosystem and reduces duplication. This also allows for better interoperability with other components that already use these domain models.

### Decision: Error Handling Strategy

**Choice**: Logger contracts return Result types for operations that may fail
**Alternatives considered**: Use panic or silent failure for validation errors
**Rationale**: Using Result types provides explicit error handling for validation failures and other potential issues. This follows Rust best practices and makes the contracts more robust and predictable for implementors.

### Decision: Severity Level Support

**Choice**: Support standard logging severity levels (Trace, Debug, Info, Warn, Error)
**Alternatives considered**: Custom severity levels or fewer levels
**Rationale**: Standard severity levels provide familiarity for developers and align with common logging practices. The five levels (Trace, Debug, Info, Warn, Error) cover the typical logging needs while being extensible for future additions.

## Data Flow

```
Application Code ──→ Logger Contract ──→ LogRecord Creation ──→ Transport Layer
       │                    │                    │
       └────────────────────┼────────────────────┘
                            │
                    LoggerFactory Contract
                            │
                    Named Logger Creation
                            │
                    Context Enrichment
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `crates/kitlogger-logger-contract/src/lib.rs` | Create | Main contract definition file |
| `crates/kitlogger-logger-contract/src/logger.rs` | Create | Logger trait definition |
| `crates/kitlogger-logger-contract/src/logger_factory.rs` | Create | LoggerFactory trait definition |

## Interfaces / Contracts

```rust
//! Logger contract for emitting structured log records.
//!
//! This trait defines the canonical interface for logging operations in the KitLogger ecosystem.
//! It is designed to be transport, exporter, and storage agnostic.

use std::collections::HashMap;
use std::result::Result;

use crate::{LogContext, LogRecord, ValidationError};

/// The Logger trait defines the interface for emitting structured log records.
///
/// Implementations of this trait are responsible for handling the actual logging
/// operations, while this trait provides a consistent interface for application code.
pub trait Logger: Send + Sync {
    /// Emits a log record with the specified severity, message, and attributes.
    ///
    /// # Arguments
    /// * `severity` - The severity level of the log record
    /// * `message` - The log message
    /// * `attributes` - Optional structured attributes to include in the log record
    ///
    /// # Returns
    /// * `Result<(), ValidationError>` - Ok if the log record was successfully created,
    ///   Err if validation failed
    fn emit(
        &self,
        severity: Severity,
        message: impl Into<String>,
        attributes: Option<Vec<LogAttribute>>,
    ) -> Result<(), ValidationError>;

    /// Emits a trace level log record.
    ///
    /// # Arguments
    /// * `message` - The log message
    /// * `attributes` - Optional structured attributes to include in the log record
    ///
    /// # Returns
    /// * `Result<(), ValidationError>` - Ok if the log record was successfully created,
    ///   Err if validation failed
    fn trace(
        &self,
        message: impl Into<String>,
        attributes: Option<Vec<LogAttribute>>,
    ) -> Result<(), ValidationError>;

    /// Emits a debug level log record.
    ///
    /// # Arguments
    /// * `message` - The log message
    /// * `attributes` - Optional structured attributes to include in the log record
    ///
    /// # Returns
    /// * `Result<(), ValidationError>` - Ok if the log record was successfully created,
    ///   Err if validation failed
    fn debug(
        &self,
        message: impl Into<String>,
        attributes: Option<Vec<LogAttribute>>,
    ) -> Result<(), ValidationError>;

    /// Emits an info level log record.
    ///
    /// # Arguments
    /// * `message` - The log message
    /// * `attributes` - Optional structured attributes to include in the log record
    ///
    /// # Returns
    /// * `Result<(), ValidationError>` - Ok if the log record was successfully created,
    ///   Err if validation failed
    fn info(
        &self,
        message: impl Into<String>,
        attributes: Option<Vec<LogAttribute>>,
    ) -> Result<(), ValidationError>;

    /// Emits a warn level log record.
    ///
    /// # Arguments
    /// * `message` - The log message
    /// * `attributes` - Optional structured attributes to include in the log record
    ///
    /// # Returns
    /// * `Result<(), ValidationError>` - Ok if the log record was successfully created,
    ///   Err if validation failed
    fn warn(
        &self,
        message: impl Into<String>,
        attributes: Option<Vec<LogAttribute>>,
    ) -> Result<(), ValidationError>;

    /// Emits an error level log record.
    ///
    /// # Arguments
    /// * `message` - The log message
    /// * `attributes` - Optional structured attributes to include in the log record
    ///
    /// # Returns
    /// * `Result<(), ValidationError>` - Ok if the log record was successfully created,
    ///   Err if validation failed
    fn error(
        &self,
        message: impl Into<String>,
        attributes: Option<Vec<LogAttribute>>,
    ) -> Result<(), ValidationError>;
}

//! LoggerFactory contract for creating named Logger instances.
//!
//! This trait defines the interface for creating Logger instances with optional
//! default context and configuration.
pub trait LoggerFactory: Send + Sync {
    /// Creates a new Logger instance with the specified name.
    ///
    /// # Arguments
    /// * `name` - The name to identify the logger
    ///
    /// # Returns
    /// * `Box<dyn Logger>` - A new Logger instance with the specified name
    fn create_logger(&self, name: impl Into<String>) -> Box<dyn Logger>;

    /// Creates a new Logger instance with the specified name and optional default context.
    ///
    /// # Arguments
    /// * `name` - The name to identify the logger
    /// * `default_context` - Optional default context to be applied to the logger
    ///
    /// # Returns
    /// * `Box<dyn Logger>` - A new Logger instance with the specified name and context
    fn create_logger_with_context(
        &self,
        name: impl Into<String>,
        default_context: Option<LogContext>,
    ) -> Box<dyn Logger>;
}
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | Logger trait methods | Mock the Logger trait and verify method calls with different severity levels and attributes |
| Unit | LoggerFactory trait methods | Mock the LoggerFactory trait and verify logger creation with different names and contexts |
| Integration | Contract integration with LogRecord | Test that Logger can create valid LogRecord instances with proper validation |
| Integration | Context handling | Verify that LogContext is properly merged and applied to log records |
| E2E | End-to-end logging flow | Test complete flow from Logger creation to LogRecord emission |

## Migration / Rollout

No migration required. These contracts are pure abstractions that will be implemented by concrete logging components. Existing code that uses the current logging implementation will need to be updated to use these contracts, but the contracts themselves are designed to be backward compatible with existing patterns.

## Open Questions

- [ ] Should the Logger trait include methods for setting minimum severity levels or other configuration options?
- [ ] Should the LoggerFactory trait include methods for retrieving existing loggers by name?
- [ ] Should the Logger trait include methods for flushing or closing loggers?