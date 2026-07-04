# JWT Authentication Provider Specification

## Purpose

Validate JWT Bearer tokens through `AuthenticationProvider`, producing `SecurityContext` for downstream authorization. Supports deterministic testing via `Clock` trait.

## Domain Types

### Requirement: Identity

`Identity` MUST expose `subject`, `roles` (`BTreeSet<String>`), `tenant_id` (`Option<String>`), `attributes` (`BTreeMap<String, String>`).

### Requirement: Claims

`Claims` MUST contain `StandardClaims` (exp, nbf, iat, iss, aud, sub) and `custom: BTreeMap<String, Value>`. Custom keys SHALL iterate in lexicographic order.

### Requirement: AuthenticationProvider Trait

`AuthenticationProvider` MUST be object-safe, `Send + Sync`, with `fn authenticate(&self, credential: &Credential) -> Result<SecurityContext, AuthenticationError>`.

### Requirement: Clock Trait

`Clock` MUST expose `fn now(&self) -> DateTime<Utc>` and be `Send + Sync`.

#### Scenario: Deterministic time

- GIVEN a `FakeClock` fixed at `2024-01-15T12:00:00Z`
- WHEN `clock.now()` is called
- THEN it SHALL return `2024-01-15T12:00:00Z`

## JWT Validation

### Requirement: exp, nbf, iat Checks (FR-001, FR-004)

`JwtAuthenticator` MUST reject expired tokens, tokens before `nbf`, tokens from the future beyond `leeway_seconds` (iat), and support HS256 + RS256.

#### Scenario: Valid HS256 token returns SecurityContext

- GIVEN a valid HS256 JWT signed with the configured key, exp > now, nbf <= now, matching issuer/audience
- WHEN `authenticate(Credential::BearerToken(token))` succeeds
- THEN it SHALL return `Ok(SecurityContext)` with identity from sub/roles/tenant/attributes

#### Scenario: Valid RS256 token succeeds

- GIVEN a valid RS256 JWT with a key resolved by `KeyResolver`
- WHEN `authenticate` is called
- THEN it SHALL return `Ok(SecurityContext)`

#### Scenario: Expired token rejected

- GIVEN a JWT where `exp` precedes the clock's `now()`
- WHEN `authenticate` is called
- THEN it SHALL return `Err(AuthenticationError::Expired)`

#### Scenario: Token before nbf rejected

- GIVEN a JWT where `nbf` follows the clock's `now()`
- WHEN `authenticate` is called
- THEN it SHALL return `Err(AuthenticationError::MalformedToken)`

#### Scenario: Future iat beyond leeway rejected

- GIVEN a JWT with `iat` exceeding `now() + leeway_seconds`
- WHEN `authenticate` is called
- THEN it SHALL return `Err(AuthenticationError::MalformedToken)`

#### Scenario: Clock-driven exp boundary

- GIVEN a JWT with `exp = 2024-01-15T12:00:00Z` and `FakeClock` at `2024-01-15T12:00:01Z`
- WHEN `authenticate` is called
- THEN it SHALL return `Err(AuthenticationError::Expired)`

### Requirement: Issuer / Audience Validation (FR-002, FR-003)

Config validation MUST reject non-matching `iss` and `aud`. Validation is skipped when the config field is `None`.

#### Scenario: Wrong issuer rejected

- GIVEN a JWT with `iss = "wrong"` and config `issuer = Some("expected")`
- WHEN `authenticate` is called
- THEN it SHALL return `Err(AuthenticationError::InvalidIssuer)`

#### Scenario: Wrong audience rejected

- GIVEN a JWT with `aud = "wrong"` and config `audience = Some("expected")`
- WHEN `authenticate` is called
- THEN it SHALL return `Err(AuthenticationError::InvalidAudience)`

#### Scenario: Missing issuer when configured

- GIVEN a JWT without `iss` and config `issuer = Some("expected")`
- WHEN `authenticate` is called
- THEN it SHALL return `Err(AuthenticationError::InvalidIssuer)`

### Requirement: Signature Validation (FR-004)

#### Scenario: Invalid signature rejected

- GIVEN a JWT with an invalid signature for the configured key
- WHEN `authenticate` is called
- THEN it SHALL return `Err(AuthenticationError::InvalidSignature)`

### Requirement: SecurityContext Production (FR-005, FR-006)

Authentication MUST produce `SecurityContext` with `Identity` + `Claims`. Raw JWT details MUST NOT be exposed. Custom claims SHALL preserve `BTreeMap` ordering.

#### Scenario: Custom claims preserved with ordering

- GIVEN a JWT with custom claims `{"zone": "a", "region": "us-east-1"}`
- WHEN authentication succeeds
- THEN `claims.custom` SHALL contain both entries AND keys SHALL iterate as "region", "zone"

### Requirement: Anonymous Endpoints (FR-007)

Missing credentials MUST NOT produce errors — the caller decides authorization.

#### Scenario: No credential means no SecurityContext

- GIVEN no `Credential` is available
- WHEN the caller maps absence to `Option::None`
- THEN no error is raised AND the endpoint proceeds without a `SecurityContext`

### Requirement: Malformed Token (FR-008)

#### Scenario: Malformed token string

- GIVEN a token that is not valid JWT
- WHEN `authenticate` is called
- THEN it SHALL return `Err(AuthenticationError::MalformedToken)`
