//! Security context produced by a successful authentication.

use crate::claims::Claims;
use crate::identity::Identity;

/// The outcome of a successful `AuthenticationProvider::authenticate` call:
/// an `Identity` paired with the `Claims` it was derived from.
#[derive(Clone, Debug, PartialEq)]
pub struct SecurityContext {
    identity: Identity,
    claims: Claims,
}

impl SecurityContext {
    /// Creates a new `SecurityContext`.
    pub fn new(identity: Identity, claims: Claims) -> Self {
        SecurityContext { identity, claims }
    }

    /// Returns the authenticated identity.
    pub fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Returns the claims the identity was derived from.
    pub fn claims(&self) -> &Claims {
        &self.claims
    }
}

#[cfg(test)]
mod tests {
    use super::SecurityContext;
    use crate::claims::{Claims, StandardClaims};
    use crate::identity::Identity;
    use std::collections::{BTreeMap, BTreeSet};

    fn sample_identity() -> Identity {
        let mut roles = BTreeSet::new();
        roles.insert("admin".to_string());
        Identity::new(
            "user-1".to_string(),
            roles,
            Some("tenant-a".to_string()),
            BTreeMap::new(),
        )
    }

    fn sample_claims() -> Claims {
        let standard = StandardClaims::new(1_700_000_100, None, 1_700_000_000, None, None, "user-1".to_string());
        Claims::new(standard, BTreeMap::new())
    }

    #[test]
    fn new_exposes_identity_and_claims() {
        let identity = sample_identity();
        let claims = sample_claims();

        let context = SecurityContext::new(identity.clone(), claims.clone());

        assert_eq!(context.identity(), &identity);
        assert_eq!(context.claims(), &claims);
    }

    #[test]
    fn two_contexts_with_different_identities_are_not_equal() {
        let context_a = SecurityContext::new(sample_identity(), sample_claims());
        let other_identity = Identity::new(
            "user-2".to_string(),
            BTreeSet::new(),
            None,
            BTreeMap::new(),
        );
        let context_b = SecurityContext::new(other_identity, sample_claims());

        assert_ne!(context_a, context_b);
    }
}
