# Telemetry Adapter Contracts — Transport Addition Specification

## Purpose

This spec covers only what change 018 adds to `telemetry-adapter-contracts`: a protocol-agnostic transport strategy, relocated from the orphaned `telemetry-transport-contract` crate per its Phase 7 handoff. The rest of `telemetry-adapter-contracts`'s existing surface (`Adapter`, `ExporterAdapter`, `AdapterRegistry`, `mapping.rs`, `AdapterId`, `AdapterHealth`, `AdapterLifecycle`) is specified elsewhere (the pre-openspec `specs/archive/2026-06-19-002-core-telemetry-domain-model-as-03-telemetry-adapter-contracts/` convention) and is out of scope here — unmodified by this change.

Traceability: proposal `openspec/changes/018-crate-removal/proposal.md`.

## Requirements

### Requirement: FR-001 Transport Trait Availability

`telemetry-adapter-contracts` MUST provide a protocol-agnostic `Transport` concept: a send operation that accepts a telemetry payload and returns the delivery mode used, without assuming any specific wire protocol (HTTP, gRPC, in-memory, or otherwise).

#### Scenario: A transport implementation is protocol-agnostic

- GIVEN a type implementing the `Transport` concept
- WHEN it is inspected
- THEN nothing in its contract requires any specific wire protocol

### Requirement: FR-002 Transport Is Optional, Not Required by Adapter

Implementing `Adapter` (or `ExporterAdapter`/`ProviderAdapter`) MUST NOT require using the `Transport` concept. An adapter with no wire-protocol concern (e.g. a local, synchronous sink) MUST be able to satisfy `Adapter` without depending on `Transport` at all.

#### Scenario: A local adapter has no Transport dependency

- GIVEN an adapter implementation with no network or wire-protocol behavior
- WHEN its dependencies are inspected
- THEN no dependency on the `Transport` concept is required to satisfy `Adapter`

### Requirement: FR-003 Delivery Mode Reporting

A successful send operation via `Transport` MUST report which delivery mode was used (e.g. fire-and-forget, request-response, batch, or streaming).

#### Scenario: Delivery mode is reported on success

- GIVEN a `Transport` implementation that completes a send
- WHEN the operation succeeds
- THEN the delivery mode actually used is reported to the caller

### Requirement: FR-004 Transport Failure Reporting

`Transport` operations MUST report failures distinctly from `AdapterError` (registry/lifecycle-management failures). At minimum, timeout, unavailability, backpressure (carrying the canonical `telemetry_types::BackpressureSignal`), payload-too-large, and unsupported-transport conditions MUST be distinguishable from one another.

#### Scenario: Distinct transport failure kinds are reported

- GIVEN a `Transport` send operation that fails
- WHEN the failure is inspected
- THEN it is reported as one of the distinguishable transport-failure kinds, not conflated with a registry/lifecycle failure

#### Scenario: Backpressure carries the canonical signal type

- GIVEN a `Transport` send operation that fails due to backpressure
- WHEN the failure is inspected
- THEN it carries a `telemetry_types::BackpressureSignal` value, not a second, competing backpressure type
