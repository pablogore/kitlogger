//! Baggage implementation for W3C Baggage propagation
//!
//! This module implements the W3C Baggage specification for propagating
//! application context across service boundaries.

/// A baggage property (key-value or flag)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaggageProperty {
    /// Key-value property
    KeyValue { key: String, value: String },
    /// Flag property (key only)
    Flag { key: String },
}

/// A single baggage entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaggageEntry {
    /// The key of the baggage entry
    pub key: String,

    /// The value of the baggage entry (optional)
    pub value: Option<String>,

    /// Properties associated with this entry
    pub properties: Vec<BaggageProperty>,
}

impl BaggageEntry {
    /// Create a new baggage entry with key-value
    pub fn new(key: String, value: String) -> Self {
        Self {
            key,
            value: Some(value),
            properties: Vec::new(),
        }
    }

    /// Create a new baggage entry with key only (flag)
    pub fn flag(key: String) -> Self {
        Self {
            key,
            value: None,
            properties: Vec::new(),
        }
    }
}

/// Baggage container for storing multiple baggage entries
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Baggage {
    /// Baggage entries
    entries: Vec<BaggageEntry>,

    /// Total size in bytes
    total_size: usize,
}

impl Baggage {
    /// Maximum number of entries allowed
    pub const MAX_ENTRIES: usize = 180;

    /// Maximum total size in bytes
    pub const MAX_SIZE: usize = 64 * 1024; // 64KB
}

impl Baggage {
    /// Create a new empty baggage
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            total_size: 0,
        }
    }

    /// Add a baggage entry
    pub fn add_entry(&mut self, entry: BaggageEntry) -> Result<(), String> {
        // Check if we've exceeded the maximum number of entries
        if self.entries.len() >= Self::MAX_ENTRIES {
            return Err("Maximum number of baggage entries exceeded".to_string());
        }

        // Calculate the size of this entry
        let entry_size = entry.key.len() + entry.value.as_ref().map_or(0, |v| v.len());

        // Check if we've exceeded the maximum size
        if self.total_size + entry_size > Self::MAX_SIZE {
            return Err("Maximum baggage size exceeded".to_string());
        }

        self.entries.push(entry);
        self.total_size += entry_size;
        Ok(())
    }

    /// Get all entries
    pub fn entries(&self) -> &[BaggageEntry] {
        &self.entries
    }

    /// Get entries by key
    pub fn get(&self, key: &str) -> Option<&BaggageEntry> {
        self.entries.iter().find(|entry| entry.key == *key)
    }

    /// Get all entries with a specific key
    pub fn get_all(&self, key: &str) -> Vec<&BaggageEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.key == *key)
            .collect()
    }
}

impl Default for Baggage {
    fn default() -> Self {
        Self::new()
    }
}
