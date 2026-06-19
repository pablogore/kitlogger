# ADR-001: Formatter vs Exporter Ownership

## Status

Accepted

## Context

The Console Exporter proposal initially included formatting responsibilities (JsonExporter, HumanReadableExporter, TxtExporter) inside the exporter boundary. This created ownership overlap with KIT-006 Formatting Pipeline, which already owns output representation.

Two options existed:
- **A**: Console Exporter owns both formatting and delivery (receives `LogRecord`, produces output text)
- **B**: Formatting Pipeline owns representation, Console Exporter owns delivery only (receives `&str`)

## Decision

**Option B.** Formatting and delivery are independently evolvable capabilities.

```
LogRecord → Formatter (KIT-006) → &str → Exporter (KIT-009) → Destination
```

## Consequences

**Positive:**
- KIT-006 owns representation; KIT-009 owns delivery — clean capability boundaries
- Future exporters (CloudWatch, Loki, OTLP, Kafka, File) consume `&str`, no formatting duplication
- Formatters swap without touching delivery; exporters swap without touching formatting

**Negative:**
- The pipeline has an extra hop: `LogRecord → String → write`. Overhead is negligible (a pointer copy).

## Interface Contract

```rust
// KIT-006 produces this:
trait Formatter {
    fn format(&self, record: &LogRecord) -> Result<String, FormatError>;
}

// KIT-009 consumes this:
trait ConsoleExporter {
    fn export(&self, msg: &str, severity: Severity) -> Result<(), ExportError>;
}
```
