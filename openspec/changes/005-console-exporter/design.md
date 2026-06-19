# Design: Console Exporter

## Technical Approach

New `console-exporter` crate. ConsoleExporter receives already-formatted `&str` from the Formatting Pipeline — it performs zero formatting. StreamRouter owns stdout/stderr I/O and level-to-stream routing. FlushStrategy decouples write timing. LifecycleStateMachine manages init→run→flush→shutdown transitions.

## Architecture Decisions

| Decision | Options | Choice & Rationale |
|----------|---------|-------------------|
| **Formatting ownership** | (a) Exporter formats (b) Exporter receives `&str` | **(b)** Formatting belongs to KIT-006. Exporter is a pure I/O sink. Violating this would duplicate concerns and couple the pipeline to specific formats |
| **New crate vs. add to existing** | (a) `telemetry-transport-contract` (b) New crate | **(b)** Different domain context, avoids entanglement |
| **Sync vs. async** | (a) `async fn` (b) Sync core | **(b)** Console I/O is synchronous. Wrap via `spawn_blocking` if needed |
| **Flush strategy** | (a) Always flush (b) Configurable strategies | **(b)** Immediate for correctness, Batch for throughput, OnShutdown for batching without timer |

## Data Flow

```
Formatting Pipeline ──→ ConsoleExporter::export(&str, Severity)
                                │
                                ▼
                      FlushStrategy (immediate | batch | on_shutdown)
                                │
                                ▼
                      StreamRouter::write(&str, Severity)
                           │          │
                      stdout/       stderr/
                      (DEBUG+       (WARN+
                       INFO+        ERROR+
                       TRACE)       FATAL)
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `Cargo.toml` | Modify | Add `crates/console-exporter` to workspace members |
| `crates/console-exporter/Cargo.toml` | Create | Deps: `kitlogger-log-domain` only — no serde |
| `crates/console-exporter/src/lib.rs` | Create | Module tree and public re-exports |
| `crates/console-exporter/src/error.rs` | Create | `ExportError` enum (I/O, lifecycle, flush variants) |
| `crates/console-exporter/src/exporter.rs` | Create | `ConsoleExporter`: `fn export(&self, &str, Severity)` |
| `crates/console-exporter/src/stream_router.rs` | Create | Owns stdout/stderr, writes `&str` by level mapping |
| `crates/console-exporter/src/lifecycle.rs` | Create | State machine: Uninitialized→Running→Flushing→Shutdown |
| `crates/console-exporter/src/flush.rs` | Create | `FlushStrategy` trait + Immediate, OnShutdown, Batch impls |

## Interfaces / Contracts

```rust
// exporter.rs — receives pre-formatted strings, no formatting
pub trait ConsoleExporter: Send + Sync {
    fn export(&self, msg: &str, severity: Severity) -> Result<(), ExportError>;
    fn flush(&self) -> Result<(), ExportError>;
    fn shutdown(&self) -> Result<(), ExportError>;
}

// stream_router.rs — owns I/O, writes &str to the correct stream
pub struct StreamRouter { .. }
impl StreamRouter {
    pub fn write(&self, msg: &str, severity: Severity) -> Result<(), ExportError>;
    pub fn set_mapping(&mut self, mapping: LevelStreamMapping);
    pub fn set_writers(stdout: Box<dyn Write>, stderr: Box<dyn Write>);
}

// lifecycle.rs — state machine
pub enum LifecycleState { Uninitialized, Running, Flushing, Shutdown, Error }
pub struct LifecycleStateMachine { .. }
impl LifecycleStateMachine {
    pub fn transition_to(&mut self, target: LifecycleState) -> Result<(), ExportError>;
    pub fn current(&self) -> LifecycleState;
}

// flush.rs — pluggable flush strategies
pub trait FlushStrategy: Send + Sync {
    fn should_flush(&self, write_count: usize) -> bool;
    fn on_shutdown(&self) -> bool;
}
pub struct ImmediateFlush;
pub struct OnShutdownFlush;
pub struct BatchFlush { threshold: usize, interval: Duration }
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | StreamRouter level mapping | Default and custom mappings resolve to correct stream |
| Unit | Lifecycle transitions | All valid + invalid transitions, delivery blocked after shutdown |
| Unit | Flush strategies | Immediate flushes every write, Batch flushes at threshold |
| Integration | Exporter + Router + Lifecycle | Export string → assert output on `Vec<u8>` writers |

## Migration / Rollout

No migration required. Console exporter is additive — crate is standalone. Integration with the pipeline and Formatting Pipeline is deferred to a follow-up change.

## Open Questions

- [ ] StreamRouter: accept generic `Write` impls or wrap `io::Stdout`/`io::Stderr`? Decision: accept `Box<dyn Write>` for testability, default to `io::stdout()`/`io::stderr()`.
- [ ] BatchFlush timer: should the batch flush run on a background thread or be triggered by the pipeline? Deferred: start with Immediate + OnShutdown only.
