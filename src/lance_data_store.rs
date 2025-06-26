// LanceDB implementation for NodeSpace data store
// Phase 1.3: Basic structure with DataStore trait compatibility

use crate::data_store::DataStore;
use crate::error::DataStoreError;
use arrow_array::RecordBatch;
use arrow_schema::{DataType, Field, Schema};
use async_trait::async_trait;
use lancedb::{Connection, Table};
use nodespace_core_types::{Node, NodeId, NodeSpaceResult};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for LanceDB data store
#[derive(Debug, Clone)]
pub struct LanceConfig {
    pub database_path: String,
    pub table_name: String,
    pub vector_dimension: usize,
    pub enable_compression: bool,
    pub index_cache_size: usize,
}

impl LanceConfig {
    /// Create new config with BGE default dimensions
    pub fn new() -> Self {
        Self {
            database_path: "./data/lance.db".to_string(),
            table_name: "nodes".to_string(),
            vector_dimension: 384, // fastembed BGE default
            enable_compression: true,
            index_cache_size: 256 * 1024 * 1024, // 256MB
        }
    }

    /// Set custom vector dimensions with validation
    pub fn with_dimensions(mut self, dims: usize) -> Self {
        assert!(dims > 0 && dims <= 4096, "Vector dimensions must be 1-4096");
        self.vector_dimension = dims;
        self
    }

    /// Set database path
    pub fn with_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.database_path = path.as_ref().to_string_lossy().to_string();
        self
    }

    /// Validate vector dimensions
    pub fn validate_vector(&self, vector: &[f32]) -> Result<(), DataStoreError> {
        if vector.len() != self.vector_dimension {
            return Err(DataStoreError::InvalidVector {
                expected: self.vector_dimension,
                actual: vector.len(),
            });
        }
        Ok(())
    }
}

impl Default for LanceConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Universal document structure for LanceDB storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalNode {
    pub id: String,
    pub node_type: String,
    pub content: String,          // serde_json::Value serialized
    pub metadata: Option<String>, // serde_json::Value serialized
    pub vector: Option<Vec<f32>>, // Embedding vector

    // Simple relationship fields (eliminates SurrealDB graph complexity!)
    pub parent_id: Option<String>,
    pub next_sibling: Option<String>,
    pub previous_sibling: Option<String>,

    pub created_at: String, // ISO 8601
    pub updated_at: String,
}

impl UniversalNode {
    /// Convert from NodeSpace Node to UniversalNode
    pub fn from_node(node: Node) -> Result<Self, DataStoreError> {
        let content =
            serde_json::to_string(&node.content).map_err(DataStoreError::Serialization)?;

        let metadata = if let Some(meta) = node.metadata {
            Some(serde_json::to_string(&meta).map_err(DataStoreError::Serialization)?)
        } else {
            None
        };

        Ok(Self {
            id: node.id.to_string(),
            node_type: "node".to_string(), // Default type, can be enhanced
            content,
            metadata,
            vector: None,    // Will be set separately
            parent_id: None, // Will be derived from relationships
            next_sibling: node.next_sibling.map(|id| id.to_string()),
            previous_sibling: node.previous_sibling.map(|id| id.to_string()),
            created_at: node.created_at,
            updated_at: node.updated_at,
        })
    }

    /// Convert UniversalNode back to NodeSpace Node
    pub fn to_node(&self) -> Result<Node, DataStoreError> {
        let content: serde_json::Value =
            serde_json::from_str(&self.content).map_err(DataStoreError::Serialization)?;

        let metadata = if let Some(meta_str) = &self.metadata {
            Some(serde_json::from_str(meta_str).map_err(DataStoreError::Serialization)?)
        } else {
            None
        };

        let mut node = Node::with_id(NodeId::from_string(self.id.clone()), content);

        if let Some(meta) = metadata {
            node = node.with_metadata(meta);
        }

        // Set sibling pointers
        node.next_sibling = self
            .next_sibling
            .as_ref()
            .map(|s| NodeId::from_string(s.clone()));
        node.previous_sibling = self
            .previous_sibling
            .as_ref()
            .map(|s| NodeId::from_string(s.clone()));

        node.created_at = self.created_at.clone();
        node.updated_at = self.updated_at.clone();

        Ok(node)
    }
}

/// LanceDB data store implementation
pub struct LanceDataStore {
    #[allow(dead_code)] // Will be used in future phases
    connection: Arc<Connection>,
    #[allow(dead_code)] // Will be used in future phases
    table: Arc<RwLock<Option<Table>>>,
    config: LanceConfig,
}

impl LanceDataStore {
    /// Create new LanceDataStore with configuration
    pub async fn new(config: LanceConfig) -> Result<Self, DataStoreError> {
        // Ensure directory exists
        if let Some(parent) = Path::new(&config.database_path).parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                DataStoreError::LanceDB(format!("Failed to create directory: {}", e))
            })?;
        }

        // Connect to LanceDB using the correct API
        let connection = lancedb::connect(&config.database_path)
            .execute()
            .await
            .map_err(|e| DataStoreError::LanceDB(format!("Failed to connect to LanceDB: {}", e)))?;

        Ok(Self {
            connection: Arc::new(connection),
            table: Arc::new(RwLock::new(None)),
            config,
        })
    }

    /// Initialize the database and create table with schema
    pub async fn initialize(&self) -> Result<(), DataStoreError> {
        // For Phase 1.3, we create a basic table structure
        // Full implementation will come in subsequent phases

        // TODO: Implement proper table creation with schema
        // For now, we'll skip table creation and focus on trait compatibility

        Ok(())
    }

    /// Get Arrow schema for universal node storage
    fn _get_arrow_schema(&self) -> Arc<Schema> {
        let fields = vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("node_type", DataType::Utf8, false),
            Field::new("parent_id", DataType::Utf8, true),
            Field::new("content", DataType::Utf8, false),
            Field::new("metadata", DataType::Utf8, true),
            Field::new(
                "vector",
                DataType::List(Arc::new(Field::new("item", DataType::Float32, true))),
                true,
            ),
            Field::new("created_at", DataType::Utf8, false),
            Field::new("updated_at", DataType::Utf8, false),
            Field::new("next_sibling", DataType::Utf8, true),
            Field::new("previous_sibling", DataType::Utf8, true),
        ];

        Arc::new(Schema::new(fields))
    }

    /// Enhanced method: Get child nodes (simple parent_id filtering vs SurrealDB graph traversal!)
    pub async fn get_child_nodes(&self, _parent_id: &NodeId) -> Result<Vec<Node>, DataStoreError> {
        // TODO: Implement when LanceDB query API is finalized
        // This represents the major simplification over SurrealDB graph traversal:
        // SELECT * FROM nodes WHERE parent_id = ? (simple filter)
        // vs SurrealDB: SELECT * FROM table_type:id->contains (complex graph traversal)
        Ok(Vec::new())
    }

    /// Enhanced method: Semantic search with filters
    pub async fn semantic_search_with_filters(
        &self,
        query_vector: Vec<f32>,
        _node_type_filter: Option<&str>,
        _parent_id_filter: Option<&str>,
        _limit: usize,
    ) -> Result<Vec<(Node, f32)>, DataStoreError> {
        self.config.validate_vector(&query_vector)?;

        // TODO: Implement when LanceDB vector search API is finalized
        // This will be: table.vector_search(query_vector).where(filters).limit(limit)
        Ok(Vec::new())
    }
}

// DataStore trait implementation for full compatibility with existing NodeSpace code
#[async_trait]
impl DataStore for LanceDataStore {
    async fn store_node(&self, node: Node) -> NodeSpaceResult<NodeId> {
        // Convert to UniversalNode for future LanceDB storage
        let _universal_node = UniversalNode::from_node(node.clone())
            .map_err(Into::<nodespace_core_types::NodeSpaceError>::into)?;

        // TODO: Implement actual RecordBatch conversion and storage
        // For Phase 1.3, return node ID as if stored
        Ok(node.id)
    }

    async fn get_node(&self, _id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        // TODO: Implement node retrieval
        Ok(None)
    }

    async fn delete_node(&self, _id: &NodeId) -> NodeSpaceResult<()> {
        // TODO: Implement node deletion
        Ok(())
    }

    async fn query_nodes(&self, _query: &str) -> NodeSpaceResult<Vec<Node>> {
        // TODO: Implement query execution
        Ok(Vec::new())
    }

    async fn create_relationship(
        &self,
        _from: &NodeId,
        _to: &NodeId,
        _rel_type: &str,
    ) -> NodeSpaceResult<()> {
        // For LanceDB, relationships are stored as simple parent_id fields
        // This is a major simplification vs SurrealDB RELATE statements!
        // TODO: Implement via simple field updates
        Ok(())
    }

    async fn store_node_with_embedding(
        &self,
        node: Node,
        embedding: Vec<f32>,
    ) -> NodeSpaceResult<NodeId> {
        self.config
            .validate_vector(&embedding)
            .map_err(Into::<nodespace_core_types::NodeSpaceError>::into)?;

        let mut _universal_node = UniversalNode::from_node(node.clone())
            .map_err(Into::<nodespace_core_types::NodeSpaceError>::into)?;
        _universal_node.vector = Some(embedding);

        // TODO: Implement embedding storage
        Ok(node.id)
    }

    async fn search_similar_nodes(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        self.semantic_search_with_filters(embedding, None, None, limit)
            .await
            .map_err(Into::<nodespace_core_types::NodeSpaceError>::into)
    }

    async fn update_node_embedding(
        &self,
        _id: &NodeId,
        embedding: Vec<f32>,
    ) -> NodeSpaceResult<()> {
        self.config
            .validate_vector(&embedding)
            .map_err(Into::<nodespace_core_types::NodeSpaceError>::into)?;

        // TODO: Implement embedding update
        Ok(())
    }

    async fn semantic_search_with_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        self.search_similar_nodes(embedding, limit).await
    }
}

// Helper methods for data conversion (will be implemented in future phases)
impl LanceDataStore {
    /// Convert UniversalNode to RecordBatch for LanceDB storage
    fn _universal_node_to_record_batch(
        &self,
        _nodes: Vec<UniversalNode>,
    ) -> Result<RecordBatch, DataStoreError> {
        // TODO: Implement proper Arrow conversion
        // This is a complex operation involving Arrow array construction
        Err(DataStoreError::LanceDB(
            "RecordBatch conversion not yet implemented".to_string(),
        ))
    }

    /// Convert RecordBatch to UniversalNode objects
    fn _record_batch_to_universal_nodes(
        &self,
        _batch: RecordBatch,
    ) -> Result<Vec<UniversalNode>, DataStoreError> {
        // TODO: Implement proper Arrow deserialization
        Err(DataStoreError::LanceDB(
            "RecordBatch deserialization not yet implemented".to_string(),
        ))
    }
}
