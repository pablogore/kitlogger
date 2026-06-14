use std::collections::HashMap;

/// Injector trait for setting key-value pairs in a carrier
pub trait Injector {
    /// Set a key-value pair in the carrier
    fn set(&mut self, key: &str, value: &str);
}

/// Extractor trait for getting values from a carrier
pub trait Extractor {
    /// Get the first value for a key from the carrier
    fn get(&self, key: &str) -> Option<&str>;
    
    /// Get all values for a key from the carrier
    fn get_all(&self, key: &str) -> Vec<&str>;
}

/// Propagator trait for injecting and extracting context from carriers
pub trait Propagator {
    /// Context type associated with this propagator
    type Context;
    
    /// Inject context into a carrier
    fn inject(&self, carrier: &mut dyn Injector, context: &Self::Context);
    
    /// Extract context from a carrier
    fn extract(&self, carrier: &dyn Extractor) -> Self::Context;
    
    /// Get the fields that this propagator uses
    fn fields(&self) -> &'static [&'static str];
}

/// A HashMap-based carrier implementation
pub struct MapCarrier {
    data: HashMap<String, Vec<String>>,
}

impl MapCarrier {
    /// Create a new empty MapCarrier
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    
    /// Get all keys in the carrier
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.data.keys()
    }
    
    /// Get all values for a specific key
    pub fn get_values(&self, key: &str) -> Vec<&String> {
        self.data.get(key).map_or(Vec::new(), |v| v.iter().collect())
    }
}

impl Default for MapCarrier {
    fn default() -> Self {
        Self::new()
    }
}

impl Injector for MapCarrier {
    fn set(&mut self, key: &str, value: &str) {
        self.data.entry(key.to_string()).or_insert_with(Vec::new).push(value.to_string());
    }
}

impl Extractor for MapCarrier {
    fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).and_then(|v| v.first().map(|s| s.as_str()))
    }
    
    fn get_all(&self, key: &str) -> Vec<&str> {
        self.data.get(key).map_or(Vec::new(), |v| v.iter().map(|s| s.as_str()).collect())
    }
}

/// HTTP header carrier implementation
pub struct HttpHeaderCarrier<'a> {
    headers: &'a mut dyn Injector,
}

impl<'a> HttpHeaderCarrier<'a> {
    /// Create a new HttpHeaderCarrier
    pub fn new(headers: &'a mut dyn Injector) -> Self {
        Self { headers }
    }
}

impl<'a> Injector for HttpHeaderCarrier<'a> {
    fn set(&mut self, key: &str, value: &str) {
        self.headers.set(key, value);
    }
}

impl<'a> Extractor for HttpHeaderCarrier<'a> {
    fn get(&self, _key: &str) -> Option<&str> {
        // This is a simplified approach - in a real implementation,
        // this would be a wrapper around a real HTTP header structure
        // For testing purposes, we'll just return None
        None
    }
    
    fn get_all(&self, _key: &str) -> Vec<&str> {
        // This is a simplified approach - in a real implementation,
        // this would be a wrapper around a real HTTP header structure
        Vec::new()
    }
}

/// gRPC metadata carrier implementation
pub struct GrpcMetadataCarrier<'a> {
    metadata: &'a mut dyn Injector,
}

impl<'a> GrpcMetadataCarrier<'a> {
    /// Create a new GrpcMetadataCarrier
    pub fn new(metadata: &'a mut dyn Injector) -> Self {
        Self { metadata }
    }
}

impl<'a> Injector for GrpcMetadataCarrier<'a> {
    fn set(&mut self, key: &str, value: &str) {
        self.metadata.set(key, value);
    }
}

impl<'a> Extractor for GrpcMetadataCarrier<'a> {
    fn get(&self, _key: &str) -> Option<&str> {
        // This is a simplified approach - in a real implementation,
        // this would be a wrapper around a real gRPC metadata structure
        // For testing purposes, we'll just return None
        None
    }
    
    fn get_all(&self, _key: &str) -> Vec<&str> {
        // This is a simplified approach - in a real implementation,
        // this would be a wrapper around a real gRPC metadata structure
        Vec::new()
    }
}

/// Result type alias for propagator operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;