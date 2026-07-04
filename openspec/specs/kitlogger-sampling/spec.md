# KITLogger Sampling Specification

## Purpose

Define the behavioral contract for deciding whether a given emission should proceed, based on `LoggingConfig.sampling` (materialized and validated by kit-config). This spec covers what the system MUST express — not how it is implemented. Sampling decisions MUST NOT depend on the content of the record being considered. `SamplingConfig` is assumed to have already passed `kit-config`'s own validation (e.g. `EveryNth` requiring `n > 0`) before reaching this capability — this capability does not re-validate it.

## Requirements

### Requirement: FR-001 None Strategy

When `SamplingConfig.strategy` is `None`, every sampling decision MUST be `true`.

#### Scenario: None strategy always proceeds

- GIVEN a `SamplingConfig` with `strategy = None`
- WHEN a sampling decision is requested any number of times
- THEN every decision is `true`

### Requirement: FR-002 EveryNth Strategy

When `SamplingConfig.strategy` is `EveryNth`, exactly one out of every `n` consecutive decisions MUST be `true`, deterministically.

#### Scenario: Deterministic sequence over n calls

- GIVEN a `SamplingConfig` with `strategy = EveryNth` and `n = 3`
- WHEN a sampling decision is requested 6 times consecutively
- THEN exactly 2 of the 6 decisions are `true`
- AND the pattern is deterministic (repeating the same 6 calls produces the same sequence)

### Requirement: FR-003 Probabilistic Strategy

When `SamplingConfig.strategy` is `Probabilistic`, each decision MUST be `true` with probability equal to `SamplingConfig.rate`, independent of prior decisions.

#### Scenario: Observed rate approximates the configured rate

- GIVEN a `SamplingConfig` with `strategy = Probabilistic` and `rate = 0.3`
- WHEN a large number of independent sampling decisions are requested
- THEN the observed proportion of `true` decisions approximates `0.3`, converging as the number of decisions grows
- AND the exact sample size and acceptable deviation are a test-implementation concern, not part of this requirement

### Requirement: FR-004 RateLimit Strategy

When `SamplingConfig.strategy` is `RateLimit`, decisions MUST be `true` up to `SamplingConfig.max_events_per_second` times within any one-second window, and `false` for any additional decision requested within that same window.

#### Scenario: Decisions within the limit proceed

- GIVEN a `SamplingConfig` with `strategy = RateLimit` and `max_events_per_second = 5`
- WHEN 5 sampling decisions are requested within the same one-second window
- THEN all 5 decisions are `true`

#### Scenario: Decisions beyond the limit are rejected within the same window

- GIVEN a `SamplingConfig` with `strategy = RateLimit` and `max_events_per_second = 5`
- WHEN a 6th sampling decision is requested within the same one-second window
- THEN the 6th decision is `false`

#### Scenario: The limit resets in the next window

- GIVEN a `SamplingConfig` with `strategy = RateLimit` and `max_events_per_second = 5`, with the limit already reached in the current window
- WHEN a sampling decision is requested after the window has elapsed
- THEN the decision is `true`

### Requirement: FR-005 Disabled Config Passthrough

When `SamplingConfig.enabled` is `false`, every sampling decision MUST be `true`, regardless of `strategy`.

#### Scenario: Disabled config never drops an emission

- GIVEN a `SamplingConfig` with `enabled = false` and `strategy = RateLimit` with `max_events_per_second = 1`
- WHEN 10 sampling decisions are requested within the same one-second window
- THEN all 10 decisions are `true`

### Requirement: FR-006 Content Independence

A sampling decision MUST NOT depend on, or require, the content of the record being considered.

#### Scenario: Sampler operates without a record

- GIVEN a `Sampler` constructed from a `SamplingConfig`
- WHEN a sampling decision is requested
- THEN no `LogRecord` or record content is required as input to make the decision

### Requirement: FR-007 Injectable Time Source

The passage of time used by the `RateLimit` strategy's one-second window MUST be sourced through an injectable clock abstraction, not read directly from the system clock. This MUST allow deterministic testing of window behavior without real time delays.

#### Scenario: Window behavior is testable without real delays

- GIVEN a `Sampler` constructed with a controllable time source
- WHEN time is advanced programmatically past the one-second window boundary
- THEN `RateLimit`'s decision behavior reflects the new window without the test needing to sleep for a real second
