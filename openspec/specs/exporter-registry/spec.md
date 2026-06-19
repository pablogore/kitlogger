# Exporter Registry Specification

## Purpose

Pluggable registry that selects, configures, and manages exporter implementations. Decouples the logging pipeline from specific exporters — new exporters can be added by registering them with the registry without modifying pipeline code.

## Requirements

### Requirement: Exporter Selection by Name

The system MUST support selecting an exporter implementation by string name (e.g., `"console"`, `"file"`). Selecting an unknown name MUST return an error.

#### Scenario: Select registered exporter

- GIVEN a `"console"` exporter is registered with the registry
- WHEN a caller requests the exporter by name `"console"`
- THEN the registry MUST return the console exporter instance

#### Scenario: Select unregistered exporter

- GIVEN no exporter named `"custom"` is registered
- WHEN a caller requests the exporter by name `"custom"`
- THEN the registry MUST return an error indicating the exporter was not found

### Requirement: Exporter Registration

The system MUST support registering new exporter implementations without modifying pipeline code. Registration SHALL accept a name string and an exporter instance. Registering a duplicate name MAY return an error or overwrite the existing entry.

#### Scenario: Register new exporter

- GIVEN a new exporter implementation named `"yaml"`
- WHEN it is registered with the registry
- THEN subsequent requests for `"yaml"` MUST return the registered instance

#### Scenario: Duplicate registration

- GIVEN an exporter named `"console"` is already registered
- WHEN a second exporter with name `"console"` is registered
- THEN the registry SHOULD return an error or document its overwrite policy

### Requirement: Default Exporter

The system SHOULD provide a configurable default exporter used when no explicit exporter is selected.

#### Scenario: Default exporter fallback

- GIVEN no explicit exporter is configured
- WHEN the registry is queried for the active exporter
- THEN the default exporter MUST be returned
