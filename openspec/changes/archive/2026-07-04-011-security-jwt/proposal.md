# Proposal: CORE-011 JWT Authentication Provider

## Intent

KitLogger lacks security primitives — no identity, no tenant, no roles. This change introduces JWT auth via `AuthenticationProvider` → `SecurityContext` as the foundation for RBAC/ABAC/OIDC.

## Scope

### In Scope
- Domain types in `kitlogger-log-domain` (minimal deps: `serde`, `serde_json`, `chrono`): `Identity`, `StandardClaims`, `Claims`, `SecurityContext`, `Credential(BearerToken)`, `AuthenticationProvider` trait, `Clock` trait.
- New `security-jwt` crate: `JwtAuthenticator`, `JwtValidator`, `JwtConfig`.
- HS256 + RS256 validation (MVP). exp/nbf/iat checks. Configurable issuer, audience.
- `Clock` trait for deterministic testing.
- `BTreeMap<String, Value>` for deterministic claims ordering.
- `AuthenticationError` enum (domain, public contract): Expired, InvalidSignature, InvalidIssuer, InvalidAudience, MalformedToken.
- `JwtError` enum (security-jwt, infrastructure-level): Decode, Algorithm, KeyResolution — converted into `AuthenticationError` via `From<JwtError>`.

### Out of Scope
- Authorization (CORE-012/013/014), OIDC, ES256/EdDSA (CORE-011A).
- ServiceContext, runtime changes — `kitlogger` stays JWT-agnostic.

## Capabilities

### New Capabilities
- `jwt-authentication-provider`: JWT validation via `AuthenticationProvider`, configurable issuer/audience/algorithms, clock-aware exp/nbf/iat checks, `SecurityContext` production.

### Modified Capabilities
- None.

## Approach

Domain types are concrete structs in `kitlogger-log-domain` — minimal external deps (`serde`, `serde_json`, and `chrono` with `default-features = false, features = ["clock", "std"]` for the `Clock` trait's `DateTime<Utc>`). `AuthenticationProvider` is object-safe; `Credential` is an enum. `security-jwt` depends on domain + `jsonwebtoken`. `JwtAuthenticator` implements `AuthenticationProvider` — decodes JWT, validates claims, maps to `Identity` + `SecurityContext`, preserves custom claims in `BTreeMap`. Runtime never sees JWT — only `Option<SecurityContext>`.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/kitlogger-log-domain/src/{identity,claims,security,credential,authentication_provider,clock}.rs` | New | Domain types + traits |
| `crates/kitlogger-log-domain/src/lib.rs` | Modified | Re-export new modules |
| `crates/kitlogger-log-domain/Cargo.toml` | Modified | Add `serde`, `serde_json`, `chrono` (minimal features, for `Clock::now() -> DateTime<Utc>`) |
| `crates/security-jwt/` | New | Full crate |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `jsonwebtoken` crate dependency | Low | Well-maintained, no runtime impact if unused |
| Domain crate gains deps (serde, chrono) | Med | Minimal — `serde` + `serde_json` + `chrono` (no default features, `clock`+`std` only), no crypto/network |
| `FakeClock` is `pub` (not `#[cfg(test)]`-gated) | Low | Deliberate — lets `security-jwt`'s own tests (PR2/PR3) reuse it instead of duplicating a fake clock; pure wrapper, no behavior risk |

## Rollback Plan

Revert the commit. Additive only — no migrations, no state. Removing workspace member + domain modules restores prior state.

## Dependencies

- `jsonwebtoken` for JWT decode/verify (HS256, RS256). Requires the `rust_crypto` feature: `jsonwebtoken` 10.x ships with no crypto backend enabled by default, and `encode`/`decode` panic at runtime without one; `rust_crypto` is pure-Rust (no native/OpenSSL toolchain dependency), keeping the build portable.
- `serde` + `serde_json` on domain for `Value` in custom claims.
- `chrono` on domain (`default-features = false`, `features = ["clock", "std"]`) for `Clock::now() -> DateTime<Utc>`.

## Success Criteria

- [ ] `kitlogger-log-domain` compiles with all new types — serde/serde_json/chrono only.
- [ ] `security-jwt` compiles; `JwtAuthenticator` implements `AuthenticationProvider`.
- [ ] Valid HS256/RS256 tokens → `Ok(SecurityContext)`.
- [ ] Expired, bad-signature, wrong-issuer, wrong-audience → correct `AuthenticationError` (via `From<JwtError> for AuthenticationError`).
- [ ] Missing credential → `Option<SecurityContext>`.
- [ ] `Clock` trait enables deterministic time-based tests.
- [ ] Custom claims preserve `BTreeMap` ordering.
