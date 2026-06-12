# Architecture Specification: [CAPABILITY NAME]

## Capability Boundary

[Define what the Capability owns and what remains outside it.]

## Domain Boundaries

[Define bounded domains, concepts, and ownership boundaries.]

## Constraints

[List architectural and interoperability constraints.]

## Decomposition Strategy

[Explain how the Capability is divided into independently evolvable Atomic Specifications.]

## Dependency Graph

```text
[AS-01] -> [AS-02]
```

## Atomic Specification Candidates

| Key | Name | Responsibility | Dependencies | Ownership Boundary |
|-----|------|----------------|--------------|--------------------|
| AS-01 | [Name] | [One responsibility] | None | [Owned concepts/contracts] |

## Expansion Contract

Each candidate becomes one independent top-level SpecKit specification through
`expand`. Architecture assigns local candidate keys only; repository
specification numbers are allocated during expansion.
