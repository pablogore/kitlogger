//! Integration test (task 4.5): a valid RS256 token, verified via an RSA
//! public key resolved through `KeyResolver`, authenticates successfully.

mod common;

use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey};
use kitlogger_log_domain::{AuthenticationProvider, Credential, FakeClock};
use security_jwt::JwtConfig;

use common::{
    build_authenticator, encode_rs256, valid_window_claims, TEST_RSA_PRIVATE_KEY_PEM,
    TEST_RSA_PUBLIC_KEY_PEM,
};

#[test]
fn valid_rs256_token_authenticates_via_key_resolver() {
    let now = Utc::now();
    let claims = valid_window_claims(now.timestamp(), "rs256-user");
    let token = encode_rs256(&claims, TEST_RSA_PRIVATE_KEY_PEM.as_bytes());

    let public_key = DecodingKey::from_rsa_pem(TEST_RSA_PUBLIC_KEY_PEM.as_bytes())
        .expect("test RSA public key PEM must parse");
    let authenticator = build_authenticator(
        JwtConfig::new(vec![Algorithm::RS256], None, None, 0),
        public_key,
        FakeClock::new(now),
    );

    let context = authenticator
        .authenticate(&Credential::BearerToken(token))
        .expect("valid RS256 token must authenticate via KeyResolver");

    assert_eq!(context.identity().subject(), "rs256-user");
    assert_eq!(context.claims().standard().sub(), "rs256-user");
}
