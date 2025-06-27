#[cfg(feature = "migration")]
mod conversions;
mod data_store;
mod error;

// SurrealDB-related modules (only compiled when migration feature is enabled)
#[cfg(feature = "migration")]
mod surrealdb_types;

// LanceDB implementation modules (for migration)
mod lance_data_store;
mod lance_data_store_simple;
// Temporarily disabled migration module due to compilation complexity
// #[cfg(feature = "migration")]
// pub mod migration;
pub mod performance;
mod schema;

// Conditional exports based on feature flags
#[cfg(feature = "migration")]
pub use data_store::{DataStore, SurrealDataStore};
#[cfg(not(feature = "migration"))]
pub use data_store::DataStore;

pub use error::DataStoreError;
pub use lance_data_store::{LanceDataStore as LanceDataStoreFull, LanceDBConfig, UniversalDocument};
pub use lance_data_store_simple::LanceDataStore;
pub use performance::{PerformanceMonitor, PerformanceConfig, OperationType, PerformanceSummary};

#[cfg(feature = "migration")]
pub use surrealdb_types::*;
