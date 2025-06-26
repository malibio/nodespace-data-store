use nodespace_core_types::NodeSpaceError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error("SurrealDB error: {0}")]
    SurrealDB(#[from] surrealdb::Error),

    #[error("LanceDB error: {0}")]
    LanceDB(String),

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
            DataStoreError::SurrealDB(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::LanceDB(_) => NodeSpaceError::DatabaseError(err.to_string()),
            DataStoreError::Serialization(_) => NodeSpaceError::SerializationError(err.to_string()),
            DataStoreError::NodeNotFound(_) => NodeSpaceError::NotFound(err.to_string()),
            DataStoreError::InvalidQuery(_) => NodeSpaceError::ValidationError(err.to_string()),
        }
    }
}
