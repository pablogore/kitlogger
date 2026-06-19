# Tasks: Console Exporter

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 250–350 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | ask-on-risk |
| Chain strategy | feature-branch-chain |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: feature-branch-chain
400-line budget risk: Low

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Full crate (src/ + tests) | Single PR | Under 400 lines, no split needed |

## Phase 1: Foundation

- [x] 1.1 Add `crates/console-exporter` to workspace members in root `Cargo.toml`
- [x] 1.2 Create `crates/console-exporter/Cargo.toml` with dep on `kitlogger-log-domain` only
- [x] 1.3 Create `crates/console-exporter/src/error.rs` with `ExportError` enum (I/O, lifecycle, flush variants)

## Phase 2: Core Implementation

- [x] 2.1 Create `crates/console-exporter/src/stream_router.rs` with level-to-stream routing, `set_mapping`, `set_writers`
- [x] 2.2 Create `crates/console-exporter/src/lifecycle.rs` with state machine: Uninitialized→Running→Flushing→Shutdown
- [x] 2.3 Create `crates/console-exporter/src/flush.rs` with `FlushStrategy` trait + Immediate, OnShutdown, Batch impls
- [x] 2.4 Create `crates/console-exporter/src/exporter.rs` with `ConsoleExporter`: `fn export(&self, &str, Severity)`, `fn flush()`, `fn shutdown()`
- [x] 2.5 Create `crates/console-exporter/src/lib.rs` with module tree and public re-exports

## Phase 3: Testing

- [x] 3.1 Unit tests for StreamRouter: default mapping, custom mapping, write error handling
- [x] 3.2 Unit tests for LifecycleStateMachine: valid transitions, delivery after shutdown, error recovery
- [x] 3.3 Unit tests for FlushStrategy: Immediate flushes every write, Batch flushes at threshold, OnShutdown buffers
- [x] 3.4 Integration tests: export string through exporter → assert output on `Vec<u8>` writers
- [x] 3.5 Integration tests: lifecycle end-to-end (init → deliver → flush → shutdown)
