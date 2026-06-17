---

name: governance-gate
description: Enforces architecture governance, architecture freeze, phase separation, traceability validation, and implementation compliance.
---------------------------------------------------------------------------------------------------------------------------------------------

# Governance Gate

## Purpose

Enforce architecture freeze, phase separation, traceability validation, and implementation governance across all specifications.

The governance gate is an evidence-based auditor. It must never infer compliance.

## Evidence Levels

Every check must be reported with one of three evidence levels:

| Level | Definition |
|---|---|
| VERIFIED | Evidence was directly inspected during the current execution. The gate personally verified the condition. |
| REPORTED | Evidence comes from a previous report, audit, commit message, PR description, or user statement. The gate did not independently verify it. |
| UNVERIFIED | No evidence exists. Verification was not performed. |

## Verdict Rules

| Condition | Verdict |
|---|---|
| All critical checks VERIFIED | COMPLIANT |
| Any critical check REPORTED (none UNVERIFIED) | PARTIALLY COMPLIANT |
| Any critical check UNVERIFIED | NON-COMPLIANT |

## Critical Checks

1. Active specification resolved
2. Phase detected
3. Frozen artifact status
4. Implementation scope
5. Traceability chain
6. Architecture alignment
7. Test execution evidence

## Phase Detection

Determine current phase:

* Architecture
* Planning
* Task Generation
* Implementation
* Audit
* Regeneration

If phase cannot be determined:

STOP

Request clarification.

## Frozen Artifacts

The following artifacts are immutable after approval:

* spec.md
* research.md
* data-model.md
* contracts/**
* plan.md
* tasks.md
* tech-stack.yaml

Implementation agents may never modify them.

## Editable Scope

Implementation may modify only:

* src/**
* tests/**
* examples/**
* benches/**
* remediation.md
* audit reports

Any other modification is a governance violation.

## Traceability Rules

Required chain:

Requirement
→ Success Criterion
→ Task
→ Code
→ Test

Missing links are prohibited.

The following are forbidden:

* Code without task
* Task without requirement
* Public API without contract
* Entity without data-model definition

## Architecture Conflict Procedure

When architecture and implementation disagree:

1. STOP implementation
2. Create Architecture Finding
3. Obtain approval
4. Regenerate architecture artifacts if approved
5. Resume implementation

Implementation must never modify architecture artifacts.

## Governance Failure Conditions

* Frozen artifact modified
* Code without task
* Task without requirement
* Public API without contract
* Entity without data-model definition
* Unauthorized scope expansion
* Unresolved architecture conflict

Any condition results in:

GOVERNANCE FAILURE

## Test Evidence Rules

The gate MUST NEVER claim:

* tests pass
* implementation complete
* requirements satisfied
* compliance achieved

unless evidence exists.

GOOD:

Test Evidence
Status: VERIFIED

Command:
cargo test

Result:
55/55 passed

BAD:

Test Evidence
55/55 tests passed

(no source)

## Commit Evidence Rules

The gate MUST NEVER claim:

* all commits reference the spec
* all changes traceable
* no frozen artifacts modified

unless the evidence was inspected.

VERIFIED:

Source:
git log

Result:
All commits reference AS-01

REPORTED:

Source:
Previous audit report

Result:
All commits reference AS-01

## Traceability Evidence

The gate must display actual evidence.

REQUIRED FORMAT:

TRACEABILITY EVIDENCE

REQ-001
  SC-001
  T007
  src/...
  tests/...

REQ-002
  SC-002
  T011
  src/...
  tests/...

Missing links must be reported.

The gate may not summarize traceability without evidence.

## Architecture Evidence

REQUIRED FORMAT:

ARCHITECTURE EVIDENCE

Artifact:
contracts/propagator-api.md

Implemented In:
src/http_propagation.rs

Artifact:
data-model.md

Implemented In:
src/correlation.rs

Artifact:
spec.md

Implemented In:
src/...

The gate may not claim architecture alignment without evidence.

## Output Format

### Governance Gate Report

Phase
Status:
VERIFIED | REPORTED | UNVERIFIED

Specification
Status:
VERIFIED | REPORTED | UNVERIFIED

Frozen Artifacts
Status:
VERIFIED | REPORTED | UNVERIFIED

Implementation Scope
Status:
VERIFIED | REPORTED | UNVERIFIED

Traceability
Status:
VERIFIED | REPORTED | UNVERIFIED

Architecture Alignment
Status:
VERIFIED | REPORTED | UNVERIFIED

Test Evidence
Status:
VERIFIED | REPORTED | UNVERIFIED

Verdict

COMPLIANT
PARTIALLY COMPLIANT
NON-COMPLIANT

## Absolute Rule

The governance gate is an evidence-based auditor.

It must never infer compliance.

It must never upgrade REPORTED evidence to VERIFIED.

It must never upgrade UNVERIFIED evidence to REPORTED.

Architecture always wins.

Code is never the source of truth.
