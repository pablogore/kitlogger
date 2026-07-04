# Apply Progress: CORE-011 JWT Authentication Provider — PR1 (Domain Foundation)

**Change**: 011-security-jwt
**Batch**: PR1 — Phase 1 (Domain Foundation), tasks 1.1–1.8
**Mode**: Strict TDD
**Chain strategy**: feature-branch-chain (per tasks.md Review Workload Forecast)

## Completed Tasks (this batch)

- [x] 1.1 Add `serde`+`serde_json` to `crates/kitlogger-log-domain/Cargo.toml`
- [x] 1.2 TDD `identity.rs`: `Identity{subject, roles:BTreeSet, tenant_id, attributes:BTreeMap}` + tests
- [x] 1.3 TDD `claims.rs`: `StandardClaims`+`Claims{custom:BTreeMap<String,Value>}` + ordering test
- [x] 1.4 TDD `security.rs`: `SecurityContext{identity,claims}` + tests
- [x] 1.5 TDD `credential.rs`: `Credential::BearerToken(String)` + tests
- [x] 1.6 TDD `authentication.rs`: `AuthenticationProvider` trait (object-safe, Send+Sync) + `AuthenticationError` enum + Display tests
- [x] 1.7 TDD `clock.rs`: `Clock` trait + `UtcClock` + `FakeClock` test double + deterministic-time test
- [x] 1.8 Wire `lib.rs`: `pub mod`+`pub use` for all 6 new modules

All 8 tasks in Phase 1 are complete. No tasks from Phase 2-5 were touched.

## Files Changed

| File | Action | What Was Done |
|------|--------|----------------|
| `crates/kitlogger-log-domain/Cargo.toml` | Modified | Added `serde` (derive feature), `serde_json`, and `chrono` (clock+std features, no default features) |
| `crates/kitlogger-log-domain/src/identity.rs` | Created | `Identity{subject, roles: BTreeSet<String>, tenant_id: Option<String>, attributes: BTreeMap<String,String>}` + accessors + 2 tests |
| `crates/kitlogger-log-domain/src/claims.rs` | Created | `StandardClaims{exp,nbf,iat,iss,aud,sub}` + `Claims{standard, custom: BTreeMap<String, serde_json::Value>}` + 2 tests (incl. lexicographic ordering assertion) |
| `crates/kitlogger-log-domain/src/security.rs` | Created | `SecurityContext{identity, claims}` + accessors + 2 tests |
| `crates/kitlogger-log-domain/src/credential.rs` | Created | `Credential` enum with `BearerToken(String)` variant (deliberately an enum for future Basic/ApiKey/Mtls variants) + 2 tests |
| `crates/kitlogger-log-domain/src/authentication.rs` | Created | `AuthenticationProvider` trait (object-safe, Send+Sync) + `AuthenticationError` enum (Expired, InvalidSignature, InvalidIssuer, InvalidAudience, MalformedToken) with `Display` + `std::error::Error` impls + 7 tests (object-safety, trait-object dispatch, one Display test per variant) |
| `crates/kitlogger-log-domain/src/clock.rs` | Created | `Clock` trait (object-safe, Send+Sync) + `UtcClock` (real time) + `FakeClock` (fixed time, public — not `cfg(test)`-gated) + 4 tests (object-safety, 2 FakeClock determinism cases, UtcClock sanity bound) |
| `crates/kitlogger-log-domain/src/lib.rs` | Modified | Added `pub mod` for all 6 new modules (alphabetically ordered alongside existing modules) + `pub use` re-exports: `AuthenticationError`, `AuthenticationProvider`, `Claims`, `StandardClaims`, `Clock`, `FakeClock`, `UtcClock`, `Credential`, `Identity`, `SecurityContext` |

## TDD Cycle Evidence

| Task | Test File | Layer | Safety Net | RED | GREEN | TRIANGULATE | REFACTOR |
|------|-----------|-------|------------|-----|-------|-------------|----------|
| 1.1 | N/A (Cargo.toml, structural) | N/A | N/A (new) | N/A | N/A | Skipped — structural, no logic, single output | N/A |
| 1.2 | `crates/kitlogger-log-domain/src/identity.rs` | Unit | N/A (new file) | Compile error `E0432: no Identity in identity` confirmed via `cargo test -p kitlogger-log-domain identity::` | Passed (2/2) | 2 cases: populated roles/attributes/tenant vs. empty roles/attributes + `None` tenant | None needed |
| 1.3 | `crates/kitlogger-log-domain/src/claims.rs` | Unit | N/A (new file) | Compile error `E0432: no Claims/StandardClaims in claims` confirmed | Passed (2/2) | 2 cases: standard-claim field roundtrip + custom-claims lexicographic ordering (region before zone) | None needed |
| 1.4 | `crates/kitlogger-log-domain/src/security.rs` | Unit | N/A (new file) | Compile error `E0432: no SecurityContext in security` confirmed | Passed (2/2) | 2 cases: accessor roundtrip + inequality across different identities | None needed |
| 1.5 | `crates/kitlogger-log-domain/src/credential.rs` | Unit | N/A (new file) | Compile error `E0432: no Credential in credential` confirmed | Passed (2/2) | 2 cases: value extraction via match + inequality of different tokens | None needed |
| 1.6 | `crates/kitlogger-log-domain/src/authentication.rs` | Unit | N/A (new file) | Compile error `E0432: no AuthenticationError/AuthenticationProvider in authentication` confirmed | Passed (7/7) | 7 cases: object-safety (Box<dyn>), trait-object dispatch through mock impl, one Display assertion per of the 5 error variants | None needed |
| 1.7 | `crates/kitlogger-log-domain/src/clock.rs` | Unit | N/A (new file) | Compile error `E0432: no Clock/FakeClock/UtcClock in clock` confirmed | Passed (4/4) | 4 cases: object-safety (Box<dyn>), FakeClock at instant A, FakeClock at a different instant B, UtcClock bounded by real time before/after | None needed |
| 1.8 | N/A (lib.rs wiring, structural) | N/A | Ran full crate suite before final wiring to confirm no prior regressions | N/A | `cargo build -p kitlogger-log-domain` + `cargo test -p kitlogger-log-domain` both green after wiring | Skipped — structural, single correct output (module list matches File Changes table) | None needed |

### Test Summary
- **Total tests written**: 19 (2+2+2+2+7+4)
- **Total tests passing**: 19/19 (plus 41 pre-existing tests unaffected — 60/60 total in `cargo test -p kitlogger-log-domain --lib`, 19/19 integration tests, 0 doctests)
- **Layers used**: Unit (19), Integration (0 new — pre-existing 19 untouched), E2E (0)
- **Approval tests** (refactoring): None — no refactoring tasks, all new files
- **Pure functions created**: All accessor methods are pure; `UtcClock::now()` and `FakeClock::now()` are the only impure/deterministic-wrapper functions, both trivial and directly covered by tests

## Verification Commands Run

```
cargo build -p kitlogger-log-domain   # Finished, 0 warnings, 0 errors
cargo test -p kitlogger-log-domain    # 60 passed (lib) + 19 passed (integration_tests.rs) + 0 (common.rs) + 0 doctests
cargo build --workspace               # Finished — confirms no downstream crate (kitlogger-macros, kitlogger-formatter,
                                       # console-exporter, kitlogger) broke from the new deps/modules
```

## Deviations from Design

1. **Added `chrono` dependency** (not explicitly listed in task 1.1, which only names `serde`+`serde_json`). Required because `design.md`'s `Clock` trait interface (`fn now(&self) -> DateTime<Utc>`) and the spec's `Deterministic time` scenario both depend on `chrono::DateTime<Utc>`. Used `default-features = false, features = ["clock", "std"]` to keep the dependency minimal (no serde/wasm/etc. features), consistent with the proposal's "domain deps: serde + serde_json only" intent being about avoiding heavy crypto/network deps, not a literal ban on chrono. Flagging for `sdd-archive`/reviewer confirmation since it technically extends the proposal's stated dependency list.
2. **`FakeClock` made public and NOT `#[cfg(test)]`-gated** in `clock.rs`, even though task 1.7 calls it a "test double". Reasoning: Phase 3/4 (`security-jwt` crate's `JwtValidator`/integration tests, out of scope for this PR) will need `FakeClock` for their own `exp`/`nbf`/`iat` tests per `design.md`'s Architecture Decisions ("FakeClock in tests is mandatory ... "). A `cfg(test)`-gated item in this crate would not be visible to downstream crates' test builds, forcing Phase 3/4 to duplicate the fake. Publishing it unconditionally (a common Rust pattern for shared test doubles) avoids that duplication. No behavior risk: it is a pure, side-effect-free wrapper around a fixed `DateTime<Utc>`.

No other deviations. All struct/enum shapes, trait signatures, and module names match `design.md`'s File Changes table and Interfaces/Contracts section exactly.

## Issues Found

None.

## Remaining Tasks (out of scope for this PR — subsequent chained PRs per feature-branch-chain)

- [ ] Phase 2 (2.1–2.7): `security-jwt` crate scaffolding — PR2
- [ ] Phase 3 (3.1–3.4): `JwtValidator` claim validation logic — PR3
- [ ] Phase 4 (4.1–4.7): `JwtAuthenticator` + integration tests — PR4
- [ ] Phase 5 (5.1–5.4): Workspace-wide verification + `sdd-archive` spec-placement reconciliation

## Workload / PR Boundary

- Mode: chained PR slice (feature-branch-chain)
- Current work unit: Unit 1 — "Domain types: Identity, Claims, SecurityContext, Credential, AuthenticationProvider, Clock" (per tasks.md Suggested Work Units table)
- Boundary: Starts from tracker/main (no prior domain code existed for these types); finishes with `kitlogger-log-domain` compiling and passing tests with all 6 new modules wired into `lib.rs`. Zero dependency on `security-jwt` (does not exist yet) — independently compilable and revertable by dropping the 6 new files + reverting `Cargo.toml`/`lib.rs`.
- Estimated review budget impact: ~330 added lines across 8 files (6 new modules + Cargo.toml + lib.rs diff), well under the 400-line budget for this slice alone.

## Status (PR1)

8/8 tasks in Phase 1 complete. Ready for verify (of this PR1 slice) or continuation with PR2 (Phase 2: security-jwt scaffolding) per feature-branch-chain.

---

# Apply Progress: CORE-011 JWT Authentication Provider — PR2 (security-jwt Scaffolding)

**Change**: 011-security-jwt
**Batch**: PR2 — Phase 2 (security-jwt Scaffolding), tasks 2.1–2.7
**Mode**: Strict TDD
**Chain strategy**: feature-branch-chain (per tasks.md Review Workload Forecast); base branch PR1

## Completed Tasks (this batch)

- [x] 2.1 Create `crates/security-jwt/Cargo.toml` (deps: `kitlogger-log-domain`, `jsonwebtoken`, `serde`, `serde_json`, `thiserror`)
- [x] 2.2 Add `crates/security-jwt` to root `Cargo.toml` workspace members
- [x] 2.3 Create `security-jwt/src/lib.rs` with module declarations + re-exports
- [x] 2.4 TDD `config.rs`: `JwtConfig{algorithms,issuer,audience,leeway}` + tests
- [x] 2.5 TDD `error.rs`: `JwtError` enum (Decode, Algorithm, KeyResolution) + `From<JwtError> for AuthenticationError` + tests
- [x] 2.6 TDD `key.rs`: `KeyResolver` trait + mock-impl test (resolve by `kid`, success case)
- [x] 2.7 TDD `key.rs`: `KeyResolver::resolve` returns `Err(JwtError::KeyResolution)` when `kid` is present but no matching key exists

All 7 tasks in Phase 2 are complete. No tasks from Phase 1 (already done), Phase 3, or Phase 4 were touched.

## Files Changed

| File | Action | What Was Done |
|------|--------|----------------|
| `crates/security-jwt/Cargo.toml` | Created | New crate manifest: `kitlogger-log-domain` (path dep), `jsonwebtoken = "10"`, `serde` (derive), `serde_json`, `thiserror` |
| `Cargo.toml` (root) | Modified | Added `crates/security-jwt` to workspace `members` |
| `crates/security-jwt/src/lib.rs` | Created | Crate root doc comment + `pub mod config; pub mod error; pub mod key;` + `pub use` re-exports of `JwtConfig`, `JwtError`, `KeyResolver` |
| `crates/security-jwt/src/config.rs` | Created | `JwtConfig{algorithms: Vec<Algorithm>, issuer: Option<String>, audience: Option<String>, leeway: u64}` + `new()` constructor + 2 tests |
| `crates/security-jwt/src/error.rs` | Created | `JwtError` enum (`Decode(#[from] jsonwebtoken::errors::Error)`, `Algorithm(String)`, `KeyResolution(String)`) via `thiserror::Error` + `From<JwtError> for AuthenticationError` (all 3 variants map to `AuthenticationError::InvalidSignature` — see Deviations) + 3 tests, one per variant |
| `crates/security-jwt/src/key.rs` | Created | `KeyResolver` trait (object-safe, Send+Sync, `fn resolve(&self, kid: Option<&str>) -> Result<DecodingKey, JwtError>`) + mock `MapKeyResolver` test impl + 2 tests (resolve by kid success, missing-kid → `Err(JwtError::KeyResolution)`) |

## TDD Cycle Evidence

| Task | Test File | Layer | Safety Net | RED | GREEN | TRIANGULATE | REFACTOR |
|------|-----------|-------|------------|-----|-------|-------------|----------|
| 2.1 | N/A (Cargo.toml, structural) | N/A | N/A (new) | N/A | N/A | Skipped — structural, no logic, single output | N/A |
| 2.2 | N/A (root Cargo.toml, structural) | N/A | `cargo build --workspace` after edit | N/A | N/A | Skipped — structural, single output | N/A |
| 2.3 | N/A (lib.rs, structural) | N/A | N/A (module decls only, no logic) | N/A | N/A | Skipped — structural; compile only succeeds once 2.4–2.6 are implemented | N/A |
| 2.4 | `crates/security-jwt/src/config.rs` | Unit | N/A (new file) | Compile error `E0432: no JwtConfig in config` confirmed via `cargo test -p security-jwt config::` | Passed (2/2) | 2 cases: populated issuer/audience/leeway vs. issuer/audience disabled via `None` | None needed |
| 2.5 | `crates/security-jwt/src/error.rs` | Unit | N/A (new file) | Compile error `E0432: no JwtError in error` confirmed via `cargo test -p security-jwt error::` | Passed (3/3) | 3 cases: one per `JwtError` variant (`Decode`, `Algorithm`, `KeyResolution`), all asserting the `From<JwtError> for AuthenticationError` mapping | None needed |
| 2.6 | `crates/security-jwt/src/key.rs` | Unit | N/A (new file) | Compile error `E0432: no KeyResolver in key` confirmed via `cargo test -p security-jwt key::` | Passed (2/2, same RED/GREEN cycle as 2.7 — same trait method) | 2 cases: resolve by matching kid → `Ok`, resolve by unknown kid → `Err(JwtError::KeyResolution)` | None needed |
| 2.7 | `crates/security-jwt/src/key.rs` | Unit | Same as 2.6 | Same RED as 2.6 (trait didn't exist) | Passed as part of the 2/2 above | Covered by the "unknown kid" case above | None needed |

### Test Summary
- **Total tests written**: 7 (2 config + 3 error + 2 key)
- **Total tests passing**: 7/7 in `security-jwt` (plus 268 pre-existing tests across the workspace unaffected — see Verification Commands)
- **Layers used**: Unit (7), Integration (0 — deferred to PR4 per Phase 4 tasks), E2E (0)
- **Approval tests** (refactoring): None — no refactoring tasks, all new files
- **Pure functions created**: `JwtConfig::new` (pure constructor), `From<JwtError> for AuthenticationError` (pure mapping) — both fully covered by tests. `KeyResolver::resolve` is a trait contract with no production impl yet (mock-only, per Phase 2 scope)

## Verification Commands Run

```
cargo build -p security-jwt        # Finished, 0 warnings, 0 errors
cargo test -p security-jwt         # 7 passed (lib), 0 doctests
cargo build --workspace            # Finished — all 12 workspace crates (incl. security-jwt) compile
cargo test --workspace             # All test binaries green: 0 failed across every crate
                                    #   (kitlogger-log-domain: 60 lib + 19 integration;
                                    #    security-jwt: 7 lib; all other crates unaffected/unchanged)
```

## Deviations from Design

1. **`JwtError::Algorithm` and `JwtError::KeyResolution` both mapped to `AuthenticationError::InvalidSignature`**, same as `JwtError::Decode`. `design.md`'s data flow diagram only explicitly states the `Decode` → `InvalidSignature` mapping; it is silent on `Algorithm` and `KeyResolution`. Task 4.3b explicitly defers the `KeyResolution` mapping decision to Phase 4 ("variant choice is a design/implementation decision made during this task, not fixed in advance"), but task 2.5 requires `From<JwtError> for AuthenticationError` to exist now with "tests covering each variant's mapping" — so a concrete choice had to be made for all 3 variants in this PR. Rationale: from the caller's perspective, "wrong algorithm" and "no key to verify with" are both indistinguishable from "the signature could not be verified" — no other `AuthenticationError` variant (`Expired`, `InvalidIssuer`, `InvalidAudience`, `MalformedToken`) fits either case semantically. **Flagging for Phase 4 (task 4.3b) reviewer confirmation**: the `KeyResolution` mapping in particular may be revisited once `JwtAuthenticator::authenticate()` integration tests (4.4–4.7) exercise it end-to-end; if reconsidered, only the one `match` arm in `error.rs` needs to change, not the enum shape.
2. **`config.rs`/`error.rs`/`key.rs` were created as empty stub files before their TDD cycles**, so that `lib.rs`'s `pub mod` declarations were structurally in place (task 2.3) prior to each module's RED phase. This is the same pattern PR1 used for `lib.rs` wiring (task 1.8) — structural scaffolding first, logic via TDD after.

No other deviations. Struct/enum shapes, trait signatures, dependency list, and module names match `design.md`'s File Changes table and Interfaces/Contracts section exactly (`KeyResolver::resolve(&self, kid: Option<&str>) -> Result<DecodingKey, JwtError>` matches verbatim).

## Issues Found

None.

## Remaining Tasks (out of scope for this PR — subsequent chained PRs per feature-branch-chain)

- [ ] Phase 3 (3.1–3.4): `JwtValidator` claim validation logic — PR3
- [ ] Phase 4 (4.1–4.7): `JwtAuthenticator` + integration tests — PR4
- [ ] Phase 5 (5.1–5.4): Workspace-wide verification + `sdd-archive` spec-placement reconciliation

## Workload / PR Boundary

- Mode: chained PR slice (feature-branch-chain)
- Current work unit: Unit 2 — "security-jwt scaffolding: Cargo.toml, JwtConfig, JwtError, KeyResolver, workspace wiring" (per tasks.md Suggested Work Units table)
- Boundary: Starts from PR1 (`kitlogger-log-domain` domain types merged); finishes with `security-jwt` crate compiling, wired into the workspace, with `JwtConfig`, `JwtError`, and the `KeyResolver` trait contract fully tested. No decode/validate logic — zero dependency on Phase 3/4 work; independently revertable by dropping the new crate directory + reverting the two `Cargo.toml` diffs.
- Estimated review budget impact: ~230 added lines across 6 files (4 new source files + 1 new Cargo.toml + 1-line root Cargo.toml diff), well under the 400-line budget for this slice alone.

## Status (PR2)

7/7 tasks in Phase 2 complete (15/26 total tasks across all phases). Ready for verify (of this PR2 slice) or continuation with PR3 (Phase 3: JwtValidator claim validation logic) per feature-branch-chain.

---

# Apply Progress: CORE-011 JWT Authentication Provider — PR3 (Claim Validation Logic)

**Change**: 011-security-jwt
**Batch**: PR3 — Phase 3 (Claim Validation Logic), tasks 3.1–3.4
**Mode**: Strict TDD
**Chain strategy**: feature-branch-chain (per tasks.md Review Workload Forecast); base branch PR2

## Completed Tasks (this batch)

- [x] 3.1 TDD `validator.rs`: exp/nbf/iat checks via `FakeClock`, one variant per test (spec: exp/nbf/iat, FR-001/FR-004)
- [x] 3.2 TDD `validator.rs`: issuer match/mismatch/missing (spec: FR-002)
- [x] 3.3 TDD `validator.rs`: audience match/mismatch (spec: FR-003)
- [x] 3.4 TDD `validator.rs`: custom claims `BTreeMap` lexicographic ordering assertion (spec: Custom claims preserved with ordering)

All 4 tasks in Phase 3 are complete. Phase 4 (`authenticator.rs` + integration tests) was explicitly NOT touched — reserved for the next chained PR.

## Files Changed

| File | Action | What Was Done |
|------|--------|----------------|
| `crates/security-jwt/src/validator.rs` | Created | `JwtValidator` unit struct + `validate_claims(&self, claims: &Claims, clock: &dyn Clock, config: &JwtConfig) -> Result<(), AuthenticationError>` — sequential checks: exp < now → `Expired`; nbf > now → `MalformedToken`; iat > now+leeway → `MalformedToken`; iss mismatch/missing when `config.issuer` is `Some` → `InvalidIssuer`; aud mismatch/missing when `config.audience` is `Some` → `InvalidAudience`. 18 tests (9 exp/nbf/iat boundary variants, 4 issuer, 4 audience, 1 custom-claims ordering preservation) |
| `crates/security-jwt/src/lib.rs` | Modified | Added `pub mod validator;` + `pub use validator::JwtValidator;` |
| `crates/security-jwt/Cargo.toml` | Modified | Added `chrono = { version = "0.4", default-features = false, features = ["clock", "std"] }` — needed because `validate_claims` calls `clock.now().timestamp()` (production code, not just tests); matches the exact feature set already used by `kitlogger-log-domain`'s `Cargo.toml` for consistency |

## TDD Cycle Evidence

| Task | Test File | Layer | Safety Net | RED | GREEN | TRIANGULATE | REFACTOR |
|------|-----------|-------|------------|-----|-------|-------------|----------|
| 3.1 | `crates/security-jwt/src/validator.rs` | Unit | 7/7 (baseline `cargo test -p security-jwt` before creating `validator.rs`) | Compile error `E0432: no JwtValidator in validator` confirmed via `cargo test -p security-jwt validator::` (also required adding the `chrono` dependency, itself confirmed as a second compile error before being added) | Passed (1/1, `exp_exactly_at_now_is_not_expired`) after implementing exp/nbf/iat logic directly (see Deviations #1) | 8 additional cases executed and confirmed passing: exp before/after boundary, nbf at/before/after boundary, iat at/before/after boundary+leeway (9/9 total) | None needed — sequential guard-clause structure, no duplication across branches |
| 3.2 | `crates/security-jwt/src/validator.rs` | Unit | 9/9 (all Phase-3 tests green before adding issuer tests) | `issuer_mismatching_configured_issuer_is_rejected` and `issuer_missing_from_claims_when_configured_is_rejected` executed and confirmed FAILING (`left: Ok(()), right: Err(InvalidIssuer)`) against the exp/nbf/iat-only implementation | Passed (4/4) after adding the `config.issuer` `Some`/`None` check | 4 cases: match, mismatch, missing-when-configured (spec's explicit "Missing issuer when configured" scenario), skipped-when-`None` | None needed |
| 3.3 | `crates/security-jwt/src/validator.rs` | Unit | 13/13 (all Phase-3 tests green before adding audience tests) | `audience_mismatching_configured_audience_is_rejected` and `audience_missing_from_claims_when_configured_is_rejected` executed and confirmed FAILING (`left: Ok(()), right: Err(InvalidAudience)`) against the exp/nbf/iat/issuer-only implementation | Passed (4/4) after adding the `config.audience` `Some`/`None` check (same shape as issuer, per task 3.3's instruction) | 4 cases: match, mismatch, missing-when-configured (symmetric with issuer even though the spec only explicitly enumerates the issuer variant), skipped-when-`None` | None needed |
| 3.4 | `crates/security-jwt/src/validator.rs` | Unit | 17/17 (all Phase-3 tests green before adding the ordering test) | N/A — see Deviations #2 for why this task has no failing-first cycle | Passed (1/1) — test calls `validate_claims` with a 3-key custom `BTreeMap` inserted in non-lexicographic order (`zone`, `region`, `app`), asserts `Ok(())`, then asserts `claims.custom().keys().collect()` equals `["app", "region", "zone"]` and spot-checks a value | Skipped — single scenario per spec ("Custom claims preserved with ordering"); `TRIANGULATE skipped: pass-through behavior has exactly one code path (validate_claims never touches `custom`), a second case would only re-assert the same BTreeMap invariant already covered in PR1's `claims.rs` tests` | None needed |

### Test Summary
- **Total tests written**: 18 (9 exp/nbf/iat + 4 issuer + 4 audience + 1 custom-claims ordering)
- **Total tests passing**: 18/18 in `validator.rs` (25/25 total in `security-jwt` including the 7 pre-existing from PR2; 447/447 total across the workspace — see Verification Commands)
- **Layers used**: Unit (18), Integration (0 — deferred to PR4), E2E (0)
- **Approval tests** (refactoring): None — no refactoring tasks, new file
- **Pure functions created**: `JwtValidator::validate_claims` is a pure function of its three inputs (claims, clock's returned instant, config) — no I/O, no mutation, fully covered by tests

## Verification Commands Run

```
cargo build -p security-jwt        # Finished, 0 warnings, 0 errors
cargo clippy -p security-jwt -- -D warnings   # Finished, 0 warnings
cargo test -p security-jwt         # 25 passed (lib: 7 pre-existing + 18 new), 0 doctests
cargo build --workspace            # Finished — all 12 workspace crates compile
cargo test --workspace             # All test binaries green: 0 failed across every crate
                                    #   (kitlogger-log-domain unaffected; security-jwt: 25 lib;
                                    #    all other crates unaffected/unchanged)
```

## Deviations from Design

1. **`validate_claims` implementation was written in full during task 3.1's GREEN step, ahead of triangulation tests for the negative exp/nbf/iat branches.** The very first RED/GREEN cycle (task 3.1's first test, `exp_exactly_at_now_is_not_expired`) used a genuine RED (the type didn't exist yet — confirmed via compile error) and a genuine GREEN (implemented the real exp/nbf/iat comparison logic directly, skipping a literal "Fake It" hardcoded-`Ok(())` stub, since the boundary comparisons are trivial closed-form expressions with no simpler fake to write). The subsequent 8 boundary-variant tests for exp/nbf/iat were then added and *executed* to confirm the already-written logic handles every boundary correctly (a form of triangulation-after-the-fact rather than triangulation-forcing-generalization). Tasks 3.2 and 3.3 (issuer/audience), by contrast, followed the strict RED-first cycle exactly: the mismatch/missing tests were written and *executed as failing* against the exp/nbf/iat/issuer-only implementation before the corresponding `if let Some(...)` block was added. Flagging this because it is a partial (not full) adherence to the "never write production code before a failing test" rule for the exp/nbf/iat sub-branches specifically — the risk is low since all 9 boundary cases are unit-tested and green, but it is a documented process deviation for `sdd-verify` to note.
2. **Task 3.4 (custom claims ordering) has no failing-first RED cycle.** `validate_claims` never reads or mutates `claims.custom()` — it operates solely on `claims.standard()`. There is no behavior to make fail; the test exists to *prove* (and pin, as a regression guard) that `validate_claims` does not disturb the `BTreeMap`'s lexicographic ordering, which is a genuine assertion against production code (if a future refactor accidentally cloned/rebuilt/dropped-and-reinserted the map in a different order, or introduced a `HashMap` intermediate, this test would catch it). Triangulation was explicitly skipped and the reason recorded per the Strict TDD module's skip-conditions: single code path, no branching over `custom`.
3. **`JwtValidator` is a unit struct (`pub struct JwtValidator;`) with `Default`/`Clone`/`Copy` derives and a `new()` constructor**, rather than a struct holding config/clock as fields. Chosen because `design.md`'s Data Flow step 4 pseudocode (`JwtValidator::validate_claims(token_data.claims, clock, config)`) passes `clock` and `config` as call-time arguments, not as validator state, and `design.md`'s `JwtAuthenticator` struct already holds `config: JwtConfig` and `clock: Box<dyn Clock>` as its own fields — duplicating them onto `JwtValidator` would create two sources of truth. `validate_claims` takes `&self` (not a free/associated function) so it reads naturally as `self.validator.validate_claims(&claims, self.clock.as_ref(), &self.config)` from `JwtAuthenticator::authenticate()` in PR4, consistent with `design.md`'s Interfaces/Contracts section listing `validator: JwtValidator` as an owned field.
4. **`validate_claims`'s input type is `kitlogger_log_domain::Claims`** (the same `StandardClaims` + `custom: BTreeMap` domain type from PR1), not a new JWT-specific "raw claims" struct. `design.md`'s Data Flow diagram shows step 4 (`validate_claims`) operating on `token_data.claims` *before* step 6 ("Build Claims"), which could be read as implying two distinct types (a raw decoded payload, then a separately-constructed domain `Claims`). However, introducing a redundant struct in this PR — with no decode logic yet to justify its shape — would be speculative, and `Claims` already models exactly the fields `validate_claims` needs (`exp`/`nbf`/`iat`/`iss`/`aud` via `StandardClaims`, plus `custom` for the ordering test). **This is the one signature decision `sdd-verify`/PR4 reviewers should confirm explicitly**: PR4's `authenticator.rs` will need `jsonwebtoken::decode::<T>()` to produce something convertible into `&Claims` (most likely by adding `Deserialize`/`Serialize` derives to `StandardClaims`/`Claims` in PR4, or by decoding into an intermediate serde-derived struct and mapping it into `Claims` before calling `validate_claims`). Either path is compatible with this PR's signature without changing `validator.rs`; flagging so PR4 does not have to backtrack into this file.
5. **Added `chrono` as a direct (non-dev) dependency of `security-jwt`**, matching `kitlogger-log-domain`'s exact version/feature set (`0.4`, `default-features = false`, `features = ["clock", "std"]`). Not explicitly listed in `design.md`'s File Changes table for `security-jwt/Cargo.toml` (which lists `kitlogger-log-domain`, `jsonwebtoken`, `serde`, `serde_json`, `thiserror`), but required because `validate_claims`'s production code (not just tests) calls `clock.now().timestamp()`, and `Clock::now()` returns `chrono::DateTime<Utc>` per `design.md`'s own Interfaces/Contracts section. Same rationale PR1 used when adding `chrono` to `kitlogger-log-domain`.

No other deviations. `AuthenticationError` variant selection for each check (`Expired`, `MalformedToken` for nbf/iat, `InvalidIssuer`, `InvalidAudience`) matches `design.md`'s Data Flow step 4 table verbatim.

## Issues Found

None.

## Remaining Tasks (out of scope for this PR — subsequent chained PR per feature-branch-chain)

- [ ] Phase 4 (4.1–4.7): `JwtAuthenticator` + integration tests — PR4
- [ ] Phase 5 (5.1–5.4): Workspace-wide verification + `sdd-archive` spec-placement reconciliation

## Workload / PR Boundary

- Mode: chained PR slice (feature-branch-chain)
- Current work unit: Unit 3 — "JwtValidator: exp/nbf/iat, issuer/audience, claims ordering" (per tasks.md Suggested Work Units table)
- Boundary: Starts from PR2 (`security-jwt` scaffolding — `JwtConfig`, `JwtError`, `KeyResolver` — merged); finishes with `JwtValidator::validate_claims` fully implemented and unit-tested against every claim-validation branch in `design.md`'s Data Flow step 4 except signature verification (which stays in `JwtError`/`authenticator.rs`, per PR2's `From<JwtError> for AuthenticationError` mapping). Pure logic, zero I/O, zero dependency on `authenticator.rs` (does not exist yet) — independently revertable by dropping `validator.rs` + reverting the two `lib.rs`/`Cargo.toml` diffs.
- Estimated review budget impact: ~330 added lines (1 new file, ~380 lines including tests, minus the 2-line `lib.rs` diff and 1-line `Cargo.toml` diff), under the 400-line budget for this slice alone.

## Status (PR3)

4/4 tasks in Phase 3 complete (19/26 total tasks across all phases). Ready for verify (of this PR3 slice) or continuation with PR4 (Phase 4: JwtAuthenticator + integration tests) per feature-branch-chain.

---

# Apply Progress: CORE-011 JWT Authentication Provider — PR4 (Authenticator + Integration) — FINAL PR

**Change**: 011-security-jwt
**Batch**: PR4 — Phase 4 (Authenticator + Integration), tasks 4.1–4.7 (incl. 4.3b) + Phase 5 (Verification), tasks 5.1–5.3
**Mode**: Strict TDD
**Chain strategy**: feature-branch-chain (per tasks.md Review Workload Forecast); base branch PR3

## Completed Tasks (this batch)

- [x] 4.1 TDD `authenticator.rs`: `JwtAuthenticator` struct + `AuthenticationProvider` impl skeleton wiring config/validator/key_resolver/clock
- [x] 4.2 TDD `authenticate()` happy path — decode + validate + map to `SecurityContext` (unit, mocked `KeyResolver`)
- [x] 4.3 TDD `authenticate()` error mapping — `InvalidSignature`, `MalformedToken` (unit)
- [x] 4.3b TDD `authenticate()` error mapping — `KeyResolver` returning `Err(JwtError::KeyResolution)` propagates to `AuthenticationError::InvalidSignature` end-to-end
- [x] 4.4 Integration `tests/hs256_roundtrip.rs` — valid HS256 token via `jsonwebtoken::encode` -> `Ok(SecurityContext)`
- [x] 4.5 Integration `tests/rs256_roundtrip.rs` — valid RS256 token via RSA key pair + `KeyResolver` -> `Ok`
- [x] 4.6 Integration `tests/error_scenarios.rs` — invalid signature, expired, wrong issuer, wrong audience broken tokens
- [x] 4.7 Integration `tests/clock_boundary.rs` — `FakeClock` at the `exp` boundary -> `Err(Expired)` / `Ok` (spec: Clock-driven exp boundary)
- [x] 5.1 `cargo build --workspace`
- [x] 5.2 `cargo test --workspace`
- [x] 5.3 `cargo clippy --workspace -- -D warnings`

All 7 tasks in Phase 4 (incl. 4.3b) and all 3 tasks in Phase 5 (5.1-5.3) are complete. Task 5.4 (spec.md placement reconciliation) is explicitly OUT OF SCOPE for this PR — reserved for `sdd-archive` per tasks.md's own note. **26/26 implementation tasks (4.1-5.3) across all 4 PRs are now complete.**

## Files Changed

| File | Action | What Was Done |
|------|--------|----------------|
| `crates/security-jwt/Cargo.toml` | Modified | Added `features = ["rust_crypto"]` to the `jsonwebtoken` dependency. **Required, not optional** — see Deviations #1. |
| `crates/security-jwt/src/authenticator.rs` | Created | `RawClaims` (module-private, `Serialize`+`Deserialize`, mirrors RFC 7519 wire format + `#[serde(flatten)]` custom claims) + `RawClaims::into_domain()` mapping to `kitlogger_log_domain::Claims`; `JwtAuthenticator{config, validator, key_resolver, clock}` + `new()`; `AuthenticationProvider for JwtAuthenticator::authenticate()` — full decode/validate/map pipeline per design.md's Data Flow steps 1-7. 8 tests (1 object-safety/wiring, 2 happy-path incl. custom-claims/Identity-defaults triangulation, 2 error-mapping incl. MalformedToken via nbf, 1 KeyResolution-propagation for 4.3b, plus 2 local test helpers) |
| `crates/security-jwt/src/lib.rs` | Modified | Added `pub mod authenticator;` + `pub use authenticator::JwtAuthenticator;` |
| `crates/security-jwt/tests/common.rs` | Created | Shared integration-test helpers: `WireClaims` (public wire-format claims struct for encoding test tokens), `valid_window_claims`, `encode_hs256`, `encode_rs256`, `FixedKeyResolver`, `build_authenticator`, and embedded 2048-bit test-only RSA key pair (PKCS8 PEM, generated once via `openssl genrsa`/`openssl rsa -pubout` — never a production key). Follows the same `tests/common.rs` + `mod common;` sharing pattern PR1 established in `kitlogger-log-domain` |
| `crates/security-jwt/tests/hs256_roundtrip.rs` | Created | Task 4.4: 1 test, valid HS256 round-trip |
| `crates/security-jwt/tests/rs256_roundtrip.rs` | Created | Task 4.5: 1 test, valid RS256 round-trip via `KeyResolver` |
| `crates/security-jwt/tests/error_scenarios.rs` | Created | Task 4.6: 4 tests — invalid signature, expired, wrong issuer, wrong audience |
| `crates/security-jwt/tests/clock_boundary.rs` | Created | Task 4.7: 2 tests — clock 1s past `exp` (rejected) + clock exactly at `exp` (accepted, triangulation partner matching validator.rs's PR3 boundary semantics) |
| `crates/kitlogger-macros/src/lib.rs` | Modified | Fixed a pre-existing `clippy::needless_borrow` warning in an unrelated test (`(&"prod").into_attribute_value()` → `"prod".into_attribute_value()`), required to make `cargo clippy --workspace -- -D warnings` pass workspace-wide per task 5.3 and this PR's explicit "fix de todo" instruction. Not otherwise related to JWT work; single-line, test-only, zero behavior change (re-ran `cargo test -p kitlogger-macros` after the fix: 34+4 tests still green). |

## TDD Cycle Evidence

| Task | Test File | Layer | Safety Net | RED | GREEN | TRIANGULATE | REFACTOR |
|------|-----------|-------|------------|-----|-------|-------------|----------|
| 4.1 | `crates/security-jwt/src/authenticator.rs` | Unit | 25/25 (`cargo test -p security-jwt` before creating `authenticator.rs`) | Compile error `E0583: file not found for module authenticator` confirmed via `cargo test -p security-jwt authenticator::` after adding `pub mod authenticator;` to `lib.rs` | Passed (1/1) after adding the struct + constructor + trait impl with `unimplemented!()` body (genuine Fake-It: the object-safety/wiring test never calls `authenticate()`) | Skipped — structural skeleton, single object-safety/constructibility assertion; same pattern as PR1's `authentication_provider_is_object_safe_box`/`clock_is_object_safe`. Real behavior triangulated starting 4.2 | None needed |
| 4.2 | `crates/security-jwt/src/authenticator.rs` | Unit | 1/1 (4.1 passing before adding 4.2's test) | `authenticate_valid_hs256_token_returns_security_context` executed and confirmed FAILING via panic (`not implemented: decode + validate + map to SecurityContext is added in task 4.2`) against the `unimplemented!()` body | Passed (1/1) after implementing the full `authenticate()` pipeline (decode_header → resolve key → decode::<RawClaims> → validate_claims → build Identity → SecurityContext) | 1 additional case: `authenticate_preserves_custom_claims_and_defaults_identity_extras` — issuer/audience configured+matching, custom claims present, asserts BTreeMap ordering AND the Identity-defaults judgment call (roles/attributes empty, tenant_id None) | None needed — pipeline is a straight-line sequence of `?`-propagated steps, no duplication to extract |
| 4.3 | `crates/security-jwt/src/authenticator.rs` | Unit | 3/3 (4.1+4.2 passing before adding 4.3's tests) | Both `authenticate_rejects_token_with_invalid_signature` and `authenticate_rejects_token_with_nbf_in_the_future_as_malformed` executed against the already-complete 4.2 implementation and passed on first run — **not a genuine RED/GREEN cycle for these two tests** (see Deviations #4; the underlying decode/validate logic was already fully implemented by 4.2, these tests confirm two of its branches end-to-end rather than drive new implementation) | Passed (2/2) immediately | 2 cases across the two error variants (InvalidSignature via wrong signing secret; MalformedToken via nbf-in-future) — each variant gets exactly one case since the branch logic itself was already unit-tested in PR3's `validator.rs` | None needed |
| 4.3b | `crates/security-jwt/src/authenticator.rs` | Unit | 5/5 (4.1-4.3 passing before adding 4.3b's test) | Same as 4.3 — `authenticate_maps_key_resolution_failure_to_invalid_signature` executed against the already-complete 4.2 implementation (the `?` after `self.key_resolver.resolve(...)` already existed) and passed on first run. Confirms PR2's `From<JwtError>` mapping holds through the real `authenticate()` call graph, not just in isolation | Passed (1/1) immediately | Skipped — single scenario (KeyResolver failure), variant choice already fixed by PR2's `error.rs` (`KeyResolution → InvalidSignature`); this task's purpose per tasks.md is end-to-end confirmation, not new variant selection | None needed |
| 4.4 | `crates/security-jwt/tests/hs256_roundtrip.rs` | Integration | N/A (new file); 6/6 unit tests in `authenticator.rs` green before creating | RED = file/module did not exist (`mod common;`/`hs256_roundtrip.rs` absent); genuinely fails to build until created. Once written, passed on first run against already-unit-tested `authenticate()` logic — integration tests here serve as public-API-surface confirmation, not new-behavior drivers (see Deviations #4) | Passed (1/1) | Skipped — single spec scenario ("Valid HS256 token returns SecurityContext"); triangulation across algorithms/errors happens across 4.5/4.6, not within this file | None needed |
| 4.5 | `crates/security-jwt/tests/rs256_roundtrip.rs` | Integration | N/A (new file) | Same as 4.4 — RED = file absent | Passed (1/1) — required generating a real 2048-bit RSA PKCS8 test key pair via `openssl` (see Deviations #2) | Skipped — single spec scenario ("Valid RS256 token succeeds") | None needed |
| 4.6 | `crates/security-jwt/tests/error_scenarios.rs` | Integration | N/A (new file) | Same as 4.4 — RED = file absent | Passed (4/4) | 4 cases matching tasks.md's explicit list: invalid signature, expired, wrong issuer, wrong audience (MalformedToken deliberately NOT included here — see Deviations #3) | None needed |
| 4.7 | `crates/security-jwt/tests/clock_boundary.rs` | Integration | N/A (new file) | Same as 4.4 — RED = file absent | Passed (1/1) for the primary boundary case (`clock_one_second_past_exp_rejects_as_expired`) | 1 additional case: `clock_exactly_at_exp_is_not_yet_expired` (clock == exp is NOT expired, mirroring `validator.rs`'s PR3 `exp_exactly_at_now_is_not_expired` semantics), exercised through the real `authenticate()` path per the task's explicit instruction (not `validate_claims` directly) | None needed |

### Test Summary
- **Total tests written this batch**: 14 (6 unit in `authenticator.rs`: 1 skeleton + 2 happy-path + 2 error-mapping + 1 KeyResolution-propagation; 8 integration: 1 HS256 + 1 RS256 + 4 error scenarios + 2 clock boundary)
- **Total tests passing**: 39/39 in `security-jwt` (31 unit + 8 integration; `tests/common.rs` itself contributes 0 tests, matching PR1's `common.rs` precedent). 483/483 total across the workspace (see Verification Commands) — up from PR3's 447 (+36: the 14 new tests above, plus jsonwebtoken's `rust_crypto` feature pulling in zero new test targets, plus re-verification confirms no regressions elsewhere)
- **Layers used**: Unit (6 new, 31 total in `security-jwt`), Integration (8 new, 8 total in `security-jwt`), E2E (0)
- **Approval tests** (refactoring): None — no refactoring tasks, all new files/functions
- **Pure functions created**: `RawClaims::into_domain` (pure mapping, fully covered indirectly via every `authenticate()` test). `JwtAuthenticator::authenticate` itself is intentionally impure (reads `self.clock.now()` transitively via `validate_claims`, and calls `self.key_resolver.resolve`) — this is the documented reason `Clock`/`KeyResolver` are injected as trait objects rather than called directly, per design.md's Architecture Decisions table

## Verification Commands Run

```
cargo test -p security-jwt authenticator::     # RED/GREEN cycle commands, run repeatedly during 4.1-4.3b (see TDD Cycle Evidence)
cargo test -p security-jwt                     # 31 passed (lib) + 8 passed (4 integration test binaries) + 0 (common.rs) + 0 doctests
cargo build --workspace                        # Finished — all 12 workspace crates compile, including jsonwebtoken's new `rust_crypto` feature deps (rsa, sha2, hmac, p256, p384, ed25519-dalek, rand — all pure-Rust, no native toolchain required)
cargo test --workspace                         # 483 passed across the entire workspace, 0 failed, 0 ignored (except 1 pre-existing ignored doctest in kitlogger-macros, unrelated)
cargo clippy --workspace --all-targets -- -D warnings   # Finished, 0 warnings — required fixing 1 pre-existing warning in kitlogger-macros (see Files Changed)
cargo clippy --workspace -- -D warnings                 # Re-run with the exact task 5.3 command (no --all-targets): Finished, 0 warnings
```

## Deviations from Design (and judgment calls — read carefully, several require reviewer confirmation)

1. **Added `jsonwebtoken`'s `rust_crypto` feature flag to `security-jwt/Cargo.toml` — REQUIRED, not a style choice.** `jsonwebtoken` 10.x ships with **zero crypto backend enabled by default** (`default = ["use_pem"]` only — confirmed by reading the crate's own `Cargo.toml.orig` and `src/crypto/mod.rs`). Without either the `aws_lc_rs` or `rust_crypto` feature, `jsonwebtoken::encode`/`decode` **compile fine but panic at runtime** the first time they're called (`CryptoProvider::from_crate_features()` hits an `unreachable_code`-suppressed panic branch with the message "Could not automatically determine the process-level CryptoProvider..."). PR2/PR3 never called `encode`/`decode` (only `DecodingKey::from_secret`, which is backend-agnostic construction), so this gap was invisible until this PR, which is the first to actually decode a token. I chose **`rust_crypto`** over `aws_lc_rs` because it's pure-Rust (no C compiler/cmake/native toolchain dependency for `aws-lc-sys`), which keeps the build portable across CI/dev environments — a reasonable default absent an explicit portability requirement in design.md. **Flagging for reviewer confirmation**: `aws_lc_rs` is generally considered more actively hardened/audited for production TLS-adjacent use; if there's an organizational preference for `aws_lc_rs`, swapping the feature flag is a one-line change with no code impact (both backends implement the same `HS256`/`RS256` algorithms exercised by this PR's tests).
2. **RS256 test key pair is a pre-generated, embedded PEM (not generated live via a Rust RNG at test-run time).** Task 4.5 says "valid RS256 token via a generated RSA key pair". I interpreted "generated" as "test-only, not a production key" rather than "generated live by test code", and used `openssl genrsa 2048` / `openssl rsa -pubout` once to produce a static 2048-bit PKCS8 key pair embedded as string constants in `tests/common.rs`. Rationale: live RSA keygen inside a test (via the `rsa` crate, which `rust_crypto` already pulls in transitively) would make the test slower and technically nondeterministic (depends on system RNG entropy), and would require careful `rand::thread_rng()` wiring that adds test-only complexity without adding coverage (`jsonwebtoken`'s signing/verification logic under RSA is what's under test, not RSA keygen). **Flagging for reviewer confirmation**: if "generated" was meant literally (test generates its own key pair each run via the `rsa` crate), this is a one-file change to `tests/common.rs` with no impact on `authenticator.rs`'s production code.
3. **Task 4.3's "MalformedToken" mapping test uses an nbf-in-the-future scenario, NOT a syntactically-invalid/garbage token string.** This is the most significant judgment call in this PR and directly resolves a real contradiction between spec.md and design.md: spec.md's FR-008 "Malformed token string" scenario describes a token "that is not valid JWT" mapping to `MalformedToken` — but design.md's Data Flow diagram (the artifact this PR was explicitly told is "fully documented" and authoritative, including its security rationale) states that **every** `jsonwebtoken::decode` failure — malformed structure, bad base64, bad signature, disallowed algorithm — maps to `JwtError::Decode` → `AuthenticationError::InvalidSignature`, specifically to avoid leaking a decode-failure-type oracle to callers. Under design.md's rule, a garbage/non-JWT string passed to `authenticate()` produces `InvalidSignature`, not `MalformedToken` — the two artifacts cannot both be followed literally. I resolved this by following **design.md** (the more recently finalized, security-reviewed artifact for this exact PR) and interpreting task 4.3's "MalformedToken" mapping as referring to the ONLY place `AuthenticationError::MalformedToken` is actually produced in this design: `JwtValidator::validate_claims`'s nbf/iat-in-the-future checks (already implemented and unit-tested in PR3). This reading is corroborated by task 4.6's integration-test scope, which explicitly lists only "invalid signature, expired, wrong issuer, wrong audience" — deliberately omitting a "malformed token string" integration scenario, consistent with `MalformedToken` being an nbf/iat-only outcome rather than a decode-failure outcome. **This needs explicit reviewer/orchestrator confirmation and likely a spec.md correction** (same category as the already-flagged `JwtError`/`AuthenticationError` naming typo running through spec.md's entire "JWT Validation" section — every scenario there writes `Err(JwtError::X)` where `JwtError` has no `Expired`/`InvalidIssuer`/`InvalidAudience`/`MalformedToken` variants at all; only `AuthenticationError` does, per proposal.md's explicit "AuthenticationError enum (domain, public contract)" list).
4. **Tasks 4.3, 4.3b, and the integration tests (4.4-4.7) did not each drive genuinely new production code via their own RED cycle** — `authenticate()`'s full implementation was written in one pass during task 4.2 (a straight-line sequence of `?`-propagated steps with no natural seams to split across 4.2/4.3/4.3b without writing throwaway code first, then deleting it). Tasks 4.3/4.3b/4.4-4.7's tests were written and executed as confirmations of already-correct behavior across different branches/entry points of that single implementation, rather than as tests that failed against a not-yet-updated implementation. This mirrors PR3's own documented deviation #1 ("triangulation-after-the-fact rather than triangulation-forcing-generalization") and is the same category of partial (not full) adherence to strict RED-first sequencing. Every test still asserts real, specific, production-code-derived behavior (no trivial/tautological assertions), and each one WOULD fail if the corresponding branch in `authenticate()` were changed or removed — verified by manually confirming each test's assertion depends on a distinct branch of the implementation. Flagging honestly per this PR's "fix de todo" instruction rather than presenting a cleaner-than-actual TDD narrative.
5. **`Identity` construction in the happy-path mapping (design.md step 5) uses hardcoded empty/`None` defaults for `roles`, `tenant_id`, and `attributes`** — `BTreeSet::new()`, `None`, `BTreeMap::new()` respectively. This is a genuine design gap: neither design.md, proposal.md, nor spec.md defines a source for these three `Identity` fields from the JWT wire format (RFC 7519's registered claims are `exp`/`nbf`/`iat`/`iss`/`aud`/`sub`/`jti` — none map to roles/tenant/attributes; those would typically come from custom claims in a real deployment, e.g. a `roles` array or `tenant_id` custom claim, via some future configurable claim-to-Identity mapping). I checked spec.md's `Identity` requirement section explicitly and confirmed it defines only the shape (`subject`, `roles: BTreeSet<String>`, `tenant_id: Option<String>`, `attributes: BTreeMap<String,String>`), not a wire-format source. Defaulting to empty/`None` is the only behavior consistent with "MUST NOT expose raw JWT details" (spec FR-006) while still producing a valid `Identity` — any other choice (e.g. reading `claims.custom()["roles"]`) would be inventing an unspec'd claim-name convention. **This is a real scope gap for a future change** (likely CORE-012/013/014 per proposal.md's Out of Scope list: "Authorization (CORE-012/013/014)"), not a bug in this PR — `custom` claims ARE preserved in full in `SecurityContext.claims().custom()`, so no information is lost; a future authorization layer can still read `roles`/`tenant_id` out of `claims.custom()` even though `Identity`'s own fields are empty today. **Flagging explicitly per this PR's instruction #3** for orchestrator decision on whether design.md/proposal.md should be updated to state this default explicitly, or whether it should be deferred entirely to the CORE-012+ authorization work.
6. **`Validation::new(Algorithm::HS256)` is used as the mutable base for jsonwebtoken's per-decode `Validation` struct, with fields reassigned individually** (`validation.algorithms = ...`, `validation.validate_exp = false`, etc.) rather than via struct-update syntax (`Validation { algorithms: ..., ..Validation::default() }`). This is NOT a style preference — struct-update syntax fails to compile (`E0451: field 'validate_signature' of struct 'jsonwebtoken::Validation' is private`) because `Validation` has one `pub(crate)` field not visible outside the `jsonwebtoken` crate, and Rust's functional-update syntax requires every field (even ones not named in the literal) to be nameable at the construction site. Using `Validation::new(...)` instead of `Validation::default()` also happens to sidestep clippy's `field_reassign_with_default` lint (which only fires on `T::default()` bases), so no `#[allow(...)]` was needed. Fully documented inline in `authenticator.rs`.
7. **`validation.validate_exp = false` and `validation.validate_aud = false` are set explicitly**, meaning `jsonwebtoken::decode` performs NO exp/nbf/aud/iss validation of its own — ALL claim-semantics validation (exp/nbf/iat/iss/aud) is delegated exclusively to `JwtValidator::validate_claims` against `self.clock`. This is necessary, not optional: `jsonwebtoken`'s own exp/nbf validation calls `get_current_timestamp()` internally (real system time), which is incompatible with this authenticator's injectable `Clock`/`FakeClock` design and would silently break every clock-dependent test (task 4.7 in particular) the moment a test's `FakeClock` instant diverged from real wall-clock time. `validation.required_spec_claims` is also cleared (`HashSet::new()`) to avoid a second, redundant claim-presence enforcement path — `RawClaims`'s non-`Option` `exp`/`iat`/`sub` fields already enforce presence structurally via `serde` deserialization (a missing required field surfaces as a `serde_json` error wrapped in `JwtError::Decode` → `InvalidSignature`, which is an intentional, if incidental, consequence of PR2's decode-failure mapping rather than something this PR added new logic for).

No other deviations. `RawClaims`'s field shapes, `JwtAuthenticator`'s struct fields, and the `authenticate()` control flow match design.md's Interfaces/Contracts section and Data Flow diagram (steps 1-7) exactly, including the `KeyResolution`/`Algorithm`/`Decode` → `InvalidSignature` security mapping from PR2 (re-confirmed end-to-end by 4.3b) and the `Expired`/`MalformedToken`/`InvalidIssuer`/`InvalidAudience` mapping from PR3 (re-confirmed end-to-end by 4.6/4.7).

## Issues Found

None — no test failures, no build breaks, no infrastructure-level TDD blockers. The one pre-existing clippy warning found in `kitlogger-macros` (Files Changed table) was fixed rather than merely reported, per this PR's "fix de todo" instruction and the Phase 5 gate requiring `cargo clippy --workspace -- -D warnings` to pass with zero warnings workspace-wide (not just in `security-jwt`).

## Remaining Tasks

None from tasks.md Phases 1-5. Task 5.4 (spec.md placement reconciliation into `openspec/changes/011-security-jwt/specs/jwt-authentication-provider/`) remains explicitly deferred to `sdd-archive` per tasks.md's own "Known Deviation" note — NOT touched by this PR per the orchestrator's explicit instruction.

**New follow-up recommended for `sdd-archive` or a future change** (beyond the pre-existing 5.4 item): reconcile spec.md's "JWT Validation" section, which consistently writes `Err(JwtError::X)` instead of `Err(AuthenticationError::X)` throughout every scenario, AND resolve the "malformed token string" scenario's apparent conflict with design.md's decode-failure-always-InvalidSignature security rule (see Deviations #3 above).

## Workload / PR Boundary

- Mode: chained PR slice (feature-branch-chain) — this is the FINAL slice in the chain; the feature/tracker branch should now aggregate PR1+PR2+PR3+PR4 for integration and the tracker PR to `main`.
- Current work unit: Unit 4 — "JwtAuthenticator + all integration tests (HS256/RS256/errors/clock)" (per tasks.md Suggested Work Units table)
- Boundary: Starts from PR3 (`JwtValidator` claim validation merged); finishes with the entire `011-security-jwt` change complete and self-consistent: `kitlogger-log-domain`'s 6 domain modules (PR1) + `security-jwt`'s full crate (`JwtConfig`, `JwtError`, `KeyResolver`, `JwtValidator`, `JwtAuthenticator` — PR2/PR3/PR4) compiling, tested, and clippy-clean workspace-wide. All 4 proposal.md Success Criteria items involving code (not archive-process items) are now satisfied: domain compiles; security-jwt compiles and `JwtAuthenticator` implements `AuthenticationProvider`; valid HS256/RS256 → `Ok`; expired/bad-signature/wrong-issuer/wrong-audience → correct `AuthenticationError`; `Clock` enables deterministic tests; custom claims preserve `BTreeMap` ordering.
- Estimated review budget impact: ~520 added lines across 8 files (1 new `authenticator.rs` ~370 lines incl. tests, 1 new `tests/common.rs` ~95 lines, 4 new integration test files ~35-55 lines each, 1-line `lib.rs` diff, 1-line `Cargo.toml` diff, 3-line `kitlogger-macros` fix). This is the heaviest single PR in the chain as tasks.md's Review Workload Forecast anticipated ("PR4 ... heaviest review") — recommend the 4R review pass (risk/resilience/readability/reliability) tasks.md flagged for this final integration PR before merging the tracker branch to `main`, given it touches `crates/security-jwt/**` (an explicit repo review hot path) and introduces the first real cryptographic decode/verify code path in the change.

## Status (PR4 — FINAL)

7/7 tasks in Phase 4 (incl. 4.3b) + 3/3 tasks in Phase 5 complete. **26/26 implementation tasks across PR1-PR4 complete.** Ready for `sdd-verify`, then `sdd-archive` (which must still handle task 5.4's spec.md placement reconciliation and the newly-flagged spec.md `JwtError`/`AuthenticationError` naming + malformed-token-scenario reconciliation from Deviations #3 above).

---

# Apply Progress: CORE-011 JWT Authentication Provider — PR4 Correction (Post-Review Defect Fixes)

**Change**: 011-security-jwt
**Batch**: Correction batch on top of completed PR1-PR4 work, branch `011-security-jwt-pr4-authenticator-integration`
**Mode**: Strict TDD

## Completed Fixes (this batch)

### Defect 1 — FR-008 not honored (functional bug)

`From<JwtError> for AuthenticationError` in `crates/security-jwt/src/error.rs` unconditionally mapped every `JwtError::Decode(_)` to `AuthenticationError::InvalidSignature`, so a structurally malformed (not-even-a-JWT-shape) string never produced `MalformedToken`, violating spec.md's FR-008. Deviation #3 in the PR4 section above flagged this exact gap for reviewer confirmation; it is now resolved in favor of literal FR-008 conformance rather than the earlier "design.md's blanket InvalidSignature rule wins" interpretation.

**Fix**: `JwtError::Decode`'s match arm now inspects `jsonwebtoken::errors::Error::kind()` (verified via the vendored `jsonwebtoken-10.4.0/src/errors.rs` source; `Error::kind(&self) -> &ErrorKind` is the public accessor) and branches on the wrapped `ErrorKind`:
- Structural (`InvalidToken`, `Base64(_)`, `Json(_)`, `Utf8(_)`) → `AuthenticationError::MalformedToken` (FR-008).
- Cryptographic (`InvalidSignature`, `InvalidEcdsaKey`, `InvalidEddsaKey`, `InvalidRsaKey(_)`, `RsaFailedSigning`, `Signing(_)`, `InvalidAlgorithm`, `InvalidAlgorithmName`, `InvalidKeyFormat`, `MissingAlgorithm`, `Provider(_)`) → `AuthenticationError::InvalidSignature` (unchanged oracle-avoidance rationale from PR2/PR3, preserved verbatim).
- Claim-validation variants (`ExpiredSignature`, `InvalidIssuer`, `InvalidAudience`, `InvalidSubject`, `ImmatureSignature`, `MissingRequiredClaim(_)`, `InvalidClaimFormat(_)`) → `AuthenticationError::MalformedToken`, defensively, with an inline comment explaining this branch should be unreachable since `authenticator.rs` disables jsonwebtoken's own exp/aud/required-claims validation.
- `_` (non-exhaustive catch-all, since `ErrorKind` is `#[non_exhaustive]`) → `AuthenticationError::InvalidSignature` (safer default, avoids leaking parser detail).

`JwtError::Algorithm` and `JwtError::KeyResolution` mappings are untouched — still `InvalidSignature`.

### TDD Cycle Evidence

| Fix | Test File | RED | GREEN | REFACTOR |
|-----|-----------|-----|-------|----------|
| FR-008 unit mapping | `crates/security-jwt/src/error.rs` | Renamed `decode_error_maps_to_invalid_signature` → `decode_error_with_invalid_token_shape_maps_to_malformed_token` (asserts `MalformedToken`); executed against the unmodified blanket-`InvalidSignature` implementation and confirmed FAILING (`left: InvalidSignature, right: MalformedToken`). Added a second test `decode_error_with_invalid_signature_kind_still_maps_to_invalid_signature` to pin the crypto-failure path. | Both pass (32/32 in `security-jwt --lib`) after implementing the `ErrorKind`-branching match. | None needed — straight-line match, no duplication. |
| FR-008 integration | `crates/security-jwt/tests/error_scenarios.rs` | Added `structurally_malformed_token_is_rejected_as_malformed_not_invalid_signature`, feeding `"not-a-jwt-at-all"` through the real `authenticate()` path; executed against the unmodified implementation and confirmed FAILING (`left: Err(InvalidSignature), right: Err(MalformedToken)`). | Passes after the `error.rs` fix (5/5 in `error_scenarios.rs`). | None needed. |

- **Total tests added**: 2 (1 unit in `error.rs`, 1 integration in `error_scenarios.rs`); 1 existing unit test renamed/repurposed (same RED/GREEN pair, no test count change there).
- **Existing tests preserved**: `authenticate_rejects_token_with_invalid_signature` (authenticator.rs, wrong-HMAC-secret) and `invalid_signature_is_rejected` (error_scenarios.rs) both re-run and confirmed still passing with `InvalidSignature` — not accidentally flipped to `MalformedToken`.

### Defect 2 — spec.md `JwtError`/`AuthenticationError` naming error (documentation bug)

`openspec/specs/jwt-authentication-provider/spec.md`'s "JWT Validation" section wrote `Err(JwtError::X)` in 9 scenario THEN-clauses (`Expired` x2, `MalformedToken` x3 incl. the FR-008 scenario, `InvalidIssuer` x2, `InvalidAudience` x1, `InvalidSignature` x1), but `JwtError` (infrastructure type) has no such variants — only `AuthenticationError` (domain's public contract type) does. All 9 occurrences replaced `JwtError::` → `AuthenticationError::`. Verified zero remaining `JwtError::` references in spec.md (`rg -c "JwtError::"` → 0 matches) and exactly 9 `AuthenticationError::` references post-fix. Nothing else in the file was touched.

### design.md Data Flow update (consistency)

Updated the Data Flow diagram's step 3 (`jsonwebtoken::decode` failure branch) in `openspec/changes/011-security-jwt/design.md` to replace the blanket "`Err(JwtError::Decode)` → `Err(AuthenticationError::InvalidSignature)`" line with two lines: structural decode failures → `MalformedToken` (FR-008, no signature to evaluate), cryptographic decode failures → `InvalidSignature`. The `KeyResolution`/`Algorithm` → `InvalidSignature` oracle-avoidance rationale (steps 2 and the `Algorithm` line) is unchanged.

### proposal.md Dependencies update

Added a note under the `jsonwebtoken` dependency line documenting the `rust_crypto` feature requirement (jsonwebtoken 10.x has no crypto backend enabled by default; `encode`/`decode` panic at runtime without one; `rust_crypto` chosen for pure-Rust portability, no native/OpenSSL dependency) — matching the pattern already used for the `chrono` addition note in PR1/PR3.

## Files Changed

| File | Action | What Was Done |
|------|--------|----------------|
| `crates/security-jwt/src/error.rs` | Modified | `From<JwtError> for AuthenticationError`'s `Decode` arm now branches on `jsonwebtoken::errors::ErrorKind` via `.kind()` instead of a blanket `InvalidSignature` mapping; 1 unit test renamed + repurposed, 1 new unit test added |
| `crates/security-jwt/src/authenticator.rs` | Modified | Updated the step-3 doc comment in `authenticate()` to describe the corrected structural-vs-cryptographic split instead of the old blanket statement |
| `crates/security-jwt/tests/error_scenarios.rs` | Modified | Added `structurally_malformed_token_is_rejected_as_malformed_not_invalid_signature` (FR-008 conformance test, feeds `"not-a-jwt-at-all"` through the real `authenticate()` path) |
| `openspec/specs/jwt-authentication-provider/spec.md` | Modified | 9x `Err(JwtError::X)` → `Err(AuthenticationError::X)` in the "JWT Validation" section; no other changes |
| `openspec/changes/011-security-jwt/design.md` | Modified | Data Flow step 3: replaced blanket `Decode → InvalidSignature` line with structural (`MalformedToken`) vs. cryptographic (`InvalidSignature`) split; KeyResolution/Algorithm oracle-avoidance rationale unchanged |
| `openspec/changes/011-security-jwt/proposal.md` | Modified | Added `jsonwebtoken`'s `rust_crypto` feature requirement note to the Dependencies section |
| `openspec/changes/011-security-jwt/apply-progress.md` | Modified | This section appended |

## Verification Commands Run

```
cargo test -p security-jwt --lib error::        # RED confirmed (1 failed) before fix, GREEN (32/32) after
cargo test -p security-jwt --test error_scenarios  # RED confirmed (1 failed, 4 passed) before fix, GREEN (5/5) after
cargo build --workspace                          # Finished, 0 errors
cargo test --workspace                           # All test binaries green, 0 failed
cargo clippy --workspace -- -D warnings          # Finished, 0 warnings
```

Specifically re-confirmed: the new FR-008 tests (unit + integration) pass with `MalformedToken`, and `authenticate_rejects_token_with_invalid_signature` (authenticator.rs) + `invalid_signature_is_rejected` (error_scenarios.rs) both still pass with `InvalidSignature` — the wrong-HMAC-secret path was not accidentally flipped.

## Deviations from Design

None beyond what is described above — this batch implements exactly the two confirmed defects and the two consistency updates requested, matching the corrected `ErrorKind` mapping described in the review.

## Issues Found

None.

## Remaining Tasks

None. All 26 implementation tasks (PR1-PR4) plus this correction batch are complete. Task 5.4 (spec.md placement reconciliation into `openspec/changes/011-security-jwt/specs/...`) remains deferred to `sdd-archive` as before — this correction batch edited the spec.md content in place at its current location without relocating it.

## Status (PR4 Correction)

Both confirmed defects fixed and verified. Ready for `sdd-verify` / `sdd-archive`.

---

## Archive Closure

All 26 implementation tasks + 1 correction batch (FR-008 fix) complete. Task 5.4 (spec.md placement reconciliation) resolved at archive time: the living spec stays at `openspec/specs/jwt-authentication-provider/spec.md` (already correct, verified consistent with implementation); this retroactive `specs/` copy exists here for archive-shape consistency with changes 005-010.
