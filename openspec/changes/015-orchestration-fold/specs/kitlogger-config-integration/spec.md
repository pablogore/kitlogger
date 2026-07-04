# Delta: KITLogger Config Integration

## MODIFIED Requirements

### Requirement: FR-003 Behavioral Integration via the Emission Pipeline

`LoggingConfig`'s behavioral fields (`.enabled`, `.level`, `.sampling`, `.redact`, `.buffering`, `.format`, `.output`) drive real `KITLogger::log`/`log_record` behavior, exactly as specified by the `kitlogger-emission-pipeline` capability (change 015). This supersedes this capability's original restriction ("fields other than those needed for construction-time validity MUST NOT be consulted by any runtime code path") — that restriction existed only as a scope boundary until this phase landed, per its own original wording ("gating any runtime behavior... is folded into Phase 5").

#### Scenario: LoggingConfig fields now gate real behavior

- GIVEN a `KITLogger` constructed from a `LoggingConfig`
- WHEN a log call is made
- THEN `LoggingConfig.enabled`, `.level`, `.sampling`, `.redact`, `.buffering`, and `.format` each observably affect the outcome, per `kitlogger-emission-pipeline`'s requirements

#### Scenario: Construction-time validation is unaffected

- GIVEN an invalid `LoggingConfig` (failing `kit_config`'s `Validation` trait)
- WHEN `KITLogger` construction is attempted
- THEN construction still fails at construction time, exactly as this capability originally specified (FR-002, unchanged by this delta)
