// LanceDB schema definitions for NodeSpace universal document format
// Supporting both text and multimodal (image) content types

use arrow_schema::{DataType, Field, Schema};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Universal NodeSpace document schema for LanceDB
/// This schema supports infinite entity extensibility without schema changes,
/// including multimodal support for text and image content
#[allow(dead_code)]
pub struct UniversalSchema;

impl UniversalSchema {
    /// Get the Arrow schema for the universal NodeSpace document format
    /// Supports both text and image nodes with unified vector storage
    #[allow(dead_code)]
    pub fn get_arrow_schema() -> Arc<Schema> {
        let fields = vec![
            // Core identification
            Field::new("id", DataType::Utf8, false),
            Field::new("node_type", DataType::Utf8, false), // "text", "image", "date", "task", etc.
            // Content (flexible for text and binary data)
            Field::new("content", DataType::Utf8, false), // Text content or base64 encoded binary
            Field::new("content_type", DataType::Utf8, false), // "text/plain", "image/png", etc.
            Field::new("content_size_bytes", DataType::UInt64, true), // Size for binary content
            // Metadata (JSON blob for entity-specific fields)
            Field::new("metadata", DataType::Utf8, true),
            // Vector embeddings (unified for text and image)
            Field::new(
                "vector",
                DataType::List(Arc::new(Field::new("item", DataType::Float32, false))),
                true,
            ),
            Field::new("vector_model", DataType::Utf8, true), // Model used for embedding
            Field::new("vector_dimensions", DataType::UInt32, true), // Vector size
            // Structural relationships (simplified from SurrealDB graph model)
            Field::new("parent_id", DataType::Utf8, true),
            Field::new(
                "children_ids",
                DataType::List(Arc::new(Field::new("item", DataType::Utf8, false))),
                true,
            ),
            Field::new(
                "mentions",
                DataType::List(Arc::new(Field::new("item", DataType::Utf8, false))),
                true,
            ),
            Field::new("next_sibling", DataType::Utf8, true),
            Field::new("previous_sibling", DataType::Utf8, true),
            // Temporal fields
            Field::new("created_at", DataType::Utf8, false),
            Field::new("updated_at", DataType::Utf8, false),
            // Multimodal specific fields
            Field::new("image_alt_text", DataType::Utf8, true), // Alt text for images
            Field::new("image_width", DataType::UInt32, true),
            Field::new("image_height", DataType::UInt32, true),
            Field::new("image_format", DataType::Utf8, true), // "png", "jpg", "webp", etc.
            // Extensibility (ghost properties for infinite extensibility)
            Field::new("extended_properties", DataType::Utf8, true),
            // Performance and indexing
            Field::new("search_priority", DataType::Float32, true), // Boost factor for search
            Field::new("last_accessed", DataType::Utf8, true),      // For cache management
        ];

        Arc::new(Schema::new(fields))
    }

    /// Get schema for text-only nodes (backwards compatibility)
    #[allow(dead_code)]
    pub fn get_text_schema() -> Arc<Schema> {
        let fields = vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("node_type", DataType::Utf8, false),
            Field::new("content", DataType::Utf8, false),
            Field::new("metadata", DataType::Utf8, true),
            Field::new(
                "vector",
                DataType::List(Arc::new(Field::new("item", DataType::Float32, false))),
                true,
            ),
            Field::new("parent_id", DataType::Utf8, true),
            Field::new("created_at", DataType::Utf8, false),
            Field::new("updated_at", DataType::Utf8, false),
        ];

        Arc::new(Schema::new(fields))
    }
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
    fn test_universal_schema_creation() {
        let schema = UniversalSchema::get_arrow_schema();
        assert!(schema.fields().len() > 15); // Should have all multimodal fields

        // Check for key multimodal fields
        assert!(schema.field_with_name("content_type").is_ok());
        assert!(schema.field_with_name("image_width").is_ok());
        assert!(schema.field_with_name("vector_model").is_ok());
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
