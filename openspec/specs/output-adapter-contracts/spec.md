# Output Adapter Contracts Specification

## Purpose

Define the behavioral contract for the Output Port: the single seam every log output destination (console, file, and future network/vendor destinations) is dispatched through. This spec covers what the system MUST express — not how it is implemented. This crate is deliberately independent of `telemetry-adapter-contracts` (Telemetry/OTel-provider bounded context) — see the originating architecture review for why.

Traceability: proposal `openspec/changes/014-output-consolidation/proposal.md`.

## Requirements

### Requirement: FR-001 Output Port Conformance

Every output destination MUST be dispatchable through one common Port. The Port MUST accept an already-formatted record representation and its severity — it MUST NOT require a raw, pre-format `LogRecord`, consistent with the pipeline order (buffer → format → dispatch).

#### Scenario: A conforming output receives a dispatched record

- GIVEN an output implementing the Port
- WHEN a formatted record and its severity are dispatched to it
- THEN the output receives both values without requiring any additional, output-specific wrapping

### Requirement: FR-002 Unique Registration

Each output registered with the registry MUST be uniquely identified. Registering a second output under an identifier already in use MUST be rejected.

#### Scenario: Duplicate registration is rejected

- GIVEN an output already registered under a given identifier
- WHEN a second output is registered under the same identifier
- THEN the registration is rejected
- AND the originally registered output remains registered, unchanged

### Requirement: FR-003 Dispatch to All Registered Outputs

Dispatching a record MUST deliver it to every currently registered output.

#### Scenario: All registered outputs receive the dispatched record

- GIVEN three outputs registered under distinct identifiers
- WHEN a record is dispatched
- THEN all three outputs receive it

### Requirement: FR-004 Partial Failure Isolation

A delivery failure at one registered output MUST NOT prevent delivery to any other registered output. The aggregate dispatch result MUST distinguish between: all outputs succeeded, some outputs failed, and all outputs failed.

#### Scenario: One failing output does not block the others

- GIVEN three registered outputs, one of which fails on delivery
- WHEN a record is dispatched
- THEN the two non-failing outputs still receive the record
- AND the aggregate result indicates a partial failure, naming the failing output

#### Scenario: All outputs failing is distinguishable from a partial failure

- GIVEN three registered outputs, all of which fail on delivery
- WHEN a record is dispatched
- THEN the aggregate result indicates total failure, distinguishable from a partial failure

### Requirement: FR-005 No Telemetry-Specific Coupling

The Port and registry MUST NOT require any OpenTelemetry-specific or cross-signal batch type (e.g. a payload envelope combining traces, metrics, and logs) to dispatch a single record. Any output MUST be able to conform without depending on `telemetry-adapter-contracts` or `telemetry-types`.

#### Scenario: A conforming output has no telemetry-specific dependency

- GIVEN an output implementing the Port
- WHEN its dependencies are inspected
- THEN no dependency on an OpenTelemetry-mapping or cross-signal batch type is required to satisfy the Port
