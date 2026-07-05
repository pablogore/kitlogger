# Tasks: Verification Closure — `log()` Formatter Coverage

No `design.md` — there is no implementation decision to make; `log()` already delegates to `log_record()` and shares its formatting pipeline (see `proposal.md`'s Verified Contract and Verification Notes). No `spec.md` — no capability is introduced or modified; the guarantee under test was never a formally specified requirement, only an implicit consistency expectation between two existing public methods.

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | < 100 |
| 400-line budget risk | None |
| Chained PRs recommended | No |

## Phase 1: Verification

- [x] 1.1 Re-confirm `log()` still delegates to `log_record()` and both reach `format_and_dispatch` (`crates/kitlogger/src/lib.rs`) — guards against the code having moved since the proposal was written. If that delegation no longer exists but the observable contract (every configured formatter produces its expected output through `log()`) still holds by some other path, update `proposal.md` to reflect the new path before writing tests — the proposal commits to the observable contract, not to this specific implementation.
- [x] 1.2 Note `log()`'s signature (`log(&self, severity: Severity, message: &str)`) never carries `attributes` or `LogContext` — it always constructs the record with `Vec::new()` attrs and calls `log_record(&record, None)`. Task 2 below must test what `log()` can actually produce, not copy `log_record` test fixtures that rely on attrs/context `log()` has no way to pass.

## Phase 2: Test Coverage

Add to `crates/kitlogger/tests/pipeline_integration.rs`, mirroring the existing `log_record_*_format_produces_*_output` tests but calling `.log(severity, message)` instead of `.log_record(&record, None)`:

- [x] 2.1 `log_json_format_produces_json_output` — mirrors `log_record_json_format_produces_json_output`; assert the same `"level":"INFO"` / `"msg":"login ok"` keys, plus a negative assertion that the output does not contain the Text formatter's `[INFO]` prefix — guards against a future refactor silently falling back to the legacy/Text path.
- [x] 2.2 `log_text_format_produces_text_output` — mirrors `log_record_text_format_with_logger_context`, but since `log()` always passes `context: None`, assert the no-logger-name variant: `[INFO] login ok`, not `[INFO] auth: login ok`.
- [x] 2.3 `log_human_readable_format_basic` — mirrors `log_record_human_readable_format_basic`, but drop the `service=api` attribute assertion since `log()` has no way to attach attributes; assert only `INFO` and the message text.
- [x] 2.4 `log_logfmt_format_produces_kv_pairs` — mirrors `log_record_logfmt_format_produces_kv_pairs` using `Severity::Warn`; assert `level=WARN` and `msg="slow query"` on stderr.
- [x] 2.5 `log_error_severity_goes_to_stderr` — mirrors `log_record_error_severity_goes_to_stderr`; added after verify flagged that `log()`'s Error/stderr routing had no dedicated test (SUGGESTION, non-blocking, closed anyway).

## Phase 3: Close

- [x] 3.1 Run `cargo test -p kitlogger` — confirm all five new tests pass and no existing test regresses.
- [x] 3.2 Run `cargo clippy` and `cargo fmt --check`.
- [x] 3.3 Verify every Success Criterion in `proposal.md` has explicit test coverage.
