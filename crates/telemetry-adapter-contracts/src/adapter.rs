use async_trait::async_trait;

use crate::error::AdapterResult;
use crate::health::HealthReport;
use crate::id::AdapterId;

/// Identity and health; no lifecycle concerns.
#[async_trait]
pub trait CommonAdapterBase: Send + Sync {
    fn id(&self) -> &AdapterId;
    fn health(&self) -> HealthReport;
}

/// Lifecycle operations (flush, shutdown) separated from identity/health.
/// Uses `&self` for Arc compatibility; concrete adapters own synchronization
/// via interior mutability.
#[async_trait]
pub trait LifecycleAdapter: Send + Sync {
    async fn flush(&self) -> AdapterResult<()>;
    async fn shutdown(&self) -> AdapterResult<()>;
}

/// Dedicated trait for telemetry delivery operations.
/// Uses `&self` for Arc compatibility.
#[async_trait]
pub trait TelemetryDelivery: Send + Sync {
    async fn deliver(&self, envelope: Vec<u8>) -> AdapterResult<()>;
}

/// Common supertrait for registry storage.
pub trait Adapter: CommonAdapterBase + LifecycleAdapter + TelemetryDelivery + Send + Sync {}

/// Blanket impl: any type implementing all bases automatically implements Adapter.
impl<T> Adapter for T where T: CommonAdapterBase + LifecycleAdapter + TelemetryDelivery + Send + Sync
{}

/// Provider-side operations.
#[async_trait]
pub trait ProviderAdapter: CommonAdapterBase + LifecycleAdapter + TelemetryDelivery {
    async fn initialize(&self) -> AdapterResult<()>;
    async fn start(&self) -> AdapterResult<()>;
    async fn stop(&self) -> AdapterResult<()>;
}

/// Exporter-side operations.
#[async_trait]
pub trait ExporterAdapter: CommonAdapterBase + LifecycleAdapter + TelemetryDelivery {
    async fn initialize(&self) -> AdapterResult<()>;
    async fn start(&self) -> AdapterResult<()>;
    async fn stop(&self) -> AdapterResult<()>;
}
