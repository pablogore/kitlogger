//! Error types produced by `security-jwt` and their mapping to the domain's
//! provider-agnostic `AuthenticationError`.

use kitlogger_log_domain::AuthenticationError;

/// Infrastructure-level failure produced while decoding or resolving a key
/// for a JWT.
///
/// Kept separate from `AuthenticationError` so `security-jwt` can carry
/// JWT-specific detail (useful for logging/debugging) while still exposing a
/// provider-agnostic error to `AuthenticationProvider` consumers via
/// `From<JwtError> for AuthenticationError`.
///
/// **Do not log or expose this type (or its `Display`/`Debug` output) to an
/// untrusted caller.** Several distinct `JwtError` variants (and distinct
/// `jsonwebtoken::errors::ErrorKind`s within `Decode`) deliberately collapse
/// into the same `AuthenticationError::InvalidSignature` to avoid a
/// key/algorithm-enumeration oracle (see `From<JwtError> for
/// AuthenticationError` below). Surfacing `JwtError` itself to a caller would
/// defeat that protection. It exists only for server-side logging/debugging.
#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    /// `jsonwebtoken` failed to decode or verify the token (malformed
    /// structure, bad signature, unsupported header, etc.).
    #[error("failed to decode token: {0}")]
    Decode(#[from] jsonwebtoken::errors::Error),

    /// The token's algorithm is not one of `JwtConfig::algorithms`.
    ///
    /// Reserved for external `KeyResolver` implementations that want to
    /// reject a token's algorithm before attempting key resolution (e.g. a
    /// resolver backed by a JWKS endpoint that only serves certain
    /// algorithms per `kid`). Not currently constructed by this crate's own
    /// code — `jsonwebtoken`'s algorithm mismatch surfaces via
    /// `ErrorKind::InvalidAlgorithm` through the `Decode` variant instead.
    #[error("unsupported or mismatched algorithm: {0}")]
    Algorithm(String),

    /// `KeyResolver` could not find a decoding key for the given `kid`.
    #[error("no key found for token: {0}")]
    KeyResolution(String),
}

impl From<JwtError> for AuthenticationError {
    fn from(err: JwtError) -> Self {
        match err {
            // `jsonwebtoken::errors::ErrorKind` distinguishes structural
            // failures ("this string never had a valid JWT shape") from
            // cryptographic failures ("this JWT's signature/key/algorithm
            // could not be trusted"). FR-008 requires the former to surface
            // as `MalformedToken`; the latter keeps mapping to
            // `InvalidSignature` (see the oracle-avoidance rationale below).
            JwtError::Decode(inner) => match inner.kind() {
                // Structural: not a valid JWT shape, or the base64/JSON/
                // UTF-8 layers underneath it could not even be decoded.
                jsonwebtoken::errors::ErrorKind::InvalidToken
                | jsonwebtoken::errors::ErrorKind::Base64(_)
                | jsonwebtoken::errors::ErrorKind::Json(_)
                | jsonwebtoken::errors::ErrorKind::Utf8(_) => AuthenticationError::MalformedToken,
                // Cryptographic: the token has a valid JWT shape but its
                // signature, key, or algorithm could not be trusted.
                // `security-jwt`'s own `JwtValidator` (exp/nbf/iat/iss/aud)
                // is the source of the domain's more specific error
                // variants, not `jsonwebtoken`'s internal checks.
                jsonwebtoken::errors::ErrorKind::InvalidSignature
                | jsonwebtoken::errors::ErrorKind::InvalidEcdsaKey
                | jsonwebtoken::errors::ErrorKind::InvalidEddsaKey
                | jsonwebtoken::errors::ErrorKind::InvalidRsaKey(_)
                | jsonwebtoken::errors::ErrorKind::RsaFailedSigning
                | jsonwebtoken::errors::ErrorKind::Signing(_)
                | jsonwebtoken::errors::ErrorKind::InvalidAlgorithm
                | jsonwebtoken::errors::ErrorKind::InvalidAlgorithmName
                | jsonwebtoken::errors::ErrorKind::InvalidKeyFormat
                | jsonwebtoken::errors::ErrorKind::MissingAlgorithm
                | jsonwebtoken::errors::ErrorKind::Provider(_) => {
                    AuthenticationError::InvalidSignature
                }
                // Claim-validation variants should not normally occur:
                // `JwtAuthenticator::authenticate` disables jsonwebtoken's
                // own exp/aud/required-claims validation and routes all
                // claim semantics through `JwtValidator` instead (see
                // authenticator.rs). Mapped defensively to `MalformedToken`
                // — closer to "the claims couldn't even be parsed/checked"
                // than "bad signature" — as a safe catch-all for a branch
                // that should be unreachable in normal operation.
                jsonwebtoken::errors::ErrorKind::ExpiredSignature
                | jsonwebtoken::errors::ErrorKind::InvalidIssuer
                | jsonwebtoken::errors::ErrorKind::InvalidAudience
                | jsonwebtoken::errors::ErrorKind::InvalidSubject
                | jsonwebtoken::errors::ErrorKind::ImmatureSignature
                | jsonwebtoken::errors::ErrorKind::MissingRequiredClaim(_)
                | jsonwebtoken::errors::ErrorKind::InvalidClaimFormat(_) => {
                    AuthenticationError::MalformedToken
                }
                // `ErrorKind` is `#[non_exhaustive]`: any future variant
                // jsonwebtoken adds defaults to the safer "signature could
                // not be verified" outcome rather than leaking parser
                // detail through an unmatched-variant panic.
                //
                // Maintainability: this match intentionally mirrors
                // `jsonwebtoken`'s `ErrorKind` taxonomy as of 10.4.0. Because
                // `ErrorKind` is `#[non_exhaustive]`, jsonwebtoken can add
                // variants in a *minor* release, not just a major one — a
                // new parsing-related kind (e.g. ASN.1/PEM/JWK errors) would
                // silently fall into this catch-all as `InvalidSignature`
                // instead of `MalformedToken` until this match is revisited.
                // Re-check this arm against jsonwebtoken's changelog on every
                // version bump, not just major ones.
                _ => AuthenticationError::InvalidSignature,
            },
            // An algorithm the caller does not trust is treated the same as
            // an untrusted signature (defends against algorithm-confusion
            // attacks, e.g. "alg: none").
            JwtError::Algorithm(_) => AuthenticationError::InvalidSignature,
            // No key to verify against is, from the caller's perspective,
            // indistinguishable from "the signature could not be verified".
            JwtError::KeyResolution(_) => AuthenticationError::InvalidSignature,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::JwtError;
    use kitlogger_log_domain::AuthenticationError;

    #[test]
    fn decode_error_with_invalid_token_shape_maps_to_malformed_token() {
        // `ErrorKind::InvalidToken` means the string never had a valid JWT
        // shape at all (FR-008) — structural, not cryptographic.
        let err = JwtError::Decode(jsonwebtoken::errors::ErrorKind::InvalidToken.into());

        assert_eq!(
            AuthenticationError::from(err),
            AuthenticationError::MalformedToken
        );
    }

    #[test]
    fn decode_error_with_invalid_signature_kind_still_maps_to_invalid_signature() {
        // A structurally valid JWT with a bad signature is a cryptographic
        // failure and must keep mapping to `InvalidSignature`.
        let err = JwtError::Decode(jsonwebtoken::errors::ErrorKind::InvalidSignature.into());

        assert_eq!(
            AuthenticationError::from(err),
            AuthenticationError::InvalidSignature
        );
    }

    #[test]
    fn algorithm_error_maps_to_invalid_signature() {
        let err = JwtError::Algorithm("ES256 not permitted".to_string());

        assert_eq!(
            AuthenticationError::from(err),
            AuthenticationError::InvalidSignature
        );
    }

    #[test]
    fn key_resolution_error_maps_to_invalid_signature() {
        let err = JwtError::KeyResolution("no key for kid=abc".to_string());

        assert_eq!(
            AuthenticationError::from(err),
            AuthenticationError::InvalidSignature
        );
    }
}
