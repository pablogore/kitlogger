//! Output Adapter Contracts: the Output Port and dispatch-mechanism
//! registry every log output destination (console, file, and future
//! network/vendor destinations) conforms to.
//!
//! Deliberately independent of `telemetry-adapter-contracts` — that crate
//! is a Telemetry/OTel-provider bounded context (Canonical<->OTel mapping,
//! cross-signal delivery envelope), not a generic Output Port. Depends
//! only on `kitlogger-log-domain`, for `Severity`.
//!
//! See `openspec/changes/014-output-consolidation/design.md` Q1 and Q6.

mod output;
mod registry;

pub use output::{Output, OutputError};
pub use registry::{DispatchOutcome, OutputId, RegistrationError, Registry};
