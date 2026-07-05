# Delta: Exporter Registry

## REMOVED Requirements

### Requirement: Exporter Selection by Name

**Reason**: Superseded by `output-adapter-contracts` (change 014), which registers and dispatches to *all* currently registered outputs simultaneously (ADR-008 §6), not one exporter selected by name at a time. This requirement was never implemented (`rg -rn "ExporterRegistry" crates` — zero matches), was never mentioned in its originating change's (`005-console-exporter`) own `proposal.md`/`design.md`, and that change's own `verify-report.md` already flagged it as out of scope.

### Requirement: Exporter Registration

**Reason**: Same as above. `output-adapter-contracts::FR-002` (Unique Registration) covers registration in the surviving model — by unique identifier, for simultaneous multi-output dispatch, not name-based single selection.

### Requirement: Default Exporter

**Reason**: Same as above. The surviving model (`output-adapter-contracts`) has no single "active exporter" concept to default — `kitlogger` registers a fixed set of outputs at construction (`kitlogger-emission-pipeline::FR-009`, change 015), not a runtime-selectable one with a fallback.
