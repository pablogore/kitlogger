//! `JwtAuthenticator`: the `AuthenticationProvider` implementation that
//! decodes and validates JWT bearer tokens into a `SecurityContext`.
//!
//! Wires together the pieces built in earlier PRs:
//! - `KeyResolver` (PR2) resolves the `DecodingKey` for the token's `kid`.
//! - `JwtValidator` (PR3) validates already-decoded claims against a `Clock`
//!   and `JwtConfig`.
//! - `jsonwebtoken::decode` (this PR) performs the actual signature
//!   verification and claim deserialization.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use jsonwebtoken::{decode, decode_header, Algorithm, Validation};
use kitlogger_log_domain::{
    AuthenticationError, AuthenticationProvider, Claims, Clock, Credential, Identity,
    SecurityContext, StandardClaims,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::JwtConfig;
use crate::error::JwtError;
use crate::key::KeyResolver;
use crate::validator::JwtValidator;

/// Wire-format JWT payload, deserialized directly from the token body.
///
/// `kitlogger_log_domain::Claims`/`StandardClaims` deliberately do not derive
/// `Serialize`/`Deserialize` — the domain crate's public API stays stable
/// regardless of wire format (see design.md's rationale for the two-layer
/// error type split, which follows the same "domain stays wire-format
/// agnostic" principle). This module-private struct mirrors RFC 7519's
/// registered claims plus arbitrary custom claims and is mapped into the
/// domain `Claims` type after a successful decode.
///
/// `#[serde(flatten)]` on `custom` captures every JSON key not already
/// claimed by a named field above it, so custom claims never collide with
/// registered ones.
#[derive(Debug, Serialize, Deserialize)]
struct RawClaims {
    exp: i64,
    #[serde(default)]
    nbf: Option<i64>,
    iat: i64,
    #[serde(default)]
    iss: Option<String>,
    #[serde(default)]
    aud: Option<String>,
    sub: String,
    #[serde(flatten)]
    custom: BTreeMap<String, Value>,
}

impl RawClaims {
    fn into_domain(self) -> Claims {
        let standard = StandardClaims::new(self.exp, self.nbf, self.iat, self.iss, self.aud, self.sub);
        Claims::new(standard, self.custom)
    }
}

/// JWT-based `AuthenticationProvider`: decodes a `Credential::BearerToken`,
/// verifies its signature via `KeyResolver`, validates its claims via
/// `JwtValidator`, and produces a `SecurityContext`.
pub struct JwtAuthenticator {
    config: JwtConfig,
    validator: JwtValidator,
    key_resolver: Box<dyn KeyResolver>,
    clock: Box<dyn Clock>,
}

impl JwtAuthenticator {
    /// Creates a new `JwtAuthenticator`.
    pub fn new(
        config: JwtConfig,
        validator: JwtValidator,
        key_resolver: Box<dyn KeyResolver>,
        clock: Box<dyn Clock>,
    ) -> Self {
        JwtAuthenticator {
            config,
            validator,
            key_resolver,
            clock,
        }
    }
}

impl AuthenticationProvider for JwtAuthenticator {
    fn authenticate(&self, credential: &Credential) -> Result<SecurityContext, AuthenticationError> {
        // 1. Match Credential::BearerToken(token) (design.md Data Flow step 1).
        let Credential::BearerToken(token) = credential;

        // 2. Resolve the decoding key by `kid` (step 2). A missing/unknown
        //    `kid` surfaces as `JwtError::KeyResolution`, which `?` converts
        //    via `From<JwtError> for AuthenticationError` into
        //    `AuthenticationError::InvalidSignature` (task 4.3b: deliberate,
        //    documented in error.rs, to avoid a key-enumeration oracle).
        let header = decode_header(token).map_err(JwtError::Decode)?;
        let key = self.key_resolver.resolve(header.kid.as_deref())?;

        // 3. Decode + verify the signature (step 3). Structural decode
        //    failures (not a valid JWT shape, bad base64/JSON/UTF-8) map to
        //    `AuthenticationError::MalformedToken` (FR-008); signature/key/
        //    algorithm failures map to `AuthenticationError::InvalidSignature`
        //    — both via the same `From<JwtError>` mapping (see error.rs).
        //
        //    `validate_exp`/`validate_aud` are disabled and
        //    `required_spec_claims` is cleared: jsonwebtoken's own claim
        //    validation runs against the *real* system clock
        //    (`get_current_timestamp()`), which would bypass this
        //    authenticator's injected `Clock` and break deterministic
        //    testing (see task 4.7). All claim semantics (exp/nbf/iat/
        //    iss/aud) are instead handled exclusively by
        //    `JwtValidator::validate_claims` in step 4, against `self.clock`.
        // `Validation::new` (not `Validation::default()`) is used as the base
        // on purpose: `validate_signature` is a private field of `Validation`
        // that struct-update syntax (`..Validation::default()`) cannot copy
        // across the crate boundary, so fields are set individually instead.
        let mut validation = Validation::new(Algorithm::HS256);
        validation.algorithms = self.config.algorithms.clone();
        validation.validate_exp = false;
        validation.validate_aud = false;
        validation.required_spec_claims = HashSet::new();
        let token_data = decode::<RawClaims>(token, &key, &validation).map_err(JwtError::Decode)?;
        let claims = token_data.claims.into_domain();

        // 4. Validate claims against `self.clock` and `self.config`.
        self.validator.validate_claims(&claims, self.clock.as_ref(), &self.config)?;

        // 5. Build Identity. The JWT wire format carries no roles, tenant, or
        //    attributes claims (out of scope for this change per proposal.md
        //    — RBAC/ABAC land in CORE-012/013/014), so these default to
        //    empty/`None`. `subject` comes from the validated `sub` claim.
        let identity = Identity::new(
            claims.standard().sub().to_string(),
            BTreeSet::new(),
            None,
            BTreeMap::new(),
        );

        // 6-7. Build Claims (already produced in step 3) and return the
        //      SecurityContext.
        Ok(SecurityContext::new(identity, claims))
    }
}

#[cfg(test)]
mod tests {
    use super::JwtAuthenticator;
    use crate::config::JwtConfig;
    use crate::error::JwtError;
    use crate::key::KeyResolver;
    use crate::validator::JwtValidator;
    use jsonwebtoken::DecodingKey;
    use kitlogger_log_domain::{AuthenticationProvider, UtcClock};

    struct AlwaysResolves;

    impl KeyResolver for AlwaysResolves {
        fn resolve(&self, _kid: Option<&str>) -> Result<DecodingKey, JwtError> {
            Ok(DecodingKey::from_secret(b"secret"))
        }
    }

    #[test]
    fn jwt_authenticator_is_object_safe_and_constructible() {
        // Compile-time + wiring check only: if JwtAuthenticator did not wire
        // config/validator/key_resolver/clock per design.md's Interfaces
        // section, or did not implement AuthenticationProvider, this would
        // fail to compile. Behavior is covered starting task 4.2.
        let _authenticator: Box<dyn AuthenticationProvider> = Box::new(JwtAuthenticator::new(
            JwtConfig::new(vec![], None, None, 0),
            JwtValidator::new(),
            Box::new(AlwaysResolves),
            Box::new(UtcClock),
        ));
    }

    // ── 4.2: happy path ─────────────────────────────────────────────────

    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use kitlogger_log_domain::{Credential, FakeClock};
    use serde::Serialize;
    use serde_json::json;

    const TEST_SECRET: &[u8] = b"unit-test-hmac-secret";

    /// Wire-format claims used only to encode test tokens. Mirrors the shape
    /// `authenticator.rs`'s internal `RawClaims` expects to decode.
    #[derive(Serialize)]
    struct WireClaims {
        exp: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        nbf: Option<i64>,
        iat: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        iss: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        aud: Option<String>,
        sub: String,
        #[serde(flatten)]
        custom: std::collections::BTreeMap<String, serde_json::Value>,
    }

    fn encode_hs256(claims: &WireClaims) -> String {
        encode(&Header::new(Algorithm::HS256), claims, &EncodingKey::from_secret(TEST_SECRET)).unwrap()
    }

    struct FixedSecretResolver;

    impl KeyResolver for FixedSecretResolver {
        fn resolve(&self, _kid: Option<&str>) -> Result<DecodingKey, JwtError> {
            Ok(DecodingKey::from_secret(TEST_SECRET))
        }
    }

    fn authenticator_at(
        now: chrono::DateTime<chrono::Utc>,
        config: JwtConfig,
    ) -> JwtAuthenticator {
        JwtAuthenticator::new(
            config,
            JwtValidator::new(),
            Box::new(FixedSecretResolver),
            Box::new(FakeClock::new(now)),
        )
    }

    #[test]
    fn authenticate_valid_hs256_token_returns_security_context() {
        let now = chrono::Utc::now();
        let claims = WireClaims {
            exp: now.timestamp() + 100,
            nbf: None,
            iat: now.timestamp(),
            iss: None,
            aud: None,
            sub: "user-42".to_string(),
            custom: std::collections::BTreeMap::new(),
        };
        let token = encode_hs256(&claims);
        let config = JwtConfig::new(vec![Algorithm::HS256], None, None, 0);
        let authenticator = authenticator_at(now, config);

        let result = authenticator.authenticate(&Credential::BearerToken(token));

        let context = result.expect("valid HS256 token must authenticate successfully");
        assert_eq!(context.identity().subject(), "user-42");
        assert_eq!(context.claims().standard().sub(), "user-42");
    }

    #[test]
    fn authenticate_preserves_custom_claims_and_defaults_identity_extras() {
        let now = chrono::Utc::now();
        let mut custom = std::collections::BTreeMap::new();
        custom.insert("zone".to_string(), json!("a"));
        custom.insert("region".to_string(), json!("us-east-1"));
        let claims = WireClaims {
            exp: now.timestamp() + 100,
            nbf: None,
            iat: now.timestamp(),
            iss: Some("kitlogger".to_string()),
            aud: Some("kitlogger-api".to_string()),
            sub: "user-99".to_string(),
            custom,
        };
        let token = encode_hs256(&claims);
        let config = JwtConfig::new(
            vec![Algorithm::HS256],
            Some("kitlogger".to_string()),
            Some("kitlogger-api".to_string()),
            0,
        );
        let authenticator = authenticator_at(now, config);

        let context = authenticator
            .authenticate(&Credential::BearerToken(token))
            .expect("valid HS256 token with issuer/audience/custom claims must authenticate");

        // Custom claims round-trip with BTreeMap lexicographic ordering
        // (spec: "Custom claims preserved with ordering").
        let keys: Vec<&String> = context.claims().custom().keys().collect();
        assert_eq!(keys, vec!["region", "zone"]);
        assert_eq!(context.claims().custom().get("region"), Some(&json!("us-east-1")));

        // Identity extras have no source in the JWT wire format (see
        // apply-progress.md for this PR): roles/attributes default empty,
        // tenant_id defaults to None.
        assert!(context.identity().roles().is_empty());
        assert_eq!(context.identity().tenant_id(), None);
        assert!(context.identity().attributes().is_empty());
    }

    // ── 4.3: error mapping ──────────────────────────────────────────────

    use kitlogger_log_domain::AuthenticationError;

    fn valid_window_claims(now: chrono::DateTime<chrono::Utc>, sub: &str) -> WireClaims {
        WireClaims {
            exp: now.timestamp() + 100,
            nbf: None,
            iat: now.timestamp(),
            iss: None,
            aud: None,
            sub: sub.to_string(),
            custom: std::collections::BTreeMap::new(),
        }
    }

    #[test]
    fn authenticate_rejects_token_with_invalid_signature() {
        let now = chrono::Utc::now();
        // Signed with a secret the resolver does NOT resolve to.
        let token = encode(
            &Header::new(Algorithm::HS256),
            &valid_window_claims(now, "user-1"),
            &EncodingKey::from_secret(b"a-different-secret-than-the-resolver-returns"),
        )
        .unwrap();
        let config = JwtConfig::new(vec![Algorithm::HS256], None, None, 0);
        let authenticator = authenticator_at(now, config);

        let result = authenticator.authenticate(&Credential::BearerToken(token));

        assert_eq!(result, Err(AuthenticationError::InvalidSignature));
    }

    #[test]
    fn authenticate_rejects_token_with_nbf_in_the_future_as_malformed() {
        let now = chrono::Utc::now();
        let mut claims = valid_window_claims(now, "user-1");
        claims.nbf = Some(now.timestamp() + 3600);
        let token = encode_hs256(&claims);
        let config = JwtConfig::new(vec![Algorithm::HS256], None, None, 0);
        let authenticator = authenticator_at(now, config);

        let result = authenticator.authenticate(&Credential::BearerToken(token));

        assert_eq!(result, Err(AuthenticationError::MalformedToken));
    }

    // ── 4.3b: KeyResolver failure propagation ───────────────────────────

    struct NeverResolves;

    impl KeyResolver for NeverResolves {
        fn resolve(&self, _kid: Option<&str>) -> Result<DecodingKey, JwtError> {
            Err(JwtError::KeyResolution("no key configured for this test".to_string()))
        }
    }

    #[test]
    fn authenticate_maps_key_resolution_failure_to_invalid_signature() {
        let now = chrono::Utc::now();
        let token = encode_hs256(&valid_window_claims(now, "user-1"));
        let authenticator = JwtAuthenticator::new(
            JwtConfig::new(vec![Algorithm::HS256], None, None, 0),
            JwtValidator::new(),
            Box::new(NeverResolves),
            Box::new(FakeClock::new(now)),
        );

        let result = authenticator.authenticate(&Credential::BearerToken(token));

        // Confirms end-to-end, through the real authenticate() flow (not
        // just the From<JwtError> unit test in error.rs from PR2), that
        // "unknown key" and "bad signature" are indistinguishable to the
        // caller — the documented key-enumeration-oracle defense holds when
        // KeyResolver is exercised from JwtAuthenticator itself.
        assert_eq!(result, Err(AuthenticationError::InvalidSignature));
    }
}
