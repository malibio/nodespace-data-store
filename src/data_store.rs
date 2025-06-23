use crate::conversions::node_id_to_thing;
use crate::error::DataStoreError;
use async_trait::async_trait;
use nodespace_core_types::{Node, NodeId, NodeSpaceResult};
use surrealdb::engine::local::{Db, Mem, RocksDb};
use surrealdb::Surreal;

// DataStore trait - authoritative interface owned by this repository
#[async_trait]
pub trait DataStore {
    async fn store_node(&self, node: Node) -> NodeSpaceResult<NodeId>;
    async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>>;
    async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()>;
    async fn query_nodes(&self, query: &str) -> NodeSpaceResult<Vec<Node>>;
    async fn create_relationship(
        &self,
        from: &NodeId,
        to: &NodeId,
        rel_type: &str,
    ) -> NodeSpaceResult<()>;

    // Vector search capabilities
    async fn store_node_with_embedding(
        &self,
        node: Node,
        embedding: Vec<f32>,
    ) -> NodeSpaceResult<NodeId>;
    async fn search_similar_nodes(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>>;
    async fn update_node_embedding(&self, id: &NodeId, embedding: Vec<f32>) -> NodeSpaceResult<()>;
}

pub struct SurrealDataStore {
    db: Surreal<Db>,
}

impl SurrealDataStore {
    pub async fn new(path: &str) -> Result<Self, DataStoreError> {
        let db = if path == "memory" {
            Surreal::new::<Mem>(()).await?
        } else {
            // Use RocksDB for file-based storage
            Surreal::new::<RocksDb>(path).await?
        };

        // Use the database and namespace
        db.use_ns("nodespace").use_db("nodes").await?;

        Ok(Self { db })
    }
}

#[async_trait]
impl DataStore for SurrealDataStore {
    async fn store_node(&self, node: Node) -> NodeSpaceResult<NodeId> {
        let thing = node_id_to_thing(&node.id);

        // Create a node without the id field to avoid conflict
        let node_data = serde_json::json!({
            "content": node.content,
            "metadata": node.metadata,
            "created_at": node.created_at,
            "updated_at": node.updated_at
        });

        let _: Option<serde_json::Value> = self
            .db
            .create(thing)
            .content(node_data)
            .await
            .map_err(DataStoreError::from)?;

        Ok(node.id)
    }

    async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        let thing = node_id_to_thing(id);

        let result: Option<serde_json::Value> =
            self.db.select(thing).await.map_err(DataStoreError::from)?;

        match result {
            Some(data) => {
                let node = Node {
                    id: id.clone(),
                    content: data["content"].clone(),
                    metadata: if data["metadata"].is_null() {
                        None
                    } else {
                        Some(data["metadata"].clone())
                    },
                    created_at: data["created_at"].as_str().unwrap_or("").to_string(),
                    updated_at: data["updated_at"].as_str().unwrap_or("").to_string(),
                };
                Ok(Some(node))
            }
            None => Ok(None),
        }
    }

    async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()> {
        let thing = node_id_to_thing(id);

        let _deleted: Option<serde_json::Value> =
            self.db.delete(thing).await.map_err(DataStoreError::from)?;

        Ok(())
    }

    async fn query_nodes(&self, query: &str) -> NodeSpaceResult<Vec<Node>> {
        let mut result = self
            .db
            .query(query.to_string())
            .await
            .map_err(DataStoreError::from)?;

        // Take the first result and convert to Vec<Node>
        let values: Vec<serde_json::Value> = result.take(0).map_err(DataStoreError::Database)?;

        let mut nodes = Vec::new();
        for value in values {
            if let Some(node_data) = value.as_object() {
                // Extract node ID from SurrealDB record - handle Thing format
                let id_str = if let Some(id_val) = node_data.get("id") {
                    if let Some(id_obj) = id_val.as_object() {
                        // Handle nested SurrealDB Thing structure: {"tb": "nodes", "id": {"String": "uuid"}}
                        if let Some(id_inner) = id_obj.get("id") {
                            if let Some(string_obj) = id_inner.as_object() {
                                if let Some(uuid_str) = string_obj.get("String") {
                                    if let Some(uuid) = uuid_str.as_str() {
                                        // Convert underscores back to hyphens for UUID format
                                        uuid.replace("_", "-")
                                    } else {
                                        String::new()
                                    }
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        }
                    } else if let Some(id_str) = id_val.as_str() {
                        // Handle simple string format
                        if let Some(stripped) = id_str.strip_prefix("nodes:") {
                            stripped.replace("_", "-")
                        } else {
                            id_str.to_string()
                        }
                    } else {
                        String::new()
                    }
                } else {
                    continue;
                };

                let node = Node {
                    id: NodeId::from_string(id_str.to_string()),
                    content: node_data
                        .get("content")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                    metadata: node_data.get("metadata").cloned(),
                    created_at: node_data
                        .get("created_at")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    updated_at: node_data
                        .get("updated_at")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                };
                nodes.push(node);
            }
        }

        Ok(nodes)
    }

    async fn create_relationship(
        &self,
        from: &NodeId,
        to: &NodeId,
        rel_type: &str,
    ) -> NodeSpaceResult<()> {
        let from_thing = node_id_to_thing(from);
        let to_thing = node_id_to_thing(to);

        let relate_query = format!("RELATE {}->{}->{}", from_thing, rel_type, to_thing);

        let mut result = self
            .db
            .query(relate_query)
            .await
            .map_err(DataStoreError::from)?;

        // Extract the relationship ID from the result
        let _value: surrealdb::sql::Value = result.take(0).map_err(DataStoreError::Database)?;

        Ok(())
    }

    async fn store_node_with_embedding(
        &self,
        node: Node,
        embedding: Vec<f32>,
    ) -> NodeSpaceResult<NodeId> {
        let thing = node_id_to_thing(&node.id);

        // Create a node with embedding vector
        let node_data = serde_json::json!({
            "content": node.content,
            "metadata": node.metadata,
            "created_at": node.created_at,
            "updated_at": node.updated_at,
            "embedding": embedding
        });

        let _: Option<serde_json::Value> = self
            .db
            .create(thing)
            .content(node_data)
            .await
            .map_err(DataStoreError::from)?;

        Ok(node.id)
    }

    async fn search_similar_nodes(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        // Use SurrealDB's vector search functionality
        let query = format!(
            "SELECT *, vector::similarity::cosine(embedding, {}) AS score FROM nodes WHERE embedding IS NOT NULL ORDER BY score DESC LIMIT {}",
            serde_json::to_string(&embedding).map_err(DataStoreError::from)?,
            limit
        );

        let mut result = self.db.query(query).await.map_err(DataStoreError::from)?;

        let values: Vec<serde_json::Value> = result.take(0).map_err(DataStoreError::Database)?;

        let mut results = Vec::new();
        for value in values {
            if let Some(node_data) = value.as_object() {
                // Extract node ID and score - handle SurrealDB Thing format
                let id_str = if let Some(id_val) = node_data.get("id") {
                    if let Some(id_obj) = id_val.as_object() {
                        // Handle nested SurrealDB Thing structure: {"tb": "nodes", "id": {"String": "uuid"}}
                        if let Some(id_inner) = id_obj.get("id") {
                            if let Some(string_obj) = id_inner.as_object() {
                                if let Some(uuid_str) = string_obj.get("String") {
                                    if let Some(uuid) = uuid_str.as_str() {
                                        // Convert underscores back to hyphens for UUID format
                                        uuid.replace("_", "-")
                                    } else {
                                        String::new()
                                    }
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        }
                    } else if let Some(id_str) = id_val.as_str() {
                        // Handle simple string format
                        if let Some(stripped) = id_str.strip_prefix("nodes:") {
                            stripped.replace("_", "-")
                        } else {
                            id_str.to_string()
                        }
                    } else {
                        String::new()
                    }
                } else {
                    continue;
                };

                let score = node_data
                    .get("score")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32;

                let node = Node {
                    id: NodeId::from_string(id_str.to_string()),
                    content: node_data
                        .get("content")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                    metadata: node_data.get("metadata").cloned(),
                    created_at: node_data
                        .get("created_at")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    updated_at: node_data
                        .get("updated_at")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                };

                results.push((node, score));
            }
        }

        Ok(results)
    }

    async fn update_node_embedding(&self, id: &NodeId, embedding: Vec<f32>) -> NodeSpaceResult<()> {
        let thing = node_id_to_thing(id);

        // Update the embedding field
        let update_data = serde_json::json!({
            "embedding": embedding
        });

        let _: Option<serde_json::Value> = self
            .db
            .update(thing)
            .merge(update_data)
            .await
            .map_err(DataStoreError::from)?;

        Ok(())
    }
}

impl SurrealDataStore {
    /// Get all relationships for a given node
    pub async fn get_node_relationships(
        &self,
        node_id: &NodeId,
    ) -> NodeSpaceResult<serde_json::Value> {
        let thing = node_id_to_thing(node_id);

        // For now, return a simple query result - relationship query syntax can be improved later
        let query = format!("SELECT * FROM {}", thing);

        let mut result = self.db.query(query).await.map_err(DataStoreError::from)?;

        let value: surrealdb::sql::Value = result.take(0).map_err(DataStoreError::Database)?;

        let json = serde_json::to_value(&value).map_err(DataStoreError::from)?;

        Ok(json)
    }

    /// Create or get a date node for hierarchical organization
    pub async fn create_or_get_date_node(
        &self,
        date_value: &str,
        description: Option<&str>,
    ) -> NodeSpaceResult<NodeId> {
        // Check if date node already exists
        let query = format!("SELECT * FROM date WHERE date_value = '{}'", date_value);
        let mut result = self.db.query(query).await.map_err(DataStoreError::from)?;
        let existing: Vec<serde_json::Value> = result.take(0).map_err(DataStoreError::Database)?;

        if !existing.is_empty() {
            // Extract existing node ID
            if let Some(node_data) = existing.first().and_then(|v| v.as_object()) {
                if let Some(id_val) = node_data.get("id") {
                    let id_str = self.extract_node_id_from_value(id_val);
                    return Ok(NodeId::from_string(id_str));
                }
            }
        }

        // Create new date node
        let node_id = NodeId::new();
        let date_data = serde_json::json!({
            "date_value": date_value,
            "description": description.unwrap_or(""),
            "created_at": chrono::Utc::now().to_rfc3339(),
            "updated_at": chrono::Utc::now().to_rfc3339()
        });

        let thing_id = ("date", node_id.as_str().replace("-", "_"));
        let _: Option<serde_json::Value> = self
            .db
            .create(thing_id)
            .content(date_data)
            .await
            .map_err(DataStoreError::from)?;

        Ok(node_id)
    }

    /// Create a text node with hierarchical parent relationship
    pub async fn create_text_node(
        &self,
        content: &str,
        parent_node_id: Option<&NodeId>,
    ) -> NodeSpaceResult<NodeId> {
        let node_id = NodeId::new();
        let text_data = serde_json::json!({
            "content": content,
            "parent_node": parent_node_id.map(|id| format!("nodes:{}", id.as_str().replace("-", "_"))),
            "created_at": chrono::Utc::now().to_rfc3339(),
            "updated_at": chrono::Utc::now().to_rfc3339()
        });

        let thing_id = ("text", node_id.as_str().replace("-", "_"));
        let _: Option<serde_json::Value> = self
            .db
            .create(thing_id)
            .content(text_data)
            .await
            .map_err(DataStoreError::from)?;

        // Create relationship if parent exists
        if let Some(parent_id) = parent_node_id {
            let from_thing = format!("date:{}", parent_id.as_str().replace("-", "_"));
            let to_thing = format!("text:{}", node_id.as_str().replace("-", "_"));
            let relate_query = format!("RELATE {}->contains->{}", from_thing, to_thing);

            let mut _result = self
                .db
                .query(relate_query)
                .await
                .map_err(DataStoreError::from)?;
        }

        Ok(node_id)
    }

    /// Get all text nodes for a specific date
    pub async fn get_nodes_for_date(&self, date_value: &str) -> NodeSpaceResult<Vec<Node>> {
        // First get the date node ID, then get related text nodes
        let date_query = format!("SELECT * FROM date WHERE date_value = '{}'", date_value);
        let mut result = self.db.query(date_query).await.map_err(DataStoreError::from)?;
        let date_nodes: Vec<serde_json::Value> = result.take(0).map_err(DataStoreError::Database)?;
        
        if date_nodes.is_empty() {
            return Ok(Vec::new());
        }
        
        // Extract the date node ID
        if let Some(date_node) = date_nodes.first().and_then(|v| v.as_object()) {
            if let Some(id_val) = date_node.get("id") {
                let date_id = self.extract_node_id_from_value(id_val);
                let date_thing = format!("date:{}", date_id.replace("-", "_"));
                
                // Query text nodes that have a relationship from this date node
                let text_query = format!(
                    "SELECT * FROM text WHERE id IN (SELECT ->contains->id FROM {})",
                    date_thing
                );
                return self.query_nodes(&text_query).await;
            }
        }
        
        Ok(Vec::new())
    }

    /// Get all children of a date node (hierarchical query)
    pub async fn get_date_children(
        &self,
        date_node_id: &NodeId,
    ) -> NodeSpaceResult<Vec<serde_json::Value>> {
        let from_thing = format!("date:{}", date_node_id.as_str().replace("-", "_"));
        let query = format!(
            "SELECT * FROM text WHERE id IN (SELECT ->contains->id FROM {})",
            from_thing
        );

        let mut result = self.db.query(query).await.map_err(DataStoreError::from)?;
        let values: Vec<serde_json::Value> = result.take(0).map_err(DataStoreError::Database)?;
        Ok(values)
    }

    /// Helper method to extract node ID from SurrealDB value
    fn extract_node_id_from_value(&self, id_val: &serde_json::Value) -> String {
        if let Some(id_obj) = id_val.as_object() {
            // Handle nested SurrealDB Thing structure: {"tb": "date", "id": {"String": "uuid"}}
            if let Some(id_inner) = id_obj.get("id") {
                if let Some(string_obj) = id_inner.as_object() {
                    if let Some(uuid_str) = string_obj.get("String") {
                        if let Some(uuid) = uuid_str.as_str() {
                            return uuid.replace("_", "-");
                        }
                    }
                }
            }
        } else if let Some(id_str) = id_val.as_str() {
            // Handle simple string format
            if let Some(stripped) = id_str.strip_prefix("date:") {
                return stripped.replace("_", "-");
            }
            return id_str.to_string();
        }
        String::new()
    }
}
