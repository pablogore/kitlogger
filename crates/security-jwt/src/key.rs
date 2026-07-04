//! Resolves the `DecodingKey` used to verify a JWT's signature, keyed by the
//! token header's `kid` (key ID).

use jsonwebtoken::DecodingKey;

use crate::error::JwtError;

/// Resolves a `DecodingKey` for verifying a JWT's signature.
///
/// Object-safe and `Send + Sync` so implementations can be shared as
/// `Box<dyn KeyResolver>` / `Arc<dyn KeyResolver>` across threads.
/// Implementations may resolve a single static key, a set of keys keyed by
/// `kid` (JWKS-style rotation), or fetch keys from a remote source.
pub trait KeyResolver: Send + Sync {
    /// Resolves the decoding key for the given `kid`, or the default key
    /// when `kid` is `None`.
    ///
    /// # Errors
    ///
    /// Returns `JwtError::KeyResolution` when no matching key can be found.
    fn resolve(&self, kid: Option<&str>) -> Result<DecodingKey, JwtError>;
}

#[cfg(test)]
mod tests {
    use super::KeyResolver;
    use crate::error::JwtError;
    use jsonwebtoken::DecodingKey;
    use std::collections::HashMap;

    /// Minimal `KeyResolver` backed by an in-memory map, used only to
    /// exercise the trait contract in tests.
    struct MapKeyResolver {
        keys: HashMap<String, DecodingKey>,
    }

    impl KeyResolver for MapKeyResolver {
        fn resolve(&self, kid: Option<&str>) -> Result<DecodingKey, JwtError> {
            let kid = kid.ok_or_else(|| JwtError::KeyResolution("missing kid".to_string()))?;
            self.keys
                .get(kid)
                .cloned()
                .ok_or_else(|| JwtError::KeyResolution(format!("no key for kid={kid}")))
        }
    }

    fn resolver_with_one_key() -> MapKeyResolver {
        let mut keys = HashMap::new();
        keys.insert("key-1".to_string(), DecodingKey::from_secret(b"secret"));
        MapKeyResolver { keys }
    }

    #[test]
    fn resolves_key_by_matching_kid() {
        let resolver = resolver_with_one_key();

        let result = resolver.resolve(Some("key-1"));

        assert!(result.is_ok());
    }

    #[test]
    fn returns_key_resolution_error_when_kid_has_no_matching_key() {
        let resolver = resolver_with_one_key();

        let result = resolver.resolve(Some("unknown-kid"));

        assert!(matches!(result, Err(JwtError::KeyResolution(_))));
    }
}
