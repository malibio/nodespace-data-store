// LanceDB schema definitions perfectly aligned with core-types Node structure
// Fresh schema for NS-125 breaking changes - no migration needed

use arrow_schema::{DataType, Field, Schema};
use nodespace_core_types::{Node, NodeId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Fresh LanceDB schema perfectly aligned with core-types Node (NS-125)
/// 1:1 mapping between Node fields and LanceDB columns - no conversion complexity
#[allow(dead_code)]
pub struct NodeSchema;

impl NodeSchema {
    /// Create Arrow schema that exactly matches core-types Node structure
    #[allow(dead_code)]
    pub fn create_node_schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            // Perfect 1:1 mapping with Node struct
            Field::new("id", DataType::Utf8, false),
            Field::new("type", DataType::Utf8, false),         // maps to Node.r#type
            Field::new("content", DataType::Utf8, false),       // JSON string of Node.content
            Field::new("metadata", DataType::Utf8, true),       // JSON string of Node.metadata
            Field::new("created_at", DataType::Utf8, false),
            Field::new("updated_at", DataType::Utf8, false),
            Field::new("parent_id", DataType::Utf8, true),
            Field::new("next_sibling", DataType::Utf8, true),   // Unidirectional only
            Field::new("root_id", DataType::Utf8, true),        // NS-115 hierarchy optimization
            
            // Vector embedding support (optional)
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, false)),
                    384, // Default FastEmbed dimension
                ),
                true,
            ),
            Field::new("vector_model", DataType::Utf8, true),   // Embedding model used
        ]))
    }

}

/// LanceDB document struct with perfect 1:1 mapping to core-types Node
/// No field name translation needed - direct conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanceDocument {
    pub id: String,
    pub r#type: String,              // Direct mapping to Node.r#type
    pub content: String,             // JSON string of Node.content
    pub metadata: Option<String>,    // JSON string of Node.metadata
    pub created_at: String,
    pub updated_at: String,
    pub parent_id: Option<String>,
    pub next_sibling: Option<String>, // Unidirectional only (no previous_sibling)
    pub root_id: Option<String>,     // NS-115 hierarchy optimization
    
    // Vector embedding support
    pub vector: Option<Vec<f32>>,    // 384-dim FastEmbed vectors
    pub vector_model: Option<String>,
}

/// Clean conversion from Node to LanceDocument
impl From<Node> for LanceDocument {
    fn from(node: Node) -> Self {
        Self {
            id: node.id.to_string(),
            r#type: node.r#type,                    // Perfect 1:1 mapping!
            content: serde_json::to_string(&node.content).unwrap_or_default(),
            metadata: node.metadata.map(|m| serde_json::to_string(&m).unwrap_or_default()),
            created_at: node.created_at,
            updated_at: node.updated_at,
            parent_id: node.parent_id.map(|id| id.to_string()),
            next_sibling: node.next_sibling.map(|id| id.to_string()),
            root_id: node.root_id.map(|id| id.to_string()),
            vector: None,        // Set by embedding service
            vector_model: None,  // Set by embedding service
        }
    }
}

/// Clean conversion from LanceDocument to Node
impl TryFrom<LanceDocument> for Node {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(doc: LanceDocument) -> Result<Self, Self::Error> {
        let content = serde_json::from_str(&doc.content)?;
        let mut node = Node::with_id(
            NodeId::from_string(doc.id),
            doc.r#type,                           // Perfect 1:1 mapping!
            content,
        );

        // Set optional fields
        if let Some(metadata_str) = doc.metadata {
            let metadata = serde_json::from_str(&metadata_str)?;
            node = node.with_metadata(metadata);
        }

        node.created_at = doc.created_at;
        node.updated_at = doc.updated_at;
        node.parent_id = doc.parent_id.map(NodeId::from_string);
        node.next_sibling = doc.next_sibling.map(NodeId::from_string);
        node.root_id = doc.root_id.map(NodeId::from_string);

        Ok(node)
    }
}

/// Helper constructors for common node types (NS-125 migration helpers)
/// Since we can't impl on external Node type, these are standalone functions
#[allow(dead_code)]
pub fn create_text_node(content: serde_json::Value) -> Node {
    Node::new("text".to_string(), content)
}

#[allow(dead_code)]
pub fn create_date_node(content: serde_json::Value) -> Node {
    Node::new("date".to_string(), content)
}

#[allow(dead_code)]
pub fn create_task_node(content: serde_json::Value) -> Node {
    Node::new("task".to_string(), content)
}

#[allow(dead_code)]
pub fn create_image_node(content: serde_json::Value) -> Node {
    Node::new("image".to_string(), content)
}

#[allow(dead_code)]
pub fn create_project_node(content: serde_json::Value) -> Node {
    Node::new("project".to_string(), content)
}

/// Node types supported by the universal schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    Text,
    Image,
    Date,
    Task,
    Customer,
    Project,
    Document,
    Audio,
    Video,
    Link,
    Contact,
    Event,
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Text => write!(f, "text"),
            NodeType::Image => write!(f, "image"),
            NodeType::Date => write!(f, "date"),
            NodeType::Task => write!(f, "task"),
            NodeType::Customer => write!(f, "customer"),
            NodeType::Project => write!(f, "project"),
            NodeType::Document => write!(f, "document"),
            NodeType::Audio => write!(f, "audio"),
            NodeType::Video => write!(f, "video"),
            NodeType::Link => write!(f, "link"),
            NodeType::Contact => write!(f, "contact"),
            NodeType::Event => write!(f, "event"),
        }
    }
}

impl From<&str> for NodeType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "text" => NodeType::Text,
            "image" => NodeType::Image,
            "date" => NodeType::Date,
            "task" => NodeType::Task,
            "customer" => NodeType::Customer,
            "project" => NodeType::Project,
            "document" => NodeType::Document,
            "audio" => NodeType::Audio,
            "video" => NodeType::Video,
            "link" => NodeType::Link,
            "contact" => NodeType::Contact,
            "event" => NodeType::Event,
            _ => NodeType::Text, // Default fallback
        }
    }
}

/// Image node specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub alt_text: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: String, // "png", "jpg", "webp", etc.
    pub file_size_bytes: Option<u64>,
    pub original_filename: Option<String>,
    pub camera_info: Option<CameraInfo>,
}

/// Camera/device information for images
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraInfo {
    pub make: Option<String>,
    pub model: Option<String>,
    pub software: Option<String>,
    pub timestamp: Option<String>,
    pub gps_location: Option<GpsLocation>,
}

/// GPS location data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
}

/// Content type enumeration for multimodal support
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    TextPlain,
    TextMarkdown,
    TextHtml,
    ImagePng,
    ImageJpeg,
    ImageWebp,
    ImageGif,
    AudioMp3,
    AudioWav,
    VideoMp4,
    VideoWebm,
    ApplicationPdf,
    ApplicationJson,
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mime_type = match self {
            ContentType::TextPlain => "text/plain",
            ContentType::TextMarkdown => "text/markdown",
            ContentType::TextHtml => "text/html",
            ContentType::ImagePng => "image/png",
            ContentType::ImageJpeg => "image/jpeg",
            ContentType::ImageWebp => "image/webp",
            ContentType::ImageGif => "image/gif",
            ContentType::AudioMp3 => "audio/mp3",
            ContentType::AudioWav => "audio/wav",
            ContentType::VideoMp4 => "video/mp4",
            ContentType::VideoWebm => "video/webm",
            ContentType::ApplicationPdf => "application/pdf",
            ContentType::ApplicationJson => "application/json",
        };
        write!(f, "{}", mime_type)
    }
}

impl From<&str> for ContentType {
    fn from(mime_type: &str) -> Self {
        match mime_type.to_lowercase().as_str() {
            "text/plain" => ContentType::TextPlain,
            "text/markdown" => ContentType::TextMarkdown,
            "text/html" => ContentType::TextHtml,
            "image/png" => ContentType::ImagePng,
            "image/jpeg" | "image/jpg" => ContentType::ImageJpeg,
            "image/webp" => ContentType::ImageWebp,
            "image/gif" => ContentType::ImageGif,
            "audio/mp3" | "audio/mpeg" => ContentType::AudioMp3,
            "audio/wav" => ContentType::AudioWav,
            "video/mp4" => ContentType::VideoMp4,
            "video/webm" => ContentType::VideoWebm,
            "application/pdf" => ContentType::ApplicationPdf,
            "application/json" => ContentType::ApplicationJson,
            _ => ContentType::TextPlain, // Default fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_schema_creation() {
        let schema = NodeSchema::create_node_schema();
        assert!(schema.fields().len() >= 10); // Should have core Node fields

        // Check for key Node fields aligned with core-types
        assert!(schema.field_with_name("id").is_ok());
        assert!(schema.field_with_name("type").is_ok());
        assert!(schema.field_with_name("content").is_ok());
        assert!(schema.field_with_name("vector").is_ok());
        assert!(schema.field_with_name("vector_model").is_ok());
        assert!(schema.field_with_name("root_id").is_ok());
    }

    #[test]
    fn test_node_type_conversion() {
        assert_eq!(NodeType::from("image"), NodeType::Image);
        assert_eq!(NodeType::from("TEXT"), NodeType::Text);
        assert_eq!(NodeType::from("unknown"), NodeType::Text);

        assert_eq!(NodeType::Image.to_string(), "image");
        assert_eq!(NodeType::Text.to_string(), "text");
    }

    #[test]
    fn test_content_type_conversion() {
        assert_eq!(ContentType::from("image/png"), ContentType::ImagePng);
        assert_eq!(ContentType::from("IMAGE/JPEG"), ContentType::ImageJpeg);
        assert_eq!(ContentType::from("unknown"), ContentType::TextPlain);

        assert_eq!(ContentType::ImagePng.to_string(), "image/png");
        assert_eq!(ContentType::TextPlain.to_string(), "text/plain");
    }
}
