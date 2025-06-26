use crate::error::DataStoreError;
use arrow_array::RecordBatch;
use arrow_schema::{DataType, Field, Schema};
use async_trait::async_trait;
use lancedb::{connect, Connection, Table};
use nodespace_core_types::{Node, NodeId, NodeSpaceResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// LanceDB DataStore implementation using Universal Document Schema
/// Replaces complex SurrealDB multi-table architecture with single flexible table
pub struct LanceDataStore {
    connection: Connection,
    table: Option<Table>,
}

/// Universal Node structure for LanceDB entity-centric storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalNode {
    pub id: String,
    pub node_type: String, // "text", "date", "task", "customer", "project", etc.
    pub content: String,
    pub vector: Vec<f32>, // 384-dimensional embedding from FastEmbed

    // JSON-based relationships (replaces complex SurrealDB graph traversal)
    pub parent_id: Option<String>,
    pub children_ids: Vec<String>,
    pub mentions: Vec<String>, // References to other entities

    pub created_at: String, // ISO 8601 timestamp
    pub updated_at: String,

    // Flexible metadata for entity-specific fields
    pub metadata: Option<serde_json::Value>,
}

/// Node type-specific metadata examples
#[derive(Serialize, Deserialize)]
pub struct TextMetadata {
    pub word_count: Option<u32>,
    pub language: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DateMetadata {
    pub date_value: String, // YYYY-MM-DD format
    pub timezone: Option<String>,
    pub is_recurring: bool,
}

#[derive(Serialize, Deserialize)]
pub struct TaskMetadata {
    pub priority: String, // "low", "medium", "high", "urgent"
    pub status: String,   // "todo", "in_progress", "done", "blocked"
    pub due_date: Option<String>,
    pub estimated_hours: Option<f32>,
    pub assignee: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CustomerMetadata {
    pub company: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub tier: Option<String>, // "basic", "premium", "enterprise"
    pub revenue: Option<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub status: String, // "planning", "active", "on_hold", "completed"
    pub budget: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub team_size: Option<u32>,
}

impl LanceDataStore {
    /// Initialize new LanceDB connection with Universal Document Schema
    pub async fn new(db_path: &str) -> Result<Self, DataStoreError> {
        let connection = connect(db_path)
            .execute()
            .await
            .map_err(|e| DataStoreError::LanceDB(format!("LanceDB connection failed: {}", e)))?;

        Ok(Self {
            connection,
            table: None,
        })
    }

    /// Initialize the universal nodes table with schema
    pub async fn initialize_table(&mut self) -> Result<(), DataStoreError> {
        let schema = Self::create_universal_schema();

        // Create empty table with the schema
        let empty_batch = RecordBatch::new_empty(schema);
        let reader = Box::new(arrow_array::RecordBatchIterator::new(
            std::iter::once(Ok(empty_batch.clone())),
            empty_batch.schema(),
        ));

        let table = self
            .connection
            .create_table("nodes", reader)
            .execute()
            .await
            .map_err(|e| DataStoreError::LanceDB(format!("Failed to create table: {}", e)))?;

        self.table = Some(table);
        Ok(())
    }

    /// Create the universal document schema for LanceDB
    fn create_universal_schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            // Core identification
            Field::new("id", DataType::Utf8, false),
            Field::new("node_type", DataType::Utf8, false),
            // Content and AI
            Field::new("content", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 384),
                false,
            ), // 384-dim embeddings from FastEmbed
            // Relationships (JSON-based, no complex graph traversal)
            Field::new("parent_id", DataType::Utf8, true),
            Field::new(
                "children_ids",
                DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
                true,
            ),
            Field::new(
                "mentions",
                DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
                true,
            ),
            // Temporal data
            Field::new("created_at", DataType::Utf8, false),
            Field::new("updated_at", DataType::Utf8, false),
            // Flexible metadata (JSON blob for entity-specific fields)
            Field::new("metadata", DataType::Utf8, true),
        ]))
    }

    /// Convert NodeSpace Node to UniversalNode
    fn node_to_universal(&self, node: Node, embedding: Option<Vec<f32>>) -> UniversalNode {
        let now = chrono::Utc::now().to_rfc3339();

        // Infer node type from content or metadata
        let node_type = if let Some(metadata) = &node.metadata {
            if let Some(node_type) = metadata.get("node_type").and_then(|v| v.as_str()) {
                node_type.to_string()
            } else {
                "text".to_string() // Default type
            }
        } else {
            "text".to_string()
        };

        UniversalNode {
            id: node.id.to_string(),
            node_type,
            content: node.content.to_string(),
            vector: embedding.unwrap_or_else(|| vec![0.0; 384]), // Default embedding
            parent_id: None,      // TODO: Extract from metadata if available
            children_ids: vec![], // TODO: Extract from metadata if available
            mentions: vec![],     // TODO: Extract from content/metadata
            created_at: if node.created_at.is_empty() {
                now.clone()
            } else {
                node.created_at
            },
            updated_at: if node.updated_at.is_empty() {
                now
            } else {
                node.updated_at
            },
            metadata: node.metadata,
        }
    }

    /// Convert UniversalNode back to NodeSpace Node
    #[allow(dead_code)]
    fn universal_to_node(&self, universal: UniversalNode) -> Node {
        let content = serde_json::Value::String(universal.content);

        let mut metadata = universal.metadata.unwrap_or_else(|| serde_json::json!({}));

        // Store additional fields in metadata
        metadata["node_type"] = serde_json::Value::String(universal.node_type);
        if let Some(parent_id) = universal.parent_id {
            metadata["parent_id"] = serde_json::Value::String(parent_id);
        }
        if !universal.children_ids.is_empty() {
            metadata["children_ids"] = serde_json::Value::Array(
                universal
                    .children_ids
                    .into_iter()
                    .map(serde_json::Value::String)
                    .collect(),
            );
        }
        if !universal.mentions.is_empty() {
            metadata["mentions"] = serde_json::Value::Array(
                universal
                    .mentions
                    .into_iter()
                    .map(serde_json::Value::String)
                    .collect(),
            );
        }

        Node {
            id: NodeId::from_string(universal.id),
            content,
            metadata: Some(metadata),
            created_at: universal.created_at,
            updated_at: universal.updated_at,
            next_sibling: None,
            previous_sibling: None,
        }
    }

    /// Get or create the nodes table
    #[allow(dead_code)]
    async fn get_table(&mut self) -> Result<&Table, DataStoreError> {
        if self.table.is_none() {
            // Try to open existing table first
            match self.connection.open_table("nodes").execute().await {
                Ok(table) => {
                    self.table = Some(table);
                }
                Err(_) => {
                    // Table doesn't exist, create it
                    self.initialize_table().await?;
                }
            }
        }

        self.table
            .as_ref()
            .ok_or_else(|| DataStoreError::LanceDB("Failed to get table".to_string()))
    }
}

// Implement the DataStore trait for compatibility with existing NodeSpace architecture
#[async_trait]
impl crate::data_store::DataStore for LanceDataStore {
    async fn store_node(&self, node: Node) -> NodeSpaceResult<NodeId> {
        // For now, store without embedding - embedding will be added via store_node_with_embedding
        let _universal = self.node_to_universal(node.clone(), None);

        // TODO: Convert UniversalNode to RecordBatch and insert into LanceDB
        // This is a simplified implementation - full LanceDB insertion would require
        // converting to Arrow format and batch insertion

        Ok(node.id)
    }

    async fn get_node(&self, _id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        // TODO: Query LanceDB by ID and convert back to Node
        // For now, return None as placeholder
        Ok(None)
    }

    async fn delete_node(&self, _id: &NodeId) -> NodeSpaceResult<()> {
        // TODO: Delete from LanceDB by ID
        Ok(())
    }

    async fn query_nodes(&self, _query: &str) -> NodeSpaceResult<Vec<Node>> {
        // TODO: Execute LanceDB query and convert results to Nodes
        Ok(vec![])
    }

    async fn create_relationship(
        &self,
        _from: &NodeId,
        _to: &NodeId,
        _rel_type: &str,
    ) -> NodeSpaceResult<()> {
        // TODO: Update parent_id/children_ids in the universal document
        // This replaces complex SurrealDB RELATE statements with simple JSON field updates
        Ok(())
    }

    async fn store_node_with_embedding(
        &self,
        node: Node,
        embedding: Vec<f32>,
    ) -> NodeSpaceResult<NodeId> {
        let _universal = self.node_to_universal(node.clone(), Some(embedding));

        // TODO: Convert to RecordBatch and insert with embedding vector

        Ok(node.id)
    }

    async fn search_similar_nodes(
        &self,
        _embedding: Vec<f32>,
        _limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        // TODO: Use LanceDB's native vector search
        // This replaces SurrealDB's vector::similarity::cosine with native LanceDB capabilities
        Ok(vec![])
    }

    async fn update_node_embedding(
        &self,
        _id: &NodeId,
        _embedding: Vec<f32>,
    ) -> NodeSpaceResult<()> {
        // TODO: Update the vector field for the specified node
        Ok(())
    }

    async fn semantic_search_with_embedding(
        &self,
        _embedding: Vec<f32>,
        _limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        // TODO: Native LanceDB vector search implementation
        // This is the key method that NS-69 needs for simplified core logic
        Ok(vec![])
    }
}

impl LanceDataStore {
    /// Get child nodes using simple JSON filtering (replaces complex SurrealDB traversal)
    pub async fn get_child_nodes(&self, _parent_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        // TODO: Filter nodes where parent_id equals the given ID
        // This eliminates the complex multi-table traversal that NS-69 mentions
        Ok(vec![])
    }

    /// Create or update relationship using JSON metadata (simplified from SurrealDB RELATE)
    pub async fn update_relationship(
        &self,
        _node_id: &NodeId,
        _parent_id: Option<NodeId>,
        _children_ids: Vec<NodeId>,
    ) -> NodeSpaceResult<()> {
        // TODO: Update the node's parent_id and children_ids fields
        // This is the simplified relationship model that NS-69 requires
        Ok(())
    }

    /// Hybrid search combining semantic search with metadata filtering
    pub async fn hybrid_search(
        &self,
        _embedding: Vec<f32>,
        _node_type_filter: Option<String>,
        _metadata_filter: Option<serde_json::Value>,
        _limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        // TODO: Combine vector search with structured filtering
        // This enables the advanced search capabilities mentioned in NS-69
        Ok(vec![])
    }
}
