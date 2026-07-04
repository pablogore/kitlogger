//! Integration test (task 4.4): a valid HS256 token, signed with
//! `jsonwebtoken::encode`, round-trips through
//! `JwtAuthenticator::authenticate` into a `SecurityContext`.

mod common;

use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey};
use kitlogger_log_domain::{AuthenticationProvider, Credential, FakeClock};
use security_jwt::JwtConfig;

use common::{build_authenticator, encode_hs256, valid_window_claims};

const SECRET: &[u8] = b"hs256-roundtrip-integration-secret";

#[test]
fn valid_hs256_token_authenticates_into_security_context() {
    let now = Utc::now();
    let claims = valid_window_claims(now.timestamp(), "hs256-user");
    let token = encode_hs256(&claims, SECRET);

    let authenticator = build_authenticator(
        JwtConfig::new(vec![Algorithm::HS256], None, None, 0),
        DecodingKey::from_secret(SECRET),
        FakeClock::new(now),
    );

    let context = authenticator
        .authenticate(&Credential::BearerToken(token))
        .expect("valid HS256 token must authenticate");

    assert_eq!(context.identity().subject(), "hs256-user");
    assert_eq!(context.claims().standard().sub(), "hs256-user");
    assert_eq!(context.claims().standard().exp(), claims.exp);
}
