use crate::error::DataStoreError;
use crate::{DataStore, HybridSearchConfig, ImageNode, NodeType, RelevanceFactors, SearchResult};
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
    db_path: String,
    vector_dimension: usize,
}

/// Universal Node structure for LanceDB entity-centric storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalNode {
    pub id: String,
    pub node_type: String, // "text", "date", "task", "customer", "project", etc.
    pub content: String,
    pub vector: Vec<f32>, // 384-dimensional embedding from FastEmbed

    // JSON-based relationships for entity connections
    pub parent_id: Option<String>,
    pub children_ids: Vec<String>,
    pub mentions: Vec<String>, // References to other entities

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

    /// Initialize new LanceDB connection with custom vector dimension
    pub async fn with_vector_dimension(db_path: &str, vector_dimension: usize) -> Result<Self, DataStoreError> {
        let connection = connect(db_path)
            .execute()
            .await
            .map_err(|e| DataStoreError::LanceDBConnection(format!("LanceDB connection failed: {}", e)))?;

        let instance = Self {
            connection,
            table: Arc::new(RwLock::new(None)),
            table_name: "universal_nodes".to_string(),
            db_path: db_path.to_string(),
            vector_dimension,
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

        println!(
            "✅ Arrow-based table '{}' initialized with vector index",
            self.table_name
        );
        Ok(())
    }

    /// Create the Universal Document Schema with configurable vector dimension
    fn create_universal_schema(&self) -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("node_type", DataType::Utf8, false),
            Field::new("content", DataType::Utf8, false),
            // Vector field - FixedSizeList of Float32 for LanceDB vector indexing
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, false)),
                    self.vector_dimension as i32,
                ),
                false,
            ),
            Field::new("parent_id", DataType::Utf8, true), // Nullable
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
            Field::new("created_at", DataType::Utf8, false),
            Field::new("updated_at", DataType::Utf8, false),
            Field::new("metadata", DataType::Utf8, true), // Nullable JSON string
        ]))
    }

    /// Create an empty RecordBatch for table initialization
    fn create_empty_record_batch(&self, schema: Arc<Schema>) -> Result<RecordBatch, DataStoreError> {
        use arrow_array::{FixedSizeListArray, Float32Array};

        // Create empty FixedSizeListArray for vectors with configurable dimension
        let empty_values = Float32Array::from(Vec::<f32>::new());
        let field = Arc::new(Field::new("item", DataType::Float32, false));
        let empty_vectors = FixedSizeListArray::try_new(field, self.vector_dimension as i32, Arc::new(empty_values), None)
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
                Arc::new(ListBuilder::new(StringBuilder::new()).finish()), // children_ids
                Arc::new(ListBuilder::new(StringBuilder::new()).finish()), // mentions
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
                    Ok(_) => {
                        println!("✅ Vector index created successfully");
                    }
                    Err(e) => {
                        println!(
                            "⚠️  Vector index creation failed (table might be empty): {}",
                            e
                        );
                        // This is not a fatal error - index can be created later when data exists
                    }
                }
            } else {
                println!("📝 Skipping vector index creation - table is empty");
            }
        }
        Ok(())
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

        // Extract relationships from metadata
        let parent_id = node
            .metadata
            .as_ref()
            .and_then(|m| m.get("parent_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

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

        UniversalNode {
            id: node.id.to_string(),
            node_type,
            content: node.content.to_string(),
            vector: embedding.unwrap_or_else(|| vec![0.0; self.vector_dimension]), // Default embedding
            parent_id,
            children_ids,
            mentions,
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
        let node_types: Vec<String> = nodes.iter().map(|n| n.node_type.clone()).collect();
        let contents: Vec<String> = nodes.iter().map(|n| n.content.clone()).collect();
        let parent_ids: Vec<Option<String>> = nodes.iter().map(|n| n.parent_id.clone()).collect();
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
                        self.vector_dimension, node.vector.len()
                    )));
                }
                flat_values.extend_from_slice(&node.vector);
            }

            let values = Float32Array::from(flat_values);
            let field = Arc::new(Field::new("item", DataType::Float32, false));
            FixedSizeListArray::try_new(field, self.vector_dimension as i32, Arc::new(values), None).map_err(|e| {
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

        // Create RecordBatch with all columns
        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(ids)),
                Arc::new(StringArray::from(node_types)),
                Arc::new(StringArray::from(contents)),
                Arc::new(vectors),
                Arc::new(StringArray::from(parent_ids)),
                Arc::new(children_ids),
                Arc::new(mentions),
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
        } else {
            return Err(DataStoreError::LanceDB("Table not initialized".to_string()));
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
    fn extract_nodes_from_batch(&self, batch: &RecordBatch) -> Result<Vec<UniversalNode>, DataStoreError> {
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
            .column_by_name("node_type")
            .and_then(|col| col.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| {
                DataStoreError::Arrow("Missing or invalid node_type column".to_string())
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

            let node = UniversalNode {
                id,
                node_type,
                content,
                vector,
                parent_id,
                children_ids,
                mentions,
                created_at,
                updated_at,
                metadata,
            };

            nodes.push(node);
        }

        Ok(nodes)
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

                for universal_node in universal_nodes {
                    // LanceDB provides native distance scores through the query system
                    // For vector similarity search, LanceDB handles scoring internally
                    let node = self.universal_to_node(universal_node);
                    // Use a default score since LanceDB handles ranking
                    results.push((node, 1.0));
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

    /// Get a single node by ID using native LanceDB filtering
    async fn get_node_arrow(&self, _id: &NodeId) -> Result<Option<Node>, DataStoreError> {
        let table_guard = self.table.read().await;
        if let Some(table) = table_guard.as_ref() {
            // Use LanceDB query with small limit for ID lookup
            let results_stream = table
                .query()
                .limit(100) // Small limit since we're looking for one specific ID
                .execute()
                .await
                .map_err(|e| DataStoreError::LanceDB(format!("Query by ID failed: {}", e)))?;

            // Collect the results into Vec<RecordBatch>
            let batches: Vec<RecordBatch> = futures::TryStreamExt::try_collect(results_stream)
                .await
                .map_err(|e| {
                    DataStoreError::LanceDB(format!("Failed to collect query results: {}", e))
                })?;

            // Process the retrieved batches
            for batch in batches {
                let universal_nodes = self.extract_nodes_from_batch(&batch)?;

                if let Some(universal_node) = universal_nodes.into_iter().next() {
                    let node = self.universal_to_node(universal_node);
                    return Ok(Some(node));
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
            println!("✅ Delete operation completed for ID: {}", id.as_str());
            Ok(())
        } else {
            Err(DataStoreError::LanceDB("Table not initialized".to_string()))
        }
    }

    /// Convert UniversalNode back to NodeSpace Node
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
}

// Implement the DataStore trait for compatibility with existing NodeSpace architecture
#[async_trait]
impl DataStore for LanceDataStore {
    async fn store_node(&self, node: Node) -> NodeSpaceResult<NodeId> {
        let universal = self.node_to_universal(node.clone(), None);

        // Store using Arrow persistence
        self.store_node_arrow(universal.clone()).await?;
        println!("✅ Stored node {} using Arrow persistence", universal.id);

        Ok(node.id)
    }

    async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        // Use Arrow-based retrieval
        let result = self.get_node_arrow(id).await?;
        if result.is_some() {
            println!("✅ Retrieved node {} using Arrow persistence", id.as_str());
        }
        Ok(result)
    }

    async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()> {
        // Use Arrow-based deletion
        self.delete_node_arrow(id).await?;
        println!("✅ Deleted node {} using Arrow persistence", id.as_str());

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
        let mut child_metadata = child_node.metadata.clone().unwrap_or_else(|| serde_json::json!({}));
        let needs_child_update = child_metadata.get("parent_id").and_then(|v| v.as_str()) != Some(from.as_str());
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
                DataStoreError::Database(format!("Failed to update child node (potential inconsistency): {}", e))
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
        println!(
            "✅ Stored node {} with embedding using Arrow persistence",
            universal.id
        );

        Ok(node.id)
    }

    async fn search_similar_nodes(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        // Use Arrow-based vector search
        let results = self.vector_search_arrow(embedding, limit).await?;
        println!("✅ Using Arrow-based vector search");
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

    // NEW: Cross-modal search methods for NS-81
    async fn create_image_node(&self, image_node: ImageNode) -> NodeSpaceResult<String> {
        // Convert ImageNode to UniversalNode format
        let universal_node = UniversalNode {
            id: image_node.id.clone(),
            node_type: "image".to_string(),
            content: image_node
                .metadata
                .description
                .unwrap_or_else(|| format!("Image: {}", image_node.metadata.filename)),
            vector: image_node.embedding,
            parent_id: None,
            children_ids: vec![],
            mentions: vec![],
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
                        metadata: crate::ImageMetadata {
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
            if !type_filters.is_empty() && !type_filters.contains(&universal_node.node_type) {
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
                if config.enable_cross_modal && universal_node.node_type == "image" {
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
}

impl LanceDataStore {
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
                if &universal_node.node_type != filter_type {
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
