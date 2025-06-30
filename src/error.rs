use nodespace_core_types::NodeSpaceError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataStoreError {
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

    #[error(
        "Performance threshold exceeded: {operation} took {actual_ms}ms, limit is {threshold_ms}ms"
    )]
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

    #[error("Feature not implemented: {0}")]
    NotImplemented(String),
}

impl From<DataStoreError> for NodeSpaceError {
    fn from(err: DataStoreError) -> Self {
        use nodespace_core_types::{DatabaseError, ProcessingError, ValidationError};

        match err {
            // Database-related errors
            DataStoreError::LanceDB(_) => NodeSpaceError::Database(
                DatabaseError::connection_failed("lancedb", &err.to_string()),
            ),
            DataStoreError::LanceDBConnection(_) => NodeSpaceError::Database(
                DatabaseError::connection_failed("lancedb", &err.to_string()),
            ),
            DataStoreError::LanceDBTable(_) => {
                NodeSpaceError::Database(DatabaseError::TransactionFailed {
                    operation: "table_operation".to_string(),
                    reason: err.to_string(),
                    can_retry: true,
                })
            }
            DataStoreError::LanceDBIndex(_) => {
                NodeSpaceError::Database(DatabaseError::IndexCorruption {
                    index_name: "vector_index".to_string(),
                    table: "universal_nodes".to_string(),
                    repair_command: Some("recreate index".to_string()),
                })
            }
            DataStoreError::LanceDBSchema(_) => {
                NodeSpaceError::Database(DatabaseError::MigrationFailed {
                    version: "current".to_string(),
                    target_version: "latest".to_string(),
                    reason: err.to_string(),
                    rollback_available: false,
                })
            }
            DataStoreError::LanceDBQuery(_) => {
                NodeSpaceError::Database(DatabaseError::QueryTimeout {
                    seconds: 30,
                    query: "lancedb_query".to_string(),
                    suggested_limit: Some(1000),
                })
            }
            DataStoreError::ArrowConversion(_) => {
                NodeSpaceError::Database(DatabaseError::TransactionFailed {
                    operation: "arrow_conversion".to_string(),
                    reason: err.to_string(),
                    can_retry: false,
                })
            }
            DataStoreError::VectorIndexCreation(_) => {
                NodeSpaceError::Database(DatabaseError::IndexCorruption {
                    index_name: "vector_index".to_string(),
                    table: "universal_nodes".to_string(),
                    repair_command: Some("recreate vector index".to_string()),
                })
            }
            DataStoreError::VectorSearchError(_) => {
                NodeSpaceError::Processing(ProcessingError::VectorSearchFailed {
                    reason: err.to_string(),
                    index_name: "vector_index".to_string(),
                    query_dimensions: 384,
                    similarity_threshold: Some(0.7),
                })
            }
            DataStoreError::Arrow(_) => {
                NodeSpaceError::Database(DatabaseError::TransactionFailed {
                    operation: "arrow_operation".to_string(),
                    reason: err.to_string(),
                    can_retry: false,
                })
            }
            DataStoreError::Database(_) => NodeSpaceError::Database(
                DatabaseError::connection_failed("database", &err.to_string()),
            ),
            DataStoreError::Migration(_) => {
                NodeSpaceError::Database(DatabaseError::MigrationFailed {
                    version: "current".to_string(),
                    target_version: "target".to_string(),
                    reason: err.to_string(),
                    rollback_available: true,
                })
            }

            // Validation-related errors
            DataStoreError::Serialization(_) => {
                NodeSpaceError::Validation(ValidationError::InvalidFormat {
                    field: "data".to_string(),
                    expected: "valid_json".to_string(),
                    actual: "invalid_format".to_string(),
                    examples: vec!["Valid JSON structure".to_string()],
                })
            }
            DataStoreError::NodeNotFound(_) => NodeSpaceError::Database(DatabaseError::NotFound {
                entity_type: "Node".to_string(),
                id: "unknown".to_string(),
                suggestions: vec!["Check node ID format".to_string()],
            }),
            DataStoreError::InvalidQuery(_) => {
                NodeSpaceError::Validation(ValidationError::InvalidFormat {
                    field: "query".to_string(),
                    expected: "valid_query_format".to_string(),
                    actual: "invalid_query".to_string(),
                    examples: vec!["SELECT * FROM table".to_string()],
                })
            }
            DataStoreError::InvalidVector { expected, actual } => {
                NodeSpaceError::Validation(ValidationError::OutOfRange {
                    field: "vector_dimensions".to_string(),
                    value: actual.to_string(),
                    min: expected.to_string(),
                    max: expected.to_string(),
                })
            }
            DataStoreError::PerformanceThresholdExceeded {
                operation: _,
                actual_ms,
                threshold_ms,
            } => NodeSpaceError::Validation(ValidationError::OutOfRange {
                field: "execution_time".to_string(),
                value: format!("{}ms", actual_ms),
                min: "0ms".to_string(),
                max: format!("{}ms", threshold_ms),
            }),
            DataStoreError::SchemaValidation(_) => {
                NodeSpaceError::Validation(ValidationError::SchemaValidationFailed {
                    schema_path: "universal_document_schema".to_string(),
                    violations: vec![err.to_string()],
                    schema_version: "1.0".to_string(),
                })
            }
            DataStoreError::InvalidNode(_) => {
                NodeSpaceError::Validation(ValidationError::RequiredFieldMissing {
                    field: "node_data".to_string(),
                    context: "Node creation".to_string(),
                    suggestion: Some("Ensure all required node fields are provided".to_string()),
                })
            }

            // Processing-related errors
            DataStoreError::ImageError(_) => {
                NodeSpaceError::Processing(ProcessingError::EmbeddingFailed {
                    reason: err.to_string(),
                    input_type: "image".to_string(),
                    dimensions: Some(384),
                    model_info: Some("image_processing_model".to_string()),
                })
            }
            DataStoreError::CrossModalError(_) => {
                NodeSpaceError::Processing(ProcessingError::VectorSearchFailed {
                    reason: err.to_string(),
                    index_name: "cross_modal_index".to_string(),
                    query_dimensions: 384,
                    similarity_threshold: Some(0.7),
                })
            }
            DataStoreError::EmbeddingError(_) => {
                NodeSpaceError::Processing(ProcessingError::EmbeddingFailed {
                    reason: err.to_string(),
                    input_type: "text".to_string(),
                    dimensions: Some(384),
                    model_info: Some("embedding_model".to_string()),
                })
            }
            DataStoreError::MultimodalError(_) => {
                NodeSpaceError::Processing(ProcessingError::VectorSearchFailed {
                    reason: err.to_string(),
                    index_name: "multimodal_index".to_string(),
                    query_dimensions: 384,
                    similarity_threshold: Some(0.7),
                })
            }

            // Legacy compatibility variants
            DataStoreError::IoError(_) => NodeSpaceError::IoError {
                message: err.to_string(),
            },
            DataStoreError::NotImplemented(_) => NodeSpaceError::InternalError {
                message: err.to_string(),
                service: "data-store".to_string(),
            },
        }
    }
}
