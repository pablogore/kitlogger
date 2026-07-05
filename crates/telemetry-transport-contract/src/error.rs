use serde::{Deserialize, Serialize};
use telemetry_types::BackpressureSignal;

/// Error type for transport operations.
///
/// This enum represents various errors that can occur during telemetry transport.
/// It is non-exhaustive to allow concrete transport implementations to add
/// their own error variants without breaking changes.
///
/// Does not derive `PartialEq`/`Eq`: `telemetry_types::BackpressureSignal` (used
/// by the `Backpressure` variant) does not implement either, unlike the local
/// `BackpressureSignal` this crate used to define — a necessary consequence of
/// repointing to the canonical type, not an independent decision.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportError {
    /// The operation timed out.
    Timeout,

    /// The transport is currently unavailable.
    Unavailable,

    /// The transport is experiencing backpressure.
    ///
    /// This variant contains a `BackpressureSignal` with information
    /// about when to retry the operation.
    Backpressure(BackpressureSignal),

    /// The payload is too large for the transport.
    PayloadTooLarge,

    /// The transport protocol is not supported.
    UnsupportedTransport,
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::Timeout => write!(f, "transport timeout"),
            TransportError::Unavailable => write!(f, "transport unavailable"),
            TransportError::Backpressure(_) => write!(f, "transport backpressure"),
            TransportError::PayloadTooLarge => write!(f, "payload too large"),
            TransportError::UnsupportedTransport => write!(f, "unsupported transport"),
        }
    }
}

impl std::error::Error for TransportError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Regression test for the `telemetry_types::BackpressureSignal` repoint:
    /// `TransportError::Backpressure` must still construct, round-trip
    /// through serde, `Display`, and satisfy `std::error::Error` with the
    /// canonical type in place of the crate's former local one.
    #[test]
    fn backpressure_variant_survives_the_canonical_type_repoint() {
        let mut attributes = HashMap::new();
        attributes.insert("reason".to_string(), "rate_limited".to_string());
        let error = TransportError::Backpressure(BackpressureSignal {
            retry_after: Some(30),
            attributes,
        });

        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: TransportError = serde_json::from_str(&serialized).unwrap();
        match deserialized {
            TransportError::Backpressure(signal) => {
                assert_eq!(signal.retry_after, Some(30));
                assert_eq!(
                    signal.attributes.get("reason"),
                    Some(&"rate_limited".to_string())
                );
            }
            other => panic!("expected Backpressure, got {other:?}"),
        }

        assert_eq!(error.to_string(), "transport backpressure");
        let _: &dyn std::error::Error = &error;
    }
}
