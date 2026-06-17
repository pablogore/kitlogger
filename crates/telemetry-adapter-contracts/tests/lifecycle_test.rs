use std::sync::Arc;
use std::time::SystemTime;

use async_trait::async_trait;

use telemetry_adapter_contracts::{
    AdapterError, AdapterHealth, AdapterId, AdapterLifecycle, AdapterRegistry, AdapterResult,
    CommonAdapterBase, ExporterAdapter, HealthReport, LifecycleAdapter, LifecycleState,
    ProviderAdapter, TelemetryDelivery,
};

/// Mock adapter that tracks lifecycle invocations.
struct LifecycleMock {
    id: AdapterId,
    should_fail_flush: bool,
}

impl LifecycleMock {
    fn new(id: AdapterId) -> Self {
        LifecycleMock {
            id,
            should_fail_flush: false,
        }
    }
}

#[async_trait]
impl CommonAdapterBase for LifecycleMock {
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
impl LifecycleAdapter for LifecycleMock {
    async fn flush(&self) -> AdapterResult<()> {
        if self.should_fail_flush {
            Err(AdapterError::FlushFailed("mock flush failure".into()))
        } else {
            Ok(())
        }
    }
    async fn shutdown(&self) -> AdapterResult<()> {
        // Real impl would call flush() then transition
        Ok(())
    }
}

#[async_trait]
impl TelemetryDelivery for LifecycleMock {
    async fn deliver(&self, _envelope: Vec<u8>) -> AdapterResult<()> {
        Ok(())
    }
}

#[async_trait]
impl ProviderAdapter for LifecycleMock {
    async fn initialize(&self) -> AdapterResult<()> {
        Ok(())
    }
    async fn start(&self) -> AdapterResult<()> {
        Ok(())
    }
    async fn stop(&self) -> AdapterResult<()> {
        Ok(())
    }
}

#[async_trait]
impl ExporterAdapter for LifecycleMock {
    async fn initialize(&self) -> AdapterResult<()> {
        Ok(())
    }
    async fn start(&self) -> AdapterResult<()> {
        Ok(())
    }
    async fn stop(&self) -> AdapterResult<()> {
        Ok(())
    }
}

// ─── Lifecycle state machine tests ───

#[test]
fn test_lifecycle_starts_registered() {
    let lc = AdapterLifecycle::new();
    assert_eq!(lc.state(), LifecycleState::Registered);
}

#[test]
fn test_lifecycle_registered_to_initialized() {
    let mut lc = AdapterLifecycle::new();
    lc.transition(LifecycleState::Initialized).unwrap();
    assert_eq!(lc.state(), LifecycleState::Initialized);
}

#[test]
fn test_lifecycle_initialized_to_started() {
    let mut lc = AdapterLifecycle::new();
    lc.transition(LifecycleState::Initialized).unwrap();
    lc.transition(LifecycleState::Started).unwrap();
    assert_eq!(lc.state(), LifecycleState::Started);
}

#[test]
fn test_lifecycle_full_chain() {
    let mut lc = AdapterLifecycle::new();
    lc.transition(LifecycleState::Initialized).unwrap();
    lc.transition(LifecycleState::Started).unwrap();
    lc.transition(LifecycleState::Stopping).unwrap();
    lc.transition(LifecycleState::Stopped).unwrap();
    lc.transition(LifecycleState::Shutdown).unwrap();
    assert_eq!(lc.state(), LifecycleState::Shutdown);
}

#[test]
fn test_lifecycle_startup_failure_registered_to_shutdown() {
    let mut lc = AdapterLifecycle::new();
    lc.transition(LifecycleState::Shutdown).unwrap();
    assert_eq!(lc.state(), LifecycleState::Shutdown);
}

#[test]
fn test_lifecycle_startup_failure_initialized_to_shutdown() {
    let mut lc = AdapterLifecycle::new();
    lc.transition(LifecycleState::Initialized).unwrap();
    lc.transition(LifecycleState::Shutdown).unwrap();
    assert_eq!(lc.state(), LifecycleState::Shutdown);
}

#[test]
fn test_lifecycle_invalid_transition_rejected() {
    let mut lc = AdapterLifecycle::new();
    // Registered -> Started is invalid (skip Initialized)
    let result = lc.transition(LifecycleState::Started);
    assert!(matches!(
        result,
        Err(AdapterError::InvalidTransition { .. })
    ));
    // Still registered
    assert_eq!(lc.state(), LifecycleState::Registered);
}

#[test]
fn test_lifecycle_transition_from_shutdown_rejected() {
    let mut lc = AdapterLifecycle::new();
    lc.transition(LifecycleState::Shutdown).unwrap();
    let result = lc.transition(LifecycleState::Started);
    assert!(matches!(
        result,
        Err(AdapterError::InvalidTransition { .. })
    ));
}

// ─── Shutdown-flush semantics tests ───

#[test]
fn test_lifecycle_stopping_to_stopped_after_flush() {
    let mut lc = AdapterLifecycle::new();
    lc.transition(LifecycleState::Initialized).unwrap();
    lc.transition(LifecycleState::Started).unwrap();
    lc.transition(LifecycleState::Stopping).unwrap();
    // Flush succeeds → Stopped
    lc.transition(LifecycleState::Stopped).unwrap();
    assert_eq!(lc.state(), LifecycleState::Stopped);
}

#[test]
fn test_lifecycle_stopping_to_shutdown_on_flush_failure() {
    let mut lc = AdapterLifecycle::new();
    lc.transition(LifecycleState::Initialized).unwrap();
    lc.transition(LifecycleState::Started).unwrap();
    lc.transition(LifecycleState::Stopping).unwrap();
    // Flush fails → Shutdown
    lc.transition(LifecycleState::Shutdown).unwrap();
    assert_eq!(lc.state(), LifecycleState::Shutdown);
}

// ─── Integration test ───

#[tokio::test]
async fn test_full_lifecycle_with_multiplexing() {
    let mut registry = AdapterRegistry::new();
    let id1 = AdapterId::new("adapter-1").unwrap();
    let id2 = AdapterId::new("adapter-2").unwrap();

    registry
        .register(id1.clone(), Arc::new(LifecycleMock::new(id1.clone())))
        .unwrap();
    registry
        .register(id2.clone(), Arc::new(LifecycleMock::new(id2.clone())))
        .unwrap();
    registry.freeze();

    // Both adapters should receive delivery
    let result = telemetry_adapter_contracts::registry::deliver_to_all(
        &registry,
        &[id1, id2],
        vec![1, 2, 3],
    )
    .await;
    assert!(result.is_ok());
}
