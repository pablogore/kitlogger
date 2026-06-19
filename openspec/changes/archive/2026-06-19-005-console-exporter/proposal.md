# Proposal: Console Exporter

## Intent

Deliver formatted log output to console streams (stdout/stderr) with stream routing, lifecycle management, and flush control. The exporter receives pre-formatted strings from the Formatting Pipeline and owns delivery only — formatting is not its responsibility.

## Scope

### In Scope
- Deliver pre-formatted strings to stdout/stderr
- Level-to-stream routing (errors to stderr, others to stdout)
- Pluggable flush strategies (immediate, batch, on-shutdown)
- Lifecycle management (init, run, flush, shutdown, error recovery)
- Integrate with the existing logging pipeline

### Out of Scope
- Log formatting or record transformation — owned by KIT-006 Formatting Pipeline
- File storage, log rotation, remote transport
- Centralized aggregation, metrics, or trace export

## Capabilities

### New Capabilities
- `console-stream-router`: Routes already-formatted strings to the correct console stream (stdout/stderr) based on log level
- `console-exporter-core`: Core exporter that receives pre-formatted strings, manages lifecycle, and coordinates flush strategies

### Modified Capabilities
- None — this is a new capability orthogonal to existing ones

## Approach

The Console Exporter sits downstream of the Formatting Pipeline. It receives fully formatted strings and writes them to stdout/stderr via the StreamRouter. A lifecycle state machine manages initialization, active output, flush, and shutdown transitions. Flush strategies decouple write timing from the pipeline.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| Pipeline sink | New | Console delivery sink consuming `&str` |
| Configuration | New | Stream routing, flush strategy, lifecycle config |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Stream mixing (stdout vs stderr) | Low | Clear level-to-stream mapping, documented behavior |
| Data loss on shutdown | Low | Flush lifecycle phase guarantees in-flight writes complete |

## Rollback Plan

Remove the console exporter config or switch to a no-op exporter. No data loss — console output is ephemeral.

## Dependencies

- KIT-006 Formatting Pipeline — produces the `&str` input this exporter consumes

## Success Criteria

- [ ] Pre-formatted strings appear on stdout/stderr with correct stream routing
- [ ] Flush strategies (immediate, batch, on-shutdown) work correctly
- [ ] Lifecycle transitions (init → run → flush → shutdown) complete without data loss
- [ ] I/O errors are reported without panicking
- [ ] No regression in existing pipeline throughput
