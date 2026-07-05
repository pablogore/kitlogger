# Archive Report: KITLOGGER-001 — Verification Closure: `log()` Formatter Coverage (Change 020)

## Status

Shipped. All tasks in `tasks.md` complete (`[x]`). Merged into `develop` via PR #60.

## What shipped

Zero functional or API changes, as scoped — `log()` already delegated to `log_record()` before this change and shared its formatting pipeline (`crates/kitlogger/src/lib.rs`). This closure adds the regression coverage that guarantee never had, in `crates/kitlogger/tests/pipeline_integration.rs`:

- `log_json_format_produces_json_output` — asserts `log()` produces JSON output, plus a negative assertion against the Text formatter's `[INFO]` prefix leaking through.
- `log_text_format_produces_text_output` — asserts the no-logger-name variant (`log()` always passes `context: None`).
- `log_human_readable_format_basic` — asserts severity and message text (no attributes, since `log()` has no way to attach them).
- `log_logfmt_format_produces_kv_pairs` — asserts `level=WARN` / `msg="slow query"` on stderr.
- `log_error_severity_goes_to_stderr` — closes a gap flagged during verification (SUGGESTION, non-blocking) where `log()`'s Error/stderr routing had no dedicated test.

## Why no code change

CORE-017 originally reported `log()` bypasses the configured formatter. Re-verification against the current codebase (post-#44, which introduced the `log()` → `log_record()` delegation) found that finding stale: both methods already reach `format_and_dispatch`. What was real is that this equivalence was never asserted for `log()` — the existing suite proved each formatter's output only through `log_record()`, leaving `log()` one refactor away from silently regressing without any test catching it. This closure is that missing test coverage, not a bypass fix.

## Verification

- `cargo test -p kitlogger` — all five new tests pass, no existing test regresses.
- `cargo clippy` / `cargo fmt --check` — clean.
- Every Success Criterion in `proposal.md` has explicit test coverage.
- No spec merge required — no capability was introduced or modified; the guarantee under test was an implicit consistency expectation between two existing public methods, never a formally specified requirement.
