# Console Stream Router Specification

## Purpose

Routes already-formatted strings to the correct console stream (stdout or stderr) based on log level. The router is the final output stage — it takes a formatted string and a severity level, and writes to the appropriate stream. It performs no formatting.

## Requirements

### Requirement: Level-to-Stream Routing

The system MUST route output to stdout or stderr based on severity level. By default, ERROR and WARN MUST go to stderr; DEBUG, INFO, and TRACE MUST go to stdout. The mapping SHALL be configurable.

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

#### Scenario: Stderr write failure

- GIVEN stderr returns a broken pipe error
- WHEN the router attempts to write
- THEN the router MUST return an error to the caller
- AND the router MUST NOT panic or abort
