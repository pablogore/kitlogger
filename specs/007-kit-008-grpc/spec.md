# Feature Specification: KIT-008 gRPC Middleware Observability

**Feature Branch**: `007-kit-008-grpc`
**Created**: 2026-06-10
**Status**: Draft
**Input**: Implementar middleware gRPC server y client para Kit Logger que permita observabilidad consistente en servicios gRPC mediante logging estructurado, correlation IDs, request metadata, timing y manejo uniforme de errores.

## Overview

Kit Logger already provides structured logging foundations — a `Logger` interface, correlation IDs, context propagation, redaction, sampling, multiple outputs, and a formatting subsystem (KIT-001, KIT-005, KIT-006). This feature adds official middleware support for gRPC services, enabling consistent observability across RPC-based microservices without coupling to any specific logging implementation.

The middleware layer provides:

- **Server middleware** — inject automatic structured logging into unary and streaming RPC handlers, capturing request metadata, timing, status codes, and errors without per-handler boilerplate.
- **Client middleware** — instrument outgoing RPC calls with the same structured logging discipline, recording target service, method, duration, and response status.
- **Correlation propagation** — automatically propagate `request_id`, `correlation_id`, and `traceparent` across RPC boundaries via standard gRPC metadata, enabling end-to-end tracing across service meshes and distributed systems.
- **Error logging** — structured error records that capture gRPC status codes and messages while redacting sensitive information.
- **Payload logging** — optional, configurable logging of request/response payloads that respects the existing redaction and masking rules.
- **Slow request detection** — configurable thresholds that emit warning-level log events when RPC duration exceeds the threshold.
- **Metadata filtering** — allowlist and denylist support for gRPC metadata headers, preventing sensitive metadata from appearing in logs.

**Middleware Model**: Unary RPC instrumentation MAY use the tonic `Interceptor` trait. Streaming RPC instrumentation MUST use Tower `Layer`/`Service` middleware. All middleware, regardless of implementation approach, MUST compose correctly within the same tonic `Server` or `Channel` builder stack.

All middleware is designed to be provider-agnostic — it consumes Kit Logger's abstract `Logger` interface (or equivalent) and has no dependency on any concrete logging backend, OpenTelemetry SDK, or external framework.

**Implementation Target**: The primary implementation target is **tonic** (Rust gRPC framework). The following are explicitly **Out of Scope**:
- grpc-rs
- ConnectRPC
- gRPC-Web

## Clarifications

### Session 2026-06-10

- Q: What is the acceptable latency overhead the middleware should add per unary RPC? → A: <10μs per unary RPC (excluding downstream logger I/O).
- Q: Under log write pressure, how should the middleware handle backpressure? → A: Buffer with bounded capacity (configurable), drop oldest events when full.
- Q: What log level should each middleware event type use? → A: Derived from both event type and gRPC status code per the three-tier INFO/WARN/ERROR mapping (see Log Level Assignment section).
- Q: Should the middleware auto-generate correlation IDs when none are present in incoming metadata? → A: Auto-generate request_id always; do NOT auto-generate correlation_id, trace_id, or traceparent unless explicitly enabled via configuration.
- Q: When payload logging is enabled for streaming RPCs, should every message payload be logged? → A: No. Payload logging for streaming RPCs is not supported. Payload logging applies to unary RPCs only.

## User Scenarios & Testing _(mandatory)_

### User Story 1 — Automatically Log Incoming RPC Requests on the Server (Priority: P1)

As a service developer using Kit Logger, I want every incoming gRPC call on my server to be automatically logged with structured metadata (service name, method name, duration, status code, peer address, and correlation identifiers) so that I can observe and debug RPC traffic without adding logging code to each handler.

**Why this priority**: Automatic request logging is the foundational capability. Without it, every handler would need manual logging code, leading to inconsistency and maintenance burden.

**Independent Test**: A standalone test registers a simple unary RPC handler, invokes it through the interceptor, and verifies that a log event is produced containing the expected service name, method name, duration, and status code — without the handler itself calling any logging API.

**Acceptance Scenarios**:

1. **Given** a gRPC server with the unary server interceptor enabled, **When** a client sends a valid unary request, **Then** a log event is emitted at the start and end of the request, containing the service name, method name, duration in milliseconds, and final gRPC status code.
2. **Given** a gRPC server with the unary server interceptor enabled, **When** a client sends a request that results in an error, **Then** the log event captures the error status code and error message, without exposing sensitive data.
3. **Given** the interceptor is active, **When** a request is received, **Then** the log event includes the peer address of the caller.
4. **Given** the interceptor is active, **When** a request carries a `request_id` or `correlation_id` in metadata, **Then** those identifiers are present in the log event.

---

### User Story 2 — Automatically Log Streaming RPC Requests on the Server (Priority: P1)

As a service developer using streaming gRPC patterns (server streams, client streams, bidirectional streams), I want stream lifecycle events to be automatically logged so that I can observe when streams open, close, and how long they remain active.

**Why this priority**: Streaming RPCs have different lifecycle semantics than unary calls. Without dedicated stream interceptors, stream open/close events and their durations would go unlogged or require invasive per-handler instrumentation.

**Independent Test**: A test creates a server streaming RPC handler, invokes it through the stream interceptor, sends and receives messages, closes the stream, and verifies that stream open and stream close log events are produced with the correct service, method, and duration.

**Acceptance Scenarios**:

1. **Given** a gRPC server with the streaming server interceptor enabled, **When** a client initiates a server-streaming RPC, **Then** a `stream_open` log event is emitted when the stream is established.
2. **Given** an active stream, **When** the stream is closed by either the client or server, **Then** a `stream_close` log event is emitted with the total stream duration and final status code.
3. **Given** a bidirectional streaming RPC, **When** the stream is active, **Then** both `stream_open` and `stream_close` events are logged, and optional per-message events can be emitted if `log_stream_events` is enabled.

---

### User Story 3 — Automatically Log Outgoing RPC Calls from a Client (Priority: P2)

As a developer building a service that calls other gRPC services, I want outgoing client calls to be automatically logged so that downstream service dependencies are observable without wrapping each call in manual timing and logging code.

**Why this priority**: Client-side observability is critical for understanding dependency latencies and failure modes in a microservice architecture. Without client interceptors, each outgoing call would need manual instrumentation.

**Independent Test**: A test creates a gRPC client connection with the client interceptor, makes a unary call to a test server, and verifies that a log event is produced with the target service address, method name, call duration, and response status code — without the caller adding any manual logging.

**Acceptance Scenarios**:

1. **Given** a gRPC client with the unary client interceptor enabled, **When** the client makes an outgoing unary call, **Then** a log event is emitted containing the target service, method, duration in milliseconds, and response status code.
2. **Given** a client interceptor, **When** an outgoing call fails (network error, deadline exceeded, server error), **Then** the log event captures the error with the appropriate gRPC status code and message.
3. **Given** a client interceptor, **When** the outgoing call carries a `request_id` or `correlation_id`, **Then** those identifiers are propagated to the target service via gRPC metadata.

---

### User Story 4 — Correlate Logs Across RPC Boundaries (Priority: P1)

As an operator debugging a multi-service workflow, I want correlation identifiers to be automatically propagated across gRPC calls so that I can trace a request's journey through multiple services without manually plumbing identifiers through each service boundary.

**Why this priority**: Cross-service correlation is the essential value proposition of middleware-based instrumentation. Manual propagation is error-prone and rarely applied consistently across all service boundaries.

**Independent Test**: A test creates a server interceptor that extracts `correlation_id` and `traceparent` from incoming gRPC metadata, passes them through a mock downstream call with the client interceptor, and verifies the downstream call's metadata contains the propagated identifiers.

**Acceptance Scenarios**:

1. **Given** an incoming gRPC request with `x-correlation-id` metadata, **When** the server interceptor processes the request, **Then** the correlation ID is injected into the logger context for that request scope.
2. **Given** an incoming gRPC request with `traceparent` metadata (W3C Trace Context), **When** the server interceptor processes the request, **Then** the trace context is extracted and made available in the logger context.
3. **Given** a logger context with an active `correlation_id`, **When** a client interceptor sends an outgoing gRPC call, **Then** the `correlation_id` is automatically added to the outgoing gRPC metadata.
4. **Given** a logger context with an active `trace_id` (from either traceparent or automatic generation), **When** a client interceptor sends an outgoing gRPC call, **Then** the `traceparent` header is set in the outgoing metadata.

---

### User Story 5 — Receive Alerts for Slow gRPC Calls (Priority: P2)

As a platform engineer responsible for service latency, I want slow gRPC requests to be automatically detected and logged separately so that I can identify performance regressions without sifting through all request logs.

**Why this priority**: Slow request detection is a common operational need. Making it a built-in, configurable feature ensures consistent slow-request monitoring across all gRPC services using Kit Logger.

**Independent Test**: A test configures a slow request threshold of 100ms, creates a server handler that deliberately sleeps for 200ms, makes a request, and verifies that a `grpc_slow_request` event is emitted in addition to the standard request completion event.

**Acceptance Scenarios**:

1. **Given** a slow request threshold of 500ms configured, **When** a request completes in 300ms, **Then** no slow request event is emitted.
2. **Given** a slow request threshold of 500ms configured, **When** a request completes in 700ms, **Then** a `grpc_slow_request` event is emitted with the actual duration.
3. **Given** no slow request threshold is configured, **When** any request completes, **Then** no slow request event is emitted (feature is opt-in).

---

### User Story 6 — Control What Metadata Appears in Logs (Priority: P3)

As a security-conscious operator, I want to control which gRPC metadata headers appear in logs via an allowlist or denylist so that sensitive headers (e.g., authorization tokens, internal routing keys) are never recorded, even when payload logging is enabled.

**Why this priority**: Metadata filtering is a security and compliance requirement. Hard-coded filtering is fragile; a configurable allowlist/denylist ensures that metadata logging policies can be managed without code changes.

**Independent Test**: A test configures a denylist containing `authorization`, sends a request with that header, and verifies the header value is not present in the log event, while other headers are logged normally.

**Acceptance Scenarios**:

1. **Given** a metadata allowlist configured with `["service", "method"]`, **When** a request has headers `{service: "users.UserService", method: "GetUser", authorization: "Bearer xyz"}`, **Then** only `service` and `method` appear in the log event.
2. **Given** a metadata denylist configured with `["authorization", "x-internal-token"]`, **When** a request has those headers, **Then** their values are omitted from the log event.
3. **Given** no allowlist or denylist is configured, **When** a request is received, **Then** no metadata headers appear in log events (metadata logging is opt-in).
4. **Given** both an allowlist and a denylist are configured, **Then** the allowlist takes precedence (only allowlisted headers are considered, then the denylist is applied to that subset).

---

### User Story 7 — Log Request and Response Payloads with Redaction (Priority: P3)

As a developer debugging a production issue, I want to optionally log request and response payloads for specific services, with sensitive fields automatically redacted, so that I can inspect the data flowing through my gRPC services without exposing secrets or PII in logs.

**Why this priority**: Payload logging is a powerful debugging tool, but dangerous if sensitive data is exposed. Integrating with the existing redaction subsystem ensures that payload logging is both useful and safe.

**Independent Test**: A test enables payload logging and configures a redaction rule for the field `password`, sends a request with `password: "s3cret"`, and verifies the log event shows `password` as redacted while non-sensitive fields are present.

**Acceptance Scenarios**:

1. **Given** payload logging is disabled (default), **When** a request is processed, **Then** no payload content appears in log events.
2. **Given** payload logging is enabled and a redaction rule exists for `email`, **When** a request contains `email: "user@example.com"`, **Then** the log event shows the email field as redacted.
3. **Given** payload logging is enabled and no redaction rules are configured, **When** a request is processed, **Then** the payload is logged in full.
4. **Given** payload logging is enabled for response payloads, **When** a response is sent, **Then** the response payload is logged (subject to the same redaction rules).

### Edge Cases

- **Context cancellation**: When a gRPC context is cancelled (client disconnects, deadline exceeded), the interceptor must still produce log events with the cancellation status and partial duration, rather than silently dropping the record.
- **Deadline exceeded**: When a request exceeds the gRPC deadline, the interceptor must log the event with `DEADLINE_EXCEEDED` status and the duration up to the deadline, without leaking internal deadline-handling details.
- **Empty metadata**: When a request arrives with empty metadata, or metadata that contains only headers on the denylist, the interceptor must handle this gracefully without errors or omissions.
- **Concurrent requests**: Multiple concurrent requests on the same server must not interleave or corrupt each other's log context. Each request maintains an independent correlation scope.
- **Very large payloads**: When payload logging is enabled for requests with large payloads (e.g., >1 MB), the interceptor must either truncate the payload or handle the size gracefully without excessive memory use.
- **Nil/empty correlation IDs**: When no `request_id` is present in incoming metadata, the middleware MUST auto-generate one. When no `correlation_id` or `traceparent` is present, the middleware MUST NOT auto-generate them (generation requires explicit configuration).
- **Interceptor composition**: The middleware must compose correctly with other gRPC interceptors (e.g., authentication, rate limiting), preserving both logging behaviour and the other interceptor's semantics.
- **Logger backpressure**: When the downstream logger is congested and the bounded buffer is full, the middleware must drop the oldest pending log event and continue without blocking the RPC. A dropped-event counter SHOULD be exposed for observability.

## Requirements _(mandatory)_

### Functional Requirements

#### Server Unary Middleware

- **FR-001**: The system MUST provide server-side unary middleware that automatically logs the start and completion of each unary RPC call. This MAY use the tonic `Interceptor` trait.
- **FR-002**: The start-of-request log event MUST capture at minimum: service name, method name, and request metadata (subject to filtering rules).
- **FR-003**: The end-of-request log event MUST capture at minimum: service name, method name, duration in milliseconds, gRPC status code, and any error details.
- **FR-004**: The middleware MUST capture the peer (client) address and include it in the log event when available.

#### Server Streaming Middleware

- **FR-005**: The system MUST provide server-side streaming middleware that logs the lifecycle of server-streaming, client-streaming, and bidirectional streaming RPCs. This MUST be implemented as a Tower `Layer`/`Service` — the tonic `Interceptor` trait does not support streaming RPCs.
- **FR-006**: The streaming middleware MUST emit a `stream_open` event when a stream is established, containing service name and method.
- **FR-007**: The streaming middleware MUST emit a `stream_close` event when a stream terminates, containing service name, method, duration, and final status code.
- **FR-008**: The streaming middleware MAY emit per-message metadata-only events (no payloads) for sent and received messages when `log_stream_events` configuration is enabled. Payload logging is not supported for streaming RPCs.

#### Client Unary Middleware

- **FR-009**: The system MUST provide client-side unary middleware that automatically logs each outgoing unary RPC call. This MAY use the tonic `Interceptor` trait.
- **FR-010**: The client middleware MUST capture the target service address, method name, duration in milliseconds, and response status code.
- **FR-011**: The client middleware MUST automatically propagate correlation identifiers (request_id, correlation_id, traceparent) from the active logger context to the outgoing gRPC metadata.

#### Client Streaming Middleware

- **FR-012**: The system MUST provide client-side streaming middleware that logs the lifecycle of outgoing streaming RPCs (client streams, server streams, and bidirectional). This MUST be implemented as a Tower `Layer`/`Service` — the tonic `Interceptor` trait does not support streaming RPCs.
- **FR-013**: The client streaming middleware MUST emit `stream_open` and `stream_close` events with the same field set as the server streaming middleware.
- **FR-014**: The client streaming middleware MUST propagate correlation identifiers on stream establishment.

#### Correlation Propagation

- **FR-015**: The middleware MUST automatically extract `x-request-id`, `x-correlation-id`, and `traceparent` (W3C Trace Context) from incoming gRPC metadata and make them available in the logger context for the duration of the request.
- **FR-016**: The middleware MUST automatically inject active correlation identifiers (`request_id`, `correlation_id`, `traceparent`) into outgoing gRPC metadata when sending client calls.
- **FR-017**: Correlation propagation MUST work without any OpenTelemetry SDK dependency. The middleware handles traceparent parsing at the HTTP/gRPC header level without requiring a full trace SDK.
- **FR-018**: When `traceparent` is present in incoming metadata, the middleware MUST parse it and expose the trace_id in log events. When absent, the middleware MUST NOT generate a synthetic trace_id unless explicitly enabled via configuration.

**Request ID auto-generation rules**:

- **FR-019**: When `x-request-id` is present in incoming metadata, the middleware MUST preserve and propagate it without modification.
- **FR-020**: When `x-request-id` is absent from incoming metadata, the middleware MUST generate a new unique `request_id`, store it in the request context, include it in all middleware log events for that request, and propagate it to downstream gRPC calls.
- **FR-021**: The middleware MUST NOT auto-generate `correlation_id`, `trace_id`, or `traceparent` under any circumstance unless explicitly enabled via configuration. A `correlation_id` present in incoming metadata MUST be preserved and propagated. A `traceparent` present in incoming metadata MUST be preserved and propagated.

#### Log Level Assignment

Middleware log levels MUST be derived from both the event type and the gRPC status code. The following mapping is normative and MUST be implemented exactly as specified.

| gRPC Status        | Level | Rationale                                         |
| ------------------ | ----- | ------------------------------------------------- |
| OK                 | INFO  | Normal operation                                  |
| Cancelled          | INFO  | Client-driven, expected                           |
| InvalidArgument    | INFO  | Business/domain-level failure                     |
| NotFound           | INFO  | Business/domain-level failure                     |
| AlreadyExists      | INFO  | Business/domain-level failure                     |
| PermissionDenied   | INFO  | Business/domain-level failure                     |
| Unauthenticated    | INFO  | Business/domain-level failure                     |
| Unimplemented      | INFO  | Expected routing outcome                          |
| DeadlineExceeded   | WARN  | Timeout / resource pressure                       |
| ResourceExhausted  | WARN  | Resource pressure condition                       |
| Aborted            | WARN  | Contention / race condition                       |
| FailedPrecondition | WARN  | Operation preconditions not met                   |
| Internal           | ERROR | Infrastructure or internal failure                |
| Unavailable        | ERROR | Infrastructure or internal failure                |
| DataLoss           | ERROR | Infrastructure or internal failure                |
| Unknown            | ERROR | Unexpected runtime failure                        |

**Event-based overrides** (applied before status-based level):

| Event                              | Level | Notes                                               |
| ---------------------------------- | ----- | --------------------------------------------------- |
| `grpc_request_started`             | INFO  | Always INFO regardless of eventual status           |
| `grpc_request_completed`           | *     | Level follows the gRPC status mapping above         |
| `grpc_stream_opened`               | INFO  | Always INFO                                         |
| `grpc_stream_closed`               | *     | Level follows the gRPC status mapping above         |
| `grpc_slow_request`                | WARN  | Fixed WARN; emitted in addition to completion event |
| `grpc_request_payload`             | INFO  | Optional payload log                                 |
| `grpc_response_payload`            | INFO  | Optional payload log                                 |

#### Error Logging

- **FR-022**: The middleware MUST log the gRPC status code and status message on both successful and failed RPCs. The log level MUST follow the Log Level Assignment mapping above.
- **FR-023**: The middleware MUST distinguish between business/domain-level failures (INFO), resource pressure conditions (WARN), and infrastructure or internal failures (ERROR) per the Log Level Assignment mapping. This distinction MUST be driven solely by the gRPC status code, not by inspecting error message text.
- **FR-024**: The middleware MUST NOT expose sensitive information (request payloads, stack traces, internal error details) in error log messages. Error types must be generic and safe for production logging.

**Rationale**: Not every non-OK gRPC status represents a system failure. Statuses such as NotFound, InvalidArgument, AlreadyExists, PermissionDenied, or Unauthenticated are often expected business outcomes and should not generate ERROR-level noise. WARN is reserved for latency and resource pressure signals, while ERROR indicates genuine operational problems requiring investigation. This approach provides meaningful operational observability while avoiding alert fatigue.

#### Configuration

- **FR-025**: The middleware MUST provide a `GrpcMiddlewareConfig` (or equivalent) with the following configurable options:

  | Option                    | Type    | Default | Description                                    |
  | ------------------------- | ------- | ------- | ---------------------------------------------- |
  | enabled                   | bool    | true    | Master switch for the middleware               |
  | log_requests              | bool    | true    | Log incoming/outgoing request metadata         |
  | log_responses             | bool    | true    | Log response metadata                          |
  | log_stream_events         | bool    | false   | Log per-message stream events                  |
  | log_payloads              | bool    | false   | Log request/response payloads (opt-in)         |
  | slow_request_threshold    | duration| none    | Duration threshold for slow request detection  |
  | log_buffer_capacity       | uint    | 4096    | Max buffered log events (0 = unbounded, use with caution) |

- **FR-026**: The configuration MUST be composable — separate configuration for server and client interceptors should be possible without global state.
- **FR-027**: Configuration MUST support runtime changes where the middleware architecture permits it (e.g., the config is checked on each request, not cached at startup).

#### Payload Logging

- **FR-028**: Payload logging MUST be opt-in (disabled by default).
- **FR-029**: Payload logging applies to unary RPCs only. Streaming RPC payload logging is not supported.
- **FR-030**: When payload logging is enabled, the middleware MUST respect the existing redaction and masking rules from the Kit Logger redaction subsystem.
- **FR-031**: Payload logging MUST support configurable size limits to prevent logging excessively large payloads.

#### Slow Request Detection

- **FR-032**: The middleware MUST support a configurable `slow_request_threshold` duration.
- **FR-033**: When a request's duration exceeds the configured threshold, the middleware MUST emit a `grpc_slow_request` event at WARN level in addition to the standard completion event.
- **FR-034**: The slow request event MUST include the actual duration and the configured threshold for reference.
- **FR-035**: When no threshold is configured, slow request detection MUST be disabled (no additional events).

#### Metadata Filtering

- **FR-036**: The middleware MUST support an allowlist of gRPC metadata keys. When an allowlist is configured, only metadata keys in the list are logged.
- **FR-037**: The middleware MUST support a denylist of gRPC metadata keys. When a denylist is configured, metadata keys in the list are excluded from logs.
- **FR-038**: When both an allowlist and denylist are configured, the allowlist is applied first (only allowlisted keys are candidates), then the denylist is applied to that subset.
- **FR-039**: When neither an allowlist nor a denylist is configured, metadata headers are NOT logged (safe default — metadata logging is opt-in).

### Non-Functional Requirements

- **NFR-001**: The middleware MUST be provider-agnostic — it must consume the abstract `Logger` interface (or equivalent) and must not depend on any concrete logging backend implementation.
- **NFR-002**: The middleware MUST NOT introduce any mandatory dependency on OpenTelemetry SDKs or OpenTelemetry protocol libraries.
- **NFR-003**: The middleware MUST NOT introduce any dependency on external web frameworks, routers, or HTTP libraries beyond the core gRPC library.
- **NFR-004**: The middleware MUST be thread-safe — multiple concurrent RPCs must not corrupt logging state or produce interleaved output.
- **NFR-005**: The middleware MUST introduce less than 10μs of latency overhead per unary RPC (excluding downstream logger I/O). The middleware SHOULD minimise heap allocations on the hot path — logically necessary allocations (e.g., building the log event payload) are expected, but repeated per-request allocations in non-varying fields should be avoided.
- **NFR-006**: All middleware MUST compose correctly within the same tonic `Server` or `Channel` builder stack — adding logging middleware must not break authentication middleware, rate limiters, or other middleware in the chain.
- **NFR-007**: Streaming middleware MUST be implemented as Tower `Layer`/`Service`. Unary middleware MAY use the tonic `Interceptor` trait. Implementers MUST NOT attempt to use tonic's `Interceptor` trait for streaming RPCs; it is unsupported by tonic and will fail at compile time for streaming service descriptors.
- **NFR-008**: The middleware MUST NOT produce side effects beyond logging — it must not modify requests, responses, metadata, or context in ways that alter application behaviour beyond observability.
- **NFR-009**: The middleware MUST use bounded buffering for log events under backpressure. When the internal buffer (configurable via `log_buffer_capacity`) is full, the middleware MUST drop the oldest unprocessed event before enqueuing a new one. The middleware MUST NOT block the gRPC handler on log writes under any circumstance.

### Key Entities

- **Unary Server Middleware**: A middleware component that intercepts individual request-response RPC calls on the server side, logging each call's start, completion, timing, and result. MAY be implemented via the tonic `Interceptor` trait.
- **Streaming Server Middleware**: A middleware component that wraps streaming RPC calls (server, client, and bidirectional streams) on the server side, logging stream lifecycle events (open, close, duration, status). MUST be implemented as a Tower `Layer`/`Service`.
- **Unary Client Middleware**: A middleware component that intercepts outgoing individual request-response RPC calls on the client side, logging each call's target, method, timing, and result, and propagating correlation metadata. MAY be implemented via the tonic `Interceptor` trait.
- **Streaming Client Middleware**: A middleware component that wraps outgoing streaming RPC calls on the client side, logging stream lifecycle events and propagating correlation metadata. MUST be implemented as a Tower `Layer`/`Service`.
- **Correlation Context**: The per-request scope that carries `request_id`, `correlation_id`, and `traceparent` state across RPC boundaries. Extracted from incoming metadata and injected into outgoing metadata automatically.
- **GrpcMiddlewareConfig**: Configuration structure controlling middleware behaviour — what to log, when to log slow requests, whether to log payloads, and which metadata headers to include or exclude.
- **Redaction Rules**: The existing Kit Logger redaction configuration, referenced by payload logging to ensure sensitive fields are masked before being written to logs.
- **Metadata Filter**: A rule set (allowlist and/or denylist) that controls which gRPC metadata headers appear in log output. Ensures sensitive headers like `authorization` are never recorded.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: A developer can add the server unary interceptor to their gRPC server, and every incoming unary call produces structured log events with service name, method name, duration (ms), status code, peer address, and any present correlation IDs — without adding logging code to any handler. Verified by a test that makes a unary call through the interceptor and inspects the emitted log events.
- **SC-002**: A developer can add the server stream interceptor to their gRPC server, and every streaming RPC (server stream, client stream, bidirectional) produces `stream_open` and `stream_close` events with service name, method name, duration, and final status. Verified by a test that opens a stream, exchanges messages, closes it, and inspects the log events.
- **SC-003**: A developer can add the client interceptor to their gRPC client connection, and every outgoing call produces log events with target service, method, duration, and status — with correlation IDs automatically propagated to the downstream service. Verified by a test that makes an outgoing call and inspects both the local log event and the downstream service's received metadata.
- **SC-004**: A correlation ID present in incoming gRPC metadata is automatically extracted and available in the logger context for the duration of the request. When the same scope makes an outgoing client call, the correlation ID is propagated in the outgoing metadata. Verified by a full round-trip test across two services.
- **SC-005**: A slow request threshold can be configured, and requests exceeding the threshold emit a `grpc_slow_request` event with actual duration. Verified by a test that configures a threshold and makes a request that exceeds it.
- **SC-006**: Payload logging can be enabled and respects redaction rules — sensitive fields are masked in logged payloads. Verified by a test that enables payload logging with a redaction rule for a known field name.
- **SC-007**: Metadata filtering works via allowlist and denylist — only allowed metadata appears in logs, and denied metadata is excluded. Verified by tests that configure each filtering mode and inspect resulting log events.
- **SC-008**: All interceptors (unary server, stream server, unary client, stream client) produce correct log events under concurrent load — no state corruption, no interleaved log contexts. Verified by a concurrent test that runs multiple RPCs in parallel and checks log event integrity.
- **SC-009**: The middleware composes correctly with a simple authentication interceptor — both middleware functions execute without errors and both log events and auth checks proceed as expected. Verified by a test that chains the logging interceptor with an auth interceptor.
- **SC-010**: The middleware can be integrated into a gRPC server and client, enabled/disabled via configuration, and all logging behaviour respects the configuration settings (log_requests, log_responses, log_payloads, etc.). Verified by a configuration-driven test suite.

## Assumptions

- The middleware targets **tonic**, the primary Rust gRPC framework used by this project. Support for other gRPC frameworks (grpc-go, grpc-rs, ConnectRPC, gRPC-Web) is out of scope for this feature.
- The middleware auto-generates a `request_id` when none is present in incoming metadata. The middleware does NOT auto-generate `correlation_id`, `trace_id`, or `traceparent` unless explicitly enabled via configuration.
- Payload logging is disabled by default for security and performance reasons. When enabled, it integrates with the existing redaction subsystem to prevent sensitive data exposure.
- Slow request detection is opt-in — no threshold means no slow request events are generated.
- Metadata logging is opt-in — by default, no metadata headers are logged. Operators must explicitly configure an allowlist to capture metadata, or configure both allowlist and denylist to log all metadata except specific keys.
- The middleware model follows tonic's architecture: unary middleware MAY use `tonic::Interceptor`; streaming middleware MUST use Tower `Layer`/`Service`. This is not a negotiable design choice — tonic's interceptor trait does not support streaming service descriptors.
- The middleware does not implement distributed tracing (span creation, span parenting, trace exporting). Correlation propagation is limited to passing `traceparent` headers. Full distributed tracing integration is a separate feature (KIT-002 or equivalent).
- Zero-allocation requirements apply to the hot path only. One-time startup allocations (configuration loading, interceptor registration) are exempt.
- The middleware is a new addition to the Kit Logger codebase. In this project's modular architecture (workspace members, cargo crates), the middleware lives in a `grpc/` package, separate from the core logging abstractions.

## Dependencies

- **KIT-001 / KIT-005 Foundational Abstractions and Logger API**: Provides the `Logger` interface (or equivalent), `LogRecord`, correlation ID support, context propagation, and redaction subsystem that the middleware consumes.
- **KIT-006 Formatting Subsystem**: Provides structured event formatting that the middleware uses to produce consistent log output.
- **gRPC framework**: The middleware targets **tonic**, the Rust gRPC framework used by this project. Other frameworks (grpc-rs, ConnectRPC, gRPC-Web) are out of scope.
