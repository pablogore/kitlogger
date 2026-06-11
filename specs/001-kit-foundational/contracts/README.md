# Contracts: KIT-001 Foundational Observability Abstractions

This directory contains interface contracts for the observability abstractions.

## Overview

The contracts define the public API surface for the Kit framework's observability system. These contracts are designed to be stable and backward-compatible, ensuring that downstream consumers can rely on consistent interfaces.

## Core Contracts

### Context Contract

Defines the interface for trace context management, including trace_id, span_id, correlation_id, and attributes.

### Resource Contract

Defines the interface for resource metadata management, including arbitrary attributes for describing service instances.

### InstrumentationScope Contract

Defines the interface for instrumentation scope management, including name and optional version.

### Span Contract

Defines the interface for span creation and management in distributed tracing.

### LogRecord Contract

Defines the interface for structured log record creation and management.

### Metric Contract

Defines the interface for metric creation and recording across the four instrument types.

## Versioning

Contracts follow semantic versioning principles. Breaking changes to contracts will result in major version bumps.

## Compatibility

All contracts are designed to be:
- Backend agnostic
- Vendor neutral
- Domain agnostic
- OpenTelemetry compatible