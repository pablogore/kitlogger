use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::adapter::Adapter;
use crate::error::{AdapterError, AdapterResult};
use crate::id::AdapterId;
use telemetry_types::PayloadEnvelope;

/// Thread-safe registry with Arc-based storage.
pub struct AdapterRegistry {
    adapters: RwLock<HashMap<AdapterId, Arc<dyn Adapter>>>,
    frozen: bool,
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AdapterRegistry {
    pub fn new() -> Self {
        AdapterRegistry {
            adapters: RwLock::new(HashMap::new()),
            frozen: false,
        }
    }

    pub fn register(&mut self, id: AdapterId, adapter: Arc<dyn Adapter>) -> AdapterResult<()> {
        if self.frozen {
            return Err(AdapterError::Frozen);
        }
        let mut map = self
            .adapters
            .write()
            .map_err(|_| AdapterError::InitializationFailed("Lock poisoned".into()))?;
        if map.contains_key(&id) {
            return Err(AdapterError::AlreadyRegistered(id));
        }
        map.insert(id, adapter);
        Ok(())
    }

    pub fn get(&self, id: &AdapterId) -> Option<Arc<dyn Adapter>> {
        let map = self.adapters.read().ok()?;
        map.get(id).cloned()
    }

    pub fn contains(&self, id: &AdapterId) -> bool {
        self.adapters
            .read()
            .ok()
            .is_some_and(|map| map.contains_key(id))
    }

    pub fn list(&self) -> Vec<AdapterId> {
        self.adapters
            .read()
            .ok()
            .map_or_else(Vec::new, |map| map.keys().cloned().collect())
    }

    pub fn freeze(&mut self) {
        self.frozen = true;
    }
}

/// Delivers telemetry to multiple adapters via TelemetryDelivery trait.
/// Collects per-adapter failures and returns aggregated error.
pub async fn deliver_to_all(
    registry: &AdapterRegistry,
    ids: &[AdapterId],
    envelope: PayloadEnvelope,
) -> AdapterResult<()> {
    let mut failures = Vec::new();
    for id in ids {
        if let Some(adapter) = registry.get(id) {
            match adapter.deliver(envelope.clone()).await {
                Ok(()) => {}
                Err(e) => failures.push((id.clone(), e.to_string())),
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else if failures.len() < ids.len() {
        Err(AdapterError::PartialDelivery(failures))
    } else {
        Err(AdapterError::DeliveryFailed(failures))
    }
}
