# NodeSpace Data Store Schema Documentation

This document describes the actual LanceDB schema implementation used by the NodeSpace Data Store.

## Database Configuration

**Database**: LanceDB (High-performance vector database)  
**Schema**: Universal Document Schema for multimodal data  
**Storage**: Apache Arrow columnar format  
**Default Vector Dimensions**: 384 (configurable)

## Core Data Structure

### UniversalNode Schema

The data store uses a single `UniversalNode` structure that maps directly to LanceDB's Arrow columnar storage:

```rust
pub struct UniversalNode {
    pub id: String,                    // Unique entity identifier
    pub r#type: String,               // "text", "image", "date", "task", etc.
    pub content: String,              // Primary content as string
    
    // Multi-level embedding support
    pub individual_vector: Vec<f32>,     // Content embedding (384-dim default)
    pub contextual_vector: Option<Vec<f32>>,    // Context-aware embedding
    pub hierarchical_vector: Option<Vec<f32>>,  // Hierarchical path embedding
    pub vector: Vec<f32>,                // Backward compatibility mapping
    
    // Embedding metadata
    pub embedding_model: Option<String>,
    pub embeddings_generated_at: Option<String>,
    
    // Hierarchical relationships
    pub parent_id: Option<String>,
    pub before_sibling_id: Option<String>,    // Backward linking
    pub children_ids: Vec<String>,
    pub mentions: Vec<String>,               // Cross-references
    pub root_id: Option<String>,            // Hierarchy optimization
    
    // Temporal tracking
    pub created_at: String,              // ISO 8601 timestamp
    pub updated_at: String,              // ISO 8601 timestamp
    
    // Flexible metadata
    pub metadata: Option<serde_json::Value>,
}
```

## LanceDB Arrow Schema

The actual Arrow schema used for LanceDB storage:

```rust
Schema::new(vec![
    Field::new("id", DataType::Utf8, false),
    Field::new("type", DataType::Utf8, false),
    Field::new("content", DataType::Utf8, false),
    Field::new("vector", DataType::FixedSizeList(Field::new("item", DataType::Float32, false), 384), false),
    Field::new("parent_id", DataType::Utf8, true),
    Field::new("before_sibling_id", DataType::Utf8, true),
    Field::new("children_ids", DataType::List(Field::new("item", DataType::Utf8, false)), true),
    Field::new("mentions", DataType::List(Field::new("item", DataType::Utf8, false)), true),
    Field::new("root_id", DataType::Utf8, true),
    Field::new("created_at", DataType::Utf8, false),
    Field::new("updated_at", DataType::Utf8, false),
    Field::new("metadata", DataType::Utf8, true),  // JSON string
])
```

## Node Types

### 1. Text Nodes (`r#type: "text"`)

**Content**: Text content stored directly in the `content` field  
**Embeddings**: 384-dimensional vectors (default)  
**Metadata**: Simplified approach - metadata is cleared for text nodes to reduce storage overhead

### 2. Image Nodes (`r#type: "image"`)

**Content**: Image description or filename  
**Image Data**: Base64-encoded binary data stored in metadata  
**Embeddings**: Same 384-dimensional vector space  
**Metadata**: Contains image-specific properties:

```json
{
  "image_data": "base64_encoded_image_data",
  "filename": "image.jpg",
  "width": 1920,
  "height": 1080,
  "format": "jpeg"
}
```

### 3. Date Nodes (`r#type: "date"`)

**Content**: Date representation with description  
**Purpose**: Container nodes for temporal organization  
**Metadata**: Simplified approach - metadata is cleared for date nodes

### 4. Other Node Types

**Supported Types**: "task", "customer", "project", etc.  
**Metadata**: Preserved for non-text/date node types  
**Content**: Type-specific content stored as strings

## Embedding Architecture

### Single Vector Field

All embeddings are stored in the primary `vector` field with configurable dimensions (default: 384).

### Multi-level Embedding Support

The system supports three levels of embeddings:

1. **Individual**: Content-based embedding
2. **Contextual**: Context-aware embedding (with siblings/parent)
3. **Hierarchical**: Path-based embedding for hierarchy navigation

Multi-level embeddings are stored in the metadata field and synchronized with the primary vector field.

### Embedding Models

- **Configurable**: No hardcoded model dependencies
- **Default Dimension**: 384 (FastEmbed/BGE-small-en-v1.5 compatible)
- **Pluggable**: Via `EmbeddingGenerator` trait
- **Fallback**: Zero vectors when no embedding provided

## Hierarchical Relationships

### Parent-Child Structure

Relationships are managed through JSON-based fields:

- **parent_id**: Direct reference to parent entity
- **children_ids**: Array of child entity identifiers  
- **before_sibling_id**: Backward linking for sibling navigation
- **root_id**: Optimization field pointing to hierarchy root

### Hierarchy Optimization

The `root_id` field enables efficient hierarchy queries:

```rust
// O(1) root-based queries (filtered in application layer)
let nodes = data_store.get_nodes_by_root(&root_id).await?;
let typed_nodes = data_store.get_nodes_by_root_and_type(&root_id, "text").await?;
```

## Search Operations

### Vector Search

```rust
// Basic similarity search
let results = data_store.search_similar_nodes(embedding, limit).await?;

// Multi-level embedding search
let results = data_store.search_by_individual_embedding(embedding, limit).await?;
let results = data_store.search_by_contextual_embedding(embedding, limit).await?;
let results = data_store.search_by_hierarchical_embedding(embedding, limit).await?;
```

### Cross-Modal Search

```rust
// Search across node types
let results = data_store.search_multimodal(
    query_embedding, 
    vec![NodeType::Text, NodeType::Image]
).await?;
```

### Hybrid Search

```rust
// Weighted search with configurable scoring
let config = HybridSearchConfig {
    semantic_weight: 0.7,
    structural_weight: 0.2,
    temporal_weight: 0.1,
    // ... other config options
};
let results = data_store.hybrid_multimodal_search(embedding, &config).await?;
```

## Performance Characteristics

### Current Implementation

- **Vector Search**: Native LanceDB operations with cosine similarity
- **Hierarchy Queries**: Application-level filtering (not indexed)
- **Text Search**: Content string matching
- **Batch Operations**: Individual insert/update operations

### Optimization Opportunities

1. **Indexing**: Implement native LanceDB filtering
2. **Batch Operations**: Bulk insert/update support
3. **Connection Pooling**: Multi-connection support
4. **Caching**: Query result caching

## Schema Evolution

### Metadata Flexibility

The metadata field supports schema evolution through JSON:

```json
{
  "custom_field": "value",
  "version": "1.0",
  "tags": ["important", "reviewed"]
}
```

### Backward Compatibility

- Multi-level embeddings stored in metadata don't break existing queries
- Primary vector field maintains compatibility
- New node types can be added without schema changes

## Integration Patterns

### Basic Usage

```rust
use nodespace_data_store::{DataStore, LanceDataStore};

// Initialize with default 384-dimensional vectors
let data_store = LanceDataStore::new("./data/nodes.db").await?;

// Store node with automatic embedding generation
let node_id = data_store.store_node(node).await?;

// Store with provided embedding
let node_id = data_store.store_node_with_embedding(node, embedding).await?;

// Vector similarity search
let results = data_store.search_similar_nodes(query_embedding, 10).await?;
```

### Custom Vector Dimensions

```rust
// Initialize with custom dimensions
let data_store = LanceDataStore::with_vector_dimension("./data/nodes.db", 512).await?;
```

### Multi-level Embeddings

```rust
use nodespace_data_store::MultiLevelEmbeddings;

let embeddings = MultiLevelEmbeddings {
    individual: content_embedding,
    contextual: Some(context_embedding),
    hierarchical: Some(hierarchy_embedding),
    embedding_model: Some("bge-small-en-v1.5".to_string()),
    generated_at: Utc::now(),
};

let node_id = data_store.store_node_with_multi_embeddings(node, embeddings).await?;
```

## Testing

The repository includes comprehensive integration tests:

```bash
# Run all tests
cargo test

# Test specific functionality
cargo test test_cross_modal_search
cargo test test_hierarchical_relationships
cargo test test_vector_search_functionality
```

## Best Practices

### Node Organization

1. Use meaningful content in the `content` field for both search and human readability
2. Store structured data in metadata as JSON
3. Maintain consistent node types across your application
4. Use hierarchy optimization with `root_id` for deep tree structures

### Performance

1. Provide embeddings when storing nodes to avoid fallback to zero vectors
2. Use appropriate vector dimensions for your use case
3. Consider batch operations for bulk data loading
4. Filter by node type when possible to reduce search space

### Schema Design

1. Keep the `content` field meaningful for search
2. Use metadata for structured, type-specific data
3. Maintain relationship consistency between parent/child references
4. Include temporal information for time-based queries