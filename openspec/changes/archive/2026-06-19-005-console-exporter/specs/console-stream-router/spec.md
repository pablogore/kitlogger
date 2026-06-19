# Delta for Console Stream Router

## ADDED Requirements

### Requirement: Console Stream Router Integration

The system MUST integrate the Console Stream Router with the Console Exporter to ensure proper routing of log output to stdout/stderr based on severity levels.

#### Scenario: Integration with Console Exporter

- GIVEN a formatted string from the Console Exporter
- WHEN the Stream Router processes it
- THEN the router MUST route it to the correct stream based on severity

#### Scenario: Stream routing with Console Exporter

- GIVEN the Console Exporter is configured with default settings
- WHEN formatted log records are delivered
- THEN the records MUST be routed to stdout/stderr by the Stream Router

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

## REMOVED Requirements

### Requirement: Console Stream Router Core

The Console Stream Router Core requirement is now integrated into the overall Console Stream Router specification and no longer exists as a separate requirement.

(Reason: The Console Stream Router Core functionality is now part of the integrated Console Stream Router specification)
(Migration: All functionality previously in Console Stream Router Core is now part of the Console Stream Router specification)