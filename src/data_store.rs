use async_trait::async_trait;
use nodespace_core_types::{Node, NodeId, NodeSpaceResult};
use surrealdb::engine::local::{Db, Mem};
use surrealdb::Surreal;
use crate::error::DataStoreError;
use crate::conversions::node_id_to_thing;

// DataStore trait - authoritative interface owned by this repository
#[async_trait]
pub trait DataStore {
    async fn store_node(&self, node: Node) -> NodeSpaceResult<NodeId>;
    async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>>;
    async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()>;
    async fn query_nodes(&self, query: &str) -> NodeSpaceResult<Vec<Node>>;
    async fn create_relationship(&self, from: &NodeId, to: &NodeId, rel_type: &str) -> NodeSpaceResult<()>;
}

pub struct SurrealDataStore {
    db: Surreal<Db>,
}

impl SurrealDataStore {
    pub async fn new(path: &str) -> Result<Self, DataStoreError> {
        let db = if path == "memory" {
            Surreal::new::<Mem>(()).await?
        } else {
            // For now, use memory storage as file storage requires additional features
            Surreal::new::<Mem>(()).await?
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
        
        let _: Option<serde_json::Value> = self.db
            .create(thing)
            .content(node_data)
            .await
            .map_err(DataStoreError::from)?;
            
        Ok(node.id)
    }
    
    async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        let thing = node_id_to_thing(id);
        
        let result: Option<serde_json::Value> = self.db
            .select(thing)
            .await
            .map_err(DataStoreError::from)?;
            
        match result {
            Some(data) => {
                let node = Node {
                    id: id.clone(),
                    content: data["content"].clone(),
                    metadata: if data["metadata"].is_null() { None } else { Some(data["metadata"].clone()) },
                    created_at: data["created_at"].as_str().unwrap_or("").to_string(),
                    updated_at: data["updated_at"].as_str().unwrap_or("").to_string(),
                };
                Ok(Some(node))
            },
            None => Ok(None)
        }
    }
    
    async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()> {
        let thing = node_id_to_thing(id);
        
        let _deleted: Option<serde_json::Value> = self.db
            .delete(thing)
            .await
            .map_err(DataStoreError::from)?;
            
        Ok(())
    }
    
    async fn query_nodes(&self, query: &str) -> NodeSpaceResult<Vec<Node>> {
        let mut result = self.db
            .query(query.to_string())
            .await
            .map_err(DataStoreError::from)?;
            
        // Take the first result and convert to Vec<Node>
        let values: Vec<serde_json::Value> = result.take(0)
            .map_err(|e| DataStoreError::Database(e))?;
        
        let mut nodes = Vec::new();
        for value in values {
            if let Some(node_data) = value.as_object() {
                // Extract node ID from SurrealDB record
                let id_str = if let Some(id_val) = node_data.get("id") {
                    id_val.as_str().unwrap_or_default()
                } else {
                    continue;
                };
                
                let node = Node {
                    id: NodeId::from_string(id_str.to_string()),
                    content: node_data.get("content").cloned().unwrap_or(serde_json::Value::Null),
                    metadata: node_data.get("metadata").cloned(),
                    created_at: node_data.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    updated_at: node_data.get("updated_at").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                };
                nodes.push(node);
            }
        }
        
        Ok(nodes)
    }
    
    async fn create_relationship(&self, from: &NodeId, to: &NodeId, rel_type: &str) -> NodeSpaceResult<()> {
        let from_thing = node_id_to_thing(from);
        let to_thing = node_id_to_thing(to);
        
        let relate_query = format!(
            "RELATE {}->{}->{}",
            from_thing,
            rel_type,
            to_thing
        );
        
        let mut result = self.db
            .query(relate_query)
            .await
            .map_err(DataStoreError::from)?;
            
        // Extract the relationship ID from the result
        let _value: surrealdb::sql::Value = result.take(0)
            .map_err(|e| DataStoreError::Database(e))?;
            
        Ok(())
    }
}

impl SurrealDataStore {
    /// Get all relationships for a given node
    pub async fn get_node_relationships(&self, node_id: &NodeId) -> NodeSpaceResult<serde_json::Value> {
        let thing = node_id_to_thing(node_id);
        
        // For now, return a simple query result - relationship query syntax can be improved later
        let query = format!(
            "SELECT * FROM {}",
            thing
        );
        
        let mut result = self.db
            .query(query)
            .await
            .map_err(DataStoreError::from)?;
            
        let value: surrealdb::sql::Value = result.take(0)
            .map_err(|e| DataStoreError::Database(e))?;
        
        let json = serde_json::to_value(&value)
            .map_err(DataStoreError::from)?;
            
        Ok(json)
    }
}