# File Exporter Specification

## Purpose

Define the behavioral contract for writing dispatched records to a file, including rotation. `LoggingConfig.rotation` (materialized and validated by kit-config) is the canonical, owning configuration for rotation behavior. This spec covers observable behavior only — the specific rotation algorithm (e.g. numbered backup renaming vs. any other scheme) is a design decision, not part of this contract, as long as the bounds below hold.

Traceability: proposal `openspec/changes/014-output-consolidation/proposal.md`.

## Requirements

### Requirement: FR-001 File Write

Every record dispatched to the file exporter MUST be appended to the configured file path.

#### Scenario: A dispatched record is appended to the file

- GIVEN a file exporter configured with a file path
- WHEN a formatted record is dispatched to it
- THEN the record's content appears in the file, appended after any prior content

### Requirement: FR-002 Size-Based Rotation Trigger

When appending a record would cause the file's size to exceed `RotationConfig.max_size_mb`, rotation MUST occur at or before that write, per the design's chosen rotation mechanism.

#### Scenario: Rotation occurs at the size boundary

- GIVEN a file exporter with `RotationConfig.max_size_mb` set, and the file currently near that size
- WHEN a record is dispatched that would push the file over the configured size
- THEN rotation occurs
- AND the new record ends up in a file that has not exceeded the configured size

### Requirement: FR-003 Backup Retention Bound

After rotation, at most `RotationConfig.max_backups` prior versions of the file MUST be retained. Once that bound is exceeded, the oldest retained version MUST be discarded.

#### Scenario: Backups beyond the configured maximum are discarded

- GIVEN `RotationConfig.max_backups = 2` and rotation has already occurred twice, producing two retained backups
- WHEN a third rotation occurs
- THEN exactly two backups remain retrievable afterward
- AND the oldest of the three (the one from before the third rotation) is no longer retained

### Requirement: FR-004 Rotation Disabled Passthrough

When `RotationConfig.enabled` is `false`, no rotation MUST occur regardless of file size.

#### Scenario: Disabled rotation lets the file grow unbounded

- GIVEN `RotationConfig.enabled = false`
- WHEN records are dispatched well past what `max_size_mb` would otherwise trigger
- THEN no rotation occurs and all records remain in the single file

### Requirement: FR-005 Output Port Conformance

The file exporter MUST be dispatchable through the same Output Port `console-exporter` implements (see `output-adapter-contracts`'s spec).

#### Scenario: File exporter is registrable alongside other outputs

- GIVEN a file exporter and a console exporter, each conforming to the Output Port
- WHEN both are registered
- THEN both are dispatchable identically through the same registration and dispatch mechanism
