# Archive Report: Redaction & Sampling

**Date**: 2026-07-04
**Change ID**: 013
**Change Name**: 013-redaction-sampling
**Status**: ARCHIVED
**Implementation PRs**: [#32](https://github.com/pablogore/kitlogger/pull/32) (`kitlogger-redaction`), [#33](https://github.com/pablogore/kitlogger/pull/33) (`kitlogger-sampling`), both merged to `develop`

## Executive Summary

Implements Migration Plan Phase 3 (Leaf Capability Absorption) of ADR-008. Two new, independent crates: `kitlogger-redaction` (sensitive-attribute redaction over `LogRecord`) and `kitlogger-sampling` (emission-volume sampling decisions, all four `SamplingStrategy` variants). Landed as two separate PRs per this change's own review-workload split (no shared code between the crates). Neither crate is wired into `KITLogger` — that remains a later phase (Orchestration Fold).

## Capabilities Merged Into Canonical Specs

- `openspec/specs/kitlogger-redaction/spec.md` — new capability, FR-001 through FR-003.
- `openspec/specs/kitlogger-sampling/spec.md` — new capability, FR-001 through FR-007.

## Terminology Correction Applied During Archive

A post-merge code review of PR #33 found `RateLimit`'s window described as a "sliding window" in both code comments and this change's own spec (FR-007), when the implemented (and specified) behavior — count resets to zero once the window elapses, rather than decaying continuously — is a fixed-window reset, not a classic sliding window. The code comments were corrected in PR #33's follow-up commit. The canonical spec merged here carries the same correction (FR-007: "sliding window" → "one-second window"); FR-004 already used the less specific "rolling one-second window" and did not need changing. Behavior is unchanged — this is a terminology-only correction, applied consistently to both code and spec.

## Requirements Delivered

`kitlogger-redaction`: FR-001 (Sensitive Field Detection, algorithm-agnostic), FR-002 (Immutability Preserved), FR-003 (Disabled Config Passthrough) — all delivered as specified.

`kitlogger-sampling`: FR-001 through FR-006 delivered as specified. FR-007 (Injectable Time Source) delivered by reusing `kitlogger-log-domain`'s existing `Clock` trait (per ADR-010 — no competing time abstraction introduced), with a documented precondition added post-review: `Sampler::new()` assumes `SamplingConfig` has already passed `kit-config`'s own validation and does not re-validate it, to avoid a second place those rules could drift from `kit-config`'s.

## Verification Performed

- `cargo build --workspace`, `cargo test --workspace` (11 new tests: 4 redaction, 7 sampling), `cargo clippy --workspace -- -D warnings`, `cargo fmt --check` — all clean on the merge commit.
- Grep-confirmed: neither new crate is referenced from `crates/kitlogger/`; neither new crate depends on the other.
- A merge conflict between PR #32 and PR #33 (both added a workspace member to the same `Cargo.toml` array and independently regenerated `Cargo.lock`) was resolved by keeping both members and regenerating `Cargo.lock` from scratch; re-verified green after resolution.

## Noted Implementation Deviations (Both Within Spec Latitude, Not Divergences)

1. `kitlogger-redaction`'s case-insensitivity test uses a mixed-case *configured field* rather than a mixed-case *attribute name* — `kitlogger_log_domain::LogAttribute` validates names as lowercase-only (`^[a-z][a-z0-9._]{0,63}$}`), so the latter cannot be constructed. Same FR-001 contract, compliant with the existing domain model.
2. `kitlogger-sampling`'s `RateLimit` window-reset tests use a test-local `AdvanceableClock` (implementing the same canonical `Clock` trait with `Mutex`-based interior mutability) rather than `kitlogger_log_domain::FakeClock`, which is immutable after construction and cannot exercise window-reset scenarios. Explicitly licensed by FR-007/tasks.md's "or equivalent injectable time source" — no competing production `Clock` abstraction was introduced.

## Not Delivered By This Change (Explicitly Deferred)

Per this change's own `proposal.md`: wiring `Redactor`/`Sampler` into `KITLogger`'s emission path, buffering, formatting, output/dispatch, and `telemetry-transport-contract` removal are not part of this change. Tracked by later, separate changes (014 onward) already frozen in `openspec/changes/`.

## Follow-Up Items

None blocking.
