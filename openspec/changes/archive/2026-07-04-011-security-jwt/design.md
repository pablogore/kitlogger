# Design: CORE-011 JWT Authentication Provider

## Technical Approach

Additive change: domain types in `kitlogger-log-domain` (zero heavy deps) + new `security-jwt` crate that implements `AuthenticationProvider` via `jsonwebtoken`. Runtime stays JWT-agnostic — only sees `Option<SecurityContext>`.

## Architecture Decisions

| Option | Tradeoffs | Decision |
|--------|-----------|----------|
| `AuthenticationError` in domain vs. only in security-jwt | Domain enum lets ANY auth provider use the trait without depending on JWT crate | **In domain** — keeps trait fully agnostic |
| `SecurityContext` as trait vs. concrete struct | Trait adds indirection for no benefit at MVP | **Concrete struct** — extensibility lives in `AuthenticationProvider` |
| Domain deps: serde + serde_json only | Enables `Value` in custom claims without crypto/network | **Accepted** — validated as zero-runtime-cost derive |
| Domain deps: + chrono (`clock`,`std` features, no defaults) | `Clock::now()` needs a concrete `DateTime<Utc>` return type; no way around it without a hand-rolled timestamp type | **Accepted** — minimal-feature `chrono`, no additional runtime cost |
| Two error types: `AuthenticationError` (domain) + `JwtError` (crate) | Domain variants: Expired, InvalidSignature, etc. JwtError wraps `jsonwebtoken` internal failures | **Two layers** — trait consumers get a clean domain enum; JwtError provides debug detail |
| BTreeMap vs. HashMap for claims | Deterministic iteration (lexicographic key order) vs. O(1) lookup | **BTreeMap** — iteration order matters for serialization/comparison |
| Clock trait for time vs. `Utc::now()` inline | Testability vs. simplicity | **Clock trait** — FakeClock in tests is mandatory for deterministic exp/nbf/iat checks |
| `FakeClock` visibility: `pub` vs. `#[cfg(test)]`-gated | `#[cfg(test)]` keeps it out of the public API but forces `security-jwt`'s tests to duplicate a fake clock impl | **`pub`** — single shared test double across domain and security-jwt test suites; pure wrapper, no production behavior exposed |

## Data Flow

```
Request Headers
    │
    ▼
Credential::BearerToken(raw_token: String)
    │
    ▼
JwtAuthenticator::authenticate(&self, credential: &Credential)
    │
    ├── 1. Match Credential::BearerToken(token)
    ├── 2. KeyResolver::resolve(&self, kid: Option<&str>) → DecodingKey
    │      │
    │      ▼ (on missing key)
    │      Err(JwtError::KeyResolution) ──→ Err(AuthenticationError::InvalidSignature)
    │      (deliberate: distinguishing "unknown kid" from "bad signature" would leak
    │       a key-enumeration oracle to callers)
    │
    ├── 3. jsonwebtoken::decode(token, key, validation) → TokenData
    │      │
    │      ▼ (on decode/validation/algorithm error)
    │      Err(JwtError::Decode), structural  ──→ Err(AuthenticationError::MalformedToken)
    │      (not a valid JWT shape, bad base64/JSON/UTF-8 — FR-008; the token
    │       never had a signature to evaluate in the first place)
    │      Err(JwtError::Decode), cryptographic → Err(AuthenticationError::InvalidSignature)
    │      (bad signature, bad key, disallowed algorithm)
    │      Err(JwtError::Algorithm) ───────────→ Err(AuthenticationError::InvalidSignature)
    │      (Algorithm rejection folds into InvalidSignature too — same "alg: none"
    │       confusion-attack defense as above)
    │
    ├── 4. JwtValidator::validate_claims(token_data.claims, clock, config)
    │      │
    │      ├── exp < clock.now()      → Err(AuthenticationError::Expired)
    │      ├── nbf > clock.now()      → Err(AuthenticationError::MalformedToken)
    │      ├── iat > now + leeway     → Err(AuthenticationError::MalformedToken)
    │      ├── iss != config.issuer   → Err(AuthenticationError::InvalidIssuer)
    │      ├── aud != config.audience → Err(AuthenticationError::InvalidAudience)
    │      └── all pass               → continue
    │
    ├── 5. Build Identity { subject, roles, tenant_id, attributes }
    ├── 6. Build Claims { standard, custom: BTreeMap }
    └── 7. Return Ok(SecurityContext { identity, claims })
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `crates/kitlogger-log-domain/src/identity.rs` | Create | `Identity` struct with subject, roles (BTreeSet), tenant_id (Option), attributes (BTreeMap) |
| `crates/kitlogger-log-domain/src/claims.rs` | Create | `StandardClaims` + `Claims` structs, custom claims as `BTreeMap<String, Value>` |
| `crates/kitlogger-log-domain/src/security.rs` | Create | `SecurityContext` struct wrapping Identity + Claims |
| `crates/kitlogger-log-domain/src/credential.rs` | Create | `Credential` enum with `BearerToken(String)` variant. Deliberately an enum (not a `BearerToken` struct/wrapper) to leave room for future variants (`Basic`, `ApiKey`, `Mtls`) without breaking `AuthenticationProvider::authenticate` callers |
| `crates/kitlogger-log-domain/src/authentication.rs` | Create | `AuthenticationProvider` trait (object-safe, Send+Sync), `AuthenticationError` enum (Expired, InvalidSignature, InvalidIssuer, InvalidAudience, MalformedToken) |
| `crates/kitlogger-log-domain/src/clock.rs` | Create | `Clock` trait (fn now() → DateTime<Utc>), `UtcClock` impl, `pub struct FakeClock` test double (not `#[cfg(test)]`-gated — shared with `security-jwt`'s tests) |
| `crates/kitlogger-log-domain/src/lib.rs` | Modify | Add `pub mod` + `pub use` for each new module |
| `crates/kitlogger-log-domain/Cargo.toml` | Modify | Add `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`, `chrono = { version = "0.4", default-features = false, features = ["clock", "std"] }` |
| `crates/security-jwt/Cargo.toml` | Create | Crate manifest depending on `kitlogger-log-domain`, `jsonwebtoken`, `serde`, `serde_json`, `thiserror` |
| `crates/security-jwt/src/lib.rs` | Create | Crate root, module declarations, re-exports |
| `crates/security-jwt/src/config.rs` | Create | `JwtConfig` struct (algorithms, issuer, audience, leeway) |
| `crates/security-jwt/src/error.rs` | Create | `JwtError` enum (Decode, Algorithm, KeyResolution) + `From<JwtError> for AuthenticationError` |
| `crates/security-jwt/src/key.rs` | Create | `KeyResolver` trait (fn resolve → DecodingKey) |
| `crates/security-jwt/src/validator.rs` | Create | `JwtValidator` — claim-by-claim validation against Clock + Config |
| `crates/security-jwt/src/authenticator.rs` | Create | `JwtAuthenticator` implementing `AuthenticationProvider` |
| `Cargo.toml` (root) | Modify | Add `crates/security-jwt` to workspace members |

## Interfaces / Contracts

**Domain (kitlogger-log-domain):**

```rust
pub trait AuthenticationProvider: Send + Sync {
    fn authenticate(&self, credential: &Credential)
        -> Result<SecurityContext, AuthenticationError>;
}

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}
```

**Security-JWT crate:**

```rust
pub trait KeyResolver: Send + Sync {
    fn resolve(&self, kid: Option<&str>) -> Result<DecodingKey, JwtError>;
}

pub struct JwtAuthenticator {
    config: JwtConfig,
    validator: JwtValidator,
    key_resolver: Box<dyn KeyResolver>,
    clock: Box<dyn Clock>,
}

impl AuthenticationProvider for JwtAuthenticator { ... }
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit (domain) | Identity/Claims/Credential construction, Debug/Clone/PartialEq derives | Pure data tests, no deps |
| Unit (domain) | AuthenticationError Display messages | Match each variant's output string |
| Unit (validator) | exp/nbf/iat boundary checks | `FakeClock` + known timestamps — one variant per test |
| Unit (validator) | Issuer/audience match/mismatch | Config with Some/None variants |
| Unit (validator) | Custom claims ordering | Assert `BTreeMap` key iteration order |
| Unit (key) | KeyResolver resolves by kid | Mock trait impl with HashMap<Option<&str>, DecodingKey> |
| Integration | HS256 valid token round-trip | Generate token with `jsonwebtoken`, authenticate, assert Ok + fields |
| Integration | RS256 valid token round-trip | Generate with RSA key pair, resolve via KeyResolver |
| Integration | Invalid signature, expired, wrong issuer | Create broken tokens, assert correct error variant |
| Integration | Clock-driven boundary (exp = now-1s) | FakeClock + token at boundary → Err(Expired) |

## Migration / Rollout

No migration required. Additive only — new modules in existing crate + new workspace member. `kitlogger` crate gains no new deps (stays agnostic). Consumers of `AuthenticationProvider` depend on `security-jwt` themselves.

## Open Questions

None. All decisions documented and consistent with proposal + spec.
