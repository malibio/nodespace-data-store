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

    // Semantic search with provided embedding vector
    async fn semantic_search_with_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>>;
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
                let mut node = Node::with_id(id.clone(), data["content"].clone());
                if let Some(metadata) = data.get("metadata").filter(|v| !v.is_null()) {
                    node = node.with_metadata(metadata.clone());
                }
                // Set timestamps from database
                node.created_at = data["created_at"].as_str().unwrap_or("").to_string();
                node.updated_at = data["updated_at"].as_str().unwrap_or("").to_string();
                // Sibling pointers default to None (not stored in current schema)
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

                let content = node_data
                    .get("content")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);

                let mut node = Node::with_id(NodeId::from_string(id_str.to_string()), content);

                if let Some(metadata) = node_data.get("metadata") {
                    node = node.with_metadata(metadata.clone());
                }

                // Set timestamps from database
                node.created_at = node_data
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                node.updated_at = node_data
                    .get("updated_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Sibling pointers default to None (not stored in current schema)
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

                let content = node_data
                    .get("content")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);

                let mut node = Node::with_id(NodeId::from_string(id_str.to_string()), content);

                if let Some(metadata) = node_data.get("metadata") {
                    node = node.with_metadata(metadata.clone());
                }

                // Set timestamps from database
                node.created_at = node_data
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                node.updated_at = node_data
                    .get("updated_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Sibling pointers default to None (not stored in current schema)

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

    async fn semantic_search_with_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        // Use SurrealDB's vector search functionality for semantic search
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

                let content = node_data
                    .get("content")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);

                let mut node = Node::with_id(NodeId::from_string(id_str.to_string()), content);

                if let Some(metadata) = node_data.get("metadata") {
                    node = node.with_metadata(metadata.clone());
                }

                // Set timestamps from database
                node.created_at = node_data
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                node.updated_at = node_data
                    .get("updated_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Sibling pointers default to None (not stored in current schema)

                results.push((node, score));
            }
        }

        Ok(results)
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
        // Use date value directly as the record ID
        let thing_id = ("date", date_value);

        // Check if date node already exists by trying to select it
        let existing: Option<serde_json::Value> = self
            .db
            .select(thing_id)
            .await
            .map_err(DataStoreError::from)?;

        if existing.is_some() {
            // Date node already exists, return the NodeId based on date_value
            return Ok(NodeId::from_string(date_value.to_string()));
        }

        // Create new date node with date_value as the ID
        let date_data = serde_json::json!({
            "date_value": date_value,
            "description": description.unwrap_or(""),
            "created_at": chrono::Utc::now().to_rfc3339(),
            "updated_at": chrono::Utc::now().to_rfc3339()
        });

        let _: Option<serde_json::Value> = self
            .db
            .create(thing_id)
            .content(date_data)
            .await
            .map_err(DataStoreError::from)?;

        Ok(NodeId::from_string(date_value.to_string()))
    }

    /// Create a text node with hierarchical parent relationship
    pub async fn create_text_node(
        &self,
        content: &str,
        parent_date_value: Option<&str>,
    ) -> NodeSpaceResult<NodeId> {
        let node_id = NodeId::new();
        let text_data = serde_json::json!({
            "content": content,
            "parent_date": parent_date_value,
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

        // Create relationship if parent date exists
        if let Some(date_value) = parent_date_value {
            let from_thing = format!("date:`{}`", date_value);
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
        // Use direct date ID lookup - no need to search by date_value field
        let date_thing = format!("date:`{}`", date_value);

        // Query text nodes that have a relationship from this date node
        let text_query = format!("SELECT * FROM {}->contains->text", date_thing);

        let mut text_result = self
            .db
            .query(text_query)
            .await
            .map_err(DataStoreError::from)?;
        let text_values: Vec<serde_json::Value> =
            text_result.take(0).map_err(DataStoreError::Database)?;

        let mut nodes = Vec::new();
        for value in text_values {
            if let Some(text_data) = value.as_object() {
                // Convert text record to Node format
                let id_str = self.extract_node_id_from_text_value(text_data.get("id"));
                let content = text_data
                    .get("content")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);

                let mut node = Node::with_id(NodeId::from_string(id_str), content);

                if let Some(metadata) = text_data.get("metadata") {
                    node = node.with_metadata(metadata.clone());
                }

                // Set timestamps from database
                node.created_at = text_data
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                node.updated_at = text_data
                    .get("updated_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Sibling pointers default to None (not stored in current schema)
                nodes.push(node);
            }
        }

        Ok(nodes)
    }

    /// Get all children of a date node (hierarchical query)
    pub async fn get_date_children(
        &self,
        date_value: &str,
    ) -> NodeSpaceResult<Vec<serde_json::Value>> {
        let from_thing = format!("date:`{}`", date_value);
        let query = format!("SELECT * FROM {}->contains", from_thing);

        let mut result = self.db.query(query).await.map_err(DataStoreError::from)?;
        let values: Vec<serde_json::Value> = result.take(0).map_err(DataStoreError::Database)?;
        Ok(values)
    }

    /// Helper method to extract node ID from text table SurrealDB value
    fn extract_node_id_from_text_value(&self, id_val: Option<&serde_json::Value>) -> String {
        if let Some(id_val) = id_val {
            if let Some(id_obj) = id_val.as_object() {
                // Handle nested SurrealDB Thing structure: {"tb": "text", "id": {"String": "uuid"}}
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
                if let Some(stripped) = id_str.strip_prefix("text:") {
                    return stripped.replace("_", "-");
                }
                return id_str.to_string();
            }
        }
        String::new()
    }
}
