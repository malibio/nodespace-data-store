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
#[cfg(not(feature = "migration"))]
pub use data_store::{
    DataStore, HybridSearchConfig, ImageMetadata, ImageNode, MultiLevelEmbeddings, NodeType,
    QueryEmbeddings, RelevanceFactors, SearchResult,
};
#[cfg(feature = "migration")]
pub use data_store::{DataStore, SurrealDataStore};

pub use error::DataStoreError;
pub use lance_data_store::{
    LanceDBConfig, LanceDataStore as LanceDataStoreFull, UniversalDocument,
};
pub use lance_data_store_simple::{EmbeddingGenerator, LanceDataStore};
pub use performance::{OperationType, PerformanceConfig, PerformanceMonitor, PerformanceSummary};

#[cfg(feature = "migration")]
pub use surrealdb_types::*;
