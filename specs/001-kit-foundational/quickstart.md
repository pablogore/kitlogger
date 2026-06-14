# Quickstart Guide: KIT-001 Foundational Observability Abstractions

This guide provides a validation workflow to demonstrate that the foundational observability abstractions work end-to-end without any backend or exporter configuration.

## Prerequisites

- Rust 1.75 or later installed
- Cargo package manager
- Basic understanding of Rust programming

## Setup

1. Create a new Rust project:
   ```bash
   cargo new observability-demo
   cd observability-demo
   ```

2. Add the Kit framework as a dependency in `Cargo.toml`:
   ```toml
   [dependencies]
   kit-observability = { git = "https://github.com/your-org/kit.git", branch = "001-kit-foundational" }
   ```

3. Run `cargo build` to fetch dependencies

## Validation Scenarios

### Scenario 1: Create Root Context and Child Spans

1. Create a root context:
   ```rust
   use kit_observability::context::Context;
   
   let root_context = Context::create_root_context();
   ```

2. Create child contexts:
   ```rust
   let child_context = Context::create_child_context(&root_context);
   ```

3. Verify trace and span IDs:
   ```rust
   assert!(root_context.trace_id().is_some());
   assert!(root_context.span_id().is_some());
   assert_eq!(child_context.trace_id(), root_context.trace_id());
   ```

### Scenario 2: Emit Log Records with Correlation ID

1. Create a log record with correlation ID:
   ```rust
   use kit_observability::{LogRecord, Severity};
   
   let log_record = LogRecord::emit(
       "User registered successfully".to_string(),
       &root_context,
       &default_resource,
       &default_scope,
       Severity::Info,
   );
   ```

2. Verify log record structure:
   ```rust
   assert_eq!(log_record.message(), "User registered successfully");
   assert_eq!(log_record.severity(), Severity::Info);
   assert_eq!(log_record.trace_id(), root_context.trace_id());
   ```

### Scenario 3: Record Metrics with All Four Instrument Types

1. Create and record with all four metric instrument types:
   ```rust
   use kit_observability::{Counter, Gauge, Histogram, UpDownCounter};
   
   // Counter
   let mut counter = Counter::create("request_count", &default_resource, &default_scope);
   counter.record(1.0);
   
   // Gauge
   let mut gauge = Gauge::create("memory_usage", &default_resource, &default_scope);
   gauge.record(1024.0);
   
   // Histogram
   let mut histogram = Histogram::create("response_time", &default_resource, &default_scope);
   histogram.record(150.0);
   
   // UpDownCounter
   let mut up_down_counter = UpDownCounter::create("active_connections", &default_resource, &default_scope);
   up_down_counter.record(5.0);
   ```

2. Verify metric values:
   ```rust
   assert_eq!(counter.value(), 1.0);
   assert_eq!(gauge.value(), 1024.0);
   assert_eq!(histogram.count(), 1);
   assert_eq!(up_down_counter.value(), 5.0);
   ```

### Scenario 4: Resource Association

1. Create a resource with custom attributes:
   ```rust
   use kit_observability::Resource;
   
   let resource = Resource::new(vec![
       ("service.name".to_string(), "api-gateway".to_string()),
       ("deployment.environment".to_string(), "production".to_string()),
   ]);
   ```

2. Associate with telemetry:
   ```rust
   let span = Span::start("user_login", &root_context, &resource, &default_scope);
   let log_record = LogRecord::emit("Login successful".to_string(), &root_context, &resource, &default_scope, Severity::Info);
   ```

3. Verify resource attributes:
   ```rust
   assert_eq!(span.resource().attributes().get("service.name"), Some(&"api-gateway".to_string()));
   assert_eq!(log_record.resource().attributes().get("service.name"), Some(&"api-gateway".to_string()));
   ```

### Scenario 5: NoOp Implementations

1. Verify that all NoOp implementations work without errors:
   ```rust
   use kit_observability::{NoOpLogger, NoOpTracer, NoOpMeter};
   
   let logger = NoOpLogger;
   let tracer = NoOpTracer;
   let meter = NoOpMeter;
   
   // These should not panic or produce errors
   logger.emit(log_record);
   let span = tracer.start_span("test", &root_context, &resource, &default_scope);
   meter.create_counter("test_counter", &resource, &default_scope);
   ```

## Expected Outcomes

After running these validation scenarios, you should observe:

1. All telemetry records are created successfully without any backend or exporter configuration
2. Context propagation works correctly between parent and child contexts
3. All four metric instrument types (Counter, Gauge, Histogram, UpDownCounter) function correctly
4. Resource attributes are properly associated with telemetry records
5. Correlation IDs can be attached to telemetry records independently of trace context
6. NoOp implementations accept all API calls without error and discard all data silently
7. The application compiles and runs without errors when no backend is configured

## Running Tests

To run the validation tests:
```bash
cargo test
```

The tests should pass without any configuration of exporters or backends.