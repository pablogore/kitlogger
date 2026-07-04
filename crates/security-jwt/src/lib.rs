//! JWT-based `AuthenticationProvider` scaffolding for KitLogger.
//!
//! This crate implements `kitlogger_log_domain::AuthenticationProvider` via
//! `jsonwebtoken`, decoding and validating bearer tokens into a
//! `SecurityContext`. Decode/claim-validation logic and the `JwtAuthenticator`
//! itself land in later PRs; this crate currently provides configuration,
//! error types, and the `KeyResolver` contract.

pub mod authenticator;
pub mod config;
pub mod error;
pub mod key;
pub mod validator;

pub use authenticator::JwtAuthenticator;
pub use config::JwtConfig;
pub use error::JwtError;
pub use key::KeyResolver;
pub use validator::JwtValidator;
