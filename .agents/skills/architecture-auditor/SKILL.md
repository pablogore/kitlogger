---
name: architecture-auditor
description: Performs read-only architecture audits. Detects architectural inconsistencies, contract violations, model drift, requirement gaps, and implementation divergence. Produces findings but never modifies artifacts.
------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

# Architecture Auditor

## Purpose

Perform a read-only architecture audit of a specification and its implementation.

This skill never modifies files.

Its sole responsibility is to identify architectural findings.

## Scope

Audit:

* spec.md
* research.md
* data-model.md
* contracts/**
* plan.md
* tasks.md
* implementation

Implementation includes:

* src/**
* tests/**
* examples/**
* benches/**

## Audit Rules

Architecture is the source of truth.

Code is never the source of truth.

Implementation must conform to architecture.

Architecture must never be modified during audit.

## Validation Areas

### Requirements Consistency

Verify:

* Requirements are complete
* Requirements are testable
* Requirements map to success criteria
* No contradictory requirements exist

### Data Model Consistency

Verify:

* All entities exist
* All entities are defined once
* No conflicting definitions exist
* Relationships are consistent

### Contract Consistency

Verify:

* All public APIs have contracts
* Contracts match data model
* Contracts match requirements
* No undocumented APIs exist

### Plan Consistency

Verify:

* Plan aligns with requirements
* Plan aligns with contracts
* Plan aligns with data model

### Task Consistency

Verify:

* Tasks map to requirements
* Tasks map to success criteria
* Tasks are implementable

### Implementation Alignment

Verify:

* Implementation matches contracts
* Implementation matches data model
* Implementation matches requirements
* No architectural drift exists

## Finding Categories

F-XXX

Types:

* Requirement Finding
* Contract Finding
* Data Model Finding
* Plan Finding
* Task Finding
* Implementation Finding
* Traceability Finding

## Finding Format

F-001

Type:
Contract Finding

Severity:
Critical | Major | Minor

Artifact:
contracts/example.md

Description:
Clear description of the issue.

Expected:
What architecture requires.

Observed:
What was found.

Impact:
Why this matters.

Recommended Resolution:
Concrete remediation.

## Output Format

Architecture Audit Report

Specification: <spec>

Findings:
F-001
...

F-002
...

Summary

Critical Findings:
N

Major Findings:
N

Minor Findings:
N

Verdict:

COMPLIANT
or
NON-COMPLIANT

## Compliance Rule

COMPLIANT may only be issued when:

* No findings exist
* No architectural drift exists
* No contract violations exist
* No requirement inconsistencies exist

Otherwise:

NON-COMPLIANT

## Absolute Rule

This skill is read-only.

It may never modify architecture artifacts.

It may only identify findings.
