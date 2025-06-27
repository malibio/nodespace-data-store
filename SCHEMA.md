# NodeSpace Data Store Schema Documentation

This document describes the LanceDB Universal Document Schema used by the NodeSpace Data Store, including vector embedding formats and hierarchical organization patterns.

## Database Configuration

**Database**: LanceDB (High-performance vector database)  
**Schema**: Universal Document Schema for multimodal data  
**Storage**: Apache Arrow columnar format with optimized vector indexes  
**Performance**: Sub-second search times for <2s target requirements

## Universal Document Schema

### Core Entity Structure

The Universal Document Schema stores all NodeSpace entities in a single, flexible table structure optimized for vector search and hierarchical relationships.

```rust
pub struct UniversalNode {
    pub id: String,                    // Unique entity identifier
    pub node_type: String,            // "text", "image", "date", "task", etc.
    pub content: String,              // Primary content (text/description)
    pub vector: Vec<f32>,             // 384 or 512-dimensional embeddings
    
    // Hierarchical relationships via JSON metadata
    pub parent_id: Option<String>,    // Direct parent reference
    pub children_ids: Vec<String>,    // List of child entity IDs
    pub mentions: Vec<String>,        // Referenced entity connections
    
    // Temporal tracking
    pub created_at: String,           // ISO 8601 timestamp
    pub updated_at: String,           // ISO 8601 timestamp
    
    // Flexible metadata for entity-specific fields
    pub metadata: Option<serde_json::Value>,
}
```

## Entity Types

### 1. Text Entities (`node_type: "text"`)

Primary content storage with semantic embeddings for search.

**Embedding Dimensions**: 384 (BGE-small-en-v1.5 text embeddings)
**Content**: Full text content in the `content` field
**Metadata**: Document structure, titles, sections, depth levels

Example metadata:
```json
{
  "title": "Product Launch Strategy",
  "parent_id": "2025-06-27",
  "depth": 1,
  "section_type": "main_document",
  "document_type": "strategy"
}
```

### 2. Image Entities (`node_type: "image"`)

Image content with CLIP vision embeddings for cross-modal search.

**Embedding Dimensions**: 512 (CLIP vision embeddings)
**Content**: Image description or filename
**Metadata**: Contains Base64-encoded image data, EXIF information, dimensions

Example metadata:
```json
{
  "image_data": "iVBORw0KGgoAAAANSUhEUgAA...",
  "filename": "conference_presentation.jpg",
  "mime_type": "image/jpeg",
  "width": 1920,
  "height": 1080,
  "exif_data": {
    "camera": "Canon EOS R5",
    "date": "2025-06-27T10:00:00Z"
  }
}
```

### 3. Date Entities (`node_type: "date"`)

Container entities for organizing content by date with hierarchical structure.

**Content**: Date representation with description
**Relationships**: Parent to multiple document entities
**Purpose**: Temporal organization and navigation

Example:
```json
{
  "id": "2025-06-27",
  "node_type": "date",
  "content": "📅 2025-06-27 - Product Launch Planning",
  "children_ids": ["strategy-doc-1", "meeting-notes-1"]
}
```

### 4. Task Entities (`node_type: "task"`)

Action items and workflow elements with completion tracking.

**Content**: Task description and requirements
**Metadata**: Status, assignee, due dates, priority levels

## Hierarchical Relationships

### Parent-Child Structure

The schema supports multi-level hierarchical organization through JSON-based relationships:

```
DateNode (2025-06-27)
├── Main Document (depth: 1)
│   ├── Section 1 (depth: 2)
│   │   ├── Subsection A (depth: 3)
│   │   │   ├── Detail 1 (depth: 4)
│   │   │   └── Detail 2 (depth: 4)
│   │   └── Subsection B (depth: 3)
│   └── Section 2 (depth: 2)
└── Related Document (depth: 1)
```

### Relationship Management

- **parent_id**: Direct reference to parent entity
- **children_ids**: Array of child entity identifiers
- **mentions**: Cross-references to related entities
- **depth**: Hierarchical level for navigation (stored in metadata)

## Vector Search Operations

### Semantic Search

```rust
// Text similarity search (384-dim BGE embeddings)
async fn semantic_search_with_embedding(
    &self,
    embedding: Vec<f32>,
    limit: usize,
) -> NodeSpaceResult<Vec<(Node, f32)>>
```

### Cross-Modal Search

```rust
// Search across text and images
async fn search_multimodal(
    &self,
    query_embedding: Vec<f32>,
    types: Vec<NodeType>
) -> NodeSpaceResult<Vec<Node>>
```

### Hybrid Search

Combines multiple relevance factors with configurable weights:

```rust
pub struct HybridSearchConfig {
    pub semantic_weight: f64,        // Embedding similarity (0.0-1.0)
    pub structural_weight: f64,      // Relationship proximity (0.0-1.0)  
    pub temporal_weight: f64,        // Time-based relevance (0.0-1.0)
    pub max_results: usize,          // Result limit
    pub min_similarity_threshold: f64, // Quality threshold
    pub enable_cross_modal: bool,    // Text↔Image search
    pub search_timeout_ms: u64,      // Performance limit
}
```

**Scoring Algorithm**:
```
final_score = (semantic_score × semantic_weight) +
              (structural_score × structural_weight) +
              (temporal_score × temporal_weight) +
              (cross_modal_bonus × 0.1)
```

## Performance Characteristics

### Search Performance

- **Target**: <2 seconds for complex queries
- **Achieved**: ~100 microseconds for hybrid search
- **Optimization**: LanceDB native vector operations with Arrow columnar storage

### Storage Efficiency

- **Columnar format**: Optimized for analytical workloads
- **Vector indexes**: Approximate nearest neighbor search
- **Compression**: Arrow format provides efficient encoding

### Scalability

- **Horizontal scaling**: Distributed LanceDB deployment support
- **Memory efficiency**: Lazy loading and streaming for large datasets
- **Concurrent access**: Multi-reader, single-writer model

## Schema Evolution

### Metadata Flexibility

The `metadata` field supports "ghost properties" for schema evolution:

```json
{
  "metadata": {
    // Standard fields
    "title": "Document Title",
    "depth": 2,
    
    // Future extensions (ghost properties)
    "ai_generated": true,
    "quality_score": 0.95,
    "language": "en-US",
    "custom_tags": ["important", "review"]
  }
}
```

### Version Compatibility

- **Backward compatible**: New metadata fields don't break existing code
- **Forward compatible**: Unknown fields are preserved during updates
- **Migration support**: Batch operations for schema updates

## Integration Patterns

### Cross-Component Usage

Other NodeSpace components access the data store through the `DataStore` trait:

```rust
use nodespace_data_store::{DataStore, LanceDataStore};

// Connect to shared database
let data_store = LanceDataStore::new("/path/to/shared.db").await?;

// Store with embedding
let node_id = data_store.store_node_with_embedding(node, embedding).await?;

// Cross-modal search
let results = data_store.search_multimodal(query_embedding, types).await?;
```

### E2E Testing

Sample data is available in shared directory for cross-component testing:
- **Location**: `/Users/malibio/nodespace/data/lance_db/sample_entry.db`
- **Content**: Hierarchical Product Launch Campaign Strategy
- **Structure**: 4 depth levels with realistic business content
- **Usage**: Load with `cargo run --example load_shared_sample_entry`

## Best Practices

### Entity Organization

1. **Use DateNodes** for temporal organization
2. **Maintain depth metadata** for hierarchical navigation
3. **Preserve markdown structure** in content fields
4. **Include descriptive titles** in metadata

### Performance Optimization

1. **Batch operations** for bulk inserts
2. **Filter by node_type** to reduce search space
3. **Use hybrid search** for complex relevance requirements
4. **Monitor similarity thresholds** to balance precision and recall

### Schema Design

1. **Keep content field meaningful** for both humans and search
2. **Use metadata for structure** rather than content parsing
3. **Maintain relationship consistency** between parent/child references
4. **Include temporal information** for time-based queries