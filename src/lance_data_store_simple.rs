use crate::data_store::DataStore;
use crate::error::DataStoreError;
use async_trait::async_trait;
use lancedb::{connect, Connection};
use nodespace_core_types::{Node, NodeId, NodeSpaceResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Simplified LanceDB DataStore implementation
/// This provides a working foundation that can be enhanced later
pub struct LanceDataStore {
    connection: Connection,
    // For now, use in-memory storage as a bridge until LanceDB integration is complete
    nodes: Arc<Mutex<HashMap<String, UniversalNode>>>,
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

impl LanceDataStore {
    /// Initialize new LanceDB connection with Universal Document Schema
    pub async fn new(db_path: &str) -> Result<Self, DataStoreError> {
        let connection = connect(db_path)
            .execute()
            .await
            .map_err(|e| DataStoreError::LanceDBConnection(format!("LanceDB connection failed: {}", e)))?;

        Ok(Self {
            connection,
            nodes: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Initialize the universal nodes table with schema
    pub async fn initialize_table(&mut self) -> Result<(), DataStoreError> {
        // For now, just ensure the connection is working
        // Later: Create actual LanceDB table with schema
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
        let parent_id = node.metadata.as_ref()
            .and_then(|m| m.get("parent_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let children_ids = node.metadata.as_ref()
            .and_then(|m| m.get("children_ids"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        let mentions = node.metadata.as_ref()
            .and_then(|m| m.get("mentions"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        UniversalNode {
            id: node.id.to_string(),
            node_type,
            content: node.content.to_string(),
            vector: embedding.unwrap_or_else(|| vec![0.0; 384]), // Default embedding
            parent_id,
            children_ids,
            mentions,
            created_at: if node.created_at.is_empty() { now.clone() } else { node.created_at },
            updated_at: if node.updated_at.is_empty() { now } else { node.updated_at },
            metadata: node.metadata,
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
                universal.children_ids.into_iter().map(serde_json::Value::String).collect()
            );
        }
        if !universal.mentions.is_empty() {
            metadata["mentions"] = serde_json::Value::Array(
                universal.mentions.into_iter().map(serde_json::Value::String).collect()
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
        
        // Store in memory for now (bridge implementation)
        {
            let mut nodes = self.nodes.lock().unwrap();
            nodes.insert(universal.id.clone(), universal);
        }
        
        // TODO: Actually store in LanceDB table
        
        Ok(node.id)
    }

    async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        let nodes = self.nodes.lock().unwrap();
        
        if let Some(universal_node) = nodes.get(id.as_str()) {
            let node = self.universal_to_node(universal_node.clone());
            Ok(Some(node))
        } else {
            Ok(None)
        }
    }

    async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()> {
        let mut nodes = self.nodes.lock().unwrap();
        nodes.remove(id.as_str());
        
        // TODO: Actually delete from LanceDB table
        
        Ok(())
    }

    async fn query_nodes(&self, query: &str) -> NodeSpaceResult<Vec<Node>> {
        let nodes = self.nodes.lock().unwrap();
        
        // Simple filter implementation for now
        let mut results = Vec::new();
        for universal_node in nodes.values() {
            // Simple content matching
            if query.is_empty() || universal_node.content.contains(query) {
                let node = self.universal_to_node(universal_node.clone());
                results.push(node);
            }
        }
        
        Ok(results)
    }

    async fn create_relationship(
        &self,
        from: &NodeId,
        to: &NodeId,
        _rel_type: &str,
    ) -> NodeSpaceResult<()> {
        // Update parent and child relationships
        let mut nodes = self.nodes.lock().unwrap();
        
        // Update parent node to include child
        if let Some(parent_node) = nodes.get_mut(from.as_str()) {
            if !parent_node.children_ids.contains(&to.to_string()) {
                parent_node.children_ids.push(to.to_string());
            }
        }
        
        // Update child node to set parent
        if let Some(child_node) = nodes.get_mut(to.as_str()) {
            child_node.parent_id = Some(from.to_string());
        }
        
        Ok(())
    }

    async fn store_node_with_embedding(
        &self,
        node: Node,
        embedding: Vec<f32>,
    ) -> NodeSpaceResult<NodeId> {
        let universal = self.node_to_universal(node.clone(), Some(embedding));
        
        // Store in memory for now
        {
            let mut nodes = self.nodes.lock().unwrap();
            nodes.insert(universal.id.clone(), universal);
        }
        
        Ok(node.id)
    }

    async fn search_similar_nodes(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        let nodes = self.nodes.lock().unwrap();
        
        // Simple cosine similarity implementation
        let mut similarities = Vec::new();
        
        for universal_node in nodes.values() {
            let similarity = cosine_similarity(&embedding, &universal_node.vector);
            let node = self.universal_to_node(universal_node.clone());
            similarities.push((node, similarity));
        }
        
        // Sort by similarity descending and take limit
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        similarities.truncate(limit);
        
        Ok(similarities)
    }

    async fn update_node_embedding(&self, id: &NodeId, embedding: Vec<f32>) -> NodeSpaceResult<()> {
        let mut nodes = self.nodes.lock().unwrap();
        
        if let Some(node) = nodes.get_mut(id.as_str()) {
            node.vector = embedding;
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
}

impl LanceDataStore {
    /// Get child nodes using simple JSON filtering (replaces complex SurrealDB traversal)
    pub async fn get_child_nodes(&self, parent_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        let nodes = self.nodes.lock().unwrap();
        
        let mut children = Vec::new();
        for universal_node in nodes.values() {
            if let Some(ref pid) = universal_node.parent_id {
                if pid == parent_id.as_str() {
                    let node = self.universal_to_node(universal_node.clone());
                    children.push(node);
                }
            }
        }
        
        Ok(children)
    }

    /// Create or update relationship using JSON metadata (simplified from SurrealDB RELATE)
    pub async fn update_relationship(
        &self,
        node_id: &NodeId,
        parent_id: Option<NodeId>,
        children_ids: Vec<NodeId>,
    ) -> NodeSpaceResult<()> {
        let mut nodes = self.nodes.lock().unwrap();
        
        if let Some(node) = nodes.get_mut(node_id.as_str()) {
            node.parent_id = parent_id.map(|id| id.to_string());
            node.children_ids = children_ids.into_iter().map(|id| id.to_string()).collect();
        }
        
        Ok(())
    }

    /// Hybrid search combining semantic search with metadata filtering
    pub async fn hybrid_search(
        &self,
        embedding: Vec<f32>,
        node_type_filter: Option<String>,
        _metadata_filter: Option<serde_json::Value>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        let nodes = self.nodes.lock().unwrap();
        
        let mut similarities = Vec::new();
        
        for universal_node in nodes.values() {
            // Apply node type filter if specified
            if let Some(ref filter_type) = node_type_filter {
                if &universal_node.node_type != filter_type {
                    continue;
                }
            }
            
            let similarity = cosine_similarity(&embedding, &universal_node.vector);
            let node = self.universal_to_node(universal_node.clone());
            similarities.push((node, similarity));
        }
        
        // Sort by similarity descending and take limit
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        similarities.truncate(limit);
        
        Ok(similarities)
    }
}

/// Simple cosine similarity implementation
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