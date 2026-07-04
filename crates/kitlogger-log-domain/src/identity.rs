//! Authenticated identity produced by a successful `AuthenticationProvider::authenticate` call.

use std::collections::{BTreeMap, BTreeSet};

/// Identity information extracted from a validated credential.
///
/// `roles` and `attributes` use `BTreeMap`/`BTreeSet` for deterministic,
/// lexicographically ordered iteration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    subject: String,
    roles: BTreeSet<String>,
    tenant_id: Option<String>,
    attributes: BTreeMap<String, String>,
}

impl Identity {
    /// Creates a new `Identity`.
    pub fn new(
        subject: String,
        roles: BTreeSet<String>,
        tenant_id: Option<String>,
        attributes: BTreeMap<String, String>,
    ) -> Self {
        Identity {
            subject,
            roles,
            tenant_id,
            attributes,
        }
    }

    /// Returns the subject (principal identifier).
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// Returns the assigned roles, in lexicographic order.
    pub fn roles(&self) -> &BTreeSet<String> {
        &self.roles
    }

    /// Returns the tenant identifier, if the identity is tenant-scoped.
    pub fn tenant_id(&self) -> Option<&str> {
        self.tenant_id.as_deref()
    }

    /// Returns additional identity attributes, in lexicographic key order.
    pub fn attributes(&self) -> &BTreeMap<String, String> {
        &self.attributes
    }
}

#[cfg(test)]
mod tests {
    use super::Identity;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn new_stores_subject_roles_tenant_and_attributes() {
        let mut roles = BTreeSet::new();
        roles.insert("admin".to_string());
        roles.insert("viewer".to_string());

        let mut attributes = BTreeMap::new();
        attributes.insert("department".to_string(), "engineering".to_string());

        let identity = Identity::new(
            "user-42".to_string(),
            roles.clone(),
            Some("tenant-acme".to_string()),
            attributes.clone(),
        );

        assert_eq!(identity.subject(), "user-42");
        assert_eq!(identity.roles(), &roles);
        assert_eq!(identity.tenant_id(), Some("tenant-acme"));
        assert_eq!(identity.attributes(), &attributes);
    }

    #[test]
    fn new_with_no_tenant_and_no_roles_returns_empty_collections() {
        let identity = Identity::new(
            "anonymous-service".to_string(),
            BTreeSet::new(),
            None,
            BTreeMap::new(),
        );

        assert_eq!(identity.subject(), "anonymous-service");
        assert!(identity.roles().is_empty());
        assert_eq!(identity.tenant_id(), None);
        assert!(identity.attributes().is_empty());
    }
}
