//! Maps `kit_config::LogFormat` (the logging domain's configuration-level
//! format selector) to `kitlogger_formatter::LogFormat` (the canonical
//! formatter selector owned by `formatter-contract`).
//!
//! Lives inside `kitlogger` — not `kitlogger-formatter` — specifically so
//! `kitlogger-formatter`'s existing, accepted dependency boundary (no
//! dependency on `kit_config`) remains unchanged. See design.md Q5.

use kit_config::LogFormat as ConfigLogFormat;
use kitlogger_formatter::LogFormat as FormatterLogFormat;

/// Maps a `kit_config::LogFormat` to the `kitlogger_formatter::LogFormat`
/// it corresponds to.
///
/// Total: every `kit_config::LogFormat` variant produces exactly one
/// `kitlogger_formatter::LogFormat` variant (FR-001). Deterministic: a pure
/// function with no internal state, so repeated calls with the same input
/// always produce the same output (FR-002).
///
/// | `kit_config::LogFormat` | -> | `kitlogger_formatter::LogFormat` |
/// |---|---|---|
/// | `Json`    | -> | `Json` |
/// | `Text`    | -> | `Text` |
/// | `Pretty`  | -> | `HumanReadable` |
/// | `Compact` | -> | `Logfmt` |
pub fn map_log_format(format: ConfigLogFormat) -> FormatterLogFormat {
    match format {
        ConfigLogFormat::Json => FormatterLogFormat::Json,
        ConfigLogFormat::Text => FormatterLogFormat::Text,
        ConfigLogFormat::Pretty => FormatterLogFormat::HumanReadable,
        ConfigLogFormat::Compact => FormatterLogFormat::Logfmt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_log_format_variant_maps() {
        let cases = [
            (ConfigLogFormat::Json, FormatterLogFormat::Json),
            (ConfigLogFormat::Text, FormatterLogFormat::Text),
            (ConfigLogFormat::Pretty, FormatterLogFormat::HumanReadable),
            (ConfigLogFormat::Compact, FormatterLogFormat::Logfmt),
        ];

        for (input, expected) in cases {
            assert_eq!(map_log_format(input), expected);
        }
    }

    #[test]
    fn mapping_is_deterministic() {
        let variants = [
            ConfigLogFormat::Json,
            ConfigLogFormat::Text,
            ConfigLogFormat::Pretty,
            ConfigLogFormat::Compact,
        ];

        for variant in variants {
            let first = map_log_format(variant);
            let second = map_log_format(variant);
            assert_eq!(first, second);
        }
    }
}
