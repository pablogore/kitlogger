//! JWT-derived claims: standard registered claims plus custom claims.

use serde_json::Value;
use std::collections::BTreeMap;

/// Registered JWT claims (RFC 7519 section 4.1).
#[derive(Clone, Debug, PartialEq)]
pub struct StandardClaims {
    exp: i64,
    nbf: Option<i64>,
    iat: i64,
    iss: Option<String>,
    aud: Option<String>,
    sub: String,
}

impl StandardClaims {
    /// Creates a new `StandardClaims`.
    pub fn new(
        exp: i64,
        nbf: Option<i64>,
        iat: i64,
        iss: Option<String>,
        aud: Option<String>,
        sub: String,
    ) -> Self {
        StandardClaims {
            exp,
            nbf,
            iat,
            iss,
            aud,
            sub,
        }
    }

    /// Returns the expiration time (Unix seconds).
    pub fn exp(&self) -> i64 {
        self.exp
    }

    /// Returns the not-before time (Unix seconds), if present.
    pub fn nbf(&self) -> Option<i64> {
        self.nbf
    }

    /// Returns the issued-at time (Unix seconds).
    pub fn iat(&self) -> i64 {
        self.iat
    }

    /// Returns the issuer, if present.
    pub fn iss(&self) -> Option<&str> {
        self.iss.as_deref()
    }

    /// Returns the audience, if present.
    pub fn aud(&self) -> Option<&str> {
        self.aud.as_deref()
    }

    /// Returns the subject.
    pub fn sub(&self) -> &str {
        &self.sub
    }
}

/// Full claim set produced by a decoded JWT: standard claims plus
/// implementation-specific custom claims.
///
/// `custom` uses `BTreeMap` so keys iterate in deterministic lexicographic
/// order (required for serialization/comparison stability).
#[derive(Clone, Debug, PartialEq)]
pub struct Claims {
    standard: StandardClaims,
    custom: BTreeMap<String, Value>,
}

impl Claims {
    /// Creates a new `Claims`.
    pub fn new(standard: StandardClaims, custom: BTreeMap<String, Value>) -> Self {
        Claims { standard, custom }
    }

    /// Returns the standard registered claims.
    pub fn standard(&self) -> &StandardClaims {
        &self.standard
    }

    /// Returns the custom claims, in lexicographic key order.
    pub fn custom(&self) -> &BTreeMap<String, Value> {
        &self.custom
    }
}

#[cfg(test)]
mod tests {
    use super::{Claims, StandardClaims};
    use serde_json::json;
    use std::collections::BTreeMap;

    #[test]
    fn standard_claims_new_stores_all_registered_fields() {
        let standard = StandardClaims::new(
            1_700_000_100,
            Some(1_700_000_000),
            1_700_000_000,
            Some("issuer-a".to_string()),
            Some("audience-a".to_string()),
            "user-1".to_string(),
        );

        assert_eq!(standard.exp(), 1_700_000_100);
        assert_eq!(standard.nbf(), Some(1_700_000_000));
        assert_eq!(standard.iat(), 1_700_000_000);
        assert_eq!(standard.iss(), Some("issuer-a"));
        assert_eq!(standard.aud(), Some("audience-a"));
        assert_eq!(standard.sub(), "user-1");
    }

    #[test]
    fn claims_custom_map_iterates_keys_in_lexicographic_order() {
        let standard = StandardClaims::new(
            1_700_000_100,
            None,
            1_700_000_000,
            None,
            None,
            "user-1".to_string(),
        );

        let mut custom = BTreeMap::new();
        custom.insert("zone".to_string(), json!("a"));
        custom.insert("region".to_string(), json!("us-east-1"));

        let claims = Claims::new(standard, custom);

        let keys: Vec<&String> = claims.custom().keys().collect();
        assert_eq!(keys, vec!["region", "zone"]);
        assert_eq!(claims.custom().get("region"), Some(&json!("us-east-1")));
        assert_eq!(claims.custom().get("zone"), Some(&json!("a")));
    }
}
