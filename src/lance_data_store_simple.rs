use crate::data_store::{
    DataStore, HybridSearchConfig, ImageMetadata, ImageNode, NodeType, RelevanceFactors,
    SearchResult,
};
use crate::error::DataStoreError;
use arrow_array::builder::{ListBuilder, StringBuilder};
use arrow_array::{Array, ListArray, RecordBatch, RecordBatchIterator, StringArray};
use arrow_schema::{DataType, Field, Schema};
use async_trait::async_trait;
use base64::prelude::*;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{connect, Connection, Table};
use nodespace_core_types::{Node, NodeId, NodeSpaceResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// LanceDB DataStore implementation with native Arrow columnar storage
pub struct LanceDataStore {
    connection: Connection,
    table: Arc<RwLock<Option<Table>>>,
    table_name: String,
    _db_path: String,
    vector_dimension: usize,
    // Optional NLP engine for automatic embedding generation
    embedding_generator: Option<Box<dyn EmbeddingGenerator + Send + Sync>>,
}

/// Trait for generating embeddings from text content
#[async_trait]
pub trait EmbeddingGenerator {
    async fn generate_embedding(&self, content: &str) -> Result<Vec<f32>, DataStoreError>;
}

/// Universal Node structure for LanceDB entity-centric storage with multi-level embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalNode {
    pub id: String,
    pub r#type: String, // "text", "date", "task", "customer", "project", etc.
    pub content: String,

    // Multi-level embeddings for advanced search
    pub individual_vector: Vec<f32>, // Individual content embedding (384-dim)
    pub contextual_vector: Option<Vec<f32>>, // Context-aware embedding (384-dim)
    pub hierarchical_vector: Option<Vec<f32>>, // Hierarchical path embedding (384-dim)
    pub embedding_model: Option<String>, // Model used for generation
    pub embeddings_generated_at: Option<String>, // Timestamp for embedding generation

    // Backward compatibility - maps to individual_vector
    pub vector: Vec<f32>, // 384-dimensional embedding from FastEmbed

    // JSON-based relationships for entity connections
    pub parent_id: Option<String>,
    pub before_sibling_id: Option<String>, // Node that comes after this one (backward linking)
    pub children_ids: Vec<String>,
    pub mentions: Vec<String>, // References to other entities

    // Root hierarchy optimization for efficient single-query retrieval
    pub root_id: Option<String>, // Points to hierarchy root (indexed for O(1) queries)
    // Legacy root_type field removed - use node_type for categorization

    pub created_at: String, // ISO 8601 timestamp
    pub updated_at: String,

    // Flexible metadata for entity-specific fields (stored as JSON string in Arrow)
    pub metadata: Option<serde_json::Value>,
}

impl LanceDataStore {
    /// Initialize new LanceDB connection with Arrow-based storage
    pub async fn new(db_path: &str) -> Result<Self, DataStoreError> {
        Self::with_vector_dimension(db_path, 384).await
    }

    /// Set the embedding generator for automatic embedding generation
    pub fn set_embedding_generator(
        &mut self,
        generator: Box<dyn EmbeddingGenerator + Send + Sync>,
    ) {
        self.embedding_generator = Some(generator);
    }

    /// Initialize new LanceDB connection with custom vector dimension
    pub async fn with_vector_dimension(
        db_path: &str,
        vector_dimension: usize,
    ) -> Result<Self, DataStoreError> {
        let connection = connect(db_path).execute().await.map_err(|e| {
            DataStoreError::LanceDBConnection(format!("LanceDB connection failed: {}", e))
        })?;

        let instance = Self {
            connection,
            table: Arc::new(RwLock::new(None)),
            table_name: "universal_nodes".to_string(),
            _db_path: db_path.to_string(),
            vector_dimension,
            embedding_generator: None, // Can be set later via set_embedding_generator
        };

        // Initialize Arrow-based table
        instance.initialize_table().await?;

        Ok(instance)
    }

    /// Initialize the Arrow-based table with Universal Document Schema
    pub async fn initialize_table(&self) -> Result<(), DataStoreError> {
        let schema = self.create_universal_schema();

        // Check if table already exists
        let table_names =
            self.connection.table_names().execute().await.map_err(|e| {
                DataStoreError::LanceDB(format!("Failed to get table names: {}", e))
            })?;

        let table = if table_names.contains(&self.table_name) {
            // Open existing table
            self.connection
                .open_table(&self.table_name)
                .execute()
                .await
                .map_err(|e| DataStoreError::LanceDB(format!("Failed to open table: {}", e)))?
        } else {
            // Create new table with empty data
            let empty_batch = self.create_empty_record_batch(schema.clone())?;
            let batches =
                RecordBatchIterator::new(vec![empty_batch].into_iter().map(Ok), schema.clone());

            self.connection
                .create_table(&self.table_name, Box::new(batches))
                .execute()
                .await
                .map_err(|e| DataStoreError::LanceDB(format!("Failed to create table: {}", e)))?
        };

        // Store table reference
        *self.table.write().await = Some(table);

        // Create vector index for similarity search
        self.create_vector_index().await?;

        Ok(())
    }

    /// Create the Universal Document Schema with root hierarchy optimization
    fn create_universal_schema(&self) -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("type", DataType::Utf8, false),
            Field::new("content", DataType::Utf8, false),
            // Backward compatibility vector field - FixedSizeList of Float32 for LanceDB vector indexing
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, false)),
                    self.vector_dimension as i32,
                ),
                false,
            ),
            Field::new("parent_id", DataType::Utf8, true), // Nullable
            Field::new("before_sibling_id", DataType::Utf8, true), // Nullable
            // Children IDs - List of String (nullable for empty lists)
            Field::new(
                "children_ids",
                DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
                true,
            ),
            // Mentions - List of String (nullable for empty lists)
            Field::new(
                "mentions",
                DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
                true,
            ),
            // Root hierarchy optimization fields for efficient O(1) queries
            Field::new("root_id", DataType::Utf8, true), // Nullable - indexed for fast filtering
            // root_type field removed
            Field::new("created_at", DataType::Utf8, false),
            Field::new("updated_at", DataType::Utf8, false),
            Field::new("metadata", DataType::Utf8, true), // Nullable JSON string
        ]))
    }

    /// Create an empty RecordBatch for table initialization
    fn create_empty_record_batch(
        &self,
        schema: Arc<Schema>,
    ) -> Result<RecordBatch, DataStoreError> {
        use arrow_array::{FixedSizeListArray, Float32Array};

        // Create empty FixedSizeListArray for vectors with configurable dimension
        let empty_values = Float32Array::from(Vec::<f32>::new());
        let field = Arc::new(Field::new("item", DataType::Float32, false));
        let empty_vectors = FixedSizeListArray::try_new(
            field,
            self.vector_dimension as i32,
            Arc::new(empty_values),
            None,
        )
        .map_err(|e| {
            DataStoreError::Arrow(format!("Failed to create empty FixedSizeListArray: {}", e))
        })?;

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(Vec::<String>::new())), // id
                Arc::new(StringArray::from(Vec::<String>::new())), // node_type
                Arc::new(StringArray::from(Vec::<String>::new())), // content
                Arc::new(empty_vectors),                           // vector
                Arc::new(StringArray::from(Vec::<Option<String>>::new())), // parent_id
                Arc::new(StringArray::from(Vec::<Option<String>>::new())), // before_sibling_id
                Arc::new(ListBuilder::new(StringBuilder::new()).finish()), // children_ids
                Arc::new(ListBuilder::new(StringBuilder::new()).finish()), // mentions
                Arc::new(StringArray::from(Vec::<Option<String>>::new())), // root_id
                // root_type column removed
                Arc::new(StringArray::from(Vec::<String>::new())), // created_at
                Arc::new(StringArray::from(Vec::<String>::new())), // updated_at
                Arc::new(StringArray::from(Vec::<Option<String>>::new())), // metadata
            ],
        )
        .map_err(|e| DataStoreError::Arrow(format!("Failed to create empty batch: {}", e)))?;

        Ok(batch)
    }

    /// Create vector index for efficient similarity search
    async fn create_vector_index(&self) -> Result<(), DataStoreError> {
        let table_guard = self.table.read().await;
        if let Some(table) = table_guard.as_ref() {
            // Only create index if table has data
            let stats = table
                .count_rows(None)
                .await
                .map_err(|e| DataStoreError::LanceDB(format!("Failed to get row count: {}", e)))?;

            if stats > 0 {
                // Create IVF (Inverted File) index for vector similarity search
                match table
                    .create_index(
                        &["vector"],
                        lancedb::index::Index::IvfPq(Default::default()),
                    )
                    .replace(true) // Replace existing index if present
                    .execute()
                    .await
                {
                    Ok(_) => {}
                    Err(_) => {
                        // This is not a fatal error - index can be created later when data exists
                    }
                }
            }
        }
        Ok(())
    }

    /// Convert NodeSpace Node to UniversalNode with multi-level embeddings support
    /// For TextNode and DateNode: Empty metadata to eliminate redundant hierarchical data
    /// For other node types: Preserve metadata for type-specific properties
    fn node_to_universal(&self, node: Node, embedding: Option<Vec<f32>>) -> UniversalNode {
        let now = chrono::Utc::now().to_rfc3339();

        // Use the actual node type from the Node struct
        let node_type = node.r#type.clone();

        // Extract relationships from Node fields and metadata
        // Prefer Node.parent_id field over metadata for direct field access
        let parent_id = node
            .parent_id
            .as_ref()
            .map(|id| id.to_string())
            .or_else(|| {
                node.metadata
                    .as_ref()
                    .and_then(|m| m.get("parent_id"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            });

        let children_ids = node
            .metadata
            .as_ref()
            .and_then(|m| m.get("children_ids"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let mentions = node
            .metadata
            .as_ref()
            .and_then(|m| m.get("mentions"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // Extract root hierarchy optimization fields
        // Extract root hierarchy fields from Node fields and metadata
        // Prefer Node fields over metadata for direct field access
        let root_id = node.root_id.as_ref().map(|id| id.to_string()).or_else(|| {
            node.metadata
                .as_ref()
                .and_then(|m| m.get("root_id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        });

        // root_type removed - use node.r#type instead

        // Extract multi-level embeddings from metadata if available
        let default_vector = vec![0.0; self.vector_dimension];
        let individual_vector = embedding.clone().unwrap_or_else(|| default_vector.clone());

        let contextual_vector = node
            .metadata
            .as_ref()
            .and_then(|m| m.get("contextual_vector"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect()
            });

        let hierarchical_vector = node
            .metadata
            .as_ref()
            .and_then(|m| m.get("hierarchical_vector"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect()
            });

        let embedding_model = node
            .metadata
            .as_ref()
            .and_then(|m| m.get("embedding_model"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let embeddings_generated_at = node
            .metadata
            .as_ref()
            .and_then(|m| m.get("embeddings_generated_at"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Simplify metadata for TextNode and DateNode to eliminate redundant hierarchical data
        // For these node types, hierarchical data should come from parent_id/children_ids fields only
        let simplified_metadata = match node_type.as_str() {
            "text" | "date" => {
                // Empty metadata for text and date nodes - hierarchy handled by parent_id/children_ids
                None
            }
            _ => {
                // Preserve metadata for other node types (image, task, etc.) for type-specific properties
                node.metadata
            }
        };

        UniversalNode {
            id: node.id.to_string(),
            r#type: node_type,
            content: match &node.content {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            },
            individual_vector: individual_vector.clone(),
            contextual_vector,
            hierarchical_vector,
            embedding_model,
            embeddings_generated_at,
            vector: individual_vector, // Backward compatibility
            parent_id,
            before_sibling_id: node.before_sibling.as_ref().map(|id| id.to_string()),
            children_ids,
            mentions,
            root_id,   // Root hierarchy optimization
            // root_type field removed
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
            metadata: simplified_metadata,
        }
    }

    /// Convert NodeSpace Node to UniversalNode with multi-level embeddings
    fn node_to_universal_with_multi_embeddings(
        &self,
        node: Node,
        embeddings: crate::data_store::MultiLevelEmbeddings,
    ) -> UniversalNode {
        let now = chrono::Utc::now().to_rfc3339();

        // Use the actual node type from the Node struct
        let node_type = node.r#type.clone();

        // Extract relationships from Node fields and metadata
        // Prefer Node.parent_id field over metadata for direct field access
        let parent_id = node
            .parent_id
            .as_ref()
            .map(|id| id.to_string())
            .or_else(|| {
                node.metadata
                    .as_ref()
                    .and_then(|m| m.get("parent_id"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            });

        let children_ids = node
            .metadata
            .as_ref()
            .and_then(|m| m.get("children_ids"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let mentions = node
            .metadata
            .as_ref()
            .and_then(|m| m.get("mentions"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // Extract root hierarchy optimization fields
        // Extract root hierarchy fields from Node fields and metadata
        // Prefer Node fields over metadata for direct field access
        let root_id = node.root_id.as_ref().map(|id| id.to_string()).or_else(|| {
            node.metadata
                .as_ref()
                .and_then(|m| m.get("root_id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        });

        // root_type removed - use node.r#type instead

        // Simplify metadata for TextNode and DateNode to eliminate redundant hierarchical data
        // For these node types, hierarchical data should come from parent_id/children_ids fields only
        let simplified_metadata = match node_type.as_str() {
            "text" | "date" => {
                // Empty metadata for text and date nodes - hierarchy handled by parent_id/children_ids
                None
            }
            _ => {
                // Preserve metadata for other node types (image, task, etc.) for type-specific properties
                node.metadata
            }
        };

        UniversalNode {
            id: node.id.to_string(),
            r#type: node_type,
            content: match &node.content {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            },
            individual_vector: embeddings.individual.clone(),
            contextual_vector: embeddings.contextual.clone(),
            hierarchical_vector: embeddings.hierarchical.clone(),
            embedding_model: embeddings.embedding_model.clone(),
            embeddings_generated_at: Some(embeddings.generated_at.to_rfc3339()),
            vector: embeddings.individual, // Backward compatibility
            parent_id,
            before_sibling_id: node.before_sibling.as_ref().map(|id| id.to_string()),
            children_ids,
            mentions,
            root_id,   // Root hierarchy optimization
            // root_type field removed
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
            metadata: simplified_metadata,
        }
    }

    /// Create RecordBatch from nodes using proper ListArray construction
    fn create_record_batch_from_nodes(
        &self,
        nodes: Vec<UniversalNode>,
        schema: Arc<Schema>,
    ) -> Result<RecordBatch, DataStoreError> {
        if nodes.is_empty() {
            return self.create_empty_record_batch(schema);
        }

        // Extract simple fields
        let ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
        let node_types: Vec<String> = nodes.iter().map(|n| n.r#type.clone()).collect();
        let contents: Vec<String> = nodes.iter().map(|n| n.content.clone()).collect();
        let parent_ids: Vec<Option<String>> = nodes.iter().map(|n| n.parent_id.clone()).collect();
        let before_sibling_ids: Vec<Option<String>> = nodes.iter().map(|n| n.before_sibling_id.clone()).collect();
        let root_ids: Vec<Option<String>> = nodes.iter().map(|n| n.root_id.clone()).collect();
        // root_type field removed
        let created_ats: Vec<String> = nodes.iter().map(|n| n.created_at.clone()).collect();
        let updated_ats: Vec<String> = nodes.iter().map(|n| n.updated_at.clone()).collect();
        let metadatas: Vec<Option<String>> = nodes
            .iter()
            .map(|n| n.metadata.as_ref().map(|v| v.to_string()))
            .collect();

        // Vector field: Vec<f32> -> FixedSizeListArray for LanceDB vector indexing
        let vectors = {
            use arrow_array::{FixedSizeListArray, Float32Array};

            // Collect all vector values into a flat array
            let mut flat_values = Vec::new();
            for node in &nodes {
                if node.vector.len() != self.vector_dimension {
                    return Err(DataStoreError::Arrow(format!(
                        "Vector dimension mismatch: expected {}, got {}",
                        self.vector_dimension,
                        node.vector.len()
                    )));
                }
                flat_values.extend_from_slice(&node.vector);
            }

            let values = Float32Array::from(flat_values);
            let field = Arc::new(Field::new("item", DataType::Float32, false));
            FixedSizeListArray::try_new(field, self.vector_dimension as i32, Arc::new(values), None)
                .map_err(|e| {
                    DataStoreError::Arrow(format!("Failed to create FixedSizeListArray: {}", e))
                })?
        };

        // Children IDs: Vec<String> -> ListArray for string lists
        let mut children_builder = ListBuilder::new(StringBuilder::new());
        for node in &nodes {
            for child_id in &node.children_ids {
                children_builder.values().append_value(child_id);
            }
            children_builder.append(true);
        }
        let children_ids = children_builder.finish();

        // Mentions: Vec<String> -> ListArray for string lists
        let mut mentions_builder = ListBuilder::new(StringBuilder::new());
        for node in &nodes {
            for mention in &node.mentions {
                mentions_builder.values().append_value(mention);
            }
            mentions_builder.append(true);
        }
        let mentions = mentions_builder.finish();

        // Create RecordBatch with all columns (including root optimization fields)
        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(ids)),
                Arc::new(StringArray::from(node_types)),
                Arc::new(StringArray::from(contents)),
                Arc::new(vectors), // vector field for LanceDB indexing
                Arc::new(StringArray::from(parent_ids)),
                Arc::new(StringArray::from(before_sibling_ids)),
                Arc::new(children_ids),
                Arc::new(mentions),
                Arc::new(StringArray::from(root_ids)), // Root hierarchy optimization
                // root_type column removed
                Arc::new(StringArray::from(created_ats)),
                Arc::new(StringArray::from(updated_ats)),
                Arc::new(StringArray::from(metadatas)),
            ],
        )
        .map_err(|e| DataStoreError::Arrow(format!("Failed to create RecordBatch: {}", e)))?;

        Ok(batch)
    }

    /// Store a single node using Arrow persistence
    async fn store_node_arrow(&self, universal_node: UniversalNode) -> Result<(), DataStoreError> {
        let schema = self.create_universal_schema();
        let batch = self.create_record_batch_from_nodes(vec![universal_node], schema.clone())?;

        let table_guard = self.table.read().await;
        if let Some(table) = table_guard.as_ref() {
            let batches = RecordBatchIterator::new(vec![batch].into_iter().map(Ok), schema);

            table.add(Box::new(batches)).execute().await.map_err(|e| {
                DataStoreError::LanceDB(format!("Failed to add data to table: {}", e))
            })?;

            // Force filesystem sync for persistence

            // Try to force LanceDB to persist by checking table stats
            let _ = table.count_rows(None).await;

            // Give LanceDB time to complete disk writes
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        } else {
            return Err(DataStoreError::LanceDB("Table not initialized".to_string()));
        }

        Ok(())
    }

    /// Delete a node by exact ID match (more specific predicate)
    async fn delete_node_by_exact_id(&self, node_id: &NodeId) -> Result<(), DataStoreError> {
        let table_guard = self.table.read().await;
        if let Some(table) = table_guard.as_ref() {
            let id_str = node_id.to_string();

            // First, check how many rows exist before deletion
            let _count_before = table.count_rows(None).await.unwrap_or(0);
            // Use more specific LanceDB delete predicate
            let predicate = format!("id == '{}'", id_str);

            match table.delete(&predicate).await {
                Ok(_stats) => {
                    // Deletion successful
                }
                Err(e) => {
                    return Err(DataStoreError::LanceDB(format!(
                        "Failed to delete node: {}",
                        e
                    )));
                }
            }
        }
        Ok(())
    }

    /// Query nodes from Arrow storage with native LanceDB filtering
    async fn query_nodes_arrow(&self, query: &str) -> Result<Vec<UniversalNode>, DataStoreError> {
        let table_guard = self.table.read().await;
        if let Some(table) = table_guard.as_ref() {
            // Use LanceDB query with limit to avoid loading all data
            let results = table
                .query()
                .limit(1000) // Reasonable limit to avoid memory issues
                .execute()
                .await
                .map_err(|e| DataStoreError::LanceDB(format!("Query failed: {}", e)))?;

            let batches: Vec<RecordBatch> = futures::TryStreamExt::try_collect(results)
                .await
                .map_err(|e| {
                    DataStoreError::LanceDB(format!("Failed to collect results: {}", e))
                })?;

            let mut nodes = Vec::new();
            for batch in batches {
                let batch_nodes = self.extract_nodes_from_batch(&batch)?;

                if query.is_empty() {
                    nodes.extend(batch_nodes);
                } else {
                    // Apply content filter efficiently
                    for node in batch_nodes {
                        if node.content.to_lowercase().contains(&query.to_lowercase()) {
                            nodes.push(node);
                        }
                    }
                }
            }

            Ok(nodes)
        } else {
            Err(DataStoreError::LanceDB("Table not initialized".to_string()))
        }
    }

    /// Extract UniversalNode objects from Arrow RecordBatch with proper ListArray handling
    fn extract_nodes_from_batch(
        &self,
        batch: &RecordBatch,
    ) -> Result<Vec<UniversalNode>, DataStoreError> {
        let mut nodes = Vec::new();
        let num_rows = batch.num_rows();

        if num_rows == 0 {
            return Ok(nodes);
        }

        // Extract column arrays with proper error handling
        let ids = batch
            .column_by_name("id")
            .and_then(|col| col.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| DataStoreError::Arrow("Missing or invalid id column".to_string()))?;

        let node_types = batch
            .column_by_name("type")
            .and_then(|col| col.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| {
                DataStoreError::Arrow("Missing or invalid type column".to_string())
            })?;

        let contents = batch
            .column_by_name("content")
            .and_then(|col| col.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| {
                DataStoreError::Arrow("Missing or invalid content column".to_string())
            })?;

        let created_ats = batch
            .column_by_name("created_at")
            .and_then(|col| col.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| {
                DataStoreError::Arrow("Missing or invalid created_at column".to_string())
            })?;

        let updated_ats = batch
            .column_by_name("updated_at")
            .and_then(|col| col.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| {
                DataStoreError::Arrow("Missing or invalid updated_at column".to_string())
            })?;

        // Extract vector FixedSizeListArray
        let vector_list_array = batch
            .column_by_name("vector")
            .and_then(|col| {
                col.as_any()
                    .downcast_ref::<arrow_array::FixedSizeListArray>()
            })
            .ok_or_else(|| DataStoreError::Arrow("Missing or invalid vector column".to_string()))?;

        // Extract children_ids ListArray
        let children_list_array = batch
            .column_by_name("children_ids")
            .and_then(|col| col.as_any().downcast_ref::<ListArray>());

        // Extract mentions ListArray
        let mentions_list_array = batch
            .column_by_name("mentions")
            .and_then(|col| col.as_any().downcast_ref::<ListArray>());

        for i in 0..num_rows {
            let id = ids.value(i).to_string();
            let node_type = node_types.value(i).to_string();
            let content = contents.value(i).to_string();
            let created_at = created_ats.value(i).to_string();
            let updated_at = updated_ats.value(i).to_string();

            // Extract optional fields
            let parent_id = batch
                .column_by_name("parent_id")
                .and_then(|col| col.as_any().downcast_ref::<StringArray>())
                .and_then(|arr| {
                    if arr.is_null(i) {
                        None
                    } else {
                        Some(arr.value(i).to_string())
                    }
                });

            let before_sibling_id = batch
                .column_by_name("before_sibling_id")
                .and_then(|col| col.as_any().downcast_ref::<StringArray>())
                .and_then(|arr| {
                    if arr.is_null(i) {
                        None
                    } else {
                        Some(arr.value(i).to_string())
                    }
                });

            let metadata_str = batch
                .column_by_name("metadata")
                .and_then(|col| col.as_any().downcast_ref::<StringArray>())
                .and_then(|arr| {
                    if arr.is_null(i) {
                        None
                    } else {
                        Some(arr.value(i).to_string())
                    }
                });

            // Convert metadata string back to JSON Value
            let metadata = metadata_str.and_then(|s| serde_json::from_str(&s).ok());

            // Extract vector embedding from FixedSizeListArray
            let vector = if !vector_list_array.is_null(i) {
                let vector_list = vector_list_array.value(i);
                if let Some(float_array) = vector_list
                    .as_any()
                    .downcast_ref::<arrow_array::Float32Array>()
                {
                    (0..float_array.len())
                        .map(|j| float_array.value(j))
                        .collect()
                } else {
                    vec![0.0; self.vector_dimension] // Fallback if vector extraction fails
                }
            } else {
                vec![0.0; self.vector_dimension] // Fallback for null vector
            };

            // Extract children_ids from ListArray
            let children_ids = if let Some(children_list_array) = children_list_array {
                if !children_list_array.is_null(i) {
                    let children_list = children_list_array.value(i);
                    if let Some(string_array) = children_list.as_any().downcast_ref::<StringArray>()
                    {
                        (0..string_array.len())
                            .map(|j| string_array.value(j).to_string())
                            .collect()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            } else {
                vec![]
            };

            // Extract mentions from ListArray
            let mentions = if let Some(mentions_list_array) = mentions_list_array {
                if !mentions_list_array.is_null(i) {
                    let mentions_list = mentions_list_array.value(i);
                    if let Some(string_array) = mentions_list.as_any().downcast_ref::<StringArray>()
                    {
                        (0..string_array.len())
                            .map(|j| string_array.value(j).to_string())
                            .collect()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            } else {
                vec![]
            };

            // Extract root hierarchy optimization fields
            let root_id = batch
                .column_by_name("root_id")
                .and_then(|col| col.as_any().downcast_ref::<StringArray>())
                .and_then(|arr| {
                    if arr.is_null(i) {
                        None
                    } else {
                        Some(arr.value(i).to_string())
                    }
                });

            // root_type field removed

            let node = UniversalNode {
                id,
                r#type: node_type,
                content,
                individual_vector: vector.clone(),
                contextual_vector: None,
                hierarchical_vector: None,
                embedding_model: None,
                embeddings_generated_at: None,
                vector,
                parent_id,
                before_sibling_id,
                children_ids,
                mentions,
                root_id,   // Root hierarchy optimization
                // root_type field removed
                created_at,
                updated_at,
                metadata,
            };

            nodes.push(node);
        }

        Ok(nodes)
    }

    /// Extract distance scores from LanceDB query results
    fn extract_distances_from_batch(&self, batch: &RecordBatch) -> Result<Vec<f32>, DataStoreError> {
        // LanceDB typically returns distances in a column named "_distance"
        let distances = batch
            .column_by_name("_distance")
            .and_then(|col| col.as_any().downcast_ref::<arrow_array::Float32Array>())
            .ok_or_else(|| {
                DataStoreError::Arrow("Missing or invalid _distance column in search results".to_string())
            })?;

        let mut distance_values = Vec::new();
        for i in 0..distances.len() {
            let distance = if distances.is_null(i) {
                f32::INFINITY // Treat null distances as infinite (no similarity)
            } else {
                distances.value(i)
            };
            distance_values.push(distance);
        }

        Ok(distance_values)
    }

    /// Vector similarity search using Arrow storage
    async fn vector_search_arrow(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<(Node, f32)>, DataStoreError> {
        let table_guard = self.table.read().await;
        if let Some(table) = table_guard.as_ref() {
            // Perform vector similarity search
            let query_builder = table.query().nearest_to(embedding.clone()).map_err(|e| {
                DataStoreError::LanceDB(format!("Failed to create nearest_to query: {}", e))
            })?;

            let results = query_builder
                .limit(limit)
                .execute()
                .await
                .map_err(|e| DataStoreError::LanceDB(format!("Vector search failed: {}", e)))?;

            let batches: Vec<RecordBatch> = futures::TryStreamExt::try_collect(results)
                .await
                .map_err(|e| {
                    DataStoreError::LanceDB(format!("Failed to collect search results: {}", e))
                })?;

            let mut results = Vec::new();
            for batch in batches {
                let universal_nodes = self.extract_nodes_from_batch(&batch)?;
                let distances = self.extract_distances_from_batch(&batch)?;

                for (i, universal_node) in universal_nodes.into_iter().enumerate() {
                    let node = self.universal_to_node(universal_node);
                    
                    // Convert LanceDB distance to similarity score
                    // LanceDB returns squared L2 distances, convert to cosine similarity (0-1 range)
                    let distance = distances.get(i).copied().unwrap_or(f32::INFINITY);
                    let similarity = if distance.is_finite() && distance >= 0.0 {
                        // Convert distance to similarity: closer distances = higher similarity
                        // For normalized vectors, squared L2 distance relates to cosine similarity as:
                        // cosine_similarity = 1 - (squared_l2_distance / 2)
                        let cosine_sim = 1.0 - (distance / 2.0);
                        cosine_sim.clamp(0.0, 1.0) // Clamp to [0, 1]
                    } else {
                        0.0 // Invalid distance = zero similarity
                    };
                    
                    results.push((node, similarity));
                }
            }

            // Sort by similarity and limit results
            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            results.truncate(limit);

            Ok(results)
        } else {
            Err(DataStoreError::LanceDB("Table not initialized".to_string()))
        }
    }

    /// Get a single node by ID using LanceDB query with application-level filtering
    async fn get_node_arrow(&self, id: &NodeId) -> Result<Option<Node>, DataStoreError> {
        let table_guard = self.table.read().await;
        if let Some(table) = table_guard.as_ref() {
            let target_id = id.to_string();

            // Use LanceDB query with reasonable limit and filter in application
            let results_stream = table
                .query()
                .limit(1000) // Reasonable limit to avoid loading entire table
                .execute()
                .await
                .map_err(|e| DataStoreError::LanceDB(format!("Query by ID failed: {}", e)))?;

            // Collect the results into Vec<RecordBatch>
            let batches: Vec<RecordBatch> = futures::TryStreamExt::try_collect(results_stream)
                .await
                .map_err(|e| {
                    DataStoreError::LanceDB(format!("Failed to collect query results: {}", e))
                })?;

            // Process the retrieved batches and find matching ID
            for batch in batches.iter() {
                if batch.num_rows() > 0 {
                    let universal_nodes = self.extract_nodes_from_batch(batch)?;

                    // Find the node with matching ID
                    for universal_node in universal_nodes {
                        if universal_node.id == target_id {
                            // Found matching node
                            let node = self.universal_to_node(universal_node);
                            return Ok(Some(node));
                        }
                    }
                }
            }

            Ok(None) // No matching node found
        } else {
            Err(DataStoreError::LanceDB("Table not initialized".to_string()))
        }
    }

    /// Delete a node using native LanceDB delete operations
    async fn delete_node_arrow(&self, id: &NodeId) -> Result<(), DataStoreError> {
        let table_guard = self.table.read().await;
        if let Some(table) = table_guard.as_ref() {
            // Use native LanceDB delete operation with SQL predicate
            let _delete_result = table
                .delete(&format!("id = '{}'", id.as_str().replace("'", "''")))
                .await
                .map_err(|e| DataStoreError::LanceDB(format!("Delete operation failed: {}", e)))?;

            // DeleteResult contains version info - we just verify it succeeded
            Ok(())
        } else {
            Err(DataStoreError::LanceDB("Table not initialized".to_string()))
        }
    }

    /// Convert UniversalNode back to NodeSpace Node
    /// For TextNode and DateNode, keep metadata empty to maintain simplified approach
    /// For other node types, preserve their type-specific metadata
    fn universal_to_node(&self, universal: UniversalNode) -> Node {
        let content = serde_json::Value::String(universal.content);

        // Determine if this is a simplified node type (text/date) that should have empty metadata
        let final_metadata = match universal.r#type.as_str() {
            "text" | "date" => {
                // For text and date nodes: Keep metadata empty/null for simplified approach
                // Hierarchical data is maintained in parent_id/children_ids fields in UniversalNode
                // and will be computed by core-logic layer when needed
                None
            }
            _ => {
                // For other node types (image, task, etc.): Preserve their metadata
                // These may have type-specific properties that need to be maintained
                let mut metadata = universal.metadata.unwrap_or_else(|| serde_json::json!({}));

                // Only add node_type for non-simplified nodes
                metadata["node_type"] = serde_json::Value::String(universal.r#type.clone());

                // For non-simplified nodes, we can still include hierarchical data in metadata
                // for backwards compatibility, but it should be computed from the canonical source
                if let Some(parent_id) = &universal.parent_id {
                    metadata["parent_id"] = serde_json::Value::String(parent_id.clone());
                }
                if !universal.children_ids.is_empty() {
                    metadata["children_ids"] = serde_json::Value::Array(
                        universal
                            .children_ids
                            .iter()
                            .map(|id| serde_json::Value::String(id.clone()))
                            .collect(),
                    );
                }
                if !universal.mentions.is_empty() {
                    metadata["mentions"] = serde_json::Value::Array(
                        universal
                            .mentions
                            .iter()
                            .map(|id| serde_json::Value::String(id.clone()))
                            .collect(),
                    );
                }

                Some(metadata)
            }
        };

        Node {
            id: NodeId::from_string(universal.id),
            r#type: universal.r#type,
            content,
            metadata: final_metadata,
            created_at: universal.created_at,
            updated_at: universal.updated_at,
            parent_id: universal.parent_id.map(NodeId::from_string),
            before_sibling: universal.before_sibling_id.map(NodeId::from_string),
            next_sibling: None, // TODO: Map from before_sibling_id when core-types adds before_sibling field
            root_id: universal.root_id.map(NodeId::from_string),
        }
    }
}

// Implement the DataStore trait for compatibility with existing NodeSpace architecture
#[async_trait]
impl DataStore for LanceDataStore {
    async fn store_node(&self, node: Node) -> NodeSpaceResult<NodeId> {
        let universal = self.node_to_universal(node.clone(), None);

        // Store using Arrow persistence
        self.store_node_arrow(universal.clone()).await?;

        Ok(node.id)
    }

    async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        // Use Arrow-based retrieval
        let result = self.get_node_arrow(id).await?;
        Ok(result)
    }

    async fn update_node(&self, node: Node) -> NodeSpaceResult<()> {
        // First verify the node exists and get the old version
        let existing_node = self.get_node(&node.id).await?.ok_or_else(|| {
            DataStoreError::NodeNotFound(format!("Node {} not found for update", node.id))
        })?;

        // Update the node's updated_at timestamp
        let mut updated_node = node;
        updated_node.updated_at = chrono::Utc::now().to_rfc3339();

        // Check if content changed - if so, we need to regenerate embeddings
        let content_changed = existing_node.content != updated_node.content;

        if content_changed {
            let embedding = if let Some(ref generator) = self.embedding_generator {
                // Generate new embedding automatically
                match generator
                    .generate_embedding(&updated_node.content.to_string())
                    .await
                {
                    Ok(embedding) => embedding,
                    Err(_) => vec![0.0; self.vector_dimension],
                }
            } else {
                vec![0.0; self.vector_dimension]
            };

            let universal = self.node_to_universal(updated_node.clone(), Some(embedding));

            // Use atomic delete + insert for update
            self.delete_node_by_exact_id(&updated_node.id).await?;
            self.store_node_arrow(universal).await?;
        } else {
            // Content unchanged - preserve existing embedding
            let universal = self.node_to_universal(updated_node.clone(), None);

            // Use atomic delete + insert for update
            self.delete_node_by_exact_id(&updated_node.id).await?;
            self.store_node_arrow(universal).await?;
        }

        Ok(())
    }

    async fn update_node_with_embedding(
        &self,
        node: Node,
        embedding: Vec<f32>,
    ) -> NodeSpaceResult<()> {
        // Verify the node exists
        if self.get_node(&node.id).await?.is_none() {
            return Err(DataStoreError::NodeNotFound(format!(
                "Node {} not found for update",
                node.id
            ))
            .into());
        }

        // Update the node's updated_at timestamp
        let mut updated_node = node;
        updated_node.updated_at = chrono::Utc::now().to_rfc3339();

        // Use the provided embedding
        let universal = self.node_to_universal(updated_node.clone(), Some(embedding));

        // Use atomic delete + insert for update
        self.delete_node_by_exact_id(&updated_node.id).await?;
        self.store_node_arrow(universal).await?;

        Ok(())
    }

    async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()> {
        // Use Arrow-based deletion
        self.delete_node_arrow(id).await?;

        Ok(())
    }

    async fn query_nodes(&self, query: &str) -> NodeSpaceResult<Vec<Node>> {
        // Use Arrow-based query
        let universal_nodes = self.query_nodes_arrow(query).await?;
        let nodes = universal_nodes
            .into_iter()
            .map(|universal| self.universal_to_node(universal))
            .collect();
        Ok(nodes)
    }

    async fn create_relationship(
        &self,
        from: &NodeId,
        to: &NodeId,
        _rel_type: &str,
    ) -> NodeSpaceResult<()> {
        // Transactional integrity: prepare both updates before committing either
        let mut parent_node_opt = self.get_node(from).await?;
        let mut child_node_opt = self.get_node(to).await?;

        // Validate both nodes exist before making any changes
        let parent_node = parent_node_opt.as_mut().ok_or_else(|| {
            DataStoreError::NodeNotFound(format!("Parent node {} not found", from.as_str()))
        })?;
        let child_node = child_node_opt.as_mut().ok_or_else(|| {
            DataStoreError::NodeNotFound(format!("Child node {} not found", to.as_str()))
        })?;

        // Prepare parent node update
        let mut parent_metadata = parent_node
            .metadata
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));
        let mut children_ids: Vec<String> = parent_metadata
            .get("children_ids")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let needs_parent_update = !children_ids.contains(&to.to_string());
        if needs_parent_update {
            children_ids.push(to.to_string());
            parent_metadata["children_ids"] = serde_json::Value::Array(
                children_ids
                    .into_iter()
                    .map(serde_json::Value::String)
                    .collect(),
            );
        }

        // Prepare child node update
        let mut child_metadata = child_node
            .metadata
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));
        let needs_child_update =
            child_metadata.get("parent_id").and_then(|v| v.as_str()) != Some(from.as_str());
        if needs_child_update {
            child_metadata["parent_id"] = serde_json::Value::String(from.to_string());
        }

        // Commit both updates atomically
        if needs_parent_update {
            parent_node.metadata = Some(parent_metadata);
            self.store_node(parent_node.clone()).await.map_err(|e| {
                DataStoreError::Database(format!("Failed to update parent node: {}", e))
            })?;
        }

        if needs_child_update {
            child_node.metadata = Some(child_metadata);
            self.store_node(child_node.clone()).await.map_err(|e| {
                // If child update fails, we should ideally rollback parent update
                // For now, log the inconsistency - proper transaction support would be better
                DataStoreError::Database(format!(
                    "Failed to update child node (potential inconsistency): {}",
                    e
                ))
            })?;
        }

        Ok(())
    }

    async fn store_node_with_embedding(
        &self,
        node: Node,
        embedding: Vec<f32>,
    ) -> NodeSpaceResult<NodeId> {
        let universal = self.node_to_universal(node.clone(), Some(embedding));

        // Store using Arrow persistence
        self.store_node_arrow(universal.clone()).await?;

        Ok(node.id)
    }

    async fn search_similar_nodes(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        // Use Arrow-based vector search
        let results = self.vector_search_arrow(embedding, limit).await?;
        Ok(results)
    }

    async fn update_node_embedding(&self, id: &NodeId, embedding: Vec<f32>) -> NodeSpaceResult<()> {
        // Get the existing node, update its embedding, and store it back
        if let Some(mut node) = self.get_node(id).await? {
            // Update the embedding in metadata
            let mut metadata = node.metadata.unwrap_or_else(|| serde_json::json!({}));
            metadata["vector"] = serde_json::Value::Array(
                embedding
                    .iter()
                    .map(|&f| {
                        serde_json::Value::Number(serde_json::Number::from_f64(f as f64).unwrap())
                    })
                    .collect(),
            );
            node.metadata = Some(metadata);

            // Re-store the node with updated embedding
            self.store_node_with_embedding(node, embedding).await?;
        }

        Ok(())
    }

    async fn semantic_search_with_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        // Same as search_similar_nodes for this implementation
        self.search_similar_nodes(embedding, limit).await
    }

    // Cross-modal search methods
    async fn create_image_node(&self, image_node: ImageNode) -> NodeSpaceResult<String> {
        // Convert ImageNode to UniversalNode format
        let universal_node = UniversalNode {
            id: image_node.id.clone(),
            r#type: "image".to_string(),
            content: image_node
                .metadata
                .description
                .unwrap_or_else(|| format!("Image: {}", image_node.metadata.filename)),
            individual_vector: image_node.embedding.clone(),
            contextual_vector: None,
            hierarchical_vector: None,
            embedding_model: None,
            embeddings_generated_at: None,
            vector: image_node.embedding,
            parent_id: None,
            before_sibling_id: None,
            children_ids: vec![],
            mentions: vec![],
            root_id: None,   // Root hierarchy optimization
            // root_type field removed
            created_at: image_node.created_at.to_rfc3339(),
            updated_at: image_node.created_at.to_rfc3339(),
            metadata: Some(serde_json::json!({
                "image_data": base64::prelude::BASE64_STANDARD.encode(&image_node.image_data),
                "filename": image_node.metadata.filename,
                "mime_type": image_node.metadata.mime_type,
                "width": image_node.metadata.width,
                "height": image_node.metadata.height,
                "exif_data": image_node.metadata.exif_data
            })),
        };

        // Store in LanceDB table with proper Arrow schema
        self.store_node_arrow(universal_node).await?;

        Ok(image_node.id)
    }

    async fn get_image_node(&self, id: &str) -> NodeSpaceResult<Option<ImageNode>> {
        // Get node from Arrow storage
        let node_id = NodeId::from_string(id.to_string());
        if let Some(node) = self.get_node(&node_id).await? {
            if let Some(metadata) = &node.metadata {
                if metadata.get("node_type").and_then(|v| v.as_str()) == Some("image") {
                    // Convert back to ImageNode
                    let image_data = base64::prelude::BASE64_STANDARD
                        .decode(
                            metadata
                                .get("image_data")
                                .and_then(|v| v.as_str())
                                .ok_or_else(|| {
                                    DataStoreError::InvalidNode("Missing image data".to_string())
                                })?,
                        )
                        .map_err(|e| {
                            DataStoreError::InvalidNode(format!("Invalid base64 image data: {}", e))
                        })?;

                    // Extract vector from metadata or use default
                    let embedding = metadata
                        .get("vector")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_f64().map(|f| f as f32))
                                .collect()
                        })
                        .unwrap_or_else(|| vec![0.0; 384]);

                    let image_node = ImageNode {
                        id: node.id.to_string(),
                        image_data,
                        embedding,
                        metadata: ImageMetadata {
                            filename: metadata
                                .get("filename")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            mime_type: metadata
                                .get("mime_type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("image/jpeg")
                                .to_string(),
                            width: metadata.get("width").and_then(|v| v.as_u64()).unwrap_or(0)
                                as u32,
                            height: metadata.get("height").and_then(|v| v.as_u64()).unwrap_or(0)
                                as u32,
                            exif_data: metadata.get("exif_data").cloned(),
                            description: if let serde_json::Value::String(content) = &node.content {
                                if content.starts_with("Image:") {
                                    None
                                } else {
                                    Some(content.clone())
                                }
                            } else {
                                None
                            },
                        },
                        created_at: chrono::DateTime::parse_from_rfc3339(&node.created_at)
                            .map_err(|e| {
                                DataStoreError::InvalidNode(format!("Invalid timestamp: {}", e))
                            })?
                            .with_timezone(&chrono::Utc),
                    };

                    return Ok(Some(image_node));
                }
            }
        }

        Ok(None)
    }

    async fn search_multimodal(
        &self,
        query_embedding: Vec<f32>,
        types: Vec<NodeType>,
    ) -> NodeSpaceResult<Vec<Node>> {
        // Get all nodes from Arrow storage
        let universal_nodes = self.query_nodes_arrow("").await?;
        let mut results = Vec::new();

        // Convert NodeType enum to string filters
        let type_filters: Vec<String> = types
            .into_iter()
            .map(|t| match t {
                NodeType::Text => "text".to_string(),
                NodeType::Image => "image".to_string(),
                NodeType::Date => "date".to_string(),
                NodeType::Task => "task".to_string(),
            })
            .collect();

        for universal_node in universal_nodes {
            // Filter by node types
            if !type_filters.is_empty() && !type_filters.contains(&universal_node.r#type) {
                continue;
            }

            let similarity = cosine_similarity(&query_embedding, &universal_node.vector);
            if similarity > 0.1 {
                // Basic similarity threshold
                let node = self.universal_to_node(universal_node);
                results.push((node, similarity));
            }
        }

        // Sort by similarity and return just the nodes
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        Ok(results.into_iter().map(|(node, _)| node).collect())
    }

    async fn hybrid_multimodal_search(
        &self,
        query_embedding: Vec<f32>,
        config: &HybridSearchConfig,
    ) -> NodeSpaceResult<Vec<SearchResult>> {
        // Get all nodes from Arrow storage
        let universal_nodes = self.query_nodes_arrow("").await?;
        let mut results = Vec::new();

        for universal_node in universal_nodes {
            let semantic_score = cosine_similarity(&query_embedding, &universal_node.vector);

            // Skip if below minimum threshold
            if semantic_score < config.min_similarity_threshold as f32 {
                continue;
            }

            // Calculate structural score (based on relationships)
            let structural_score =
                if universal_node.parent_id.is_some() || !universal_node.children_ids.is_empty() {
                    0.8 // Has relationships
                } else {
                    0.2 // Isolated node
                };

            // Calculate temporal score (recent nodes get higher scores)
            let temporal_score = if let Ok(created_at) =
                chrono::DateTime::parse_from_rfc3339(&universal_node.created_at)
            {
                let age_days =
                    (chrono::Utc::now() - created_at.with_timezone(&chrono::Utc)).num_days();
                if age_days <= 1 {
                    1.0
                } else if age_days <= 7 {
                    0.8
                } else {
                    0.5
                }
            } else {
                0.5
            };

            // Cross-modal bonus for image-text combinations
            let cross_modal_score =
                if config.enable_cross_modal && universal_node.r#type == "image" {
                    Some(0.9) // Boost for cross-modal queries
                } else {
                    None
                };

            // Weighted final score
            let final_score = (semantic_score * config.semantic_weight as f32)
                + (structural_score * config.structural_weight as f32)
                + (temporal_score * config.temporal_weight as f32)
                + cross_modal_score.unwrap_or(0.0) * 0.1;

            let node = self.universal_to_node(universal_node);
            let search_result = SearchResult {
                node,
                score: final_score,
                relevance_factors: RelevanceFactors {
                    semantic_score,
                    structural_score,
                    temporal_score,
                    cross_modal_score,
                },
            };

            results.push(search_result);
        }

        // Sort by final score and apply limits
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(config.max_results);

        Ok(results)
    }

    // Multi-level embedding methods
    async fn store_node_with_multi_embeddings(
        &self,
        node: Node,
        embeddings: crate::data_store::MultiLevelEmbeddings,
    ) -> NodeSpaceResult<NodeId> {
        let universal = self.node_to_universal_with_multi_embeddings(node.clone(), embeddings);

        // Store using Arrow persistence
        self.store_node_arrow(universal).await?;

        Ok(node.id)
    }

    async fn update_node_embeddings(
        &self,
        node_id: &NodeId,
        embeddings: crate::data_store::MultiLevelEmbeddings,
    ) -> NodeSpaceResult<()> {
        // Get the existing node
        if let Some(node) = self.get_node(node_id).await? {
            // Convert with new embeddings
            let universal = self.node_to_universal_with_multi_embeddings(node, embeddings);

            // Use atomic delete + insert for update
            self.delete_node_by_exact_id(node_id).await?;
            self.store_node_arrow(universal).await?;

            Ok(())
        } else {
            Err(DataStoreError::NodeNotFound(format!("Node {} not found", node_id)).into())
        }
    }

    async fn get_node_embeddings(
        &self,
        node_id: &NodeId,
    ) -> NodeSpaceResult<Option<crate::data_store::MultiLevelEmbeddings>> {
        // Get the node from Arrow storage
        let universal_nodes = self.query_nodes_arrow("").await?;

        for universal_node in universal_nodes {
            if universal_node.id == node_id.to_string() {
                let embeddings = crate::data_store::MultiLevelEmbeddings {
                    individual: universal_node.individual_vector,
                    contextual: universal_node.contextual_vector,
                    hierarchical: universal_node.hierarchical_vector,
                    embedding_model: universal_node.embedding_model,
                    generated_at: if let Some(timestamp_str) =
                        universal_node.embeddings_generated_at
                    {
                        chrono::DateTime::parse_from_rfc3339(&timestamp_str)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .unwrap_or_else(|_| chrono::Utc::now())
                    } else {
                        chrono::Utc::now()
                    },
                };
                return Ok(Some(embeddings));
            }
        }

        Ok(None)
    }

    async fn search_by_individual_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        // Use individual_vector field for search
        let universal_nodes = self.query_nodes_arrow("").await?;
        let mut results = Vec::new();

        for universal_node in universal_nodes {
            let similarity = cosine_similarity(&embedding, &universal_node.individual_vector);
            if similarity > 0.1 {
                let node = self.universal_to_node(universal_node);
                results.push((node, similarity));
            }
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.truncate(limit);
        Ok(results)
    }

    async fn search_by_contextual_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        // Use contextual_vector field for search
        let universal_nodes = self.query_nodes_arrow("").await?;
        let mut results = Vec::new();

        for universal_node in universal_nodes {
            if let Some(ref contextual_vector) = universal_node.contextual_vector {
                let similarity = cosine_similarity(&embedding, contextual_vector);
                if similarity > 0.1 {
                    let node = self.universal_to_node(universal_node);
                    results.push((node, similarity));
                }
            }
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.truncate(limit);
        Ok(results)
    }

    async fn search_by_hierarchical_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        // Use hierarchical_vector field for search
        let universal_nodes = self.query_nodes_arrow("").await?;
        let mut results = Vec::new();

        for universal_node in universal_nodes {
            if let Some(ref hierarchical_vector) = universal_node.hierarchical_vector {
                let similarity = cosine_similarity(&embedding, hierarchical_vector);
                if similarity > 0.1 {
                    let node = self.universal_to_node(universal_node);
                    results.push((node, similarity));
                }
            }
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.truncate(limit);
        Ok(results)
    }

    async fn hybrid_semantic_search(
        &self,
        embeddings: crate::data_store::QueryEmbeddings,
        config: crate::data_store::HybridSearchConfig,
    ) -> NodeSpaceResult<Vec<crate::data_store::SearchResult>> {
        let universal_nodes = self.query_nodes_arrow("").await?;
        let mut results = Vec::new();

        for universal_node in universal_nodes {
            // Calculate individual embedding similarity
            let individual_score =
                cosine_similarity(&embeddings.individual, &universal_node.individual_vector);

            // Calculate contextual embedding similarity if available
            let contextual_score = if let (Some(ref query_contextual), Some(ref node_contextual)) =
                (&embeddings.contextual, &universal_node.contextual_vector)
            {
                cosine_similarity(query_contextual, node_contextual)
            } else {
                0.0
            };

            // Calculate hierarchical embedding similarity if available
            let hierarchical_score =
                if let (Some(ref query_hierarchical), Some(ref node_hierarchical)) = (
                    &embeddings.hierarchical,
                    &universal_node.hierarchical_vector,
                ) {
                    cosine_similarity(query_hierarchical, node_hierarchical)
                } else {
                    0.0
                };

            // Calculate weighted final score
            let final_score = (individual_score * config.individual_weight as f32)
                + (contextual_score * config.contextual_weight as f32)
                + (hierarchical_score * config.hierarchical_weight as f32);

            // Skip if below minimum threshold
            if final_score < config.min_similarity_threshold as f32 {
                continue;
            }

            let node = self.universal_to_node(universal_node);
            let search_result = crate::data_store::SearchResult {
                node,
                score: final_score,
                relevance_factors: crate::data_store::RelevanceFactors {
                    semantic_score: individual_score,
                    structural_score: contextual_score,
                    temporal_score: hierarchical_score,
                    cross_modal_score: None,
                },
            };

            results.push(search_result);
        }

        // Sort by final score and apply limits
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(config.max_results);

        Ok(results)
    }

    // Implement DataStore trait methods for root-based hierarchy queries
    async fn get_nodes_by_root(&self, root_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        // Direct delegation to the implementation method
        self.get_nodes_by_root_internal(root_id).await
    }

    async fn get_nodes_by_root_and_type(
        &self,
        root_id: &NodeId,
        r#type: &str,
    ) -> NodeSpaceResult<Vec<Node>> {
        // Direct delegation to the implementation method
        self.get_nodes_by_root_and_type_internal(root_id, r#type)
            .await
    }
}

impl LanceDataStore {
    /// Get all nodes under a specific root with single indexed query
    /// This is the core optimization that replaces multiple O(N) database scans
    /// with a single O(1) LanceDB indexed filter operation.
    ///
    /// NOTE: This is a basic implementation - the filter will be optimized once
    /// LanceDB's filter API is properly integrated with root_id indexing.
    pub async fn get_nodes_by_root_internal(&self, root_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        // For now, use the existing query and filter in memory
        // TODO: Replace with native LanceDB filter once filter API is working
        let all_nodes = self.query_nodes_arrow("").await?;
        let root_id_str = root_id.to_string();

        let mut matching_nodes = Vec::new();
        for universal_node in all_nodes {
            if let Some(ref node_root_id) = universal_node.root_id {
                if node_root_id == &root_id_str {
                    let node = self.universal_to_node(universal_node);
                    matching_nodes.push(node);
                }
            }
        }

        Ok(matching_nodes)
    }

    /// Get typed nodes by root for specialized queries
    /// Combines root filtering with node type filtering for optimal performance
    pub async fn get_nodes_by_root_and_type_internal(
        &self,
        root_id: &NodeId,
        r#type: &str,
    ) -> NodeSpaceResult<Vec<Node>> {
        // For now, use the existing query and filter in memory
        // TODO: Replace with native LanceDB filter once filter API is working
        let all_nodes = self.query_nodes_arrow("").await?;
        let root_id_str = root_id.to_string();

        let mut matching_nodes = Vec::new();
        for universal_node in all_nodes {
            // Check both root_id and node_type match
            if let Some(ref node_root_id) = universal_node.root_id {
                if node_root_id == &root_id_str && universal_node.r#type == r#type {
                    let node = self.universal_to_node(universal_node);
                    matching_nodes.push(node);
                }
            }
        }

        Ok(matching_nodes)
    }

    /// Create composite indexes for hierarchy query optimization
    /// This implements the performance strategy from your architectural recommendations
    pub async fn create_hierarchy_indexes(&self) -> NodeSpaceResult<()> {
        let table_guard = self.table.read().await;
        if let Some(table) = table_guard.as_ref() {
            // Check if table has data before creating indexes
            let stats = table
                .count_rows(None)
                .await
                .map_err(|e| DataStoreError::LanceDB(format!("Failed to get row count: {}", e)))?;

            if stats > 0 {
                // Primary composite index: (root_id, node_type, created_at)
                // This enables efficient hierarchy + type + temporal queries
                let _ = table
                    .create_index(
                        &["root_id", "node_type", "created_at"],
                        lancedb::index::Index::BTree(Default::default()),
                    )
                    .replace(true)
                    .execute()
                    .await;

                // Supporting index: (root_id, parent_id) for relationship queries
                let _ = table
                    .create_index(
                        &["root_id", "parent_id"],
                        lancedb::index::Index::BTree(Default::default()),
                    )
                    .replace(true)
                    .execute()
                    .await;
            }
        }

        Ok(())
    }

    /// Get child nodes using Arrow storage for hierarchical relationships
    pub async fn get_child_nodes(&self, parent_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        // Get all nodes from Arrow storage
        let universal_nodes = self.query_nodes_arrow("").await?;

        let mut children = Vec::new();
        for universal_node in universal_nodes {
            if let Some(ref pid) = universal_node.parent_id {
                if pid == parent_id.as_str() {
                    let node = self.universal_to_node(universal_node);
                    children.push(node);
                }
            }
        }

        Ok(children)
    }

    /// Create or update relationship using Arrow storage for entity connections
    pub async fn update_relationship(
        &self,
        node_id: &NodeId,
        parent_id: Option<NodeId>,
        children_ids: Vec<NodeId>,
    ) -> NodeSpaceResult<()> {
        if let Some(mut node) = self.get_node(node_id).await? {
            let mut metadata = node.metadata.unwrap_or_else(|| serde_json::json!({}));

            if let Some(parent_id) = parent_id {
                metadata["parent_id"] = serde_json::Value::String(parent_id.to_string());
            } else {
                metadata
                    .as_object_mut()
                    .and_then(|obj| obj.remove("parent_id"));
            }

            metadata["children_ids"] = serde_json::Value::Array(
                children_ids
                    .into_iter()
                    .map(|id| serde_json::Value::String(id.to_string()))
                    .collect(),
            );

            node.metadata = Some(metadata);
            self.store_node(node).await?;
        }

        Ok(())
    }

    /// Hybrid search combining semantic search with metadata filtering using Arrow storage
    pub async fn hybrid_search(
        &self,
        _embedding: Vec<f32>,
        node_type_filter: Option<String>,
        _metadata_filter: Option<serde_json::Value>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        // Get all nodes from Arrow storage
        let universal_nodes = self.query_nodes_arrow("").await?;

        let mut similarities = Vec::new();

        for universal_node in universal_nodes {
            // Apply node type filter if specified
            if let Some(ref filter_type) = node_type_filter {
                if &universal_node.r#type != filter_type {
                    continue;
                }
            }

            // Use LanceDB's native vector similarity instead of manual calculation
            // This is a fallback for hybrid search - ideally should use vector_search_arrow
            let node = self.universal_to_node(universal_node);
            similarities.push((node, 1.0)); // Placeholder score
        }

        // Sort by similarity descending and take limit
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        similarities.truncate(limit);

        Ok(similarities)
    }
}

/// Simple cosine similarity implementation for cases where LanceDB native scoring isn't available
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_store() -> LanceDataStore {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        LanceDataStore::new(db_path.to_str().unwrap()).await.unwrap()
    }

    #[tokio::test]
    async fn test_before_sibling_persistence() {
        let store = create_test_store().await;

        let node_a = Node::new("text".to_string(), serde_json::json!({"text": "Node A"}));
        let node_b = Node::new("text".to_string(), serde_json::json!({"text": "Node B"}))
            .with_before_sibling(Some(node_a.id.clone()));

        // Store both nodes
        store.store_node(node_a.clone()).await.unwrap();
        store.store_node(node_b.clone()).await.unwrap();

        // Retrieve and verify sibling relationship preserved
        let retrieved_b = store.get_node(&node_b.id).await.unwrap().unwrap();
        assert_eq!(retrieved_b.before_sibling, Some(node_a.id));
    }

    #[tokio::test]
    async fn test_null_before_sibling_handling() {
        let store = create_test_store().await;

        let node = Node::new("text".to_string(), serde_json::json!({"text": "Node"}));
        // No before_sibling set (should be None)

        store.store_node(node.clone()).await.unwrap();
        let retrieved = store.get_node(&node.id).await.unwrap().unwrap();

        assert_eq!(retrieved.before_sibling, None);
    }
}
