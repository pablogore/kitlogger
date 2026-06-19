# Delta for Console Exporter

## ADDED Requirements

### Requirement: Console Exporter Integration

The system MUST integrate the Console Exporter with the existing logging pipeline. The exporter MUST receive pre-formatted strings from the Formatting Pipeline and deliver them to the StreamRouter for output.

#### Scenario: Exporter receives formatted string

- GIVEN a pre-formatted string from the Formatting Pipeline
- WHEN the Console Exporter receives it
- THEN the exporter MUST forward it to the StreamRouter
- AND the StreamRouter MUST write it to the appropriate stream based on severity

#### Scenario: Exporter integration with pipeline

- GIVEN the logging pipeline is configured with Console Exporter
- WHEN formatted log records are processed
- THEN the records MUST be delivered to stdout/stderr via the Console Exporter

### Requirement: Stream Routing Integration

The system MUST integrate the StreamRouter with the Console Exporter to ensure proper routing of log output to stdout/stderr based on severity levels.

#### Scenario: Level-based routing

- GIVEN a formatted string with severity ERROR
- WHEN the Console Exporter delivers it
- THEN the StreamRouter MUST write it to stderr
- AND a formatted string with severity INFO MUST be written to stdout

#### Scenario: Custom routing configuration

- GIVEN a custom level-to-stream mapping configuration
- WHEN the Console Exporter delivers strings with various severities
- THEN the StreamRouter MUST route each string to the configured stream

## MODIFIED Requirements

### Requirement: Level-to-Stream Routing

The system MUST route output to stdout or stderr based on severity level. By default, ERROR and WARN MUST go to stderr; DEBUG, INFO, and TRACE MUST go to stdout. The mapping SHALL be configurable.

(Previously: Level-to-Stream Routing from console-stream-router spec)

#### Scenario: Error output to stderr

- GIVEN a formatted string with severity ERROR
- WHEN the stream router processes it
- THEN the string MUST be written to stderr

#### Scenario: Info output to stdout

- GIVEN a formatted string with severity INFO
- WHEN the stream router processes it
- THEN the string MUST be written to stdout

#### Scenario: Custom mapping

- GIVEN a configuration mapping WARN to stdout
- WHEN a WARN-severity string is processed
- THEN the string MUST be written to stdout

### Requirement: Write Error Handling

The system MUST NOT panic on I/O errors. If a stream write fails, the router MUST return an error and MAY attempt a single retry.

(Previously: Write Error Handling from console-stream-router spec)

#### Scenario: Stderr write failure

- GIVEN stderr returns a broken pipe error
- WHEN the router attempts to write
- THEN the router MUST return an error to the caller
- AND the router MUST NOT panic or abort

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

### Requirement: Console Exporter Core

The Console Exporter Core requirement is now integrated into the overall Console Exporter specification and no longer exists as a separate requirement.

(Reason: The Console Exporter Core functionality is now part of the integrated Console Exporter specification)
(Migration: All functionality previously in Console Exporter Core is now part of the Console Exporter specification)

### Requirement: Stream Router

The Stream Router requirement is now integrated into the overall Console Exporter specification and no longer exists as a separate requirement.

(Reason: The Stream Router functionality is now part of the integrated Console Exporter specification)
(Migration: All functionality previously in Stream Router is now part of the Console Exporter specification)

## RENAMED Requirements

### Requirement: Console Exporter Integration → Console Exporter Integration with Pipeline

(Reason: The requirement was renamed to better reflect its integration with the logging pipeline)
(Migration: References to "Console Exporter Integration" should be updated to "Console Exporter Integration with Pipeline")