//! Shared test utilities for `security-jwt` integration tests (tasks
//! 4.4-4.7).
#![allow(dead_code)]

use std::collections::BTreeMap;

use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
use kitlogger_log_domain::Clock;
use security_jwt::{JwtAuthenticator, JwtConfig, JwtError, JwtValidator, KeyResolver};
use serde::Serialize;
use serde_json::Value;

/// Wire-format claims used to encode test tokens. Mirrors the shape
/// `security_jwt::authenticator`'s module-private `RawClaims` expects to
/// decode: registered claims plus flattened custom claims.
#[derive(Serialize, Clone)]
pub struct WireClaims {
    pub exp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,
    pub iat: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    pub sub: String,
    #[serde(flatten)]
    pub custom: BTreeMap<String, Value>,
}

/// Claims with a valid (non-expired, already-active) time window relative to
/// `now_ts`, no issuer/audience/custom claims set.
pub fn valid_window_claims(now_ts: i64, sub: &str) -> WireClaims {
    WireClaims {
        exp: now_ts + 100,
        nbf: None,
        iat: now_ts,
        iss: None,
        aud: None,
        sub: sub.to_string(),
        custom: BTreeMap::new(),
    }
}

pub fn encode_hs256(claims: &WireClaims, secret: &[u8]) -> String {
    encode(&Header::new(Algorithm::HS256), claims, &EncodingKey::from_secret(secret))
        .expect("encoding a well-formed HS256 test token must succeed")
}

pub fn encode_rs256(claims: &WireClaims, private_key_pem: &[u8]) -> String {
    let key = EncodingKey::from_rsa_pem(private_key_pem)
        .expect("test RSA private key PEM must parse");
    encode(&Header::new(Algorithm::RS256), claims, &key)
        .expect("encoding a well-formed RS256 test token must succeed")
}

/// A `KeyResolver` that always resolves to a fixed key, regardless of `kid`.
pub struct FixedKeyResolver(pub DecodingKey);

impl KeyResolver for FixedKeyResolver {
    fn resolve(&self, _kid: Option<&str>) -> Result<DecodingKey, JwtError> {
        Ok(self.0.clone())
    }
}

/// Builds a `JwtAuthenticator` wired with a `FixedKeyResolver` and the given
/// `clock`, for exercising `authenticate()` end-to-end.
pub fn build_authenticator(
    config: JwtConfig,
    key: DecodingKey,
    clock: impl Clock + 'static,
) -> JwtAuthenticator {
    JwtAuthenticator::new(
        config,
        JwtValidator::new(),
        Box::new(FixedKeyResolver(key)),
        Box::new(clock),
    )
}

/// Test-only 2048-bit RSA key pair (PKCS8), generated once via `openssl
/// genrsa` for deterministic, fast tests. Used ONLY by
/// `tests/rs256_roundtrip.rs` — never a production key.
pub const TEST_RSA_PRIVATE_KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIIEvwIBADANBgkqhkiG9w0BAQEFAASCBKkwggSlAgEAAoIBAQCvq/ZYtUOioqpY
wo/vKp98oEKIuYWfjziVpjHCRQtCWr6PdVx8eTsJRZtYGdGWMb9cNEKB4D0Re1kV
+jyUKKHKVOwDDc2hyJ5d29g8q1wkgkxE6TCOxfcF650Is4fEsRQRMQd0unOUQBhz
gEYzFs80wWqgKTfk32UqvQbMWC7BO4CWc1tri05EApsRznqltquM8ycRw/giO2bJ
u892Wmn3ipxLgwV7ktvVLATnSuc9nvJSQuSMZGBx4yUsUodhg11aUqV9ThGUGBPz
SaFkrRQ9+VVXGVDc9RuF59PDrCnHDSZQxNowXyn6zP4l42tdcDEFDd70TyLrFXJV
XgTyvcqjAgMBAAECggEACWMZANh+yrwCeke7RT5z00cFByVSvJn8nhKJaiYiWeNb
d3MABC6+NtqfWG/YwTvbS+QHAcWnvb1nn0MF5azeKF+tVORx7ok33TAYN/W4MMM9
rBkB3T862fgjJgnh4WLM/bYu/c/QyUU7iUB7f9NiMyF6vyA0umgtkcEUEetMZErg
/6SqUAbkT9mVqQAayZKFjrkLMFKvbf19y54TzcStcJ1xS++M/zl74YZrjL0yXud2
AB5OegT5hzl2q5YbQ1CH2blluL6oU2PWlH5k/E+ue3wCKL9rqlYu8kD8Ev7XzPyn
xgsbyxXSjptisuWVeH5J92o6i4JGMdCM9+7Li4GkwQKBgQDcm/U8n25dL6HCq/wc
+AiY2vwvx1QAANf+k6//ceyKgIyEPRmqN6R3fCRYX0kKrJc3fDJ+Fkus/dEIRLyE
qySQrX+XbqdL0/KeQ8hTAEm9tUyFW7PdIB6gYgZae5s2im9uSoYe15PYO96Mo6ts
Ko6NRk8CipEGcX3Dc/9UgPtHMwKBgQDL2n+7VEkR673dnkXhk49vJnGi1QvWvzgl
m6p79lvNPcNTwJHuugcE80BSYtE9yJltYTngzwcLKP8dRDir6R4XzevNt8eW9mAW
xq4VB4A6YLnZuOewIwtP+oK19PTSNSBeNIbSSVMq0xyMPjbM5kn+juxVMfrOiu+j
u9AuJ/yu0QKBgQCSyRp1icdPLaA9/V00kTx3KE3gJIqZrfYJp47VMqImzb2xR05K
WdeOwQUV3+Cjv46mFncqOf+ETSXGkKW2yID92tMomTGrXIz0DTm/MFPgbD9MBGY6
3WJDaLW9gr0VnyrhgaiX4Oy4Va2Qel5XDEHpcjyj/jrcxKrfX9EmEuQezQKBgQDE
BKSSUye4DcESW2DC2gswS/01IDs0EcdBkn/Kl0gLAPrTi/ZHYaXiQq11CwQ8rqmp
nbXzz97sAk3drH36AD0pkp7Nv4wyQ1J+VCmMmxvYBq5vBvfyTKBSFYaexJ6titxG
+acyicNpCHsI98HmIQLBvljOSzLqbHqmLLCv0U+OIQKBgQDKYOt7QLOegHOPJ3ml
aTE8gQSw+z4Ap7vI8te5xcIhLWpKvEVHEQadj8NO2ZWLD2pLzf8x71FCkw8BEToT
47wRgnc9Y5kAIohVoOOr7I0l6xmu2wS7XYIsrcCo4AWpirxlKaNSaNf4Dx5y1+JB
DnIolyDhRxRAfBB9aUcUDndDbg==
-----END PRIVATE KEY-----
";

pub const TEST_RSA_PUBLIC_KEY_PEM: &str = "-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAr6v2WLVDoqKqWMKP7yqf
fKBCiLmFn484laYxwkULQlq+j3VcfHk7CUWbWBnRljG/XDRCgeA9EXtZFfo8lCih
ylTsAw3NocieXdvYPKtcJIJMROkwjsX3BeudCLOHxLEUETEHdLpzlEAYc4BGMxbP
NMFqoCk35N9lKr0GzFguwTuAlnNba4tORAKbEc56pbarjPMnEcP4IjtmybvPdlpp
94qcS4MFe5Lb1SwE50rnPZ7yUkLkjGRgceMlLFKHYYNdWlKlfU4RlBgT80mhZK0U
PflVVxlQ3PUbhefTw6wpxw0mUMTaMF8p+sz+JeNrXXAxBQ3e9E8i6xVyVV4E8r3K
owIDAQAB
-----END PUBLIC KEY-----
";
