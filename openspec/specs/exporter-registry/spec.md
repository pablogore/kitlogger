# Exporter Registry Specification

## Status: Superseded (change 018, Migration Plan Phase 8)

Every requirement this capability once specified is removed. It was never implemented (`rg -rn "ExporterRegistry" crates` — zero matches, confirmed at supersession time), was never mentioned in its originating change's (`005-console-exporter`) own `proposal.md`/`design.md`, and that change's own `verify-report.md` already flagged it as out of scope.

Its model — select one exporter by string name, with a default fallback — predates and conflicts with ADR-008 §6's actual committed architecture: multiple outputs (console AND file) registered and dispatched to *simultaneously*, not one selected exporter at a time. `output-adapter-contracts` (change 014) implements the model actually adopted. This file is kept, empty of active requirements, as a record that this capability was proposed and superseded rather than silently erased — see `openspec/changes/archive/2026-07-05-018-crate-removal/` for the full removal delta and reasoning.

## Purpose

Historical only. Superseded by `output-adapter-contracts`'s dispatch-to-all model.

## Requirements

None. All requirements below were removed by change 018:

- **Exporter Selection by Name** — superseded by `output-adapter-contracts::FR-003` (Dispatch to All Registered Outputs), which dispatches to every registered output simultaneously rather than selecting one by name.
- **Exporter Registration** — superseded by `output-adapter-contracts::FR-002` (Unique Registration), which registers by unique identifier for simultaneous multi-output dispatch, not name-based single selection.
- **Default Exporter** — no counterpart in the surviving model. `kitlogger` registers a fixed set of outputs at construction (`kitlogger-emission-pipeline::FR-009`), not a runtime-selectable one with a fallback.
