# KITLogger Emission Pipeline Specification

## Purpose

Define the end-to-end behavioral contract for `KITLogger::log`/`log_record`: the order in which filtering, sampling, redaction, buffering, formatting, and dispatch occur, and what each stage does to a record's progress through the pipeline. This spec covers observable behavior only — it does not specify internal APIs, threading model, or data structures.

Traceability: proposal `openspec/changes/015-orchestration-fold/proposal.md`.

## Requirements

### Requirement: FR-001 Enabled Gate

When `LoggingConfig.enabled` is `false`, no further pipeline processing MUST occur for any log call — no filtering, sampling, redaction, buffering, formatting, or dispatch.

#### Scenario: Disabled logging performs no further processing

- GIVEN a `KITLogger` constructed from a `LoggingConfig` with `enabled = false`
- WHEN a log call is made
- THEN no sampling, redaction, buffering, formatting, or dispatch occurs for that call

### Requirement: FR-002 Level Filtering

A record MUST proceed past this stage only if its severity is at or above the threshold named by `LoggingConfig.level`. `Severity::Fatal` MUST always proceed, regardless of the configured level, since no `LogLevel` value can represent a threshold at or above `Fatal`.

#### Scenario: A record below the configured level is dropped

- GIVEN `LoggingConfig.level = Warn`
- WHEN a log call is made with `Severity::Info`
- THEN the record does not proceed past this stage

#### Scenario: A record at or above the configured level proceeds

- GIVEN `LoggingConfig.level = Warn`
- WHEN a log call is made with `Severity::Error`
- THEN the record proceeds to the next stage

#### Scenario: Fatal always proceeds

- GIVEN `LoggingConfig.level = Error` (the strictest configurable level)
- WHEN a log call is made with `Severity::Fatal`
- THEN the record proceeds to the next stage

### Requirement: FR-003 Sampling Gate

A record that passes level filtering MUST next be evaluated by the sampling decision (per `LoggingConfig.sampling`). If the decision is negative, the record MUST be dropped before redaction, buffering, formatting, or dispatch occur.

#### Scenario: A sampled-out record does not reach later stages

- GIVEN a sampling configuration that decides against a given record
- WHEN that record passes level filtering
- THEN it does not reach redaction, buffering, formatting, or dispatch

### Requirement: FR-004 Redaction Before Buffering

A record that passes sampling MUST be redacted (per `LoggingConfig.redact`) before it is added to the buffer. The buffered record MUST reflect redaction, not the pre-redaction content.

#### Scenario: The buffered record is the redacted one

- GIVEN a record containing a sensitive attribute, and a sampling decision that allows it through
- WHEN the record reaches buffering
- THEN the version held in the buffer has the sensitive attribute already redacted

### Requirement: FR-005 Buffering Defers Formatting and Dispatch

A redacted record MUST be added to the buffer (per `LoggingConfig.buffering`). Formatting and dispatch for that record MUST NOT occur until the buffer flushes it — immediately, if buffering is disabled; otherwise per the buffer's own size/time flush conditions.

#### Scenario: Formatting and dispatch wait for a flush

- GIVEN `LoggingConfig.buffering.enabled = true` with a batch size greater than 1
- WHEN a single record is added and the flush conditions have not yet been met
- THEN no formatting or dispatch has occurred for that record yet

#### Scenario: Disabled buffering makes the pipeline synchronous

- GIVEN `LoggingConfig.buffering.enabled = false`
- WHEN a record reaches this stage
- THEN formatting and dispatch occur immediately for that record

### Requirement: FR-006 Flush Drains the Pipeline

Calling `KITLogger`'s flush or shutdown operation MUST guarantee that every record currently held in the buffer is formatted and dispatched before that operation returns.

#### Scenario: Shutdown drains buffered records

- GIVEN records added to the buffer that have not yet met a flush condition
- WHEN `KITLogger`'s shutdown operation is called
- THEN every one of those records has been formatted and dispatched by the time the call returns

### Requirement: FR-007 Formatting on Flush

Each record produced by a buffer flush MUST be formatted using the formatter selected via `LoggingConfig.format` before it is dispatched.

#### Scenario: A flushed record is formatted before dispatch

- GIVEN a buffer flush producing one or more records
- WHEN those records are processed
- THEN each is formatted before any dispatch occurs for it

### Requirement: FR-008 Dispatch Only After Formatting

A record MUST NOT be dispatched to any output before it has been formatted.

#### Scenario: No dispatch occurs without prior formatting

- GIVEN the pipeline's normal operation
- WHEN any record is dispatched
- THEN a corresponding formatting step for that record has already occurred

### Requirement: FR-009 Default Output Registration

`KITLogger` MUST register a console output by default at construction, per `LoggingConfig.output.targets`. `KITLogger` MUST NOT register any file-based output by default in this capability's current scope.

#### Scenario: Console is registered by default

- GIVEN a `KITLogger` constructed from a `LoggingConfig` with default `output.targets`
- WHEN its registered outputs are inspected
- THEN a console output is present

#### Scenario: No file output is registered

- GIVEN a `KITLogger` constructed from any `LoggingConfig` value
- WHEN its registered outputs are inspected
- THEN no file-based output is present

### Requirement: FR-010 Single Dispatch Orchestrator

`KITLogger` MUST be the sole orchestrator of dispatch. No second, competing dispatch or multi-output orchestration concept MUST exist alongside it.

#### Scenario: Only one dispatch path exists

- GIVEN `KITLogger`'s implementation
- WHEN its dispatch-related types are inspected
- THEN exactly one registry/orchestration mechanism is in use, with no parallel or duplicate mechanism present
