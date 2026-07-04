//! JWT validation configuration: allowed algorithms, issuer/audience checks, and clock leeway.

use jsonwebtoken::Algorithm;

/// Configuration controlling which JWTs are accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JwtConfig {
    /// Algorithms accepted when decoding a token (e.g. HS256, RS256).
    pub algorithms: Vec<Algorithm>,
    /// Expected `iss` claim. `None` disables issuer validation.
    pub issuer: Option<String>,
    /// Expected `aud` claim. `None` disables audience validation.
    pub audience: Option<String>,
    /// Allowed clock skew, in seconds, applied to exp/nbf/iat checks.
    pub leeway: u64,
}

impl JwtConfig {
    /// Creates a new `JwtConfig`.
    pub fn new(
        algorithms: Vec<Algorithm>,
        issuer: Option<String>,
        audience: Option<String>,
        leeway: u64,
    ) -> Self {
        JwtConfig {
            algorithms,
            issuer,
            audience,
            leeway,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::JwtConfig;
    use jsonwebtoken::Algorithm;

    #[test]
    fn stores_algorithms_issuer_audience_and_leeway() {
        let config = JwtConfig::new(
            vec![Algorithm::HS256, Algorithm::RS256],
            Some("kitlogger".to_string()),
            Some("kitlogger-api".to_string()),
            30,
        );

        assert_eq!(config.algorithms, vec![Algorithm::HS256, Algorithm::RS256]);
        assert_eq!(config.issuer, Some("kitlogger".to_string()));
        assert_eq!(config.audience, Some("kitlogger-api".to_string()));
        assert_eq!(config.leeway, 30);
    }

    #[test]
    fn issuer_and_audience_can_be_disabled_with_none() {
        let config = JwtConfig::new(vec![Algorithm::HS256], None, None, 0);

        assert_eq!(config.issuer, None);
        assert_eq!(config.audience, None);
    }
}
