# Feature Specification: Audit Logging Subsystem (KIT-011 Audit Capture)

**Feature Branch**: `011-kit-011-audit`  
**Created**: 2026-06-10  
**Status**: Draft

> **Note**: This specification covers **Audit Capture** only. Storage & Query functionality is covered in KIT-012.

## GAPs Incorporados

| GAP | Tema | Cambios Aplicados |
|-----|------|-------------------|
| A | AuditEvent immutability | Campos privados, solo AuditEventBuilder puede construir |
| B | Metadata ownership | Nuevo enum AuditValue en lugar de serde_json::Value |
| C | Query API scope | Removido de MVP → KIT-012 (Audit Storage & Query) |
| D | Integrity provider | previous_hash → trait IntegrityProvider con implementaciones |
| E | Exporters obligatorios | MVP requiere: Console, JSON, File |
| F | 100k eventos/s | Cambiado a 10k sustained, 100k burst |
| G | AuditConfig | Nueva estructura de configuración |
| H | HTTP integration | Integración explícita con KIT-009 middleware |
| I | AuditStore → AuditSink | Trait renombrado para evitar confusión con almacenamiento |
| J | Retention Policies | Movido a KIT-012 (no es captura, es almacenamiento) |
| K | SC-006 formatos | Formatos: JSON (machine), Console (human), opcional JSONL/CEF |

**Input**: User description: "KIT-011 Audit Logging - Implement a complete Audit Logging subsystem for Kit Logger that provides immutable, structured, compliance-oriented audit trails independent from operational logs."

## User Scenarios & Testing _(mandatory)_

### User Story 1 - Record Security-Critical Events (Priority: P1)

A **developer building a secure application** needs to record security-relevant events (user logins, permission changes, account modifications) with full context so that security teams can review and investigate incidents later.

**Why this priority**: Security events are the primary use case for audit logging and directly impact regulatory compliance and incident response capabilities.

**Independent Test**: Can be fully tested by recording a security event with all required fields (actor, action, target, outcome) and verifying it is exported with complete fidelity.

> **Note**: Query/retrieve capabilities are provided by KIT-012.

**Acceptance Scenarios**:

1. **Given** a developer has audit logging configured, **When** they record a user login event with actor (user ID), action ("user.login"), target (resource), and outcome (success/failure), **Then** the event is persisted with a unique ID, timestamp, and all provided context.

2. **Given** a security incident is under investigation, **When** an analyst exports events related to a specific user from the audit system, **Then** all matching events are delivered with complete field fidelity.

3. **Given** an audit record has been written, **When** any attempt is made to modify or delete it, **Then** the original record remains unchanged (immutability).

---

### User Story 2 - Record Business and Compliance Events (Priority: P2)

A **developer** needs to record business-critical and compliance-relevant events (financial transactions, data exports, configuration changes, API key generation) to support regulatory requirements.

**Why this priority**: Many industries (financial services, healthcare, government) have strict compliance requirements for recording specific business events.

**Independent Test**: Can be fully tested by recording business events and verifying they are exported with full context for compliance reporting.

**Acceptance Scenarios**:

1. **Given** a financial transaction is processed, **When** the system records a transaction approval event with amount, currency, counterparty, and approver, **Then** the event is exported with all transaction details for regulatory reporting.

2. **Given** a user exports sensitive data, **When** the system records a data export event with actor, data types, and destination, **Then** compliance officers can trace all data exports.

3. **Given** configuration changes must be tracked, **When** an administrator modifies system configuration, **Then** the change is recorded with before/after state and the identity of the person making the change.

---

### User Story 3 - Export Audit Data to External Systems (Priority: P2)

A **compliance officer or security analyst** needs to export audit data in standard formats to feed into SIEM systems, compliance platforms, and security analytics tools.

**Why this priority**: Audit data must often be centralized in enterprise security infrastructure (Splunk, Elastic, Azure Sentinel, AWS Security Hub) for correlation with other security events.

**Independent Test**: Can be fully tested by configuring an exporter, generating audit events, and verifying they appear in the external system in the correct format.

**Acceptance Scenarios**:

1. **Given** audit events have been recorded, **When** a compliance officer configures a JSON file exporter and triggers an export, **Then** all events are written to a file in valid JSON format suitable for ingestion by SIEM systems.

2. **Given** real-time audit streaming is required, **When** an exporter is configured to stream events to a remote endpoint, **Then** events are transmitted with guaranteed delivery semantics.

3. **Given** different downstream systems require different formats, **When** multiple exporters are configured with different formats, **Then** each exporter receives events in its configured format simultaneously.

---

### User Story 4 - Ensure Audit Record Integrity (Priority: P1)

A **security architect** needs assurance that audit records cannot be altered or deleted after creation to support legal proceedings, regulatory audits, and forensic investigations.

**Why this priority**: Audit logs often serve as legal evidence. If records can be modified, their value as evidence is destroyed. Many compliance frameworks (SOC 2, ISO 27001, PCI-DSS) require immutable audit trails.

**Independent Test**: Can be tested by writing an audit record, attempting to modify it through any available API, and verifying the original record is unchanged.

**Acceptance Scenarios**:

1. **Given** an audit record has been written, **When** any code attempts to update or delete that record via the audit API, **Then** the operation fails and the original record remains intact.

2. **Given** an integrity verification is requested, **When** an auditor requests cryptographic proof of record integrity, **Then** the system provides verification data confirming no records have been altered.

---

### User Story 5 - Correlate Events with Context (Priority: P3)

A **developer** needs to propagate application context (request ID, correlation ID, session context) through their application so that related audit events can be correlated during investigation.

**Why this priority**: In distributed systems, a single user action may generate multiple audit events across different services. Correlation IDs enable security analysts to reconstruct complete event chains.

**Independent Test**: Can be tested by setting context (correlation ID), generating multiple events, and verifying all related events contain the correlation ID.

**Acceptance Scenarios**:

1. **Given** a correlation ID has been set in the application context, **When** audit events are recorded during request processing, **Then** each event includes the correlation ID for later correlation.

2. **Given** a security analyst investigates an incident, **When** they consume events by correlation ID, **Then** all events from the same user session or request chain are delivered together.

---

### Edge Cases

- **What happens when the audit subsystem is unavailable?**: The system must have a defined behavior (fail-fast, queue locally, or continue without audit) that is configurable and documented. Default should be fail-fast for compliance-sensitive deployments.

- **How does the system handle extremely high event volumes?**: The system must handle burst loads (sudden spikes in events) without data loss or unacceptable latency degradation on the primary application.

- **What happens when event payload exceeds configured limits?**: Events with oversized payloads should be handled gracefully, with clear error reporting and optional truncation policies.

- **How are duplicate events handled?**: The system must provide idempotent event recording (same event ID = same result) to support retry scenarios in distributed systems.

- **What happens when exporter destinations become unavailable?**: Exporters must handle destination failures gracefully with configurable retry policies and buffering strategies.

- **What happens when the internal event queue is full?**: The system must apply a configurable overflow policy (block, drop newest, drop oldest) to handle backpressure.

- **What happens when sensitive data is included in metadata?**: The system must support field-level redaction for sensitive values (passwords, tokens, secrets).

## Requirements _(mandatory)_

### Functional Requirements

#### Core Event Model

- **FR-001**: The system MUST provide a dedicated audit event type (AuditEvent) with fields for: event_id, timestamp (UTC, precise to millisecond), actor, action, target, outcome, source, metadata, correlation_id, classification, compliance_metadata, integrity_hash.

- **FR-001a**: The system MUST ensure AuditEvent is completely immutable: all fields are private, there are no setters, and only AuditEventBuilder can construct instances. No mutation is permitted after construction.

- **FR-002**: The system MUST define an AuditActor struct with: id (unique identifier), kind (enum: User, Service, ApiKey, System, Anonymous), and optional display_name.

- **FR-003**: The system MUST define an AuditTarget struct with: id (unique identifier), kind (type of resource), and optional name.

- **FR-004**: The system MUST define AuditOutcome as an enum with: Success, Failure, Denied. The "Denied" outcome is critical for security events where access was explicitly refused.

- **FR-005**: The system MUST define AuditClassification as an enum with: Security, Compliance, Business, Administrative, System.

- **FR-006**: The system MUST define ComplianceMetadata with: classification, retention_class, and optional jurisdiction.

#### Architecture

- **FR-007**: The system MUST implement an internal architecture with the following layered components: AuditLogger (public API), AuditPipeline (coordinates processing), AuditProcessor (transforms and validates), and Exporter(s) (delivers to destinations).

- **FR-008**: The system MUST provide a destination abstraction (AuditSink trait) for streaming events to external systems. Implementations: ConsoleSink, JsonSink, FileSink, SyslogSink, etc.

> **Note**: Persistent storage (AuditStore) and query functionality moved to KIT-012.

- **FR-009**: The system MUST provide an ExporterRegistry for dynamic registration and management of multiple exporters.

- **FR-010**: The system MUST implement the following exporters as **mandatory** (MVP requirement):
  - **AuditConsoleExporter**: For development and debugging.
  - **AuditJsonExporter**: For machine-readable export.
  - **AuditFileExporter**: For streaming events to files.
  
  The following exporters are **optional** and future work (KIT-013+):
  - AuditSyslogExporter (RFC 5424)
  - AuditCefExporter (Common Event Format)
  - AuditHttpExporter (custom endpoints)

#### Event Building API

- **FR-011**: The system MUST provide a builder pattern (AuditEvent::builder()) for ergonomic event creation:
  ```
  AuditEvent::builder()
      .actor(...)
      .action(...)
      .target(...)
      .outcome(...)
      .build()
  ```

#### Identifier Strategy

- **FR-012**: The system MUST use UUID v7 (time-ordered UUID) for event_id to ensure temporal ordering and uniqueness.

#### Deterministic Serialization

- **FR-013**: The system MUST generate deterministic serialization of audit events - given the same event data, the serialized output must be byte-for-byte identical. This requires stable field ordering and use of ordered maps (e.g., BTreeMap) for metadata.

- **FR-014**: The system MUST use BTreeMap<String, AuditValue> for metadata to ensure consistent serialization ordering. The AuditValue enum MUST be defined as:
  ```
  enum AuditValue {
      String(String),
      Number(f64),
      Bool(bool),
      Array(Vec<AuditValue>),
      Object(BTreeMap<String, AuditValue>)
  }
  ```
  This replaces serde_json::Value to ensure deterministic serialization without subtle differences.

#### Context Propagation

- **FR-015**: The system MUST automatically propagate the following context from Kit Logger's LoggerContext: trace_id, correlation_id, tenant_id. These fields are copied to audit events automatically when present in the context.

- **FR-015a**: The system MUST integrate with HTTP middleware context propagation (KIT-009). When AuditLogger is used in HTTP request handlers, the following fields MUST be automatically captured: trace_id, correlation_id, request_id, tenant_id. This requires explicit integration with KIT-009's HTTP middleware.

#### Redaction

- **FR-016**: The system MUST support field-level redaction for sensitive values in metadata. By default, the following fields are redacted: password, secret, token, access_token, refresh_token, authorization, api_key.

#### Immutability

- **FR-017**: The system MUST guarantee immutability of audit records after creation - no update or delete operations are permitted on stored audit events.

#### Integrity Provider (Optional)

- **FR-018**: The system MUST provide an IntegrityProvider trait for cryptographic integrity verification:
  ```
  trait IntegrityProvider {
      fn initialize() -> Self;
      fn compute_hash(&self, event_payload: &str, previous_hash: Option<&str>) -> String;
      fn verify(&self, event_payload: &str, stored_hash: &str, previous_hash: Option<&str>) -> bool;
  }
  ```
  
  The system MUST provide the following built-in implementations:
  - **NoopIntegrityProvider**: No-op implementation for performance testing.
  - **HashChainIntegrityProvider**: SHA256-based hash chain: SHA256(event_payload + previous_event_hash).
  
  Future implementations may include MerkleTreeIntegrityProvider, BlockchainAnchorIntegrityProvider, or ExternalNotaryIntegrityProvider without modifying AuditEvent.

#### Query Interface

> Query capabilities have been removed from scope. See KIT-012 (Audit Storage & Query).

#### Export and Batching

- **FR-020**: The system MUST allow configuration of multiple exporters simultaneously, each with its own format and destination.

- **FR-021**: The system MUST implement batch export capabilities (AuditBatch) supporting batched writes to exporters for high-throughput scenarios (10,000+ events/second).

- **FR-022**: The system MUST provide structured, machine-readable event formats (JSON is required; CEF, Syslog, or other standard formats are recommended).

- **FR-023**: The system MUST support filtering and routing of events to different exporters based on event attributes (e.g., security events to SIEM, business events to data warehouse).

#### Backpressure

- **FR-024**: The system MUST define an OverflowPolicy enum with: Block (wait until space available), DropNewest (discard new events), DropOldest (drop oldest buffered events). This handles scenarios where exporters (Elastic, Splunk) become unavailable.

#### Performance

- **FR-025**: The system MUST provide high-performance operation that does not significantly impact the performance of the recording application (target: <10ms overhead per event under normal load).

- **FR-026**: The system MUST support writing at least **10,000 audit events per second sustained**, with burst capability of **100,000 events per second** for short durations (up to 30 seconds).

> Note: 100k+ sustained events/s requires lock-free algorithms, aggressive batching, and multiple worker threads. This is marked as a stretch goal.

#### Event Categories

- **FR-027**: The system MUST record the following event categories: authentication (login, logout, authentication failures), authorization (permission grants/revocations, role assignments), data access (reads of sensitive data), data modification (creates, updates, deletes), configuration changes, and administrative actions.

#### Separation from Operational Logs

- **FR-028**: The system MUST separate audit logging from operational/application logging - audit events are not mixed with operational log streams.

#### Retention Policies (Moved to KIT-012)

- **FR-029**: [Moved to KIT-012] Event retention policies moved to KIT-012 Storage & Query.

### Key Entities

- **AuditEvent**: The core record representing a single auditable event. **All fields are private and immutable after construction.** Contains: event_id (UUID v7), timestamp (UTC datetime), actor (AuditActor), action (string, verb + object pattern, e.g., "user.login"), target (AuditTarget), outcome (AuditOutcome enum), source (origin service/component), metadata (BTreeMap<String, AuditValue>), correlation_id (optional), classification (AuditClassification), compliance_metadata (ComplianceMetadata), integrity_hash (optional, computed by IntegrityProvider).

- **AuditValue**: Enum for deterministic metadata values. Variants: String(String), Number(f64), Bool(bool), Array(Vec<AuditValue>), Object(BTreeMap<String, AuditValue>). Replaces serde_json::Value for deterministic serialization.

- **AuditActor**: Represents the entity performing an action. Contains: id (unique identifier), kind (ActorKind enum: User, Service, ApiKey, System, Anonymous), display_name (optional human-readable name). Private fields, constructed via AuditEventBuilder.

- **AuditTarget**: Represents the resource being acted upon. Contains: id (unique identifier), kind (string describing resource type, e.g., "user", "order", "document"), name (optional human-readable name).

- **AuditOutcome**: Enum with variants: Success (action completed successfully), Failure (action failed due to error), Denied (action was explicitly denied, critical for security).

- **AuditClassification**: Enum with variants: Security (security-relevant events), Compliance (regulatory compliance events), Business (business-critical events), Administrative (admin actions), System (system-level events).

- **ComplianceMetadata**: Contains: classification (string), retention_class (string, e.g., "7years", "permanent"), jurisdiction (optional, for data sovereignty).

- **AuditLogger**: Public API entry point for recording audit events. Provides: record(event), builder(), context().

- **AuditPipeline**: Internal component that coordinates the flow of events from Logger to Processors and Exporters.

- **AuditProcessor**: Internal component that transforms, validates, and enriches events before export.

- **AuditSink**: Destination abstraction for delivering audit events to external systems. This trait replaces the previous storage abstraction to clearly indicate that KIT-011 focuses on event delivery, not persistent storage.

  **Implementations (MVP)**: ConsoleSink, JsonSink, FileSink

  **Future implementations**: SyslogSink, HttpSink, CefSink

  > **Note**: Persistent storage is intentionally outside the scope of KIT-011 and is defined in KIT-012.

- **AuditExporter**: Trait for exporting events. Implementations handle format conversion and delivery to destinations.

- **AuditConsoleExporter**: Mandatory console exporter for development/debugging. Outputs audit events to console in a readable format.

- **AuditJsonExporter**: Mandatory JSON exporter for machine-readable output.

- **AuditFileExporter**: Mandatory file exporter for streaming events to files.

- **IntegrityProvider**: Trait for cryptographic integrity verification: initialize(), compute_hash(), verify(). Implementations: NoopIntegrityProvider, HashChainIntegrityProvider.

- **AuditConfig**: Configuration structure for AuditLogger:
  ```
  struct AuditConfig {
      enabled: bool,
      overflow_policy: OverflowPolicy,
      batch_size: usize,
      flush_interval: Duration,
      redaction_enabled: bool,
      integrity_provider: Box<dyn IntegrityProvider>,
      exporters: Vec<Box<dyn AuditExporter>>,
  }
  ```

- **AuditBatch**: Collection of events to be exported together. Supports batched delivery for high-throughput scenarios.

- **OverflowPolicy**: Enum defining backpressure behavior: Block, DropNewest, DropOldest.

- **ExporterRegistry**: Central registry for managing multiple exporters with dynamic registration.

### Acceptance Criteria

| ID | Criterion | Validation Method |
|----|-----------|-------------------|
| AC-001 | AuditEvent is immutable after creation | Attempt update/delete via API; operations must fail |
| AC-002 | event_id uses UUID v7 | Verify UUID structure includes timestamp component |
| AC-003 | JSON serialization is deterministic | Serialize same event twice; byte-for-byte identical |
| AC-004 | trace_id, correlation_id, tenant_id propagate from LoggerContext | Set context, record event, verify fields present |
| AC-005 | Multiple exporters work simultaneously | Configure JSON + Console exporters; verify both receive events |
| AC-006 | Sensitive fields are redacted by default | Include "password" in metadata; verify "*****" in output |
| AC-007 | Compliance metadata is stored | Add classification/retention; verify on retrieval |
| AC-008 | Hash chain is implementable (optional) | Enable hash chain; verify chain integrity |
| AC-009 | [Moved to KIT-012] Query interface | Query functionality moved to KIT-012 |
| AC-010 | Batch export handles 10k sustained / 100k burst | Generate 10k+ events; verify throughput matches target |
| AC-011 | Zero data corruption under load | High-volume test; verify event integrity |
| AC-012 | AuditConsoleExporter is available | Instantiate exporter; verify output format |

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: Developers can record an audit event with all required fields in under 10 milliseconds on average hardware (P95 < 25ms).

- **SC-002**: The system supports writing at least 10,000 audit events per second sustained, and 100,000 events per second in burst scenarios (up to 30 seconds) without event loss.

- **SC-003**: 100% of written audit records remain immutable - zero successful update or delete operations are possible on stored records.

- **SC-004**: [Partial - KIT-012] Single event retrieval by ID: <2 seconds. Complex queries moved to KIT-012.

- **SC-005**: Exported events arrive at configured destinations with < 1 second latency under normal operating conditions.

- **SC-006**: The system exports to: JSON (machine-readable mandatory), Console (human-readable mandatory), plus optional JSONL/CEF/Syslog formats. All exporters operate simultaneously without data loss.

- **SC-007**: [Moved to KIT-012] Compliance reporting and retention policies. Full compliance reporting functionality moved to KIT-012.

- **SC-008**: Zero security events are lost during exporter destination failures lasting up to 1 hour (using configured buffering).

- **SC-009**: Deterministic serialization produces byte-for-byte identical output for the same event content across multiple serializations.

## Assumptions

- The calling application is responsible for providing accurate actor identity information; the audit system does not authenticate users itself.
- Audit events will be stored locally initially; long-term retention may involve archival to separate storage systems.
- Network latency to export destinations is outside the control of the audit system but will be within typical data center networks (<50ms RTT).
- Regulatory requirements vary by jurisdiction; the system provides flexibility in event capture and export rather than enforcing specific compliance frameworks.
- The system integrates with Kit Logger's existing LoggerContext for context propagation.
- UUID v7 library support is available in the target language/framework.

## Dependencies

- This feature relies on the presence of a logging foundation (Kit Logger core) to build upon.
- The system requires LoggerContext from Kit Logger for context propagation (trace_id, correlation_id, tenant_id).
- **This feature MUST integrate with KIT-009 HTTP middleware for context propagation** (trace_id, correlation_id, request_id, tenant_id must be automatically captured from HTTP request headers).
- No specific destination technology is assumed. Audit events may be delivered to files, HTTP endpoints, message brokers, SIEM systems, or custom sinks. Persistent storage is covered by KIT-012.
- **Query interface and storage have been moved to KIT-012**. This spec (KIT-011) covers only capture, processing, and export.
