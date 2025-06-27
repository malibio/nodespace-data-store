// LanceDB schema definitions for NodeSpace universal document format

use arrow_schema::{DataType, Field, Schema};
use std::sync::Arc;

/// Universal NodeSpace document schema for LanceDB
/// This schema supports infinite entity extensibility without schema changes
#[allow(dead_code)]
pub struct UniversalSchema;

#[allow(dead_code)]
impl UniversalSchema {
    /// Get the Arrow schema for the universal NodeSpace document format
    pub fn get_arrow_schema() -> Arc<Schema> {
        let fields = vec![
            // Core identification
            Field::new("id", DataType::Utf8, false),
            Field::new("node_type", DataType::Utf8, false),
            // Content (flexible JSON)
            Field::new("content", DataType::Utf8, false),
            Field::new("metadata", DataType::Utf8, true),
            // Vector embeddings
            Field::new(
                "vector",
                DataType::List(Arc::new(Field::new("item", DataType::Float32, false))),
                true,
            ),
            // Structural relationships (simplified from SurrealDB graph model)
            Field::new("parent_id", DataType::Utf8, true),
            Field::new("next_sibling", DataType::Utf8, true),
            Field::new("previous_sibling", DataType::Utf8, true),
            // Temporal fields
            Field::new("created_at", DataType::Utf8, false),
            Field::new("updated_at", DataType::Utf8, false),
            // Extensibility (ghost properties for infinite extensibility)
            Field::new("extended_properties", DataType::Utf8, true),
        ];

        Arc::new(Schema::new(fields))
    }
}
