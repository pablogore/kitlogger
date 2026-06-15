//! Trace Context implementation for W3C Trace Context propagation
//!
//! This module implements the W3C Trace Context specification for
//! propagating trace information across service boundaries.

use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// A 16-byte trace identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraceId([u8; 16]);

impl TraceId {
    /// Create a new TraceId from bytes
    pub fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Create a new TraceId from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(uuid.as_bytes());
        Self(bytes)
    }

    /// Get the bytes of the trace ID
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Check if this is a zero trace ID
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl FromStr for TraceId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 32 {
            return Err("TraceId must be 32 characters".to_string());
        }

        let mut bytes = [0u8; 16];
        for i in 0..16 {
            let hex_pair = &s[i * 2..i * 2 + 2];
            bytes[i] = u8::from_str_radix(hex_pair, 16)
                .map_err(|_| format!("Invalid hex character in TraceId: {}", hex_pair))?;
        }

        Ok(Self(bytes))
    }
}

/// A 8-byte span identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpanId([u8; 8]);

impl SpanId {
    /// Create a new SpanId from bytes
    pub fn new(bytes: [u8; 8]) -> Self {
        Self(bytes)
    }

    /// Get the bytes of the span ID
    pub fn as_bytes(&self) -> &[u8; 8] {
        &self.0
    }

    /// Check if this is a zero span ID
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl fmt::Display for SpanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl FromStr for SpanId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 16 {
            return Err("SpanId must be 16 characters".to_string());
        }

        let mut bytes = [0u8; 8];
        for i in 0..8 {
            let hex_pair = &s[i * 2..i * 2 + 2];
            bytes[i] = u8::from_str_radix(hex_pair, 16)
                .map_err(|_| format!("Invalid hex character in SpanId: {}", hex_pair))?;
        }

        Ok(Self(bytes))
    }
}

/// Trace flags with sampling information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraceFlags(u8);

impl TraceFlags {
    /// Create new trace flags
    pub fn new(flags: u8) -> Self {
        Self(flags)
    }

    /// Get the raw flags value
    pub fn as_u8(&self) -> u8 {
        self.0
    }

    /// Check if the sampled flag is set
    pub fn is_sampled(&self) -> bool {
        self.0 & 0x01 != 0
    }

    /// Set the sampled flag
    pub fn set_sampled(&mut self, sampled: bool) {
        if sampled {
            self.0 |= 0x01;
        } else {
            self.0 &= 0xFE;
        }
    }
}

impl fmt::Display for TraceFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02x}", self.0)
    }
}

impl fmt::LowerHex for TraceFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02x}", self.0)
    }
}

impl FromStr for TraceFlags {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u8::from_str_radix(s, 16)
            .map(TraceFlags::new)
            .map_err(|_| "Invalid hex string for TraceFlags".to_string())
    }
}

/// Trace state for vendor-specific data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceState {
    /// Vendor entries in the trace state
    entries: Vec<(String, String)>,
}

impl TraceState {
    /// Maximum number of vendor entries allowed
    pub const MAX_ENTRIES: usize = 32;
}

impl TraceState {
    /// Create a new empty TraceState
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add a vendor entry to the trace state
    pub fn add(&mut self, key: &str, value: &str) -> Result<(), String> {
        if self.entries.len() >= Self::MAX_ENTRIES {
            return Err("Maximum number of trace state entries exceeded".to_string());
        }

        // Validate key and value
        if key.is_empty() || key.len() > 256 {
            return Err("Invalid key length".to_string());
        }

        if value.len() > 256 {
            return Err("Invalid value length".to_string());
        }

        self.entries.push((key.to_string(), value.to_string()));
        Ok(())
    }

    /// Get all entries
    pub fn entries(&self) -> &[(String, String)] {
        &self.entries
    }
}

impl Default for TraceState {
    fn default() -> Self {
        Self::new()
    }
}

/// Trace context containing all trace information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceContext {
    /// Version of the trace context
    pub version: u8,

    /// Trace identifier
    pub trace_id: TraceId,

    /// Span identifier
    pub span_id: SpanId,

    /// Parent span identifier (optional)
    pub parent_span_id: Option<SpanId>,

    /// Trace flags
    pub trace_flags: TraceFlags,

    /// Trace state
    pub trace_state: TraceState,
}

impl TraceContext {
    /// Create a new trace context
    pub fn new(
        trace_id: TraceId,
        span_id: SpanId,
        parent_span_id: Option<SpanId>,
        trace_flags: TraceFlags,
        trace_state: TraceState,
    ) -> Self {
        Self {
            version: 0,
            trace_id,
            span_id,
            parent_span_id,
            trace_flags,
            trace_state,
        }
    }

    /// Check if this trace context is valid
    pub fn is_valid(&self) -> bool {
        // Version must not be 0xFF
        if self.version == 0xFF {
            return false;
        }

        // Trace ID and Span ID must not be all zeros
        if self.trace_id.is_zero() || self.span_id.is_zero() {
            return false;
        }

        // Parent span ID must not be all zeros if present
        if let Some(parent) = self.parent_span_id {
            if parent.is_zero() {
                return false;
            }
        }

        true
    }
}

impl fmt::Display for TraceContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}-{}-{}-{:02x}",
            self.version, self.trace_id, self.span_id, self.trace_flags
        )
    }
}

impl FromStr for TraceContext {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Parse the traceparent format: version-trace-id-span-id-trace-flags
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 4 {
            return Err("Invalid traceparent format".to_string());
        }

        let version = u8::from_str_radix(parts[0], 16)
            .map_err(|_| "Invalid version in traceparent".to_string())?;

        if version == 0xFF {
            return Err("Invalid version in traceparent".to_string());
        }

        let trace_id = TraceId::from_str(parts[1])
            .map_err(|e| format!("Invalid trace_id in traceparent: {}", e))?;

        let span_id = SpanId::from_str(parts[2])
            .map_err(|e| format!("Invalid span_id in traceparent: {}", e))?;

        let trace_flags = u8::from_str_radix(parts[3], 16)
            .map(TraceFlags::new)
            .map_err(|_| "Invalid trace_flags in traceparent".to_string())?;

        // For now, we'll create a minimal trace state
        let trace_state = TraceState::new();

        Ok(TraceContext {
            version,
            trace_id,
            span_id,
            parent_span_id: None,
            trace_flags,
            trace_state,
        })
    }
}
