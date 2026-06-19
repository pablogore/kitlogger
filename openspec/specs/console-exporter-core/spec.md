# Console Exporter Core Specification

## Purpose

Core exporter that receives pre-formatted strings from the Formatting Pipeline and delivers them to the StreamRouter. Owns lifecycle management (init, run, flush, shutdown) and flush strategies that decouple write timing from the application hot path.

## Requirements

### Requirement: String Delivery

The system MUST accept a pre-formatted string and severity level, then deliver it to the StreamRouter for output. The exporter MUST NOT transform or interpret the string content.

#### Scenario: Deliver formatted string

- GIVEN a pre-formatted string and severity INFO
- WHEN the exporter delivers it
- THEN the string MUST be forwarded to the StreamRouter verbatim
- AND the StreamRouter MUST write it to stdout

#### Scenario: Empty string

- GIVEN an empty formatted string
- WHEN the exporter delivers it
- THEN the exporter MUST forward it without error
- AND the StreamRouter MUST write an empty line

### Requirement: Lifecycle Management

The exporter MUST manage lifecycle transitions: Uninitialized → Running → Flushing → Shutdown. The exporter MUST NOT accept delivery requests before initialization or after shutdown.

#### Scenario: Normal lifecycle

- GIVEN the exporter is initialized
- WHEN delivery requests arrive
- THEN the exporter MUST be in Running state and accept them
- WHEN shutdown is requested
- THEN the exporter MUST transition to Flushing, complete pending writes, then transition to Shutdown

#### Scenario: Delivery after shutdown

- GIVEN the exporter is in Shutdown state
- WHEN a delivery request arrives
- THEN the exporter MUST return an error indicating the exporter is shut down

### Requirement: Flush Strategy

The exporter MUST support configurable flush strategies: Immediate (write and flush on each call), OnShutdown (buffer writes, flush during shutdown), and Batch (flush at interval or count threshold).

#### Scenario: Immediate flush

- GIVEN flush strategy is Immediate
- WHEN a string is delivered
- THEN the stream MUST be flushed after each write

#### Scenario: OnShutdown flush

- GIVEN flush strategy is OnShutdown
- WHEN strings are delivered
- THEN they MAY be buffered
- WHEN shutdown is requested
- THEN all buffered strings MUST be flushed before transitioning to Shutdown

### Requirement: Error Handling

The exporter MUST NOT panic on I/O or lifecycle errors. Errors MUST be returned to the caller for handling.

#### Scenario: Write failure during flush

- GIVEN a write error occurs during shutdown flushing
- WHEN the exporter is in Flushing state
- THEN the exporter MUST report the error
- AND the exporter MUST still transition to Shutdown
