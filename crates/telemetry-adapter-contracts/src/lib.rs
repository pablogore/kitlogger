//! Telemetry Adapter Contracts
//!
//! Defines the canonical adapter contract for telemetry provider abstraction.
//! Owns ProviderAdapter, ExporterAdapter, CommonAdapterBase, LifecycleAdapter,
//! TelemetryDelivery, AdapterRegistry, AdapterLifecycle, mapping contracts,
//! and supporting types.

pub mod adapter;
pub mod error;
pub mod health;
pub mod id;
pub mod lifecycle;
pub mod mapping;
pub mod registry;

pub use adapter::{
    Adapter, CommonAdapterBase, ExporterAdapter, LifecycleAdapter, ProviderAdapter,
    TelemetryDelivery,
};
pub use error::{AdapterError, AdapterResult};
pub use health::{AdapterHealth, HealthReport};
pub use id::AdapterId;
pub use lifecycle::{AdapterLifecycle, LifecycleState};
pub use mapping::{
    LogRecordMappingContract, MetricMappingContract, ResourceMappingContract, SpanMappingContract,
    TraceMappingContract,
};
pub use registry::{deliver_to_all, AdapterRegistry};
