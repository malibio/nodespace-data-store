use nodespace_core_types::NodeSpaceError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataStoreError {
    #[cfg(feature = "migration")]
    #[error("SurrealDB error: {0}")]
    SurrealDB(#[from] surrealdb::Error),

    #[error("LanceDB error: {0}")]
    LanceDB(String),

    #[error("LanceDB connection error: {0}")]
    LanceDBConnection(String),

    #[error("LanceDB table error: {0}")]
    LanceDBTable(String),

    #[error("LanceDB index error: {0}")]
    LanceDBIndex(String),

    #[error("LanceDB schema error: {0}")]
    LanceDBSchema(String),

    #[error("LanceDB query error: {0}")]
    LanceDBQuery(String),

    #[error("Arrow conversion error: {0}")]
    ArrowConversion(String),

    #[error("Vector index creation failed: {0}")]
    VectorIndexCreation(String),

    #[error("Vector search failed: {0}")]
    VectorSearchError(String),

    #[error("Arrow error: {0}")]
    Arrow(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Invalid vector: expected {expected} dimensions, got {actual}")]
    InvalidVector { expected: usize, actual: usize },

    #[error("Performance threshold exceeded: {operation} took {actual_ms}ms, limit is {threshold_ms}ms")]
    PerformanceThresholdExceeded {
        operation: String,
        actual_ms: u64,
        threshold_ms: u64,
    },

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Schema validation error: {0}")]
    SchemaValidation(String),

    #[error("Multimodal operation error: {0}")]
    MultimodalError(String),

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Invalid node: {0}")]
    InvalidNode(String),

    #[error("Image processing error: {0}")]
    ImageError(String),

    #[error("Cross-modal search error: {0}")]
    CrossModalError(String),

    #[error("Embedding error: {0}")]
    EmbeddingError(String),
}

impl From<DataStoreError> for NodeSpaceError {
    fn from(err: DataStoreError) -> Self {
        match err {
            #[cfg(feature = "migration")]
            DataStoreError::SurrealDB(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::LanceDB(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::LanceDBConnection(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::LanceDBTable(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::LanceDBIndex(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::LanceDBSchema(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::LanceDBQuery(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::ArrowConversion(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::VectorIndexCreation(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::VectorSearchError(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::Arrow(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::Database(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::Serialization(_) => NodeSpaceError::SerializationError(err.to_string()),
            DataStoreError::NodeNotFound(_) => NodeSpaceError::NotFound(err.to_string()),
            DataStoreError::InvalidQuery(_) => NodeSpaceError::ValidationError(err.to_string()),
            DataStoreError::InvalidVector { .. } => {
                NodeSpaceError::ValidationError(err.to_string())
            }
            DataStoreError::PerformanceThresholdExceeded { .. } => {
                NodeSpaceError::ValidationError(err.to_string())
            }
            DataStoreError::Migration(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::SchemaValidation(_) => NodeSpaceError::ValidationError(err.to_string()),
            DataStoreError::MultimodalError(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::IoError(_) => NodeSpaceError::IoError(err.to_string()),
            DataStoreError::InvalidNode(_) => NodeSpaceError::ValidationError(err.to_string()),
            DataStoreError::ImageError(_) => NodeSpaceError::ProcessingError(err.to_string()),
            DataStoreError::CrossModalError(_) => NodeSpaceError::ProcessingError(err.to_string()),
            DataStoreError::EmbeddingError(_) => NodeSpaceError::ProcessingError(err.to_string()),
        }
    }
}
