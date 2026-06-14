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

/// Result type alias for propagator operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;