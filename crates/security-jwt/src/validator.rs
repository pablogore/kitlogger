//! Claim-by-claim validation of an already-decoded JWT payload against a
//! `Clock` and `JwtConfig`. Pure logic — no I/O, no `jsonwebtoken::decode`
//! calls (decoding happens in `authenticator.rs`).

use kitlogger_log_domain::{AuthenticationError, Claims, Clock};

use crate::config::JwtConfig;

/// Validates already-decoded JWT claims against a `Clock` and `JwtConfig`.
///
/// Stateless: all inputs (claims, clock, config) are passed per call so the
/// same `JwtValidator` instance can validate tokens against different
/// clocks/configs if ever needed. Held as a field on `JwtAuthenticator`.
#[derive(Debug, Default, Clone, Copy)]
pub struct JwtValidator;

impl JwtValidator {
    /// Creates a new `JwtValidator`.
    pub fn new() -> Self {
        JwtValidator
    }

    /// Validates `claims` against `clock` and `config`.
    ///
    /// # Errors
    ///
    /// - `AuthenticationError::Expired` if `exp` is before `clock.now()`.
    /// - `AuthenticationError::MalformedToken` if `nbf` is after `clock.now()`,
    ///   or `iat` is beyond `clock.now() + config.leeway`.
    /// - `AuthenticationError::InvalidIssuer` if `config.issuer` is `Some` and
    ///   does not match `claims`' `iss` (including when `iss` is absent).
    /// - `AuthenticationError::InvalidAudience` if `config.audience` is
    ///   `Some` and does not match `claims`' `aud` (including when `aud` is
    ///   absent).
    pub fn validate_claims(
        &self,
        claims: &Claims,
        clock: &dyn Clock,
        config: &JwtConfig,
    ) -> Result<(), AuthenticationError> {
        let standard = claims.standard();
        let now = clock.now().timestamp();

        if standard.exp() < now {
            return Err(AuthenticationError::Expired);
        }

        if let Some(nbf) = standard.nbf() {
            if nbf > now {
                return Err(AuthenticationError::MalformedToken);
            }
        }

        let leeway = config.leeway as i64;
        if standard.iat() > now + leeway {
            return Err(AuthenticationError::MalformedToken);
        }

        if let Some(expected_issuer) = &config.issuer {
            if standard.iss() != Some(expected_issuer.as_str()) {
                return Err(AuthenticationError::InvalidIssuer);
            }
        }

        if let Some(expected_audience) = &config.audience {
            if standard.aud() != Some(expected_audience.as_str()) {
                return Err(AuthenticationError::InvalidAudience);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{JwtConfig, JwtValidator};
    use chrono::{DateTime, TimeZone, Utc};
    use kitlogger_log_domain::{AuthenticationError, Claims, FakeClock, StandardClaims};
    use serde_json::json;
    use std::collections::BTreeMap;

    /// `2024-01-15T12:00:00Z`, used as the fixed `FakeClock` instant for all
    /// boundary tests below.
    fn now_instant() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap()
    }

    fn claims_at(exp: i64, nbf: Option<i64>, iat: i64) -> Claims {
        let standard = StandardClaims::new(exp, nbf, iat, None, None, "user-1".to_string());
        Claims::new(standard, Default::default())
    }

    fn no_issuer_or_audience_config(leeway: u64) -> JwtConfig {
        JwtConfig::new(vec![], None, None, leeway)
    }

    /// Claims with a valid (non-expired, already-active) time window and the
    /// given `iss`/`aud`, anchored to `now`.
    fn claims_with_iss_aud(now: DateTime<Utc>, iss: Option<&str>, aud: Option<&str>) -> Claims {
        let standard = StandardClaims::new(
            now.timestamp() + 100,
            None,
            now.timestamp(),
            iss.map(str::to_string),
            aud.map(str::to_string),
            "user-1".to_string(),
        );
        Claims::new(standard, Default::default())
    }

    // ── exp ──────────────────────────────────────────────────────────────

    #[test]
    fn exp_exactly_at_now_is_not_expired() {
        let now = now_instant();
        let clock = FakeClock::new(now);
        let claims = claims_at(now.timestamp(), None, now.timestamp());
        let config = no_issuer_or_audience_config(0);

        assert!(JwtValidator::new()
            .validate_claims(&claims, &clock, &config)
            .is_ok());
    }

    #[test]
    fn exp_one_second_before_now_is_expired() {
        let now = now_instant();
        let clock = FakeClock::new(now);
        let claims = claims_at(now.timestamp() - 1, None, now.timestamp());
        let config = no_issuer_or_audience_config(0);

        assert_eq!(
            JwtValidator::new().validate_claims(&claims, &clock, &config),
            Err(AuthenticationError::Expired)
        );
    }

    #[test]
    fn exp_one_second_after_now_is_not_expired() {
        let now = now_instant();
        let clock = FakeClock::new(now);
        let claims = claims_at(now.timestamp() + 1, None, now.timestamp());
        let config = no_issuer_or_audience_config(0);

        assert!(JwtValidator::new()
            .validate_claims(&claims, &clock, &config)
            .is_ok());
    }

    // ── nbf ──────────────────────────────────────────────────────────────

    #[test]
    fn nbf_exactly_at_now_is_accepted() {
        let now = now_instant();
        let clock = FakeClock::new(now);
        let claims = claims_at(now.timestamp() + 100, Some(now.timestamp()), now.timestamp());
        let config = no_issuer_or_audience_config(0);

        assert!(JwtValidator::new()
            .validate_claims(&claims, &clock, &config)
            .is_ok());
    }

    #[test]
    fn nbf_one_second_before_now_is_accepted() {
        let now = now_instant();
        let clock = FakeClock::new(now);
        let claims = claims_at(
            now.timestamp() + 100,
            Some(now.timestamp() - 1),
            now.timestamp(),
        );
        let config = no_issuer_or_audience_config(0);

        assert!(JwtValidator::new()
            .validate_claims(&claims, &clock, &config)
            .is_ok());
    }

    #[test]
    fn nbf_one_second_after_now_is_rejected_as_malformed() {
        let now = now_instant();
        let clock = FakeClock::new(now);
        let claims = claims_at(
            now.timestamp() + 100,
            Some(now.timestamp() + 1),
            now.timestamp(),
        );
        let config = no_issuer_or_audience_config(0);

        assert_eq!(
            JwtValidator::new().validate_claims(&claims, &clock, &config),
            Err(AuthenticationError::MalformedToken)
        );
    }

    // ── iat (with leeway) ────────────────────────────────────────────────

    #[test]
    fn iat_exactly_at_now_plus_leeway_is_accepted() {
        let now = now_instant();
        let clock = FakeClock::new(now);
        let leeway = 30;
        let claims = claims_at(now.timestamp() + 100, None, now.timestamp() + leeway as i64);
        let config = no_issuer_or_audience_config(leeway);

        assert!(JwtValidator::new()
            .validate_claims(&claims, &clock, &config)
            .is_ok());
    }

    #[test]
    fn iat_one_second_before_now_plus_leeway_is_accepted() {
        let now = now_instant();
        let clock = FakeClock::new(now);
        let leeway = 30;
        let claims = claims_at(
            now.timestamp() + 100,
            None,
            now.timestamp() + leeway as i64 - 1,
        );
        let config = no_issuer_or_audience_config(leeway);

        assert!(JwtValidator::new()
            .validate_claims(&claims, &clock, &config)
            .is_ok());
    }

    #[test]
    fn iat_one_second_after_now_plus_leeway_is_rejected_as_malformed() {
        let now = now_instant();
        let clock = FakeClock::new(now);
        let leeway = 30;
        let claims = claims_at(
            now.timestamp() + 100,
            None,
            now.timestamp() + leeway as i64 + 1,
        );
        let config = no_issuer_or_audience_config(leeway);

        assert_eq!(
            JwtValidator::new().validate_claims(&claims, &clock, &config),
            Err(AuthenticationError::MalformedToken)
        );
    }

    // ── issuer ───────────────────────────────────────────────────────────

    #[test]
    fn issuer_matching_configured_issuer_is_accepted() {
        let now = now_instant();
        let clock = FakeClock::new(now);
        let claims = claims_with_iss_aud(now, Some("kitlogger"), None);
        let config = JwtConfig::new(vec![], Some("kitlogger".to_string()), None, 0);

        assert!(JwtValidator::new()
            .validate_claims(&claims, &clock, &config)
            .is_ok());
    }

    #[test]
    fn issuer_mismatching_configured_issuer_is_rejected() {
        let now = now_instant();
        let clock = FakeClock::new(now);
        let claims = claims_with_iss_aud(now, Some("wrong"), None);
        let config = JwtConfig::new(vec![], Some("kitlogger".to_string()), None, 0);

        assert_eq!(
            JwtValidator::new().validate_claims(&claims, &clock, &config),
            Err(AuthenticationError::InvalidIssuer)
        );
    }

    #[test]
    fn issuer_missing_from_claims_when_configured_is_rejected() {
        let now = now_instant();
        let clock = FakeClock::new(now);
        let claims = claims_with_iss_aud(now, None, None);
        let config = JwtConfig::new(vec![], Some("kitlogger".to_string()), None, 0);

        assert_eq!(
            JwtValidator::new().validate_claims(&claims, &clock, &config),
            Err(AuthenticationError::InvalidIssuer)
        );
    }

    #[test]
    fn issuer_check_is_skipped_when_config_issuer_is_none() {
        let now = now_instant();
        let clock = FakeClock::new(now);
        let claims = claims_with_iss_aud(now, Some("anything"), None);
        let config = no_issuer_or_audience_config(0);

        assert!(JwtValidator::new()
            .validate_claims(&claims, &clock, &config)
            .is_ok());
    }

    // ── audience ─────────────────────────────────────────────────────────

    #[test]
    fn audience_matching_configured_audience_is_accepted() {
        let now = now_instant();
        let clock = FakeClock::new(now);
        let claims = claims_with_iss_aud(now, None, Some("kitlogger-api"));
        let config = JwtConfig::new(vec![], None, Some("kitlogger-api".to_string()), 0);

        assert!(JwtValidator::new()
            .validate_claims(&claims, &clock, &config)
            .is_ok());
    }

    #[test]
    fn audience_mismatching_configured_audience_is_rejected() {
        let now = now_instant();
        let clock = FakeClock::new(now);
        let claims = claims_with_iss_aud(now, None, Some("wrong"));
        let config = JwtConfig::new(vec![], None, Some("kitlogger-api".to_string()), 0);

        assert_eq!(
            JwtValidator::new().validate_claims(&claims, &clock, &config),
            Err(AuthenticationError::InvalidAudience)
        );
    }

    #[test]
    fn audience_missing_from_claims_when_configured_is_rejected() {
        let now = now_instant();
        let clock = FakeClock::new(now);
        let claims = claims_with_iss_aud(now, None, None);
        let config = JwtConfig::new(vec![], None, Some("kitlogger-api".to_string()), 0);

        assert_eq!(
            JwtValidator::new().validate_claims(&claims, &clock, &config),
            Err(AuthenticationError::InvalidAudience)
        );
    }

    #[test]
    fn audience_check_is_skipped_when_config_audience_is_none() {
        let now = now_instant();
        let clock = FakeClock::new(now);
        let claims = claims_with_iss_aud(now, None, Some("anything"));
        let config = no_issuer_or_audience_config(0);

        assert!(JwtValidator::new()
            .validate_claims(&claims, &clock, &config)
            .is_ok());
    }

    // ── custom claims ordering ───────────────────────────────────────────

    #[test]
    fn validate_claims_preserves_custom_claims_lexicographic_key_order() {
        let now = now_instant();
        let clock = FakeClock::new(now);
        let standard = StandardClaims::new(
            now.timestamp() + 100,
            None,
            now.timestamp(),
            None,
            None,
            "user-1".to_string(),
        );
        let mut custom = BTreeMap::new();
        custom.insert("zone".to_string(), json!("a"));
        custom.insert("region".to_string(), json!("us-east-1"));
        custom.insert("app".to_string(), json!("kitlogger"));
        let claims = Claims::new(standard, custom);
        let config = no_issuer_or_audience_config(0);

        let result = JwtValidator::new().validate_claims(&claims, &clock, &config);

        assert!(result.is_ok());
        let keys: Vec<&String> = claims.custom().keys().collect();
        assert_eq!(keys, vec!["app", "region", "zone"]);
        assert_eq!(claims.custom().get("region"), Some(&json!("us-east-1")));
    }
}
