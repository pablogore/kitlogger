use std::collections::HashMap;

/// Injector trait for setting key-value pairs in a carrier
pub trait Injector {
    fn set(&mut self, key: &str, value: &str);
}

/// Extractor trait for getting values from a carrier
pub trait Extractor {
    fn get(&self, key: &str) -> Option<&str>;
    fn get_all(&self, key: &str) -> Vec<&str>;
}

/// Propagator trait for injecting and extracting context from carriers
pub trait Propagator {
    type Context;

    fn inject(&self, carrier: &mut dyn Injector, context: &Self::Context);

    /// Extract context from a carrier.
    /// Returns None when no valid context data is present in the carrier.
    fn extract(&self, carrier: &dyn Extractor) -> Option<Self::Context>;

    fn fields(&self) -> &'static [&'static str];
}

/// A HashMap-based carrier implementation
pub struct MapCarrier {
    data: HashMap<String, Vec<String>>,
}

impl MapCarrier {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.data.keys()
    }

    pub fn get_values(&self, key: &str) -> Vec<&String> {
        self.data
            .get(key)
            .map_or(Vec::new(), |v| v.iter().collect())
    }
}

impl Default for MapCarrier {
    fn default() -> Self {
        Self::new()
    }
}

impl Injector for MapCarrier {
    fn set(&mut self, key: &str, value: &str) {
        self.data
            .entry(key.to_string())
            .or_insert_with(Vec::new)
            .push(value.to_string());
    }
}

impl Extractor for MapCarrier {
    fn get(&self, key: &str) -> Option<&str> {
        self.data
            .get(key)
            .and_then(|v| v.first().map(|s| s.as_str()))
    }

    fn get_all(&self, key: &str) -> Vec<&str> {
        self.data
            .get(key)
            .map_or(Vec::new(), |v| v.iter().map(|s| s.as_str()).collect())
    }
}

// Transport-specific carrier implementations (HttpHeaderCarrier, GrpcMetadataCarrier)
// are owned by AS-02 (transport bindings), not AS-01 (context propagation).
