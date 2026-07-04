# KITLogger Buffering Specification

## Purpose

Define the behavioral contract for batching pre-format records before they reach formatting and dispatch, per `LoggingConfig.buffering` (materialized and validated by kit-config). This spec covers observable behavior only. Per ADR-008 §5, buffering holds raw records — it never holds already-formatted strings. Flushing produces pre-format records for the next pipeline stage; this capability has no knowledge of formatting or output destinations.

Traceability: proposal `openspec/changes/014-output-consolidation/proposal.md`.

## Requirements

### Requirement: FR-001 Size-Based Flush

Once `BufferingConfig.batch_size` records have been accumulated without an intervening flush, they MUST be flushed together.

#### Scenario: Reaching the batch size triggers a flush

- GIVEN `BufferingConfig.batch_size = 3`
- WHEN 3 records are added without an intervening flush
- THEN all 3 are flushed together as one batch

### Requirement: FR-002 Time-Based Flush

Regardless of how many records have accumulated, they MUST be flushed once `BufferingConfig.flush_interval_ms` has elapsed since the first unflushed record was added.

#### Scenario: The interval elapsing triggers a flush before the batch size is reached

- GIVEN `BufferingConfig.batch_size = 100` and `BufferingConfig.flush_interval_ms = 50`
- WHEN only 2 records are added and the configured interval elapses without a size-triggered flush
- THEN those 2 records are flushed

### Requirement: FR-003 Disabled Passthrough

When `BufferingConfig.enabled` is `false`, each record MUST be passed through individually and immediately — no batching occurs.

#### Scenario: Disabled buffering never accumulates records

- GIVEN `BufferingConfig.enabled = false`
- WHEN a record is added
- THEN it is available for the next pipeline stage immediately, without waiting for other records or an interval

### Requirement: FR-004 Order Preservation

Records MUST be flushed in the order they were added.

#### Scenario: Flushed batch preserves insertion order

- GIVEN 3 records added in a specific sequence
- WHEN they are flushed as one batch
- THEN they appear in the flushed batch in the same sequence they were added

### Requirement: FR-005 Pre-Format Content

Buffering MUST hold records prior to formatting — it MUST NOT hold already-formatted strings.

#### Scenario: Buffered content is not yet formatted

- GIVEN a record added to the buffer
- WHEN it is inspected while held in the buffer, before flush
- THEN it is still in its pre-format representation, not a formatted string

### Requirement: FR-006 Injectable Time Source

The passage of time used by the time-based flush (FR-002) MUST be sourced through an injectable clock abstraction, not read directly from the system clock, so that flush-interval behavior is deterministically testable without real time delays.

#### Scenario: Time-based flush is testable without real delays

- GIVEN a buffer constructed with a controllable time source
- WHEN time is advanced programmatically past the configured flush interval
- THEN the time-based flush (FR-002) occurs without the test needing to sleep for the real interval
