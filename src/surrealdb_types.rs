// SurrealDB 2.x typed structs for proper CRUD operations
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

/// Represents a text node record in SurrealDB
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Represents a date node record in SurrealDB
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub date_value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Represents a generic node record in SurrealDB (for the nodes table)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub content: serde_json::Value,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
    // Sibling pointer fields for linked list structure
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_sibling: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_sibling: Option<String>,
}

/// Represents a relationship record in SurrealDB
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelationshipRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    #[serde(rename = "in")]
    pub in_node: Thing,
    #[serde(rename = "out")]
    pub out_node: Thing,
    #[serde(default)]
    pub created_at: String,
}

/// Represents a FETCH result that includes the fetched 'out' node
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FetchOutResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    #[serde(rename = "in")]
    pub in_node: Thing,
    #[serde(rename = "out")]
    pub out_node: NodeRecord, // This will contain the actual fetched node data
    #[serde(default)]
    pub created_at: String,
}

impl From<NodeRecord> for nodespace_core_types::Node {
    fn from(record: NodeRecord) -> Self {
        let mut node = nodespace_core_types::Node::with_id(
            // Extract ID from Thing or generate new one
            if let Some(thing) = record.id {
                // Convert underscores back to hyphens for proper UUID format
                let id_str = thing.id.to_string().replace("_", "-");
                nodespace_core_types::NodeId::from_string(id_str)
            } else {
                nodespace_core_types::NodeId::new()
            },
            record.content,
        );

        if let Some(metadata) = record.metadata {
            node = node.with_metadata(metadata);
        }

        // Set sibling pointers if they exist
        if let Some(next_id) = record.next_sibling {
            let next_node_id = nodespace_core_types::NodeId::from_string(next_id.replace("_", "-"));
            node = node.with_next_sibling(Some(next_node_id));
        }

        if let Some(prev_id) = record.previous_sibling {
            let prev_node_id = nodespace_core_types::NodeId::from_string(prev_id.replace("_", "-"));
            node = node.with_previous_sibling(Some(prev_node_id));
        }

        node.created_at = record.created_at;
        node.updated_at = record.updated_at;

        node
    }
}

impl From<nodespace_core_types::Node> for NodeRecord {
    fn from(node: nodespace_core_types::Node) -> Self {
        NodeRecord {
            id: None, // Will be set by SurrealDB
            content: node.content,
            metadata: node.metadata,
            created_at: node.created_at,
            updated_at: node.updated_at,
            embedding: None, // Will be set separately if needed
            // Convert sibling NodeIds to strings (with underscores for SurrealDB compatibility)
            next_sibling: node.next_sibling.map(|id| id.to_string().replace("-", "_")),
            previous_sibling: node
                .previous_sibling
                .map(|id| id.to_string().replace("-", "_")),
        }
    }
}

impl From<TextRecord> for nodespace_core_types::Node {
    fn from(record: TextRecord) -> Self {
        // Use the text content directly as the Node content
        let content = serde_json::Value::String(record.content);

        let mut node = nodespace_core_types::Node::with_id(
            if let Some(thing) = record.id {
                // Convert underscores back to hyphens for proper UUID format
                let id_str = thing.id.to_string().replace("_", "-");
                nodespace_core_types::NodeId::from_string(id_str)
            } else {
                nodespace_core_types::NodeId::new()
            },
            content,
        );

        // Add parent_date as metadata if it exists
        if let Some(parent_date) = record.parent_date {
            let metadata = serde_json::json!({
                "parent_date": parent_date
            });
            node = node.with_metadata(metadata);
        }

        node.created_at = record.created_at;
        node.updated_at = record.updated_at;

        node
    }
}
