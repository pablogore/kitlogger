# Tasks: CORE-011 JWT Authentication Provider

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~1000-1200 (6 domain modules + 7 security-jwt modules + 4 integration test files, all with colocated unit tests) |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR1 (domain) -> PR2 (crate scaffolding) -> PR3 (validator) -> PR4 (authenticator + integration) |
| Delivery strategy | ask-on-risk |
| Chain strategy | feature-branch-chain |

Decision needed before apply: Resolved — feature-branch-chain
Chained PRs recommended: Yes
Chain strategy: feature-branch-chain
400-line budget risk: High

This change touches `crates/security-jwt/**` (auth/security hot path per repo review triggers) — recommend `feature-branch-chain` so the full authenticator integrates before merge, with 4R review (risk/resilience/readability/reliability) on the final integration PR. Orchestrator must confirm chain strategy with user before `sdd-apply`.

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Domain types: Identity, Claims, SecurityContext, Credential, AuthenticationProvider, Clock | PR 1 | Base: tracker/main. Zero deps beyond serde. Independently compilable. |
| 2 | security-jwt scaffolding: Cargo.toml, JwtConfig, JwtError, KeyResolver, workspace wiring | PR 2 | Base: PR 1. No decode logic yet. |
| 3 | JwtValidator: exp/nbf/iat, issuer/audience, claims ordering | PR 3 | Base: PR 2. Pure logic, no I/O. |
| 4 | JwtAuthenticator + all integration tests (HS256/RS256/errors/clock) | PR 4 | Base: PR 3. Wires everything; heaviest review. |

## Phase 1: Domain Foundation (`kitlogger-log-domain`)

- [x] 1.1 Add `serde`+`serde_json` to `crates/kitlogger-log-domain/Cargo.toml`
- [x] 1.2 TDD `identity.rs`: `Identity{subject, roles:BTreeSet, tenant_id, attributes:BTreeMap}` + tests
- [x] 1.3 TDD `claims.rs`: `StandardClaims`+`Claims{custom:BTreeMap<String,Value>}` + ordering test
- [x] 1.4 TDD `security.rs`: `SecurityContext{identity,claims}` + tests
- [x] 1.5 TDD `credential.rs`: `Credential::BearerToken(String)` + tests
- [x] 1.6 TDD `authentication.rs`: `AuthenticationProvider` trait (object-safe, Send+Sync) + `AuthenticationError` enum + Display tests
- [x] 1.7 TDD `clock.rs`: `Clock` trait + `UtcClock` + `FakeClock` test double + deterministic-time test (spec: Deterministic time)
- [x] 1.8 Wire `lib.rs`: `pub mod`+`pub use` for all 6 new modules

## Phase 2: security-jwt Scaffolding

- [x] 2.1 Create `crates/security-jwt/Cargo.toml` (deps: `kitlogger-log-domain`, `jsonwebtoken`, `serde`, `serde_json`, `thiserror`)
- [x] 2.2 Add `crates/security-jwt` to root `Cargo.toml` workspace members
- [x] 2.3 Create `security-jwt/src/lib.rs` with module declarations + re-exports
- [x] 2.4 TDD `config.rs`: `JwtConfig{algorithms,issuer,audience,leeway}` + tests
- [x] 2.5 TDD `error.rs`: `JwtError` enum + `From<JwtError> for AuthenticationError` + tests
- [x] 2.6 TDD `key.rs`: `KeyResolver` trait + mock-impl test (resolve by `kid`)
- [x] 2.7 TDD `key.rs`: `KeyResolver::resolve` returns `Err(JwtError::KeyResolution)` when `kid` is present but no matching key exists

## Phase 3: Claim Validation Logic

- [x] 3.1 TDD `validator.rs`: exp/nbf/iat checks via `FakeClock`, one variant per test (spec: exp/nbf/iat, FR-001/FR-004)
- [x] 3.2 TDD `validator.rs`: issuer match/mismatch/missing (spec: FR-002)
- [x] 3.3 TDD `validator.rs`: audience match/mismatch (spec: FR-003)
- [x] 3.4 TDD `validator.rs`: custom claims `BTreeMap` lexicographic ordering assertion (spec: Custom claims preserved with ordering)

## Phase 4: Authenticator + Integration

- [x] 4.1 TDD `authenticator.rs`: `JwtAuthenticator` struct + `AuthenticationProvider` impl skeleton wiring config/validator/key_resolver/clock
- [x] 4.2 TDD `authenticate()` happy path — decode + validate + map to `SecurityContext` (unit, mocked `KeyResolver`)
- [x] 4.3 TDD `authenticate()` error mapping — `InvalidSignature`, `MalformedToken` (unit)
- [x] 4.3b TDD `authenticate()` error mapping — `KeyResolver` returning `Err(JwtError::KeyResolution)` maps to the correct `AuthenticationError` variant (variant choice is a design/implementation decision made during this task, not fixed in advance)
- [x] 4.4 Integration `tests/hs256_roundtrip.rs` — valid HS256 token via `jsonwebtoken::encode` -> `Ok(SecurityContext)`
- [x] 4.5 Integration `tests/rs256_roundtrip.rs` — valid RS256 token via RSA key pair + `KeyResolver` -> `Ok`
- [x] 4.6 Integration `tests/error_scenarios.rs` — invalid signature, expired, wrong issuer, wrong audience broken tokens
- [x] 4.7 Integration `tests/clock_boundary.rs` — `FakeClock` at `exp - 1s` boundary -> `Err(Expired)` (spec: Clock-driven exp boundary)

## Phase 5: Verification

- [x] 5.1 `cargo build --workspace` — domain + security-jwt compile
- [x] 5.2 `cargo test --workspace` — all unit + integration tests green
- [x] 5.3 `cargo clippy --workspace -- -D warnings`

## Known Deviation (flag for sdd-archive)

`spec.md` for this change lives at `openspec/specs/jwt-authentication-provider/spec.md` instead of the change-scoped `openspec/changes/011-security-jwt/specs/` delta folder used by prior changes (005-010). Not corrected here per instruction — archive phase should reconcile placement/convention.

- [x] 5.4 `sdd-archive`: reconcile `spec.md` placement — move/link into `openspec/changes/011-security-jwt/specs/jwt-authentication-provider/` convention or confirm top-level placement is intentional and update the archive process notes accordingly

---

## Summary

**Total Implementation Tasks: 26 (all complete)**
- Phase 1: 8/8 ✓
- Phase 2: 7/7 ✓
- Phase 3: 4/4 ✓
- Phase 4: 7/7 ✓
- Phase 5: 3/3 ✓ (task 5.4 deferred to archive phase, see deviation note above)

Delivery strategy: feature-branch-chain (4 chained PRs per work units table, stacked on tracker `011-security-jwt`, not yet opened to GitHub)
