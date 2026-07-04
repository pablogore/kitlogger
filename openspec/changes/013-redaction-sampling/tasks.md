# Tasks: Redaction & Sampling

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 300–420 |
| 400-line budget risk | Medium |
| Chained PRs recommended | Yes — one per crate |
| Suggested split | PR 1: `kitlogger-redaction`, PR 2: `kitlogger-sampling` |
| Delivery strategy | ask-on-risk (already resolved: split) |
| Chain strategy | stacked-to-main |

Decision needed before apply: No (already resolved — split into two PRs, no shared code between them)
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: Medium (per-crate; combined estimate would exceed budget)

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | `kitlogger-redaction` crate, full | PR 1 | Independent of Unit 2 |
| 2 | `kitlogger-sampling` crate, full | PR 2 | Independent of Unit 1; can be developed in parallel |

---

## Phase 1: `kitlogger-redaction`

- [ ] 1.1 Add `"crates/kitlogger-redaction"` to workspace `members` in root `Cargo.toml`.
- [ ] 1.2 Create `crates/kitlogger-redaction/Cargo.toml` — deps: `kit-config` (path, sibling repo, matching the pattern already used elsewhere in the workspace since Phase 2), `kitlogger-log-domain` (path).
- [ ] 1.3 **RED** — Write failing test `redacts_matching_field_case_insensitive` asserting an attribute named e.g. `Password` (mixed case) is replaced when `config.fields` contains `"password"`. Satisfies FR-001.
- [ ] 1.4 **RED** — Write failing test `leaves_non_matching_fields_untouched`. Satisfies FR-001.
- [ ] 1.5 **GREEN** — Implement `Redactor` holding `kit_config::RedactionConfig`, with the case-insensitive substring matching logic. Run `cargo test -p kitlogger-redaction` — 1.3 and 1.4 pass.
- [ ] 1.6 **RED** — Write failing test `does_not_mutate_input_record` asserting the original `LogRecord` passed in is unchanged after the redaction call. Satisfies FR-002.
- [ ] 1.7 **GREEN** — Ensure the redaction function returns a new `LogRecord` rather than mutating in place. Run — 1.6 passes.
- [ ] 1.8 **RED** — Write failing test `disabled_config_returns_record_unchanged` with `config.enabled = false`. Satisfies FR-003.
- [ ] 1.9 **GREEN** — Add the enabled-check short-circuit. Run — 1.8 passes.
- [ ] 1.10 Run `cargo clippy -p kitlogger-redaction -- -D warnings` and `cargo fmt --package kitlogger-redaction -- --check`.

---

## Phase 2: `kitlogger-sampling`

- [x] 2.1 Add `"crates/kitlogger-sampling"` to workspace `members` in root `Cargo.toml`.
- [x] 2.2 Create `crates/kitlogger-sampling/Cargo.toml` — deps: `kit-config` (path), `fastrand` (matching the orphaned original's choice for the `Probabilistic` strategy), `kitlogger-log-domain` (path, for the existing `Clock` abstraction only — not for `LogRecord`).
- [x] 2.3 **RED** — Write failing test `none_strategy_always_samples`. Satisfies FR-001.
- [x] 2.4 **GREEN** — Implement `Sampler` holding `kit_config::SamplingConfig`; `None` arm. Run — 2.3 passes.
- [x] 2.5 **RED** — Write failing test `every_nth_strategy_deterministic_sequence` asserting the exact `true`/`false` pattern over `n` calls. Satisfies FR-002.
- [x] 2.6 **GREEN** — Implement `EveryNth` arm with an internal counter. Run — 2.5 passes.
- [x] 2.7 **RED** — Write failing test `probabilistic_strategy_within_statistical_tolerance` (large sample count, assert observed rate within tolerance of `config.rate`). Satisfies FR-003.
- [x] 2.8 **GREEN** — Implement `Probabilistic` arm using `fastrand`. Run — 2.7 passes.
- [x] 2.9 **RED** — Write failing test(s) for `RateLimit`'s sliding one-second window behavior using `kitlogger_log_domain::FakeClock` (or equivalent injectable time source) — no real sleeps. Satisfies FR-004 and FR-007. (Deviation: `FakeClock` is fixed at construction with no way to advance it, so it cannot exercise the window-boundary/reset scenarios. Used a test-local `AdvanceableClock` implementing the same canonical `kitlogger_log_domain::Clock` trait — explicitly allowed by this task's "or equivalent injectable time source" — with `Mutex`-based interior mutability for test-only advancement. No competing production `Clock` abstraction was introduced.)
- [x] 2.10 **GREEN** — Implement `RateLimit` arm, sourcing time exclusively through the injected clock (never `Instant::now()`/`SystemTime::now()` directly). Run — 2.9 passes.
- [x] 2.10a Grep `kitlogger-sampling/src` for direct `Instant::now()`/`SystemTime::now()` calls outside the clock abstraction — zero matches. Satisfies FR-007.
- [x] 2.11 **RED** — Write failing test `disabled_config_always_samples`. Satisfies FR-005. (Note: passed on first run — the top-level `enabled` short-circuit in `should_sample` was already implemented as part of 2.4's minimal `Sampler`, since FR-001/FR-005 were implemented together for a coherent baseline. Regression guard, not new logic.)
- [x] 2.12 **GREEN** — Add the enabled-check short-circuit. Run — 2.11 passes.
- [x] 2.13 Run `cargo clippy -p kitlogger-sampling -- -D warnings` and `cargo fmt --package kitlogger-sampling -- --check`.

---

## Phase 3: Verification

- [x] 3.1 Run `cargo test --workspace` — all tests pass; no regressions elsewhere.
- [x] 3.2 Confirm neither `kitlogger-redaction` nor `kitlogger-sampling` is referenced from `crates/kitlogger/` yet (out of scope for this change — wiring is Phase 5).
- [x] 3.3 Confirm neither new crate depends on the other.
