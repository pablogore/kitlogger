use std::sync::Arc;
use std::time::SystemTime;

use async_trait::async_trait;

use telemetry_adapter_contracts::{
    Adapter, AdapterError, AdapterHealth, AdapterId, AdapterResult, AdapterRegistry,
    CommonAdapterBase, HealthReport, LifecycleAdapter, TelemetryDelivery,
};

/// Minimal mock for registry tests.
struct MockAdapter {
    id: AdapterId,
}

impl MockAdapter {
    fn new(id: AdapterId) -> Self {
        MockAdapter { id }
    }
}

#[async_trait]
impl CommonAdapterBase for MockAdapter {
    fn id(&self) -> &AdapterId {
        &self.id
    }
    fn health(&self) -> HealthReport {
        HealthReport {
            status: AdapterHealth::Healthy,
            reason: String::new(),
            timestamp: SystemTime::now(),
        }
    }
}

#[async_trait]
impl LifecycleAdapter for MockAdapter {
    async fn flush(&self) -> AdapterResult<()> {
        Ok(())
    }
    async fn shutdown(&self) -> AdapterResult<()> {
        Ok(())
    }
}

#[async_trait]
impl TelemetryDelivery for MockAdapter {
    async fn deliver(&self, _envelope: Vec<u8>) -> AdapterResult<()> {
        Ok(())
    }
}

#[test]
fn test_registry_register_and_get() {
    let mut registry = AdapterRegistry::new();
    let id = AdapterId::new("test-1").unwrap();
    let adapter: Arc<dyn Adapter> = Arc::new(MockAdapter::new(id.clone()));
    registry.register(id.clone(), adapter.clone()).unwrap();

    let retrieved = registry.get(&id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id(), &id);
}

#[test]
fn test_registry_contains() {
    let mut registry = AdapterRegistry::new();
    let id = AdapterId::new("test-1").unwrap();
    let adapter: Arc<dyn Adapter> = Arc::new(MockAdapter::new(id.clone()));
    registry.register(id.clone(), adapter).unwrap();

    assert!(registry.contains(&id));
    assert!(!registry.contains(&AdapterId::new("nonexistent").unwrap()));
}

#[test]
fn test_registry_list() {
    let mut registry = AdapterRegistry::new();
    let id1 = AdapterId::new("a").unwrap();
    let id2 = AdapterId::new("b").unwrap();
    registry
        .register(id1.clone(), Arc::new(MockAdapter::new(id1.clone())))
        .unwrap();
    registry
        .register(id2.clone(), Arc::new(MockAdapter::new(id2.clone())))
        .unwrap();

    let mut ids = registry.list();
    ids.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    assert_eq!(ids.len(), 2);
    assert_eq!(ids[0].as_str(), "a");
    assert_eq!(ids[1].as_str(), "b");
}

#[test]
fn test_registry_freeze_rejects_registration() {
    let mut registry = AdapterRegistry::new();
    let id = AdapterId::new("test-1").unwrap();
    let adapter: Arc<dyn Adapter> = Arc::new(MockAdapter::new(id.clone()));
    registry.register(id.clone(), adapter).unwrap();
    registry.freeze();

    let new_id = AdapterId::new("another").unwrap();
    let new_adapter: Arc<dyn Adapter> = Arc::new(MockAdapter::new(new_id.clone()));
    let result = registry.register(new_id, new_adapter);
    assert!(matches!(result, Err(AdapterError::Frozen)));
}

#[test]
fn test_registry_duplicate_rejected() {
    let mut registry = AdapterRegistry::new();
    let id = AdapterId::new("dup").unwrap();
    let adapter1: Arc<dyn Adapter> = Arc::new(MockAdapter::new(id.clone()));
    registry.register(id.clone(), adapter1).unwrap();

    let adapter2: Arc<dyn Adapter> = Arc::new(MockAdapter::new(id.clone()));
    let result = registry.register(id.clone(), adapter2);
    assert!(matches!(result, Err(AdapterError::AlreadyRegistered(_))));
}

#[test]
fn test_registry_lookup_after_freeze() {
    let mut registry = AdapterRegistry::new();
    let id = AdapterId::new("frozen-lookup").unwrap();
    let adapter: Arc<dyn Adapter> = Arc::new(MockAdapter::new(id.clone()));
    registry.register(id.clone(), adapter).unwrap();
    registry.freeze();

    // Reads still work after freeze
    assert!(registry.contains(&id));
    assert!(registry.get(&id).is_some());
}
