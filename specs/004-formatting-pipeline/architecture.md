# Architecture Specification: Formatting Pipeline

## Capability Boundary

The Formatting Pipeline capability owns the transformation of canonical LogRecord instances into serialized string representations suitable for consumption by exporters. It owns the Formatter contract, all concrete formatter implementations (Text, JSON, Logfmt), and the formatted output type. It does not own LogRecord creation, validation, transport, or output delivery.

## Domain Boundaries

| Domain | Ownership | Description |
|--------|-----------|-------------|
| Formatter Contract | AS-01 | Trait definition, FormattedRecord output type, formatting pipeline abstraction |
| Text Formatting | AS-02 | Human-readable plain text renderer |
| JSON Formatting | AS-03 | Structured JSON object renderer |
| Logfmt Formatting | AS-04 | Logfmt key=value string renderer |

## Constraints

1. Formatting MUST NOT mutate LogRecord instances (immutable input).
2. Formatting MUST remain independent from exporters, transports, and output destinations.
3. Formatting MUST NOT perform I/O operations.
4. Formatting MUST produce deterministic output for identical inputs.
5. All formatters MUST preserve severity, timestamp, message, and every structured attribute.
6. The Formatter contract MUST be implementable without knowledge of other formatters.

## Decomposition Strategy

The capability is decomposed into four atomic specifications following a hub-and-spoke pattern. The core Formatter Contract (AS-01) defines the shared abstraction that all concrete formatters implement. Each concrete formatter (AS-02, AS-03, AS-04) is an independent specification because each targets a distinct output format with unique rendering rules, escaping requirements, and field ordering semantics.

## Dependency Graph

```text
003-structured-logging-core
        |
      AS-01 (Formatter Contract)
       /    |    \
      /     |     \
AS-02   AS-03   AS-04
(Text)  (JSON)  (Logfmt)
```

## Atomic Specification Candidates

| Key | Name | Responsibility | Dependencies | Ownership Boundary |
|-----|------|----------------|--------------|--------------------|
| AS-01 | Formatter Contract | Define the Formatter trait, FormattedRecord output type, and the formatting pipeline abstraction. | 003-structured-logging-core | Formatter trait, FormattedRecord type, format pipeline dispatch |
| AS-02 | Text Formatter | Implement human-readable plain text formatting of LogRecord with ordered fields and inline attribute rendering. | AS-01 | TextFormatter implementation, default field order for text |
| AS-03 | JSON Formatter | Implement JSON object formatting of LogRecord with all attributes as typed JSON values. | AS-01 | JsonFormatter implementation, JSON escaping and encoding |
| AS-04 | Logfmt Formatter | Implement logfmt key=value formatting of LogRecord with space-separated tokens. | AS-01 | LogFmtFormatter implementation, logfmt encoding rules |

## Expansion Contract

Each candidate becomes one independent top-level SpecKit specification through `expand`. Architecture assigns local candidate keys only; repository specification numbers are allocated during expansion.
