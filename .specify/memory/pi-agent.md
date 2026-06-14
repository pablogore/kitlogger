# [PROJECT NAME] Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-06-10

## Active Technologies

- Rust 1.75+
- Cargo
- Tokio (for async support)

## Project Structure

```text
src/
├── lib/
│   ├── context.rs
│   ├── resource.rs
│   ├── instrumentation_scope.rs
│   ├── span.rs
│   ├── log_record.rs
│   ├── metric.rs
│   └── noop.rs
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/
```

##
