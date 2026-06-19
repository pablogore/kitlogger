# Proposal: Logger Contracts (AS-03)

## Intent

Define the canonical Logger and LoggerFactory domain contracts that enable structured logging across the KitLogger ecosystem. These contracts provide the foundational interface for emitting structured log records and creating named logger instances with optional context and configuration, ensuring transport, exporter, and storage agnosticism while maintaining clean separation of concerns.

## Scope

### In Scope
- Define the Logger canonical domain contract for emitting structured LogRecords with severity level, message, and optional attributes
- Define the LoggerFactory canonical domain contract for creating named Logger instances with optional default context and configuration
- Define named logger creation semantics and naming conventions
- Define optional context pre-configuration capabilities for loggers
- Define optional LoggingConfiguration consumption through Kit Config contracts
- Ensure contracts remain transport, exporter, and storage agnostic

### Out of Scope
- LogRecord entity definition (covered by AS-01)
- LogContext definition (covered by AS-02)
- Configuration integration implementation (covered by AS-05)
- Serialization contracts (covered by AS-04)
- Transport, formatting, exporter, or storage concerns
- Adapter patterns (KIT-003 may define these; AS-03 does not)

## Capabilities

### New Capabilities
- `logger-contracts`: Defines the canonical Logger and LoggerFactory interfaces for structured logging

### Modified Capabilities
- None

## Approach

The Logger Contracts capability will be implemented by defining two core interfaces:

1. **Logger Interface**:
   - Method for emitting log records with severity, message, and optional attributes
   - Must be transport/exporter/storage agnostic
   - Should support structured logging with strongly typed attributes
   - Must not contain any concrete implementation details

2. **LoggerFactory Interface**:
   - Method for creating named loggers with optional default context
   - May accept LoggingConfiguration for pre-configuration
   - Should support creating loggers with inherited context from parent factories
   - Must maintain immutability of context and configuration

The implementation will follow the established patterns from the parent specification (Structured Logging Core) and maintain consistency with the dependency chain:
- AS-01 (Log Domain Model) - provides LogRecord, Severity, LogAttribute
- AS-02 (Log Context Enrichment) - provides LogContext

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `contracts/logger.md` | New | Defines the Logger canonical domain contract |
| `contracts/logger-factory.md` | New | Defines the LoggerFactory canonical domain contract |
| `specs/003-structured-logging-core-as-03-logger-contracts/spec.md` | Modified | Updated with implementation details |
| `src/logger/**` | New | Implementation of logger contracts |
| `src/logger-factory/**` | New | Implementation of logger factory contracts |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Interface complexity leading to adoption barriers | Medium | Provide clear documentation and examples |
| Inconsistent implementation across different logger types | High | Enforce strict contract compliance through testing |
| Configuration integration challenges | Medium | Define clear contract boundaries and separation of concerns |
| Performance overhead from contract abstraction | Low | Profile and optimize critical paths |

## Rollback Plan

If issues arise with the Logger Contracts implementation:
1. Revert to previous stable version of logger contracts
2. Revert any dependent implementations that rely on new contract methods
3. Ensure backward compatibility with existing logging implementations
4. Revert any configuration integration changes that cause issues

## Dependencies

- `003-structured-logging-core-as-01-structured-log-domain-model` (AS-01) - LogRecord, Severity, LogAttribute
- `003-structured-logging-core-as-02-log-context-enrichment` (AS-02) - LogContext
- KIT-CONFIG Configuration Contracts - for LoggingConfiguration consumption

## Success Criteria

- [ ] Logger canonical contract exists and is transport/exporter/storage agnostic
- [ ] LoggerFactory canonical contract exists and supports named logger creation with optional context
- [ ] No transport-specific, exporter-specific, or storage-specific types in Logger or LoggerFactory interfaces
- [ ] Logger contracts are properly integrated with LogContext from AS-02
- [ ] LoggerFactory properly handles optional LoggingConfiguration consumption
- [ ] Clear documentation and examples provided for contract usage