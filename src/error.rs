use nodespace_core_types::NodeSpaceError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error("SurrealDB error: {0}")]
    SurrealDB(String),

    #[error("LanceDB error: {0}")]
    LanceDB(String),

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
            DataStoreError::SurrealDB(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::LanceDB(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::Database(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::Serialization(_) => NodeSpaceError::SerializationError(err.to_string()),
            DataStoreError::NodeNotFound(_) => NodeSpaceError::NotFound(err.to_string()),
            DataStoreError::InvalidQuery(_) => NodeSpaceError::ValidationError(err.to_string()),
            DataStoreError::InvalidVector { .. } => {
                NodeSpaceError::ValidationError(err.to_string())
            }
            DataStoreError::IoError(_) => NodeSpaceError::IoError(err.to_string()),
            DataStoreError::InvalidNode(_) => NodeSpaceError::ValidationError(err.to_string()),
            DataStoreError::ImageError(_) => NodeSpaceError::ProcessingError(err.to_string()),
            DataStoreError::CrossModalError(_) => NodeSpaceError::ProcessingError(err.to_string()),
            DataStoreError::EmbeddingError(_) => NodeSpaceError::ProcessingError(err.to_string()),
        }
    }
}
