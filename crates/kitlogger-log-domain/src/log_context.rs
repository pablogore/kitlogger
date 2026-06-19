//! Log context for structured logging.
//!
//! This module provides the LogContext type, which carries contextual information
//! that can be enriched onto log records.

use crate::{
    log_attribute::LogAttribute, log_attribute_value::LogAttributeValue,
    validation::ValidationError, CorrelationId, SpanId, TraceId,
};
use std::fmt::{Display, Formatter, Result as FmtResult};
/// A context that can be enriched onto log records.
///
/// LogContext carries metadata that can be added to log records to provide
/// additional information about the execution context.
///
/// Enrichment methods return new `Self` instances — the original is never mutated.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LogContext {
    /// The attributes that make up this context.
    attributes: Vec<LogAttribute>,
    /// Optional correlation identifier for cross-service correlation.
    correlation_id: Option<CorrelationId>,
    /// Optional trace identifier for distributed trace association.
    trace_id: Option<TraceId>,
    /// Optional span identifier for span-level identification.
    span_id: Option<SpanId>,
}

impl LogContext {
    /// Creates a new, empty LogContext.
    ///
    /// The returned context has no attributes and no identifiers set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a reference to the attributes in this context.
    pub fn attributes(&self) -> &[LogAttribute] {
        &self.attributes
    }

    /// Returns the correlation identifier, if set.
    pub fn correlation_id(&self) -> Option<&CorrelationId> {
        self.correlation_id.as_ref()
    }

    /// Returns the trace identifier, if set.
    pub fn trace_id(&self) -> Option<&TraceId> {
        self.trace_id.as_ref()
    }

    /// Returns the span identifier, if set.
    pub fn span_id(&self) -> Option<&SpanId> {
        self.span_id.as_ref()
    }

    /// Adds an attribute to this context, mutating it in place.
    ///
    /// # Errors
    ///
    /// Returns `ValidationError::EnrichmentError` if the attribute name already exists.
    pub fn add_attribute(&mut self, attribute: LogAttribute) -> Result<(), ValidationError> {
        if self
            .attributes
            .iter()
            .any(|attr| attr.name() == attribute.name())
        {
            return Err(ValidationError::EnrichmentError(format!(
                "Duplicate attribute name: {}",
                attribute.name()
            )));
        }
        self.attributes.push(attribute);
        Ok(())
    }

    /// Creates a new LogContext with the given attribute added.
    ///
    /// The original context is not modified.
    ///
    /// # Errors
    ///
    /// Returns `ValidationError::EnrichmentError` if the attribute name already exists.
    pub fn with_attribute(&self, attribute: LogAttribute) -> Result<Self, ValidationError> {
        if self
            .attributes
            .iter()
            .any(|attr| attr.name() == attribute.name())
        {
            return Err(ValidationError::EnrichmentError(format!(
                "Duplicate attribute name: {}",
                attribute.name()
            )));
        }
        let mut new = self.clone();
        new.attributes.push(attribute);
        Ok(new)
    }

    /// Creates a new LogContext with the correlation identifier set.
    ///
    /// If a correlation identifier was already set, it is replaced (last-wins).
    /// The original context is not modified.
    pub fn with_correlation_id(&self, id: CorrelationId) -> Result<Self, ValidationError> {
        let mut new = self.clone();
        new.correlation_id = Some(id);
        Ok(new)
    }

    /// Creates a new LogContext with the trace identifier set.
    ///
    /// If a trace identifier was already set, it is replaced (last-wins).
    /// The original context is not modified.
    pub fn with_trace_id(&self, id: TraceId) -> Result<Self, ValidationError> {
        let mut new = self.clone();
        new.trace_id = Some(id);
        Ok(new)
    }

    /// Creates a new LogContext with the span identifier set.
    ///
    /// If a span identifier was already set, it is replaced (last-wins).
    /// The original context is not modified.
    pub fn with_span_id(&self, id: SpanId) -> Result<Self, ValidationError> {
        let mut new = self.clone();
        new.span_id = Some(id);
        Ok(new)
    }
}

impl Display for LogContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut parts: Vec<String> = Vec::new();

        for attr in &self.attributes {
            let value = match attr.value() {
                LogAttributeValue::String(s) => format!("\"{}\"", s),
                LogAttributeValue::Integer(n) => n.to_string(),
                LogAttributeValue::Float(n) => n.to_string(),
                LogAttributeValue::Boolean(b) => b.to_string(),
                LogAttributeValue::Timestamp(t) => format!("{:?}", t),
                LogAttributeValue::Array(a) => format!("[{} items]", a.len()),
            };
            parts.push(format!("{}: {}", attr.name(), value));
        }
        if let Some(cid) = &self.correlation_id {
            parts.push(format!("correlation_id: {}", cid));
        }
        if let Some(tid) = &self.trace_id {
            parts.push(format!("trace_id: {}", tid));
        }
        if let Some(sid) = &self.span_id {
            parts.push(format!("span_id: {}", sid));
        }

        if parts.is_empty() {
            write!(f, "LogContext {{}}")
        } else {
            write!(f, "LogContext {{ {} }}", parts.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log_attribute_value::LogAttributeValue;

    #[test]
    fn test_new_log_context() {
        let context = LogContext::new();
        assert_eq!(context.attributes().len(), 0);
        assert!(context.correlation_id().is_none());
        assert!(context.trace_id().is_none());
        assert!(context.span_id().is_none());
    }

    #[test]
    fn test_default_log_context() {
        let context = LogContext::default();
        assert_eq!(context.attributes().len(), 0);
        assert!(context.correlation_id().is_none());
        assert!(context.trace_id().is_none());
        assert!(context.span_id().is_none());
    }

    #[test]
    fn test_add_attribute() {
        let mut context = LogContext::new();
        let attr = LogAttribute::new(
            "test_key".to_string(),
            LogAttributeValue::String("test_value".to_string()),
        )
        .unwrap();

        assert!(context.add_attribute(attr).is_ok());
        assert_eq!(context.attributes().len(), 1);
    }

    #[test]
    fn test_add_duplicate_attribute() {
        let mut context = LogContext::new();
        let attr1 = LogAttribute::new(
            "test_key".to_string(),
            LogAttributeValue::String("test_value1".to_string()),
        )
        .unwrap();
        let attr2 = LogAttribute::new(
            "test_key".to_string(),
            LogAttributeValue::String("test_value2".to_string()),
        )
        .unwrap();

        assert!(context.add_attribute(attr1).is_ok());
        assert!(context.add_attribute(attr2).is_err());
    }

    #[test]
    fn test_with_attribute() {
        let context = LogContext::new();
        let attr = LogAttribute::new(
            "test_key".to_string(),
            LogAttributeValue::String("test_value".to_string()),
        )
        .unwrap();

        let enriched = context.with_attribute(attr).unwrap();
        assert_eq!(enriched.attributes().len(), 1);
        // Original unchanged
        assert_eq!(context.attributes().len(), 0);
    }

    #[test]
    fn test_with_duplicate_attribute() {
        let context = LogContext::new();
        let attr1 = LogAttribute::new(
            "test_key".to_string(),
            LogAttributeValue::String("test_value1".to_string()),
        )
        .unwrap();
        let attr2 = LogAttribute::new(
            "test_key".to_string(),
            LogAttributeValue::String("test_value2".to_string()),
        )
        .unwrap();

        let enriched = context.with_attribute(attr1).unwrap();
        assert!(enriched.with_attribute(attr2).is_err());
    }

    #[test]
    fn test_with_correlation_id() {
        let context = LogContext::new();
        let cid = CorrelationId::new("req-123".to_string());
        let enriched = context.with_correlation_id(cid.clone()).unwrap();

        assert_eq!(enriched.correlation_id(), Some(&cid));
        // Original unchanged
        assert!(context.correlation_id().is_none());
    }

    #[test]
    fn test_with_trace_id() {
        let context = LogContext::new();
        let tid = TraceId::new("trace-456".to_string());
        let enriched = context.with_trace_id(tid.clone()).unwrap();

        assert_eq!(enriched.trace_id(), Some(&tid));
        // Original unchanged
        assert!(context.trace_id().is_none());
    }

    #[test]
    fn test_with_span_id() {
        let context = LogContext::new();
        let sid = SpanId::new("span-789".to_string());
        let enriched = context.with_span_id(sid.clone()).unwrap();

        assert_eq!(enriched.span_id(), Some(&sid));
        // Original unchanged
        assert!(context.span_id().is_none());
    }

    #[test]
    fn test_single_context_all_ids() {
        let context = LogContext::new();
        let cid = CorrelationId::new("req-1".to_string());
        let tid = TraceId::new("trace-abc".to_string());
        let sid = SpanId::new("span-42".to_string());

        let enriched = context
            .with_correlation_id(cid.clone())
            .unwrap()
            .with_trace_id(tid.clone())
            .unwrap()
            .with_span_id(sid.clone())
            .unwrap();

        assert_eq!(enriched.correlation_id(), Some(&cid));
        assert_eq!(enriched.trace_id(), Some(&tid));
        assert_eq!(enriched.span_id(), Some(&sid));
    }

    #[test]
    fn test_identifier_idempotency_last_wins() {
        let context = LogContext::new();
        let cid1 = CorrelationId::new("req-1".to_string());
        let cid2 = CorrelationId::new("req-2".to_string());

        let enriched = context
            .with_correlation_id(cid1)
            .unwrap()
            .with_correlation_id(cid2.clone())
            .unwrap();

        assert_eq!(enriched.correlation_id(), Some(&cid2));
    }

    #[test]
    fn test_display_empty_context() {
        let context = LogContext::new();
        assert_eq!(format!("{}", context), "LogContext {}");
    }

    #[test]
    fn test_display_context_with_attributes() {
        let mut context = LogContext::new();
        let attr = LogAttribute::new(
            "test_key".to_string(),
            LogAttributeValue::String("test_value".to_string()),
        )
        .unwrap();
        context.add_attribute(attr).unwrap();
        assert_eq!(
            format!("{}", context),
            r#"LogContext { test_key: "test_value" }"#
        );
    }

    #[test]
    fn test_display_context_with_ids() {
        let context = LogContext::new();
        let enriched = context
            .with_correlation_id(CorrelationId::new("req-1".to_string()))
            .unwrap();
        assert_eq!(
            format!("{}", enriched),
            "LogContext { correlation_id: req-1 }"
        );
    }

    #[test]
    fn test_enrichment_immutability() {
        let context = LogContext::new();
        let attr = LogAttribute::new(
            "env".to_string(),
            LogAttributeValue::String("prod".to_string()),
        )
        .unwrap();

        let _enriched = context.with_attribute(attr).unwrap();
        // Original must remain unchanged
        assert_eq!(context.attributes().len(), 0);
    }
}
