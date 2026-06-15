/// Transport-specific metadata required for context carriage.
///
/// PropagationMetadata captures the transport binding details needed to
/// carry telemetry context across execution boundaries. Each transport
/// (HTTP, gRPC, messaging) defines its own metadata describing how context
/// headers or fields are formatted, encoded, and transmitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropagationMetadata {
    /// Transport protocol name (e.g., "http", "grpc", "kafka")
    pub transport: String,
    /// Key-value metadata entries for the transport binding
    pub entries: Vec<(String, String)>,
}

impl PropagationMetadata {
    pub fn new(transport: &str) -> Self {
        Self {
            transport: transport.to_string(),
            entries: Vec::new(),
        }
    }

    pub fn add(&mut self, key: &str, value: &str) {
        self.entries.push((key.to_string(), value.to_string()));
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.entries.iter().map(|(k, _)| k)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for PropagationMetadata {
    fn default() -> Self {
        Self::new("unknown")
    }
}
