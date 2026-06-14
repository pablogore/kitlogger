# Remediation: AS-01 Context Propagation and Correlation

**PR-18 Review Date**: 2026-06-14

**Objective**: Bring implementation into full compliance with the AS-01 approved architecture.

**Constraints**: No new features, transports, or protocols. Specification artifacts not modified.

---

## Gap Closure Summary  

| Gap | Description | Status | Verification |
|-----|-------------|--------|-------------|
| GAP-01 | Remove transport-specific propagators | ✅ Complete | Zero references to `HttpTraceContextPropagator`, `GrpcTraceContextPropagator`, etc. remain |
| GAP-02 | Extraction must not fabricate context | ✅ Complete | `Propagator::extract` returns `Option<Self::Context>`; no fallback context creation |
| GAP-03 | Remove invalid domain entities | ✅ Complete | No `[0;16]` or `[0;8]` fallback arrays remain in propagator code |
| GAP-04 | Complete tracestate roundtrip | ✅ Complete | `TraceState` entries serialized in `inject`, parsed and reconstructed in `extract` |
| GAP-05 | Complete W3C Baggage support | ✅ Complete | `BaggageProperty::KeyValue` and `BaggageProperty::Flag` roundtrip via `;` separator |
| GAP-06 | Preserve `parent_span_id` | ✅ Complete | Serialized as `parent-span-id` header; extracted and reconstructed on roundtrip |
| GAP-07 | Implement PropagationMetadata | ✅ Complete | `src/propagation_metadata.rs` created, registered in `src/lib.rs` |
| GAP-08 | Restore behavioral tests | ✅ Complete | 35 tests with value assertions (roundtrip, extraction failure, multi-hop, carrier polymorphism) |
| GAP-09 | Align implementation with tasks | ✅ Complete | Every change maps to tasks T002–T015 in `tasks.md` |
| GAP-10 | Tighten `tech-stack.yaml` | ✅ Complete | Removed Kafka, RabbitMQ (outside AS-01 ownership) |

---

## Task Completion Matrix

| Task | Description | Files Changed | Status |
|------|-------------|---------------|--------|
| T018 | Remove transport-specific propagators | `src/http_propagation.rs`, `src/grpc_propagation.rs` | ✅ Stubs only; no exports |
| T019 | Implement HttpHeaderCarrier | `src/carrier.rs` | ✅ Wraps `&mut MapCarrier`, delegates Injector + Extractor |
| T020 | Implement GrpcMetadataCarrier | `src/carrier.rs` | ✅ Same pattern as HttpHeaderCarrier |
| T021 | Refactor propagators to use carrier abstraction | `src/propagation.rs` | ✅ Generic `Propagator` trait with `Option` extract |
| T022 | Implement PropagationMetadata | `src/propagation_metadata.rs`, `src/lib.rs` | ✅ Entity with `transport`, `entries`, `get()`, `keys()` |
| T023 | Replace synthetic extraction with Result/Option | `src/propagation.rs`, `src/carrier.rs` | ✅ `Propagator::extract` returns `Option<Self::Context>` |
| T024 | Complete tracestate roundtrip | `src/propagation.rs` | ✅ Inject serializes; extract parses and populates TraceState |
| T025 | Complete baggage property serialization | `src/propagation.rs` | ✅ Full W3C format with `;` property separator |
| T026 | Restore behavioral assertions | `tests/propagation_test.rs`, `tests/baggage_test.rs`, `tests/correlation_test.rs`, `tests/trace_context_test.rs` | ✅ 35 tests with assertions on values |
| T027 | Verify public API compatibility | — | ✅ `cargo doc --no-deps` builds clean |
| T028 | Run compliance validation | — | ✅ `cargo build` (0 warnings), `cargo test` (35/35 pass) |

---

## Architecture Compliance

### Carrier Abstraction

```text
Propagator trait
├── inject(&self, carrier: &mut dyn Injector, context: &Self::Context)
├── extract(&self, carrier: &dyn Extractor) -> Option<Self::Context>
└── fields(&self) -> &'static [&'static str]

Injector trait
└── set(&mut self, key: &str, value: &str)

Extractor trait
├── get(&self, key: &str) -> Option<&str>
└── get_all(&self, key: &str) -> Vec<&str>

Carrier implementations (Injector + Extractor):
├── MapCarrier          — HashMap-based, for testing/in-process
├── HttpHeaderCarrier   — Wraps MapCarrier, HTTP semantics
└── GrpcMetadataCarrier — Wraps MapCarrier, gRPC semantics
```

### Propagator Implementations

| Propagator | Context Type | Header | Format |
|-----------|-------------|--------|--------|
| `TraceContextPropagator` | `TraceContext` | `traceparent`, `tracestate`, `parent-span-id` | W3C Trace Context |
| `CorrelationPropagator` | `CorrelationIdentifier` | `correlation-id` | UUID v7 |
| `BaggagePropagator` | `Baggage` | `baggage` | W3C Baggage |

### Extraction Semantics

- All propagators return `Option<Self::Context>`
- `None` returned when:
  - Required header is absent
  - Header value is malformed (invalid format, zero trace/span ID, nil UUID)
- No propagator creates synthetic fallback contexts

### Roundtrip Guarantees

| Property | Preserved | Verified By |
|----------|-----------|-------------|
| `trace_id` | ✅ | `test_trace_context_roundtrip` |
| `span_id` | ✅ | `test_trace_context_roundtrip` |
| `parent_span_id` | ✅ | `test_trace_context_roundtrip_with_parent_span_id` |
| `trace_flags` | ✅ | `test_trace_context_roundtrip` |
| `trace_state` entries | ✅ | `test_tracestate_roundtrip` |
| `BaggageEntry.key` | ✅ | `test_baggage_roundtrip` |
| `BaggageEntry.value` | ✅ | `test_baggage_roundtrip` |
| `BaggageProperty::KeyValue` | ✅ | `test_baggage_roundtrip_with_properties` |
| `BaggageProperty::Flag` | ✅ | `test_baggage_roundtrip_with_properties` |
| `CorrelationIdentifier.id` | ✅ | `test_correlation_roundtrip` |
| `CorrelationIdentifier.created_at` | ✅ | `test_correlation_roundtrip` |

### Multi-Hop Propagation

- 3-hop baggage propagation (`test_baggage_multi_hop`): all entries survive the full chain
- Carrier polymorphism (`test_propagator_works_with_http_carrier`, `test_propagator_works_with_grpc_carrier`): same propagator works with all carrier types

---

## Public API Surface

### Modules

| Module | Exported Types |
|--------|---------------|
| `carrier` | `Injector`, `Extractor`, `Propagator`, `MapCarrier`, `HttpHeaderCarrier`, `GrpcMetadataCarrier` |
| `trace_context` | `TraceId`, `SpanId`, `TraceFlags`, `TraceState`, `TraceContext` |
| `correlation` | `CorrelationIdentifier` |
| `baggage` | `Baggage`, `BaggageEntry`, `BaggageProperty` |
| `propagation` | `TraceContextPropagator`, `CorrelationPropagator`, `BaggagePropagator` |
| `propagation_metadata` | `PropagationMetadata` |

### Removed (per architecture)

| Module | Rationale |
|--------|-----------|
| `http_propagation` | Transport-specific; replaced by carrier abstraction |
| `grpc_propagation` | Transport-specific; replaced by carrier abstraction |

---

## Validation

```text
$ cargo build
    Finished `dev` profile — 0 warnings

$ cargo test
    Running 35 tests
    test result: ok. 35 passed

$ cargo doc --no-deps
    Generated docs — no errors
```
