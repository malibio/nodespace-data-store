use nodespace_core_types::NodeSpaceError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error("Database error: {0}")]
    Database(#[from] surrealdb::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}

impl From<DataStoreError> for NodeSpaceError {
    fn from(err: DataStoreError) -> Self {
        match err {
            DataStoreError::Database(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::Serialization(_) => NodeSpaceError::SerializationError(err.to_string()),
            DataStoreError::NodeNotFound(_) => NodeSpaceError::NotFound(err.to_string()),
            DataStoreError::InvalidQuery(_) => NodeSpaceError::ValidationError(err.to_string()),
        }
    }
}
