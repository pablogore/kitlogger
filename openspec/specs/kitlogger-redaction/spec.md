# KITLogger Redaction Specification

## Purpose

Define the behavioral contract for redacting sensitive attribute values from a `LogRecord` before it is emitted. `LoggingConfig.redact` (materialized and validated by kit-config) is the canonical, owning configuration for this behavior. This spec covers what the system MUST express — not how it is implemented.

## Requirements

### Requirement: FR-001 Sensitive Field Detection

An attribute is sensitive when its name matches a field identifier configured in `kit_config::RedactionConfig.fields`. When a sensitive attribute is found, its value MUST be replaced with a fixed redaction marker; its name MUST NOT change. The specific matching algorithm (e.g. substring, exact, regex, glob, metadata-driven) is a design decision, not part of this requirement. This requirement is satisfied by any algorithm that correctly identifies attributes intended to be sensitive per the configured field identifiers; changing the algorithm later is not a breaking change to this requirement as long as that intent is preserved.

#### Scenario: A configured sensitive field is redacted

- GIVEN a `RedactionConfig` identifying a field as sensitive
- AND a `LogRecord` with an attribute the current matching algorithm identifies as corresponding to that field
- WHEN redaction is applied
- THEN the attribute's value is replaced with the redaction marker
- AND the attribute's name is unchanged

#### Scenario: Non-matching attributes are untouched

- GIVEN a `RedactionConfig` identifying one field as sensitive
- AND a `LogRecord` with attributes, none matching that field
- WHEN redaction is applied
- THEN all attribute values are unchanged

### Requirement: FR-002 Immutability Preserved

Redaction MUST NOT mutate the input `LogRecord`. It MUST return a new `LogRecord` value with the matching attributes' values replaced.

#### Scenario: Original record is unchanged after redaction

- GIVEN a `LogRecord` with a sensitive attribute
- WHEN redaction is applied and a new record is produced
- THEN the original `LogRecord` value's attributes are unchanged
- AND the new `LogRecord`'s matching attribute values are redacted

### Requirement: FR-003 Disabled Config Passthrough

When `RedactionConfig.enabled` is `false`, redaction MUST return a record equivalent to the input, with no attribute values replaced.

#### Scenario: Disabled config redacts nothing

- GIVEN a `RedactionConfig` with `enabled = false` and `fields = ["password"]`
- AND a `LogRecord` with an attribute named `password`
- WHEN redaction is applied
- THEN the returned record's `password` attribute value is unchanged
