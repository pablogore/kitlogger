# Quickstart: Context Propagation and Correlation

## Prerequisites

- Rust toolchain (nightly or stable with appropriate features)
- Package root determined by CORE-000 Release Engineering
- `.specify/tech-stack.yaml` declares: Rust, Tokio, cargo test, OpenTelemetry

## Setup

Package creation is owned by CORE-000 Release Engineering. The following dependency is required:

```toml
[dependencies]
uuid = { version = "1.3", features = ["v7", "serde"] }
```

## Validation Scenarios

### Scenario 1: Trace Context Round-Trip

**Goal**: Verify W3C Trace Context can be injected and extracted correctly.

```bash
cargo test -- test_trace_context_roundtrip
```

**Expected**: A `traceparent` header value, when injected then extracted, yields the original trace_id, span_id, and trace_flags.

### Scenario 2: Baggage Propagation

**Goal**: Verify W3C Baggage entries survive injection/extraction.

```bash
cargo test -- test_baggage_propagation
```

**Expected**: Three baggage entries injected into a carrier are all present after extraction.

### Scenario 3: Cross-Signal Correlation

**Goal**: Verify correlation ID is generated and can be shared across signals.

```bash
cargo test -- test_correlation_generation
```

**Expected**: A generated UUID v7 is valid, time-sortable, and non-zero.

### Scenario 4: Multi-Hop Trace Context

**Goal**: Verify context propagation across multiple simulated service hops.

```bash
cargo test -- test_multi_hop_propagation
```

**Expected**: After 5 hops, the trace_id remains the same and each hop has a unique span_id with correct parent relationships.

### Scenario 5: Malformed Context Handling

**Goal**: Verify graceful handling of malformed context headers.

```bash
cargo test -- test_malformed_context
```

**Expected**: Invalid `traceparent` headers (bad format, all-zeros, wrong length) produce empty/fallback context without panicking.

## Integration Points

| Component | Integration |
|-----------|-------------|
| AS-02 Transport-Agnostic Telemetry Flow | Consumes carrier abstraction for transport bindings |
| AS-03 Telemetry Adapter Contracts | Consumes context types for OpenTelemetry adapter mapping |
| AS-04 Telemetry Configuration Semantics | Provides configuration for propagation behavior (sampling, etc.) |

## Data Model Reference

See `data-model.md` for full entity definitions and validation rules.

## Contract Reference

See `contracts/carrier.md` and `contracts/propagation.md` for trait definitions and contract tests.
