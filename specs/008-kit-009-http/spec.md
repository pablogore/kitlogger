# Feature Specification: HTTP Middleware Logging

**Feature Branch**: `008-kit-009-http`  
**Created**: 2026-06-10  
**Status**: Draft  
**Input**: User description: "KIT-009 HTTP Middleware - Proveer middleware HTTP para servidores y clientes que registre automáticamente requests, responses, latencia, status codes, correlation IDs, request IDs, payloads opcionales, headers opcionales, errores, y panic recovery sin acoplamiento a frameworks específicos."

## Clarifications

### Session 2026-06-10

- Q: Request/Response Event Emission Model — emit two events (request at start + response at completion) or single completion event? → A: Option A — Emit `http_request` at start and `http_response` at completion (two-event model). Enables timeout detection, request lifecycle tracking, and consistency with KIT-008 gRPC middleware.
- Q: Payload Logging Content-Type Policy — how should different content types be handled when payload logging is enabled? → A: Mixed policy: JSON bodies parsed and redacted, Text/XML logged as-is, Binary/Image/Audio/Video logged as `[BINARY CONTENT]`, Multipart logged as metadata only.
- Q: Retry Ownership — should the middleware implement retries, only observe retries, or support both? → A: Option B — Middleware only observes retries performed by external HTTP clients; it does NOT own or execute retry logic.
- Q: Core Middleware Abstraction — should the core be based on `http::Request/Response`, `tower::Layer+Service`, or both? → A: Option C — Core based on `http::{Request, Response}` with optional tower::Layer adapter. Hyper and Axum work natively; Tower support is opt-in via adapter; Actix is not forced into Tower.
- Q: Request Context Storage — how should the middleware store Correlation ID, Request ID, Trace ID, and Span ID for downstream handler access? → A: Option A — `request.extensions()`. Standard mechanism in hyper/axum/tower for attaching arbitrary typed data; each framework adapter translates as needed.
- Q: Client Middleware Coverage — which HTTP clients should v1 support? → A: Option B — Generic `HttpClientObserver` trait with reqwest adapter for v1. Future adapters (hyper, surf, ureq) can be added without modifying the core client trait.
- Q: Trusted Proxy and Client IP Resolution — how should the middleware determine the real client IP when deployed behind reverse proxies? → A: Option B — Trusted proxy model. If the TCP peer address belongs to a configured trusted proxy network (`trusted_proxies`), resolve the client IP from forwarding headers (`X-Forwarded-For`, `X-Real-IP`, `Forwarded`). Otherwise, use the remote socket address directly. Prevents header spoofing while working correctly behind Nginx, Traefik, Envoy, AWS ALB, Cloudflare, Kubernetes Ingress, etc.
- Q: Trusted Proxy Chain Resolution Example — should the spec include a normative example of proxy chain resolution to ensure all implementations behave identically? → A: Yes. Add normative example to FR-033 with explicit rules for all edge cases (untrusted peer, missing headers, malformed headers, all-trusted chain).
- Q: Request Body Handling — how does the middleware read the request body for logging without consuming the stream? → A: Option C — Buffer + size limits. The middleware buffers the body stream into a `Bytes` buffer (up to the configured size limit), logs the buffered bytes, then replaces the body with the buffered bytes so the downstream handler receives the full stream. Beyond the limit, log `[TRUNCATED at N bytes]`. Response body buffering follows the same pattern.
- Q: Error Content in Logs — how much error detail is safe to include? → A: Option A — Status + classification only. Log the HTTP status code and a short classification label (e.g., `client_error`, `server_error`, `timeout`, `connection_reset`, `protocol_error`). Do NOT include the full error message or response body — these may contain stack traces, internal paths, or sensitive data. This preserves enough signal for alerting and dashboards without risking data leakage.

## User Scenarios & Testing _(mandatory)_

### User Story 1 - Server Request/Response Logging (Priority: P1)

A developer integrates a logging middleware into their HTTP server application. Every incoming HTTP request is automatically logged with its method, path, response status code, duration, and a unique request identifier. The developer needs no manual instrumentation — adding the middleware is sufficient for complete request lifecycle logging.

**Why this priority**: This is the core value proposition. Without server-side logging, the feature has no purpose. It delivers immediate observability for any HTTP service.

**Independent Test**: Can be fully tested by creating a server with the middleware, sending an HTTP request, and verifying that a structured log event is emitted containing method, path, status, and duration. This alone delivers operational visibility.

**Acceptance Scenarios**:

1. **Given** an HTTP server with the logging middleware attached, **When** a GET request is made to `/users` and returns 200, **Then** TWO structured log events are emitted: an `http_request` event at request start containing `method: "GET"` and `path: "/users"`, followed by an `http_response` event at completion containing `method: "GET"`, `path: "/users"`, `status: 200`, and `duration_ms`.
2. **Given** an HTTP server with the logging middleware attached, **When** a POST request to `/users` returns a 201 status, **Then** the `http_response` event contains `method: "POST"`, `path: "/users"`, `status: 201`, and a non-negative `duration_ms`.
3. **Given** an HTTP server with the logging middleware attached, **When** a request to a non-existent route returns 404, **Then** the `http_response` event captures the request with `status: 404` and correct path.
4. **Given** the middleware is NOT attached to a server, **When** requests are processed, **Then** no automatic HTTP logging occurs (the middleware is opt-in and non-invasive).
5. **Given** a request handler takes more than 1 second to complete, **When** the request is in flight, **Then** the `http_request` event is already logged before the `http_response` event, enabling early detection of slow or hung requests.

---

### User Story 2 - Client HTTP Call Logging (Priority: P1)

A developer configures an HTTP client with logging capabilities. Every outbound HTTP request the client makes is automatically logged with method, URL, response status code, and duration. The logging is transparent — the developer uses the client normally and observability is automatic.

**Why this priority**: Server and client logging form the complete observability picture. Client logging is independently valuable for debugging external API calls, latency analysis, and error tracking in service-to-service communication.

**Independent Test**: Can be fully tested by creating a configured HTTP client, making a request to a test endpoint, and verifying that a structured log event is emitted for the outbound call with method, URL, status, and duration.

**Acceptance Scenarios**:

1. **Given** an HTTP client configured with logging, **When** a GET request is made to `https://api.example.com/users`, **Then** a structured log event is emitted containing `method: "GET"`, `url: "https://api.example.com/users"`, `status: 200`, and `duration_ms`.
2. **Given** an HTTP client configured with logging via reqwest adapter, **When** a POST request receives a 4xx or 5xx response, **Then** the log event captures the error status code and duration (the middleware does not alter the response or error behavior).
3. **Given** an HTTP client without logging configured, **When** requests are made, **Then** no automatic HTTP client logging occurs.

---

### User Story 3 - Request Identification and Tracing (Priority: P2)

The middleware provides consistent request identification. When an incoming request has a `X-Correlation-ID` header, it is preserved for trace continuity across services. When missing, a new unique identifier is generated. The same applies to `X-Request-ID` for per-request identification. Additionally, if a W3C `traceparent` header is present, trace and span IDs are extracted for OpenTelemetry compatibility. All identifiers are stored in request context so any downstream handler can retrieve them.

**Why this priority**: Request identification enables distributed tracing and debugging across multiple services. Without it, correlating logs from different services is manual and error-prone. It builds on P1 by adding traceability.

**Independent Test**: Can be tested by sending requests with and without correlation headers, then verifying the middleware either propagates existing IDs or generates new ones, and that handlers can retrieve them from context.

**Acceptance Scenarios**:

1. **Given** an incoming request with `X-Correlation-ID: abc-123`, **When** the middleware processes it, **Then** the same `abc-123` is used in the log event and reflected in the response header `X-Correlation-ID`.
2. **Given** an incoming request without `X-Correlation-ID`, **When** the middleware processes it, **Then** a new UUID is generated, included in the log event, and added to the response as `X-Correlation-ID`.
3. **Given** an incoming request without `X-Request-ID`, **When** the middleware processes it, **Then** a new unique request ID is generated automatically.
4. **Given** an incoming request with a W3C `traceparent` header, **When** the middleware processes it, **Then** the `trace_id` and `span_id` are extracted and available in request context.
5. **Given** the middleware has stored identifiers in context, **When** a downstream handler accesses the context, **Then** it can retrieve `correlation_id`, `request_id`, `trace_id`, and `span_id`.

---

### User Story 4 - Error and Panic Handling (Priority: P2)

When an HTTP handler panics, the middleware catches the panic, logs the error details including a stack trace, and returns a 500 Internal Server Error response to the client. This prevents the server process from crashing and ensures every panic is observable. Regular HTTP errors (4xx, 5xx status codes) are also captured as structured error events distinct from normal request/response events.

**Why this priority**: Panic recovery protects service availability. Without it, a single panicking handler can crash the entire server process. Combined with structured error events, this provides critical reliability and debuggability.

**Independent Test**: Can be tested by creating a handler that deliberately panics, then verifying the server does not crash, a 500 response is returned, and a structured panic event is logged with error details.

**Acceptance Scenarios**:

1. **Given** a handler that panics during request processing, **When** the panic occurs, **Then** the middleware catches it, logs an event with `event: "http_panic"` containing the error message, and returns a 500 response to the client.
2. **Given** a handler that returns a 5xx status code without panicking, **When** the response is sent, **Then** an `http_error` event is logged alongside the standard response event.
3. **Given** the panic recovery middleware is active, **When** multiple concurrent requests cause panics, **Then** each panic is independently caught and logged without affecting other in-flight requests.

---

### User Story 5 - Configurable Logging Depth (Priority: P3)

A developer can configure the middleware to optionally log request and response bodies, headers, and request/response sizes. Sensitive fields in payloads (passwords, tokens, secrets) and sensitive headers (Authorization, Cookie, Set-Cookie, API keys) are automatically redacted. Body logging has a configurable size limit to prevent log explosion from large payloads.

**Why this priority**: Deep logging is valuable for debugging but adds overhead and privacy risk. It is P3 because the core logging (P1) is sufficient for operational visibility; these features enhance debugging at the cost of performance and require careful configuration.

**Independent Test**: Can be tested by configuring payload/header logging, sending a request with both sensitive and non-sensitive data, and verifying that non-sensitive data is logged while sensitive fields are redacted and body truncation works.

**Acceptance Scenarios**:

1. **Given** payload logging is enabled with `log_request_body: true`, **When** a request with JSON body `{"username": "jane", "password": "secret123"}` is processed, **Then** the log event includes the body with `"password": "[REDACTED]"` and `"username": "jane"` visible.
2. **Given** header logging is enabled, **When** a request with `Authorization: Bearer tok_abc` and `Content-Type: application/json` is processed, **Then** the log event shows `"authorization": "[REDACTED]"` and `"content-type": "application/json"`.
3. **Given** `max_body_log_size_bytes` is set to 1024, **When** a request body exceeds 1024 bytes, **Then** the logged body is truncated and marked with `...[TRUNCATED]`.
4. **Given** payload logging is disabled (default), **When** requests are processed, **Then** no body content appears in log events.
5. **Given** `request_size_bytes` and `response_size_bytes` are enabled, **When** a request is processed, **Then** the log event includes the byte size of the request payload and the response payload.

---

### User Story 6 - Performance and Operational Controls (Priority: P3)

A developer can tune the middleware for production environments: exclude health check or metrics endpoints from logging, apply sampling rates to reduce log volume, detect and flag slow requests exceeding a configurable threshold, and optimize health check handling with a dedicated mode.

**Why this priority**: These are production-tuning features. The middleware already works without them (P1-P2), but production deployments at scale need controls to manage log volume and identify performance issues.

**Independent Test**: Can be tested by configuring exclusions, sampling, and slow request thresholds, then sending requests that trigger each condition and verifying the expected behavior.

**Acceptance Scenarios**:

1. **Given** `exclude_paths` is set to `["/health", "/metrics"]`, **When** a GET request is made to `/health`, **Then** no log event is emitted for that request.
2. **Given** `sampling_rate` is set to `0.1`, **When** 1000 requests are processed, **Then** approximately 10% of requests (statistically) produce log events.
3. **Given** `slow_request_threshold_ms` is set to 500, **When** a request takes 812ms, **Then** a `slow_request` event is logged with `duration_ms: 812` in addition to the standard response event.
4. **Given** `slow_request_threshold_ms` is set to 500, **When** a request takes 200ms, **Then** no slow request event is emitted.
5. **Given** `health_check_mode` is set to `Suppress`, **When** requests are processed, **Then** health check endpoint requests produce no log events regardless of other configuration.

---

### User Story 7 - Advanced Protocol and Observability (Priority: P3)

The middleware recognizes advanced HTTP patterns: streaming responses (SSE, chunked transfer) are detected and logged as streaming without capturing the full stream body; WebSocket upgrade requests are identified and logged as a distinct event type; client retry attempts are recorded with attempt counters; and route templates (e.g., `/users/{id}`) are used instead of raw paths when the underlying framework provides them.

**Why this priority**: These are specialized use cases. The middleware is fully useful without them (P1-P2), but they provide completeness for modern HTTP applications.

**Independent Test**: Can be tested independently per protocol feature — e.g., open a WebSocket connection and verify a `websocket_upgrade` event, or trigger client retries and verify the retry counter.

**Acceptance Scenarios**:

1. **Given** a response uses Server-Sent Events or chunked transfer encoding, **When** the response is streamed, **Then** the log event includes `streaming: true` and does NOT log the full stream body.
2. **Given** an HTTP upgrade request for WebSocket, **When** the upgrade is initiated, **Then** a `websocket_upgrade` event is logged.
3. **Given** an HTTP client configured with retry capability, **When** the external client automatically retries and the request succeeds on the 2nd attempt, **Then** the log event includes `retry_attempt: 2`. The middleware itself does not initiate or control retries — it only records the counter provided by the client.
4. **Given** the framework provides route template information (e.g., `/users/{id}`), **When** a request is made to `/users/123`, **Then** the log event uses the template `/users/{id}` instead of the raw path `/users/123`.
5. **Given** the framework does NOT provide route template information, **When** a request is made to `/users/123`, **Then** the raw path `/users/123` is logged as-is (no error, no fallback attempt to resolve templates).

---

### Edge Cases

- What happens when the logger itself fails (e.g., disk full, network loss to log aggregator)? The middleware must never cause the request to fail — logging failures are silent and the request proceeds normally.
- What happens when a request body cannot be read (e.g., malformed, stream error)? The request proceeds; the body field in the log event is marked as unavailable rather than causing an error.
- What happens when correlation ID headers contain invalid characters or exceed reasonable length? The middleware sanitizes or replaces with a generated ID to prevent log injection.
- What happens when a handler starts a response stream but never finishes? The middleware logs what it can at the point of response initiation and marks the event as incomplete if the stream is unterminated.
- What happens when multiple middlewares are stacked and one panics? Each middleware layer handles its own panic independently; outer layers are not affected by inner layer panics.
- What happens when body logging and body limits interact with compressed payloads? The middleware logs the decompressed size if decompression occurs, otherwise logs the compressed wire size with a note.
- What happens when sampling is enabled alongside slow request detection? Slow requests are always logged regardless of sampling rate to ensure performance issues are never missed.
- What happens when a request arrives with the TCP peer address in the trusted proxy list but the forwarding headers are missing or unparseable? The middleware falls back to the remote socket address — proxy misconfiguration should not cause the request to fail.
- What happens when a request arrives from an untrusted source with spoofed `X-Forwarded-For` headers? Since the peer is not in `trusted_proxies`, the headers are ignored and the remote socket address is used — spoofing is neutralized.

## Requirements _(mandatory)_

### Functional Requirements

**Core Middleware**

- **FR-001**: System MUST emit an `http_request` event at the start of handling each server HTTP request, and an `http_response` event at request completion containing method, path, client IP address, response status code, and duration in milliseconds.
- **FR-002**: System MUST provide client middleware via a generic `HttpClientObserver` trait that logs every HTTP request made by a client, including method, URL, response status code, and duration in milliseconds. The v1 release MUST include a reqwest adapter implementing this trait; additional adapters (hyper, surf, ureq) may be added later without changing the trait.
- **FR-003**: System MUST be framework-independent at its core, built on the standard `http::{Request, Response}` types crate. This ensures native compatibility with Hyper and Axum. Tower's `Layer`/`Service` traits are supported via an optional adapter — not a required dependency of the core.
- **FR-004**: System MUST provide optional server adapters for popular frameworks (axum, actix-web, warp, tower, hyper) that wrap the core middleware without duplicating logic. Client adapters follow the same pattern via the `HttpClientObserver` trait; the v1 release includes reqwest.

**Request Identification**

- **FR-005**: System MUST propagate existing `X-Correlation-ID` header values when present in incoming requests.
- **FR-006**: System MUST generate a new UUID as `X-Correlation-ID` when no correlation ID is present in the incoming request.
- **FR-007**: System MUST generate a unique `X-Request-ID` for every incoming request that lacks one.
- **FR-008**: System MUST extract `trace_id` and `span_id` from W3C `traceparent` headers when present.
- **FR-009**: System MUST store Correlation ID, Request ID, Trace ID, and Span ID in `request.extensions()` (the standard type-map mechanism in the `http` crate) so downstream handlers can retrieve them via typed accessors. Each framework adapter translates `extensions()` to the framework's equivalent context mechanism.

**Error Handling**

- **FR-010**: System MUST catch panics that occur during request handling, log a structured panic event with error details, and return an HTTP 500 response.
- **FR-011**: System MUST NOT allow a panic in one request handler to crash the server process or affect other concurrent requests.
- **FR-012**: System MUST emit a distinct `http_error` event when a response has a 4xx or 5xx status code. The event must include the HTTP status code and a short classification label (e.g., `client_error`, `server_error`, `timeout`, `connection_reset`, `protocol_error`), but MUST NOT include the full error message or error response body.

**Structured Events**

- **FR-013**: System MUST emit structured log events with a consistent `event` field identifying the event type. Server events: `http_request` (emitted at request start), `http_response` (emitted at completion), `http_error`, `http_panic`, `slow_request`. Client events: `http_client_request` (emitted at completion). Protocol events: `websocket_upgrade`.
- **FR-014**: System MUST include `duration_ms` (elapsed time in milliseconds) in every request/response event.

**Payload and Header Logging**

- **FR-015**: System MUST support optional request body logging, disabled by default, enabled via configuration (`log_request_body: true`). When enabled, the middleware buffers the request body stream into a `Bytes` buffer up to the configured size limit (`max_body_log_size_bytes`), logs the buffered content according to content-type policy, then replaces the body with the buffered bytes so downstream handlers receive the full stream. Content-type handling: JSON bodies are parsed and redacted, text/XML logged as-is, binary/image/audio/video logged as `[BINARY CONTENT]`, and multipart bodies logged as metadata only. Bodies exceeding the limit are truncated with a `...[TRUNCATED at N bytes]` marker.
- **FR-016**: System MUST support optional response body logging, disabled by default, enabled via configuration (`log_response_body: true`). Response bodies follow the same buffering + size-limit + content-type policy as request bodies (FR-015). Response body logging must be configurable per-route (e.g., skip health check responses, log all API responses).
- **FR-017**: System MUST support optional header logging, disabled by default, enabled via configuration (`log_headers: true`).
- **FR-018**: System MUST redact sensitive header values by default for: `Authorization`, `Cookie`, `Set-Cookie`, `X-API-Key`, and `Proxy-Authorization`, replacing their values with `[REDACTED]`.
- **FR-019**: System MUST support configurable sensitive field redaction in payloads (e.g., `password`, `token`, `secret`), replacing matching field values with `[REDACTED]`.
- **FR-020**: System MUST enforce a configurable maximum body log size (`max_body_log_size_bytes`, default 65536), truncating larger bodies and appending a `...[TRUNCATED]` marker.
- **FR-021**: System MUST log request body size in bytes (`request_size_bytes`) and response body size in bytes (`response_size_bytes`) when size metrics are enabled.

**Performance Controls**

- **FR-022**: System MUST detect requests exceeding a configurable slow request threshold (`slow_request_threshold_ms`) and emit a distinct `slow_request` event.
- **FR-023**: System MUST support configurable path exclusions (`exclude_paths`) that suppress all logging for matching endpoints.
- **FR-024**: System MUST support configurable sampling (`sampling_rate`, 0.0 to 1.0) to reduce log volume, with slow requests always logged regardless of sampling.
- **FR-025**: System MUST support health check mode configuration (`health_check_mode`) with options: `Normal` (standard rules), `Suppress` (no logging), and `Sample` (reduced rate).
- **FR-026**: System MUST incur less than 1 millisecond of overhead per request when payload logging is disabled.

**Advanced Protocol Support**

- **FR-027**: System MUST detect streaming responses (SSE, chunked transfer encoding) and log them with a `streaming: true` flag without capturing the full stream body.
- **FR-028**: System MUST detect HTTP upgrade requests to WebSocket and log a `websocket_upgrade` event.
- **FR-029**: System MUST record retry attempt counters for client requests when the external HTTP client performs automatic retries. The middleware observes and logs retries but does NOT implement or execute retry logic itself.
- **FR-030**: System MUST log route templates (e.g., `/users/{id}`) instead of raw paths when the underlying framework provides template information. When the framework does NOT provide templates, the raw path is used without error.

**Reliability**

- **FR-031**: System MUST NOT cause a request to fail due to a logging failure — logging errors are silently swallowed and the request proceeds normally.
- **FR-032**: System MUST NOT leak sensitive data into logs under any default configuration.

**Client IP Resolution**

- **FR-033**: System MUST determine the client IP address using a trusted proxy model. When `trusted_proxies` is configured with a list of trusted CIDR networks, the middleware compares the TCP peer address against this list. If the peer is trusted, the client IP is resolved from forwarding headers (`X-Forwarded-For`, `X-Real-IP`, or the `Forwarded` header per RFC 7239) — using the leftmost untrusted IP in the chain. If the peer is not trusted or no `trusted_proxies` are configured, the remote socket address is used directly. This prevents header spoofing attacks while working correctly behind reverse proxies.

  **Normative Example**: Given `trusted_proxies: [10.0.0.0/8, 172.16.0.0/12]` and an incoming connection with `remote_addr = 10.0.0.6` (trusted) and header `X-Forwarded-For: 198.51.100.10, 10.0.0.5, 10.0.0.6`:
  - `198.51.100.10` → untrusted (not in any CIDR)
  - `10.0.0.5` → trusted (matches 10.0.0.0/8)
  - `10.0.0.6` → trusted (matches 10.0.0.0/8)
  - Result: `client_ip = 198.51.100.10` (leftmost untrusted IP in the chain)

  **Resolution rules** (implementations MUST follow this order):
  1. If the peer is not trusted → `client_ip = remote_addr` (headers ignored)
  2. If forwarding headers are missing → `client_ip = remote_addr`
  3. If forwarding headers are malformed/unparseable → `client_ip = remote_addr`
  4. If all forwarded addresses are trusted → `client_ip = leftmost address in the chain`
  5. Header precedence: `Forwarded` (RFC 7239) > `X-Forwarded-For` > `X-Real-IP` > `remote_addr`

### Key Entities

- **Log Event**: The primary output entity. Contains `event` type, `method`, `path`/`url`, `client_ip`, `status`, `duration_ms`, and optional fields (`request_body`, `response_body`, `headers`, `request_size_bytes`, `response_size_bytes`, `streaming`, `retry_attempt`). Events are structured (JSON-compatible) with consistent field naming.
- **Middleware Configuration**: Controls all optional behavior: payload logging flags, header logging flag, redacted field lists, body size limits, slow request threshold, excluded paths, sampling rate, health check mode, and trusted proxy CIDR networks (`trusted_proxies`). Has safe defaults (payload logging off, headers off, redaction on, 64KB body limit, no trusted proxies — uses socket address).
- **Request Context**: Stores identifiers injected by middleware: `correlation_id`, `request_id`, `trace_id`, `span_id`. Populated by the middleware and read by downstream handlers.
- **Client Adapter**: Thin layer that translates client-specific request/response types (reqwest `Request`/`Response`, future hyper client types) into the core `HttpClientObserver` trait. v1 ships with reqwest adapter.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: A developer can add request/response logging to an HTTP server by writing no more than 5 lines of integration code (instantiate middleware, attach to server).
- **SC-002**: The middleware adds less than 1 millisecond of overhead to request processing time (measured as p99 latency difference with and without middleware, payload logging disabled).
- **SC-003**: 100% of HTTP requests processed by a server with the middleware produce a corresponding structured log event (sampling disabled, exclusions empty).
- **SC-004**: 100% of handler panics are caught and logged without crashing the server process.
- **SC-005**: Zero sensitive header values (Authorization, Cookie, Set-Cookie, X-API-Key, Proxy-Authorization) appear in logs under default configuration.
- **SC-006**: A developer can trace a request end-to-end across 3+ services using only the correlation ID emitted by the middleware, without manual instrumentation.
- **SC-007**: The core middleware compiles and operates without any framework-specific dependencies, and the same core logic works unchanged across all supported framework adapters.
- **SC-008**: With sampling set to 10%, log volume is reduced by approximately 90% (±5 percentage points) while slow requests continue to be logged at 100%.

## Assumptions

- The project already has a structured logging facility (referenced as KIT-007) that the middleware integrates with. This spec assumes structured JSON log output is available.
- The project uses Rust as its implementation language (implied by crate structure and `tower::Layer` references).
- "Framework independence" means a core crate with no framework dependencies, plus separate adapter crates per framework. The core uses only the standard `http::Request`/`http::Response` types crate. Tower's `Layer`/`Service` traits are supported via an optional adapter — not a required dependency of the core.
- Payload logging is off by default for both security and performance reasons.
- Route template resolution is delegated to the framework; the middleware does not attempt to infer templates from raw paths.
- Client middleware uses a generic trait with adapters rather than being tied to a single HTTP client library.
- Slow request detection emits a separate event type rather than adding a field to the standard response event, to simplify alerting and querying.
- Body size limits default to 65536 bytes (64KB) as a reasonable balance between debuggability and log safety.
- The constitution file for this project is not yet defined; no additional constraints apply beyond those in this specification.
