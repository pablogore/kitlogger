# Proposal: KITLOGGER-001 — Verification Closure: `log()` Formatter Coverage

**Type:** Verification / regression-coverage closure — no functional changes, no API changes, no new capabilities. Do not expect behavior to change as a result of this change.

## Intent

CORE-017 reported that `KITLogger::log()` bypasses the configured formatter while `KITLogger::log_record()` respects it — producing a legacy output format regardless of `logging.format`. That finding motivated this proposal's original framing: make `log()` formatter-aware.

Re-verification against the current codebase before writing a spec/design found that finding is stale. `log()` already delegates to `log_record()` and therefore shares the same formatting pipeline — every configured format handles a missing context gracefully rather than falling back to a different output. See Verification Notes below for the specific code path this was confirmed against.

What is real: `log()` and `log_record()` are both public API and look equivalent from a caller's perspective — nothing marks one as a convenience wrapper with weaker guarantees. That equivalence is exactly why the missing verification mattered. The existing test suite proves each formatter's output only through `log_record()`; the only test touching `log()` checks enabled/disabled gating, not output format. So the guarantee "every public logging entry point honors the configured formatter" held in practice but was never asserted for `log()` — one refactor away from silently regressing without any test catching it.

This proposal is the closure of that verified contract: confirming no code change is required, and closing the verification gap CORE-017 actually pointed at.

## Verified Contract

This is not a formatter bypass to fix — it is an untested guarantee on two API surfaces that appear interchangeable. `log()` already reaches the same formatting pipeline as `log_record()`; the risk was never having a test that would fail if that stopped being true.

## Scope

### In Scope

- Add integration tests exercising `log()` across all four configured formats (`Json`, `HumanReadable`, `Text`, `Logfmt` — these cover every formatter currently supported by KITLogger), mirroring the existing `log_record_*_format_produces_*_output` tests in `pipeline_integration.rs`.
- Confirm, for the record, that `log()` and `log_record()` share one formatting pipeline today, with zero code changes.

### Out of Scope

- Choosing the default formatter.
- Introducing new formatter types.
- Changing the `LogRecord` data model.
- Structured logging enhancements, tracing, or OpenTelemetry.
- Any change to `log()`'s signature or behavior.
- Adding a `LogFormat::ALL`/`EnumIter`-style enumeration API to convert these tests to table-driven — that would change `kitlogger-formatter`'s public surface, which this closure's own Type line rules out. Considered during code review and deliberately deferred to a separate proposal if wanted; the explicit per-format tests stay as-is here.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- None.

## Approach

This is a test-coverage closure, not a design or implementation change. `log()` already follows the same formatter path as `log_record()`. The only artifact this produces is regression tests proving each configured formatter produces its expected output through `log()`, so a future refactor that reintroduces a `log()`-specific bypass fails CI instead of shipping silently.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/kitlogger/tests/pipeline_integration.rs` | Modified | Add tests exercising each configured format through `log()`, mirroring the existing `log_record_*` tests |
| `crates/kitlogger/src/lib.rs` | None | No code changes — `log()` already delegates to `log_record()` |

## Success Criteria

- An automated test proves that each configured formatter (`Json`, `HumanReadable`, `Text`, `Logfmt`) produces its expected output through `log()`, not just through `log_record()`.
- `log()` and `log_record()` produce equivalent formatter output for equivalent log events, except for the optional context fields `log()` intentionally omits (it always passes `context: None`).
- Existing callers of `log()` continue to compile unchanged.
- No duplicate formatting implementation is introduced.

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| A future refactor reintroduces a `log()`-specific bypass unnoticed | Low today; was true before #44 | New tests added by this change catch it in CI |

## Verification Notes

_Point-in-time evidence for the re-verification above. Expected to move into `verify-report.md` once verification runs — kept here only until then, so the rest of this proposal doesn't depend on line numbers or commit hashes that will drift as the code evolves._

- `log()` builds a `LogRecord` and delegates to `log_record()` (`crates/kitlogger/src/lib.rs:297-301`).
- `format_and_dispatch` calls `self.formatter.format(record, context)` for every record regardless of which public method produced it (`crates/kitlogger/src/lib.rs:333`).
- This delegation was introduced by commit `2c96685` (#44), which landed after CORE-017's original verification.
- Existing coverage: `pipeline_integration.rs`'s `log_record_*_format_produces_*_output` tests exercise all four formats only via `log_record()`. `logging_config_test.rs:88-95` is the only test touching `log()`, and it checks enabled/disabled gating, not format.
