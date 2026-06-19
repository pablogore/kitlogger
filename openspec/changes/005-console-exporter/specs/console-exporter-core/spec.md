# Delta for Console Exporter Core

## ADDED Requirements

### Requirement: Console Exporter Core Integration

The system MUST integrate the Console Exporter Core with the Console Stream Router to ensure proper delivery of log output.

#### Scenario: Integration with Stream Router

- GIVEN a formatted string from the Console Exporter
- WHEN the Console Exporter delivers it
- THEN the exporter MUST forward it to the Stream Router for output

#### Scenario: Core functionality with Stream Router

- GIVEN the Console Exporter is configured with default settings
- WHEN formatted log records are delivered
- THEN the records MUST be processed by the Console Exporter Core and forwarded to the Stream Router

## MODIFIED Requirements

### Requirement: String Delivery

The system MUST accept a pre-formatted string and severity level, then deliver it to the StreamRouter for output. The exporter MUST NOT transform or interpret the string content.

(Previously: String Delivery from console-exporter-core spec)

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

(Previously: Lifecycle Management from console-exporter-core spec)

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

(Previously: Flush Strategy from console-exporter-core spec)

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

(Previously: Error Handling from console-exporter-core spec)

#### Scenario: Write failure during flush

- GIVEN a write error occurs during shutdown flushing
- WHEN the exporter is in Flushing state
- THEN the exporter MUST report the error
- AND the exporter MUST still transition to Shutdown

## REMOVED Requirements

### Requirement: Console Exporter Core Functionality

The Console Exporter Core Functionality requirement is now integrated into the overall Console Exporter specification and no longer exists as a separate requirement.

(Reason: The Console Exporter Core functionality is now part of the integrated Console Exporter specification)
(Migration: All functionality previously in Console Exporter Core is now part of the Console Exporter specification)