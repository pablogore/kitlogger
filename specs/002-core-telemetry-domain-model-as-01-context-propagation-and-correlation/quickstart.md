# Quickstart: Context Propagation and Correlation

## Prerequisites

- Rust toolchain (edition 2021)
- All technologies declared in [tech-stack.yaml](tech-stack.yaml)

## Setup

```bash
# Navigate to project root
cd /path/to/kitlogger

# Build the crate
cargo build

# Run all tests
cargo test
```

## Validation Scenarios

### Scenario 1: Trace Context Roundtrip

Verify that Trace Context can be injected into a carrier and extracted back.

```bash
cargo test test_trace_context_roundtrip
```

**Expected**: The extracted context has the same trace_id, span_id, and trace_flags as the original.

### Scenario 2: Correlation Identifier Propagation

Verify that a CorrelationIdentifier can be injected and extracted.

```bash
cargo test test_correlation_roundtrip
```

**Expected**: The extracted identifier matches the original.

### Scenario 3: Baggage Add and Retrieve

Verify that baggage entries can be added and retrieved.

```bash
cargo test test_baggage_add_entry
cargo test test_baggage_get_entry
```

**Expected**: Entries are stored and retrievable by key.

### Scenario 4: End-to-End Context Propagation (3-Hop Simulation)

Simulate propagating Trace Context across 3 service hops.

```rust
// Hop 1 → Hop 2 → Hop 3
let mut carrier = MapCarrier::new();
let propagator = TraceContextPropagator::new();
propagator.inject(&mut carrier, &trace_context);
let extracted = propagator.extract(&carrier);
```

**Run**: `cargo test test_trace_context_propagator`

### Scenario 5: Cross-Signal Correlation

Verify that a CorrelationIdentifier correlates across signals.

```bash
cargo test test_correlation_propagator
```

## Test Suite

All tests are in `tests/`:

| Test File | Scenarios |
|-----------|-----------|
| `tests/trace_context_test.rs` | Creation, parsing, roundtrip, validation |
| `tests/correlation_test.rs` | Generation, roundtrip |
| `tests/baggage_test.rs` | Entry creation, add, get, roundtrip |
| `tests/propagation_test.rs` | Full propagator roundtrips |

Run the full suite:

```bash
cargo test
```
