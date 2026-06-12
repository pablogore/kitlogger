# Quickstart Validation Guide

This guide provides validation scenarios to prove the telemetry data model feature works end-to-end.

## Prerequisites

- Node.js 16+ installed
- npm or yarn package manager
- Basic understanding of OpenTelemetry concepts

## Setup

1. Install the telemetry library:
   ```bash
   npm install @your-org/telemetry-core
   ```

2. Create a simple Node.js application:
   ```bash
   mkdir telemetry-demo && cd telemetry-demo
   npm init -y
   ```

## Validation Scenarios

### Scenario 1: Trace Creation and Span Management

1. Create a trace with multiple spans:
   ```javascript
   import { Trace, Span } from '@your-org/telemetry-core';
   
   const trace = new Trace('trace-123');
   const span1 = new Span('span-1', 'trace-123');
   const span2 = new Span('span-2', 'trace-123', 'span-1');
   
   trace.addSpan(span1);
   trace.addSpan(span2);
   ```

2. Verify trace and span data:
   ```javascript
   console.log(trace);
   console.log(span1);
   console.log(span2);
   ```

3. Expected outcome: Trace and spans are created with correct IDs and relationships.

### Scenario 2: Metric Collection

1. Create a metric:
   ```javascript
   import { Metric } from '@your-org/telemetry-core';
   
   const metric = new Metric('cpu_usage', 85.5, 'percent', Date.now());
   ```

2. Verify metric data:
   ```javascript
   console.log(metric);
   ```

3. Expected outcome: Metric is created with correct name, value, unit, and timestamp.

### Scenario 3: Log Recording

1. Create a log:
   ```javascript
   import { Log } from '@your-org/telemetry-core';
   
   const log = new Log('info', 'Application started', Date.now());
   ```

2. Verify log data:
   ```javascript
   console.log(log);
   ```

3. Expected outcome: Log is created with correct severity, body, and timestamp.

## Contract References

- [Telemetry Data Model Contract](contracts/telemetry-data-model.md)
- [Data Model Specification](data-model.md)

## Expected Results

After running all validation scenarios, you should see:
1. Trace objects with proper span relationships
2. Metric objects with correct attributes
3. Log objects with proper severity levels

These validations confirm that the telemetry data models are correctly implemented and can be used for observability purposes.