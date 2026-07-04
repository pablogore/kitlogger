//! Integration test (task 4.7): `FakeClock` at the `exp` boundary,
//! exercised through the real `authenticate()` path (not `validate_claims`
//! directly, which PR3 already covers at the unit level).

mod common;

use chrono::{Duration, TimeZone, Utc};
use jsonwebtoken::{Algorithm, DecodingKey};
use kitlogger_log_domain::{AuthenticationError, AuthenticationProvider, Credential, FakeClock};
use security_jwt::JwtConfig;

use common::{build_authenticator, encode_hs256, valid_window_claims};

const SECRET: &[u8] = b"clock-boundary-integration-secret";

#[test]
fn clock_one_second_past_exp_rejects_as_expired() {
    let exp_instant = Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap();
    let mut claims = valid_window_claims(exp_instant.timestamp() - 100, "user-1");
    claims.exp = exp_instant.timestamp();
    let token = encode_hs256(&claims, SECRET);

    // Clock fixed 1 second AFTER exp (spec: "Clock-driven exp boundary").
    let clock_instant = exp_instant + Duration::seconds(1);
    let authenticator = build_authenticator(
        JwtConfig::new(vec![Algorithm::HS256], None, None, 0),
        DecodingKey::from_secret(SECRET),
        FakeClock::new(clock_instant),
    );

    let result = authenticator.authenticate(&Credential::BearerToken(token));

    assert_eq!(result, Err(AuthenticationError::Expired));
}

#[test]
fn clock_exactly_at_exp_is_not_yet_expired() {
    let exp_instant = Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap();
    let mut claims = valid_window_claims(exp_instant.timestamp() - 100, "user-1");
    claims.exp = exp_instant.timestamp();
    let token = encode_hs256(&claims, SECRET);

    let authenticator = build_authenticator(
        JwtConfig::new(vec![Algorithm::HS256], None, None, 0),
        DecodingKey::from_secret(SECRET),
        FakeClock::new(exp_instant),
    );

    let result = authenticator.authenticate(&Credential::BearerToken(token));

    assert!(result.is_ok());
}
