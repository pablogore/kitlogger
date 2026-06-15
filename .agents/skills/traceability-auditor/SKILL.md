---
name: traceability-auditor
description: Verifies complete Requirement → Success Criterion → Task → Code → Test traceability. Detects orphaned requirements, tasks, code, tests, contracts, and entities.
-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------

# Traceability Auditor

## Purpose

Perform a read-only traceability audit across architecture, planning, implementation, and testing artifacts.

This skill never modifies files.

Its sole responsibility is to verify traceability completeness and identify traceability gaps.

## Scope

Audit:

* spec.md
* research.md
* data-model.md
* contracts/**
* plan.md
* tasks.md

Implementation:

* src/**
* tests/**
* examples/**
* benches/**

## Traceability Model

The required chain is:

Requirement
→ Success Criterion
→ Task
→ Code
→ Test

Every implementation artifact must be traceable.

Every architectural artifact must be implemented.

Missing links are prohibited.

## Audit Rules

### Requirement Coverage

Verify:

* Every requirement exists in spec.md
* Every requirement maps to at least one success criterion
* Every requirement maps to at least one task

Failure:

TRACEABILITY GAP

### Success Criterion Coverage

Verify:

* Every success criterion maps to at least one task
* Every success criterion is testable
* Every success criterion maps to at least one test

Failure:

TRACEABILITY GAP

### Task Coverage

Verify:

* Every task maps to a requirement
* Every task maps to a success criterion
* Every implemented task maps to code
* Every implemented task maps to tests

Failure:

TRACEABILITY GAP

### Code Coverage

Verify:

* Every implementation file maps to at least one task
* Every public API maps to a contract
* Every entity maps to data-model.md

Failure:

TRACEABILITY GAP

### Test Coverage

Verify:

* Every test maps to a requirement
* Every test maps to a success criterion
* Every success criterion has at least one validating test

Failure:

TRACEABILITY GAP

### Contract Coverage

Verify:

* Every public API has a contract definition
* Every contract maps to implementation
* Every contract maps to requirements

Failure:

TRACEABILITY GAP

### Data Model Coverage

Verify:

* Every entity is defined in data-model.md
* Every entity used in implementation maps to a model definition
* Every model definition is implemented

Failure:

TRACEABILITY GAP

## Gap Categories

TG-XXX

Types:

* Requirement Gap
* Success Criterion Gap
* Task Gap
* Code Gap
* Test Gap
* Contract Gap
* Data Model Gap

## Gap Format

TG-001

Type:
Task Gap

Severity:
Critical | Major | Minor

Requirement:
REQ-001

Success Criterion:
SC-001

Task:
T-007

Description:
Missing implementation traceability.

Expected:
Task must map to implementation and tests.

Observed:
Task exists but no implementation located.

Impact:
Requirement cannot be verified.

Recommended Resolution:
Implement task and add validating tests.

## Required Evidence

The auditor must provide evidence.

Examples:

TRACEABILITY EVIDENCE

REQ-001
SC-001
T007
src/http_propagation.rs
tests/trace_context_test.rs

REQ-002
SC-002
T011
src/correlation.rs
tests/correlation_test.rs

The auditor may not claim traceability without evidence.

## Coverage Metrics

Calculate:

Requirements Covered:
X/Y

Success Criteria Covered:
X/Y

Tasks Covered:
X/Y

Code Files Traced:
X/Y

Tests Traced:
X/Y

Contracts Traced:
X/Y

Entities Traced:
X/Y

## Output Format

Traceability Audit Report

Specification: <specification>

Coverage Metrics

Requirements Covered:
X/Y

Success Criteria Covered:
X/Y

Tasks Covered:
X/Y

Code Files Traced:
X/Y

Tests Traced:
X/Y

Contracts Traced:
X/Y

Entities Traced:
X/Y

Traceability Evidence

REQ-...
...

Traceability Gaps

TG-001
...

TG-002
...

Summary

Critical Gaps:
N

Major Gaps:
N

Minor Gaps:
N

Verdict:

COMPLETE
PARTIAL
INCOMPLETE

## Verdict Rules

COMPLETE

Issued only when:

* No traceability gaps exist
* All requirements traced
* All success criteria traced
* All tasks traced
* All code traced
* All tests traced
* All contracts traced
* All entities traced

PARTIAL

Issued when:

* Some traceability gaps exist
* Coverage is incomplete

INCOMPLETE

Issued when:

* Critical gaps exist
* Requirements cannot be verified
* Tests cannot validate requirements

## Absolute Rule

Traceability must be proven.

The auditor must never infer traceability.

The auditor must provide evidence.

No evidence means no traceability.
