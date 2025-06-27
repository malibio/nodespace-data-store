use async_trait::async_trait;
use nodespace_core_types::{Node, NodeId, NodeSpaceResult};

// SurrealDB-related imports (only when migration feature is enabled)
#[cfg(feature = "migration")]
use crate::conversions::{node_id_to_record_id, node_id_to_thing};
#[cfg(feature = "migration")]
use crate::error::DataStoreError;
#[cfg(feature = "migration")]
use crate::surrealdb_types::*;
#[cfg(feature = "migration")]
use surrealdb::engine::local::{Db, Mem, RocksDb};
#[cfg(feature = "migration")]
use surrealdb::sql::Value as SurrealValue;
#[cfg(feature = "migration")]
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

#[cfg(feature = "migration")]
pub struct SurrealDataStore {
    db: Surreal<Db>,
}

#[cfg(feature = "migration")]
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

#[cfg(feature = "migration")]
#[async_trait]
impl DataStore for SurrealDataStore {
    async fn store_node(&self, node: Node) -> NodeSpaceResult<NodeId> {
        let record_id = node_id_to_record_id(&node.id);

        // Convert Node to NodeRecord for proper SurrealDB 2.x typing
        let node_record = NodeRecord::from(node.clone());

        let _created_record: Option<NodeRecord> = self
            .db
            .create(record_id)
            .content(node_record)
            .await
            .map_err(DataStoreError::from)?;

        Ok(node.id)
    }

    async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        let record_id = node_id_to_record_id(id);

        let result: Option<NodeRecord> = self
            .db
            .select(record_id)
            .await
            .map_err(DataStoreError::from)?;

        match result {
            Some(record) => Ok(Some(Node::from(record))),
            None => Ok(None),
        }
    }

    async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()> {
        let record_id = node_id_to_record_id(id);

        let _deleted: Option<NodeRecord> = self
            .db
            .delete(record_id)
            .await
            .map_err(DataStoreError::from)?;

        Ok(())
    }

    async fn query_nodes(&self, query: &str) -> NodeSpaceResult<Vec<Node>> {
        let mut result = self
            .db
            .query(query.to_string())
            .await
            .map_err(DataStoreError::from)?;

        // Take the first result and convert directly to Vec<Node>
        // SurrealDB 2.x: Use direct Node deserialization instead of serde_json::Value
        let nodes: Vec<Node> = result.take(0).map_err(DataStoreError::from)?;
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

        // Create the relationship using correct SurrealDB RELATE syntax
        let relate_query = format!("RELATE {}->{}->{}", from_thing, rel_type, to_thing);

        // Execute the query - don't try to deserialize the complex relationship result
        self.db
            .query(relate_query)
            .await
            .map_err(DataStoreError::from)?;

        Ok(())
    }

    async fn store_node_with_embedding(
        &self,
        node: Node,
        embedding: Vec<f32>,
    ) -> NodeSpaceResult<NodeId> {
        let record_id = node_id_to_record_id(&node.id);

        // Convert Node to NodeRecord and add embedding
        let mut node_record = NodeRecord::from(node.clone());
        node_record.embedding = Some(embedding);

        let _created_record: Option<NodeRecord> = self
            .db
            .create(record_id)
            .content(node_record)
            .await
            .map_err(DataStoreError::from)?;

        Ok(node.id)
    }

    async fn search_similar_nodes(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        use surrealdb::sql::Thing;

        // Create a simplified struct for search results
        #[derive(serde::Deserialize)]
        struct SearchResult {
            id: Option<Thing>,
            content: serde_json::Value,
            #[serde(default)]
            metadata: Option<serde_json::Value>,
            #[serde(default)]
            created_at: String,
            #[serde(default)]
            updated_at: String,
            score: f32,
        }

        // Use SurrealDB's vector search functionality
        let query = format!(
            "SELECT *, vector::similarity::cosine(embedding, {}) AS score FROM nodes WHERE embedding IS NOT NULL ORDER BY score DESC LIMIT {}",
            serde_json::to_string(&embedding).map_err(DataStoreError::from)?,
            limit
        );

        let mut result = self.db.query(query).await.map_err(DataStoreError::from)?;

        // Use proper type deserialization with dedicated struct
        let search_results: Vec<SearchResult> = result.take(0).map_err(DataStoreError::from)?;

        let results = search_results
            .into_iter()
            .map(|sr| {
                // Convert SearchResult to Node
                let node_id = if let Some(thing) = sr.id {
                    // Convert underscores back to hyphens for proper UUID format
                    let id_str = thing.id.to_string().replace("_", "-");
                    nodespace_core_types::NodeId::from_string(id_str)
                } else {
                    nodespace_core_types::NodeId::new()
                };

                let mut node = Node::with_id(node_id, sr.content);

                if let Some(metadata) = sr.metadata {
                    node = node.with_metadata(metadata);
                }

                node.created_at = sr.created_at;
                node.updated_at = sr.updated_at;

                (node, sr.score)
            })
            .collect();

        Ok(results)
    }

    async fn update_node_embedding(&self, id: &NodeId, embedding: Vec<f32>) -> NodeSpaceResult<()> {
        let record_id = node_id_to_record_id(id);

        // Update the embedding field using typed approach
        let update_data = serde_json::json!({
            "embedding": embedding
        });

        let _updated: Option<NodeRecord> = self
            .db
            .update(record_id)
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
        use surrealdb::sql::Thing;

        // Create a simplified struct for search results
        #[derive(serde::Deserialize)]
        struct SearchResult {
            id: Option<Thing>,
            content: serde_json::Value,
            #[serde(default)]
            metadata: Option<serde_json::Value>,
            #[serde(default)]
            created_at: String,
            #[serde(default)]
            updated_at: String,
            score: f32,
        }

        // Use SurrealDB's vector search functionality for semantic search
        let query = format!(
            "SELECT *, vector::similarity::cosine(embedding, {}) AS score FROM nodes WHERE embedding IS NOT NULL ORDER BY score DESC LIMIT {}",
            serde_json::to_string(&embedding).map_err(DataStoreError::from)?,
            limit
        );

        let mut result = self.db.query(query).await.map_err(DataStoreError::from)?;

        // Use proper type deserialization with dedicated struct
        let search_results: Vec<SearchResult> = result.take(0).map_err(DataStoreError::from)?;

        let results = search_results
            .into_iter()
            .map(|sr| {
                // Convert SearchResult to Node
                let node_id = if let Some(thing) = sr.id {
                    // Convert underscores back to hyphens for proper UUID format
                    let id_str = thing.id.to_string().replace("_", "-");
                    nodespace_core_types::NodeId::from_string(id_str)
                } else {
                    nodespace_core_types::NodeId::new()
                };

                let mut node = Node::with_id(node_id, sr.content);

                if let Some(metadata) = sr.metadata {
                    node = node.with_metadata(metadata);
                }

                node.created_at = sr.created_at;
                node.updated_at = sr.updated_at;

                (node, sr.score)
            })
            .collect();

        Ok(results)
    }
}

#[cfg(feature = "migration")]
impl SurrealDataStore {
    /// Get a text node specifically from the text table
    pub async fn get_text_node(&self, node_id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        let thing_id = ("text", node_id.as_str().replace("-", "_"));

        let result: Option<TextRecord> = self
            .db
            .select(thing_id)
            .await
            .map_err(DataStoreError::from)?;

        match result {
            Some(record) => Ok(Some(Node::from(record))),
            None => Ok(None),
        }
    }

    /// Create a relationship between text nodes
    pub async fn create_text_relationship(
        &self,
        from: &NodeId,
        to: &NodeId,
        rel_type: &str,
    ) -> NodeSpaceResult<()> {
        let from_thing = format!("text:{}", from.as_str().replace("-", "_"));
        let to_thing = format!("text:{}", to.as_str().replace("-", "_"));

        // Create the relationship using correct SurrealDB RELATE syntax for text nodes
        let relate_query = format!("RELATE {}->{}->{}", from_thing, rel_type, to_thing);

        self.db
            .query(relate_query)
            .await
            .map_err(DataStoreError::from)?;

        Ok(())
    }

    /// Get all relationships for a given node
    pub async fn get_node_relationships(
        &self,
        node_id: &NodeId,
    ) -> NodeSpaceResult<serde_json::Value> {
        let thing = node_id_to_thing(node_id);

        // For now, return a simple query result - relationship query syntax can be improved later
        let query = format!("SELECT * FROM {}", thing);

        let mut result = self.db.query(query).await.map_err(DataStoreError::from)?;

        let value: surrealdb::Value = result.take(0).map_err(DataStoreError::from)?;

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
        let existing: Option<DateRecord> = self
            .db
            .select(thing_id)
            .await
            .map_err(DataStoreError::from)?;

        if existing.is_some() {
            // Date node already exists, return the NodeId based on date_value
            return Ok(NodeId::from_string(date_value.to_string()));
        }

        // Create new date node with date_value as the ID
        let date_record = DateRecord {
            id: None, // Will be set by SurrealDB
            date_value: date_value.to_string(),
            description: description.map(|s| s.to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        let _created: Option<DateRecord> = self
            .db
            .create(thing_id)
            .content(date_record)
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
        let text_record = TextRecord {
            id: None, // Will be set by SurrealDB
            content: content.to_string(),
            parent_date: parent_date_value.map(|s| s.to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        let thing_id = ("text", node_id.as_str().replace("-", "_"));
        let _created: Option<TextRecord> = self
            .db
            .create(thing_id)
            .content(text_record)
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
        let text_records: Vec<TextRecord> =
            text_result.take(0).map_err(DataStoreError::from)?;

        let nodes: Vec<Node> = text_records.into_iter().map(Node::from).collect();

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
        let values: Vec<SurrealValue> = result.take(0).map_err(DataStoreError::from)?;

        // Convert SurrealValues to JSON
        let json_values: Vec<serde_json::Value> =
            values.into_iter().map(|v| v.into_json()).collect();

        Ok(json_values)
    }
}
