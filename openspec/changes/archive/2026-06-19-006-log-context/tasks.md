# Tasks: Log Context & Enrichment (006-log-context, AS-02)

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~220-300 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | auto-chain |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

## Phase 1: Foundation

- [x] 1.1 Add `EnrichmentError(String)` variant + Display arm — `crates/kitlogger-log-domain/src/validation.rs`

## Phase 2: Core (TDD: RED → GREEN)

- [x] 2.1 Declare `mod log_context;` + `pub use` re-export — `crates/kitlogger-log-domain/src/lib.rs`
- [x] 2.2 RED: Write failing unit tests for LogContext (new, Default, getters, enrichment, Display, duplicate rejection) — inline `#[cfg(test)]` in `crates/kitlogger-log-domain/src/log_context.rs`
- [x] 2.3 GREEN: Implement LogContext — constructor, getters, Default, Display — `crates/kitlogger-log-domain/src/log_context.rs`
- [x] 2.4 GREEN: Implement enrichment methods — `with_attribute` (duplicate name → Err), `with_correlation_id`, `with_trace_id`, `with_span_id` — `crates/kitlogger-log-domain/src/log_context.rs`

## Phase 3: Integration Tests

- [x] 3.1 Add integration tests: full enrichment pipeline chaining, attribute isolation, ID enrichment idempotency — `crates/kitlogger-log-domain/tests/integration_tests.rs`

## Phase 4: Verification

- [x] 4.1 Run `cargo test -p kitlogger-log-domain` — all tests pass (30/30)
- [x] 4.2 Run `cargo clippy -p kitlogger-log-domain` — no warnings
- [x] 4.3 Run `cargo fmt -p kitlogger-log-domain` — ensure formatting

### Test Commands

```sh
cargo test -p kitlogger-log-domain
cargo clippy -p kitlogger-log-domain
cargo fmt -p kitlogger-log-domain
```
