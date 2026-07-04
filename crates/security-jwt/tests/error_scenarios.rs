//! Integration test (task 4.6): broken tokens map to the correct
//! `AuthenticationError` variant end-to-end through `authenticate()`.

mod common;

use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey};
use kitlogger_log_domain::{AuthenticationError, AuthenticationProvider, Credential, FakeClock};
use security_jwt::JwtConfig;

use common::{build_authenticator, encode_hs256, valid_window_claims};

const SECRET: &[u8] = b"error-scenarios-integration-secret";

#[test]
fn invalid_signature_is_rejected() {
    let now = Utc::now();
    let claims = valid_window_claims(now.timestamp(), "user-1");
    // Signed with a secret different from the one the resolver will return.
    let token = encode_hs256(&claims, b"a-completely-different-signing-secret");

    let authenticator = build_authenticator(
        JwtConfig::new(vec![Algorithm::HS256], None, None, 0),
        DecodingKey::from_secret(SECRET),
        FakeClock::new(now),
    );

    let result = authenticator.authenticate(&Credential::BearerToken(token));

    assert_eq!(result, Err(AuthenticationError::InvalidSignature));
}

#[test]
fn expired_token_is_rejected() {
    let now = Utc::now();
    let mut claims = valid_window_claims(now.timestamp() - 200, "user-1");
    claims.exp = now.timestamp() - 100; // already expired relative to `now`
    let token = encode_hs256(&claims, SECRET);

    let authenticator = build_authenticator(
        JwtConfig::new(vec![Algorithm::HS256], None, None, 0),
        DecodingKey::from_secret(SECRET),
        FakeClock::new(now),
    );

    let result = authenticator.authenticate(&Credential::BearerToken(token));

    assert_eq!(result, Err(AuthenticationError::Expired));
}

#[test]
fn wrong_issuer_is_rejected() {
    let now = Utc::now();
    let mut claims = valid_window_claims(now.timestamp(), "user-1");
    claims.iss = Some("wrong-issuer".to_string());
    let token = encode_hs256(&claims, SECRET);

    let authenticator = build_authenticator(
        JwtConfig::new(vec![Algorithm::HS256], Some("kitlogger".to_string()), None, 0),
        DecodingKey::from_secret(SECRET),
        FakeClock::new(now),
    );

    let result = authenticator.authenticate(&Credential::BearerToken(token));

    assert_eq!(result, Err(AuthenticationError::InvalidIssuer));
}

#[test]
fn structurally_malformed_token_is_rejected_as_malformed_not_invalid_signature() {
    // FR-008: a string that never had a valid JWT shape at all (no header/
    // payload/signature segments) must surface as `MalformedToken`, not
    // `InvalidSignature` — those are two different failure classes: "this
    // isn't even a JWT" vs. "this JWT's signature is wrong".
    let now = Utc::now();

    let authenticator = build_authenticator(
        JwtConfig::new(vec![Algorithm::HS256], None, None, 0),
        DecodingKey::from_secret(SECRET),
        FakeClock::new(now),
    );

    let result = authenticator.authenticate(&Credential::BearerToken("not-a-jwt-at-all".to_string()));

    assert_eq!(result, Err(AuthenticationError::MalformedToken));
}

#[test]
fn wrong_audience_is_rejected() {
    let now = Utc::now();
    let mut claims = valid_window_claims(now.timestamp(), "user-1");
    claims.aud = Some("wrong-audience".to_string());
    let token = encode_hs256(&claims, SECRET);

    let authenticator = build_authenticator(
        JwtConfig::new(vec![Algorithm::HS256], None, Some("kitlogger-api".to_string()), 0),
        DecodingKey::from_secret(SECRET),
        FakeClock::new(now),
    );

    let result = authenticator.authenticate(&Credential::BearerToken(token));

    assert_eq!(result, Err(AuthenticationError::InvalidAudience));
}
