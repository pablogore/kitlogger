//! Public authentication contract: `AuthenticationProvider` trait and `AuthenticationError`.

use crate::credential::Credential;
use crate::security::SecurityContext;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Authenticates a `Credential` into a `SecurityContext`.
///
/// Object-safe and `Send + Sync` so implementations can be stored as
/// `Box<dyn AuthenticationProvider>` / `Arc<dyn AuthenticationProvider>` and
/// shared across threads. Any authentication scheme (JWT, mTLS, API key,
/// ...) can implement this trait without the domain crate depending on it.
pub trait AuthenticationProvider: Send + Sync {
    /// Authenticates the given credential.
    ///
    /// # Errors
    ///
    /// Returns an `AuthenticationError` describing why the credential was
    /// rejected.
    fn authenticate(&self, credential: &Credential) -> Result<SecurityContext, AuthenticationError>;
}

/// Domain-level authentication failure. Kept provider-agnostic so consumers
/// of `AuthenticationProvider` never need to depend on a specific auth
/// scheme's crate (e.g. `security-jwt`) to handle failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthenticationError {
    /// The credential's expiration time has passed.
    Expired,
    /// The credential's signature could not be verified.
    InvalidSignature,
    /// The credential's issuer does not match the configured issuer.
    InvalidIssuer,
    /// The credential's audience does not match the configured audience.
    InvalidAudience,
    /// The credential is not well-formed.
    MalformedToken,
}

impl Display for AuthenticationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            AuthenticationError::Expired => write!(f, "token expired"),
            AuthenticationError::InvalidSignature => write!(f, "invalid token signature"),
            AuthenticationError::InvalidIssuer => write!(f, "invalid token issuer"),
            AuthenticationError::InvalidAudience => write!(f, "invalid token audience"),
            AuthenticationError::MalformedToken => write!(f, "malformed token"),
        }
    }
}

impl std::error::Error for AuthenticationError {}

#[cfg(test)]
mod tests {
    use super::{AuthenticationError, AuthenticationProvider};
    use crate::credential::Credential;
    use crate::security::SecurityContext;

    // ── Object-safety smoke tests ───────────────────────────────────────────

    #[test]
    fn authentication_provider_is_object_safe_box() {
        // Compile-time check only: if AuthenticationProvider were NOT
        // object-safe, this line would fail to compile.
        let _: Option<Box<dyn AuthenticationProvider>> = None;
    }

    struct AlwaysRejects;

    impl AuthenticationProvider for AlwaysRejects {
        fn authenticate(
            &self,
            _credential: &Credential,
        ) -> Result<SecurityContext, AuthenticationError> {
            Err(AuthenticationError::MalformedToken)
        }
    }

    #[test]
    fn provider_impl_returns_declared_error_through_trait_object() {
        let provider: Box<dyn AuthenticationProvider> = Box::new(AlwaysRejects);
        let result = provider.authenticate(&Credential::BearerToken("x".to_string()));

        assert_eq!(result, Err(AuthenticationError::MalformedToken));
    }

    // ── AuthenticationError Display messages ────────────────────────────────

    #[test]
    fn display_message_for_expired() {
        assert_eq!(format!("{}", AuthenticationError::Expired), "token expired");
    }

    #[test]
    fn display_message_for_invalid_signature() {
        assert_eq!(
            format!("{}", AuthenticationError::InvalidSignature),
            "invalid token signature"
        );
    }

    #[test]
    fn display_message_for_invalid_issuer() {
        assert_eq!(
            format!("{}", AuthenticationError::InvalidIssuer),
            "invalid token issuer"
        );
    }

    #[test]
    fn display_message_for_invalid_audience() {
        assert_eq!(
            format!("{}", AuthenticationError::InvalidAudience),
            "invalid token audience"
        );
    }

    #[test]
    fn display_message_for_malformed_token() {
        assert_eq!(
            format!("{}", AuthenticationError::MalformedToken),
            "malformed token"
        );
    }
}
