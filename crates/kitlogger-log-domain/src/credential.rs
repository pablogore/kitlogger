//! Credentials presented by a caller for authentication.

/// A credential presented for authentication.
///
/// Deliberately an enum (not a `BearerToken` wrapper struct) to leave room
/// for future variants (`Basic`, `ApiKey`, `Mtls`, ...) without breaking
/// `AuthenticationProvider::authenticate` callers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Credential {
    /// A raw JWT presented as an `Authorization: Bearer <token>` header.
    BearerToken(String),
}

#[cfg(test)]
mod tests {
    use super::Credential;

    #[test]
    fn bearer_token_variant_holds_raw_token_string() {
        let credential = Credential::BearerToken("raw.jwt.token".to_string());

        match credential {
            Credential::BearerToken(token) => assert_eq!(token, "raw.jwt.token"),
        }
    }

    #[test]
    fn two_bearer_tokens_with_different_values_are_not_equal() {
        let a = Credential::BearerToken("token-a".to_string());
        let b = Credential::BearerToken("token-b".to_string());

        assert_ne!(a, b);
    }
}
