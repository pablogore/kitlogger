# KITLogger Format Selection Specification

## Purpose

Define the behavioral contract for mapping `kit_config::LogFormat` (the logging domain's configuration-level format selector) to `kitlogger_formatter::LogFormat` (the canonical formatter selector owned by `formatter-contract`). This mapping exists inside `kitlogger` specifically so `kitlogger-formatter`'s existing, accepted dependency boundary (no dependency on `kit_config`) remains unchanged — see `design.md` Q5.

Traceability: proposal `openspec/changes/014-output-consolidation/proposal.md`.

## Requirements

### Requirement: FR-001 Mapping Completeness

Every `kit_config::LogFormat` variant MUST map to exactly one `kitlogger_formatter::LogFormat` variant. No `kit_config::LogFormat` variant MAY be left unmapped.

#### Scenario: Every kit_config::LogFormat variant is handled

- GIVEN each of the four `kit_config::LogFormat` variants (`Json`, `Pretty`, `Compact`, `Text`)
- WHEN each is mapped
- THEN a `kitlogger_formatter::LogFormat` variant is produced for all four, without panic or error

### Requirement: FR-002 Deterministic Mapping

The same `kit_config::LogFormat` input MUST always produce the same `kitlogger_formatter::LogFormat` output.

#### Scenario: Repeated mapping is stable

- GIVEN a `kit_config::LogFormat` value
- WHEN it is mapped multiple times
- THEN every mapping produces the same result

### Requirement: FR-003 No Change to Formatter's Dependency Boundary

This mapping capability MUST NOT require any change to `kitlogger-formatter`'s dependencies. `kitlogger-formatter` MUST remain unaware of `kit_config`.

#### Scenario: kitlogger-formatter's dependency list is unaffected

- GIVEN `kitlogger-formatter`'s Cargo manifest before and after this capability is introduced
- WHEN its dependency list is compared
- THEN it is unchanged
