mod data_store;
mod error;

// LanceDB implementation modules
mod lance_data_store;
mod lance_data_store_simple;
pub mod performance;
mod schema;

pub use data_store::{
    DataStore, HybridSearchConfig, ImageMetadata, ImageNode, MultiLevelEmbeddings, NodeType,
    QueryEmbeddings, RelevanceFactors, SearchResult,
};

pub use error::DataStoreError;
pub use lance_data_store::{
    LanceDBConfig, LanceDataStore as LanceDataStoreFull, UniversalDocument,
};
pub use lance_data_store_simple::{EmbeddingGenerator, LanceDataStore};
pub use performance::{OperationType, PerformanceConfig, PerformanceMonitor, PerformanceSummary};
