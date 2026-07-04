# Design: Redaction & Sampling

## Technical Approach

### `kitlogger-redaction`

- `Redactor` holds a `kit_config::RedactionConfig` (fields: `enabled`, `fields: Vec<String>`, default `["password", "token", "secret", "authorization"]`).
- **Matching algorithm (design-level choice, not part of the spec's contract — see `specs/kitlogger-redaction/spec.md` FR-001)**: for this phase, an attribute name is sensitive if it case-insensitively contains any configured field substring — preserves the orphaned original's validated behavior exactly. The spec intentionally does not commit to this algorithm so it can evolve (exact match, regex, glob, metadata-driven) without being a breaking change to FR-001, as long as it still correctly identifies attributes intended to be sensitive.
- Redaction operates on `kitlogger_log_domain::LogRecord`'s `attributes: Vec<LogAttribute>` (each `LogAttribute` pairs a name with a typed `LogAttributeValue`). Because `LogRecord` is immutable (structured-logging-core's own domain contract), redaction produces a **new** `LogRecord` with matching attributes' values replaced by a fixed redaction marker — it does not mutate the input.
- **Redaction marker ownership**: the literal replacement value (`LogAttributeValue::String("**REDACTED**")`, matching the orphaned original) is a `kitlogger-redaction` domain constant for this phase, not a `RedactionConfig` field. `RedactionConfig` decides *which* fields are sensitive; it does not decide *what* they are replaced with. Making the marker configurable later would be a new `RedactionConfig` field addition — not something owed to this change.
- If `config.enabled` is `false`, the record is returned unchanged.

### `kitlogger-sampling`

- `Sampler` holds a `kit_config::SamplingConfig` (fields: `enabled`, `strategy: SamplingStrategy`, `rate: f64`, `n: u32`, `max_events_per_second: u32`).
- `should_sample()` implements all four `SamplingStrategy` variants, matching the orphaned original:
  - `None` — always `true`.
  - `Probabilistic` — random draw against `rate`.
  - `EveryNth` — deterministic counter, `true` every `n`th call.
  - `RateLimit` — sliding one-second window against `max_events_per_second`.
- `Sampler` takes no `LogRecord` parameter — sampling is a volume-control decision independent of record content, matching the orphaned original exactly.
- If `config.enabled` is `false`, `should_sample()` always returns `true` (sampling disabled means nothing is dropped).
- **Time source for `RateLimit`**: the sliding one-second window MUST read time through an injectable clock, not `std::time::Instant::now()` directly — otherwise the window is untestable without real sleeps. `kitlogger-log-domain` already owns a canonical `Clock` trait (with `UtcClock`/`FakeClock` implementations); `kitlogger-sampling` consumes that existing abstraction rather than defining a competing one (ADR-010). This is the one reason `kitlogger-sampling` depends on `kitlogger-log-domain` — not for `LogRecord`, and not in tension with FR-006's content independence, which is about record *content*, not about reusing an existing time-source type.

## Architecture Decisions

| Decision | Choice | Rejected | Rationale |
|----------|--------|----------|-----------|
| Crate boundary | Two new crates, `kitlogger-redaction` and `kitlogger-sampling` | One shared crate; inline in `kitlogger` | Matches the existing crate-per-domain pattern (`kitlogger-formatter`, `console-exporter`, `kitlogger-macros`); ADR-010 treats them as two distinct concepts with two distinct owners |
| Redaction mechanism | Build a new `LogRecord` with replaced attribute values | Mutate `LogRecord` in place | `LogRecord` is immutable by domain contract; mutation is not an option, not a style preference |
| Sampler record dependency | None — `Sampler` depends on `kit_config` and, for `Clock` only, `kitlogger-log-domain`; never on `LogRecord`/content | Depend on `kitlogger-log-domain` for future content-based sampling | No content-based sampling requirement exists today; adding a content dependency speculatively would violate the "don't design for hypothetical requirements" principle. The `Clock` dependency is different in kind — it consumes an existing, canonical utility type, it does not touch record content |
| Redaction matching algorithm | Case-insensitive substring (this phase) | Commit the algorithm to the spec | The orphaned original's validated behavior, but kept out of FR-001's contract text so the algorithm (exact/regex/glob/metadata) can evolve without a breaking spec change |
| Time source for `RateLimit` | Injectable clock, reusing `kitlogger_log_domain::Clock` | Hardcode `Instant::now()`; define a new local `Clock` trait | Hardcoding makes the sliding window untestable without real sleeps; defining a competing `Clock` trait would violate ADR-010 given one already exists and is canonical |
| Randomness source for `Probabilistic` | `fastrand` | `rand`; a hand-rolled PRNG | Matches the orphaned original's already-validated choice; `fastrand` is a minimal, dependency-light crate consistent with `kitlogger-sampling`'s leaf-crate footprint |
| Redaction marker value | `kitlogger-redaction` domain constant | Expose via `RedactionConfig` | `RedactionConfig` owns *which* fields are sensitive, not *what* replaces them; making the marker configurable is a future, separate field addition, not part of this phase |

## Interfaces / Contracts

Not specified here beyond what the spec deltas require (`Redactor`, `Sampler` as named concepts) — exact method signatures are an implementation decision for the apply phase, not this design document.

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit (`kitlogger-redaction`) | This phase's chosen matching algorithm (case-insensitive substring) against configured fields | Table-driven test over field name variants: exact match (`password` ~ `password`), case-insensitive match (`Password` ~ `password`), substring match (`auth_token` ~ `token`) |
| Unit (`kitlogger-redaction`) | Non-matching attributes are untouched; matching ones are replaced | Assert full `LogRecord` equality except redacted fields |
| Unit (`kitlogger-redaction`) | Input `LogRecord` is not mutated (immutability preserved) | Assert original record reference/value unchanged after redaction call |
| Unit (`kitlogger-redaction`) | `enabled = false` returns the record unchanged | Direct assertion |
| Unit (`kitlogger-sampling`) | Each of the four strategies produces the documented decision pattern | `None` always true; `EveryNth` deterministic sequence; `Probabilistic` — the concrete test methodology (this phase: 10,000 draws, asserting the observed rate is within a tolerance band around the configured rate) is an implementation/test decision, not part of FR-003's contract; `RateLimit` window behavior using `kitlogger_log_domain::FakeClock` (or equivalent), never real sleeps |
| Unit (`kitlogger-sampling`) | `enabled = false` always samples | Direct assertion |
| Unit (`kitlogger-sampling`) | `RateLimit` reads time through the injectable clock, never the system clock directly | Code-level check (no direct `Instant::now()`/`SystemTime::now()` call outside the clock abstraction) |

## Migration / Rollout

No migration required — both crates are additive with zero consumers in this change. Rollback: remove both crate directories and their workspace member entries.

## Composability

Both crates are fully reusable and do not assume they will be used exclusively by `KITLogger` — nothing in either crate's design references `kitlogger` or the specific pipeline sequencing ADR-008 defines. A different consumer (a future pipeline, a test harness, another host application) can depend on either crate independently.

## Open Questions

- None blocking this change. Wiring these two capabilities into `KITLogger`'s emission path is explicitly deferred to Phase 5 (Orchestration Fold), where the full filter → sample → redact → buffer → format → dispatch sequencing is designed as one piece.
