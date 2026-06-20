# Proposal: Formatting Pipeline (KIT-006)

## Intent

`LogRecord → String` conversion does not exist. The domain model (`LogRecord`,
`LogContext`, `LogAttribute`, `Logger`, `LoggerFactory`) is complete and the
`ConsoleExporter` already consumes a pre-formatted `&str`, but nothing produces
that string. ADR-001 designed this stage (KIT-006) and was never built. Without
it the pipeline is broken end-to-end: records can be created and an exporter can
write, but the two cannot connect. This change closes exactly that gap.

## Scope

### In Scope

- New crate `kitlogger-formatter`.
- `Formatter` trait: `format(&self, record: &LogRecord, context: Option<&LogContext>) -> Result<String, FormatError>`.
- 4 formatters: `JsonFormatter`, `HumanReadableFormatter`, `TextFormatter`, `LogfmtFormatter`.
- `LogFormat` enum: `Json | HumanReadable | Text | Logfmt`.
- `FormatError` type.
- Pipeline wiring in the top-level `kitlogger` adapter: `LogRecord → Formatter → &str → ConsoleExporter`.

### Out of Scope

- Dead code cleanup of `telemetry-transport-contract/src/formatter.rs` (separate change).
- Exporters other than `ConsoleExporter` (File, OTLP, Loki, etc.).
- Async / buffered / batched formatting.
- Adding a `logger_name` field to `LogRecord` (see Risks).

## Capabilities

### New Capabilities

- `log-formatting`: Convert a `LogRecord` (plus optional `LogContext`) into a destination-agnostic `String` across 4 selectable formats.

### Modified Capabilities

- None.

## Approach

`kitlogger-formatter` defines a `Formatter` trait and 4 implementations. Each is
a pure, stateless `LogRecord (+ optional LogContext) → String` transform.
`LogContext` and `LogRecord` stay separate concerns: context is an optional
co-input, never copied into the record.

- **JsonFormatter**: structured JSON object (`ts`, `level`, `msg`, attributes, context).
- **HumanReadableFormatter**: human-friendly `timestamp level message` plus attributes (replaces the old PrettyFormatter intent).
- **TextFormatter**: simple `[LEVEL] message`, no timestamp.
- **LogfmtFormatter**: `ts= level= msg= key=value ...`; `Array` values serialized as inline JSON (`tags=["api","auth"]`), no silent data loss.

The top-level `kitlogger` adapter selects a formatter via `LogFormat`, formats
the record, and hands the resulting `&str` to `ConsoleExporter::export(msg, severity)`.

`FormatError` covers serialization failures (e.g. `serde_json` errors on JSON /
logfmt arrays) and any value-rendering failure. `Timestamp` values render as
RFC3339 (UTC).

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `crates/kitlogger-formatter/` | New | New crate: trait, 4 formatters, `LogFormat`, `FormatError` |
| top-level `kitlogger` adapter | Modified/New | Wires `LogRecord → Formatter → &str → ConsoleExporter` |
| `Cargo.toml` (workspace) | Modified | Register new crate member |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `LogRecord` has no `logger_name` field; formats referencing logger name cannot read it from the record | High | Source logger name from `LogContext` attributes when present; otherwise omit. Do not add a field to `LogRecord` in this change |
| `LogContext` is optional; callers may omit it | High | Treat `None` as "no enrichment"; formats degrade gracefully (skip context fields) |
| `LogAttributeValue::Timestamp` rendering ambiguity | Med | Fixed policy: RFC3339 UTC for all formats |
| ADR-001 trait signature omits `context` | Low | This change evolves the ADR contract to add `context: Option<&LogContext>`; document the evolution in the spec |

## Rollback Plan

The change is purely additive (new crate + adapter wiring). To roll back: remove
`crates/kitlogger-formatter`, revert its workspace member entry, and revert the
adapter wiring commit. No domain or exporter types change, so no migration.

## Dependencies

- `kitlogger-log-domain` (only domain dependency).
- `serde_json` (JSON output + inline-JSON array serialization for logfmt).
- ADR-001: Formatter vs Exporter Ownership (Option B) — establishes this stage.

## Success Criteria

- [ ] `kitlogger-formatter` crate compiles and is a workspace member.
- [ ] All 4 formatters implement `Formatter` and produce documented output.
- [ ] `LogFormat` enum selects the matching formatter.
- [ ] `FormatError` surfaces serialization/rendering failures (no panics).
- [ ] Top-level adapter formats a `LogRecord` and delivers via `ConsoleExporter`.
- [ ] Logfmt arrays serialize as inline JSON; timestamps render RFC3339 UTC.
