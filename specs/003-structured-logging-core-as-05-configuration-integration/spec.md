# Feature Specification: Configuration Integration

**SPEC_ID**: `003-structured-logging-core-as-05-configuration-integration`

**Parent**: Structured Logging Core (`003-structured-logging-core`)

**Candidate Key**: AS-05

**Created**: 2026-06-18

**Status**: Draft

## Scope

Define how KIT-005 consumes LoggingConfiguration (owned by KIT-CONFIG) through Kit Config contracts for controlling logging behavior — severity thresholds, attribute filtering rules, context propagation settings, and serialization preferences.

In scope:
- Define consumption contracts for LoggingConfiguration through Kit Config
- Define severity threshold configuration consumption
- Define attribute filtering rule consumption
- Define context propagation setting consumption
- Define serialization preference consumption
- Define that configuration is consumed exclusively through Kit Config contracts

## Non-Scope

- LoggingConfiguration entity shape definition (owned by KIT-CONFIG)
- LogRecord entity definition (covered by AS-01)
- LogContext definition (covered by AS-02)
- Logger and LoggerFactory contracts (covered by AS-03)
- Serialization contracts (covered by AS-04)
- Configuration file loading, parsing (TOML, YAML, JSON)
- Environment variable interpretation
- Runtime configuration hot reload implementation
- Any direct configuration source access

## Responsibility

Define how KIT-005 consumes LoggingConfiguration (owned by KIT-CONFIG) through Kit Config contracts for controlling logging behavior — severity thresholds, attribute filtering, context propagation settings, and serialization preferences. No direct configuration file loading, parsing, or environment variable interpretation.

## Dependencies

- `003-structured-logging-core-as-03-logger-contracts` (AS-03) — LoggerFactory consumes configuration
- `003-structured-logging-core-as-04-serialization-contracts` (AS-04) — serialization configuration contracts
- `002-core-telemetry-domain-model` (KIT-CONFIG Configuration Contracts) — LoggingConfiguration entity shape

## Requirements

### Functional Requirements

1. Logging behavior MUST be configurable through Kit Config contracts.
2. Severity thresholds MUST be consumable from LoggingConfiguration.
3. Attribute filtering rules MUST be consumable from LoggingConfiguration.
4. Context propagation settings MUST be consumable from LoggingConfiguration.
5. Serialization preferences MUST be consumable from LoggingConfiguration.
6. Configuration MUST NOT be loaded directly from files.
7. Configuration MUST NOT be parsed from TOML, YAML, JSON, or environment variables.
8. LoggingConfiguration entity shape is owned by KIT-CONFIG; KIT-005 defines consumption points only.

### Key Entities

- **LoggingConfiguration** — Configuration contract (entity shape owned by KIT-CONFIG) consumed through Kit Config contracts for controlling severity thresholds, attribute filtering, context propagation, and serialization preferences.

## User Scenarios & Testing

### Scenario 1: Operator configures severity threshold through Kit Config

An operator sets a severity threshold of Warn through Kit Config. The logging core reads the resolved configuration and suppresses records below the threshold without parsing any configuration file.

### Scenario 2: Operator configures attribute filtering

An operator configures attribute include/exclude rules through Kit Config. The logging core applies filtering based on the resolved rules.

### Testing

- Severity threshold is readable from LoggingConfiguration
- Attribute filtering rules are readable from LoggingConfiguration
- Configuration consumption uses Kit Config contracts only
- No direct file loading or parsing occurs

## Success Criteria

### Measurable Outcomes

1. Severity thresholds are consumable through Kit Config contracts.
2. Attribute filtering rules are consumable through Kit Config contracts.
3. Context propagation settings are consumable through Kit Config contracts.
4. Serialization preferences are consumable through Kit Config contracts.
5. No configuration file loading, parsing (TOML/YAML/JSON), or environment variable interpretation code exists in KIT-005.

## Assumptions

1. The Kit Config framework provides typed configuration contracts that KIT-005 can consume as dependencies.
2. LoggingConfiguration entity shape is owned by KIT-CONFIG; KIT-005 defines consumption points only.
3. LoggingConfiguration is consumed at LoggerFactory creation time and may be scoped to individual loggers.
