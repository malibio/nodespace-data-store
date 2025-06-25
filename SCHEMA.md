# NodeSpace Data Store Schema Documentation

This document describes the SurrealDB schema used by the NodeSpace Data Store, including the vector embedding format and hierarchical organization patterns.

## Database Configuration

**Database**: SurrealDB (Multi-model: Document + Graph + Vector)  
**Namespace**: `nodespace`  
**Database**: `nodes`  
**Storage**: RocksDB (file-based) or Memory (testing)

## Core Tables

### 1. Text Table (`text`)

Primary storage for text content with semantic embeddings.

```sql
-- Table: text
-- Purpose: Text content with vector embeddings for semantic search
CREATE text:uuid SET
  content = "Strategic planning session revealed key market opportunities...", 
  parent_date = "2025-05-01",
  embedding = [0.123, -0.456, 0.789, ...], -- 384 dimensions
  created_at = "2025-06-25T14:30:45.123Z",
  updated_at = "2025-06-25T14:30:45.123Z";
```

**Fields:**
- `id`: Record ID format `text:{uuid}` where hyphens are converted to underscores
- `content`: Text content as string
- `parent_date`: Optional date value for hierarchical organization
- `embedding`: Vector<float, 384> - BAAI/bge-small-en-v1.5 embeddings
- `created_at`: ISO 8601 timestamp (RFC3339)
- `updated_at`: ISO 8601 timestamp (RFC3339)

**Indexing:**
- Vector similarity search on `embedding` field
- Text search on `content` field
- Date filtering on `parent_date` field

### 2. Date Table (`date`)

Hierarchical organization nodes for date-based grouping.

```sql
-- Table: date  
-- Purpose: Hierarchical date organization
CREATE date:`2025-05-01` SET
  date_value = "2025-05-01",
  description = "Strategic Planning Session",
  created_at = "2025-06-25T14:30:45.123Z",
  updated_at = "2025-06-25T14:30:45.123Z";
```

**Fields:**
- `id`: Record ID format `date:{date_value}` (e.g., `date:2025-05-01`)
- `date_value`: Date string in YYYY-MM-DD format
- `description`: Optional descriptive text for the date
- `created_at`: ISO 8601 timestamp
- `updated_at`: ISO 8601 timestamp

**Note**: Date nodes typically do not have embeddings as they are organizational nodes.

### 3. Nodes Table (`nodes`)

Generic node storage compatible with NodeSpace core types.

```sql
-- Table: nodes
-- Purpose: Generic node storage with full NodeSpace compatibility
CREATE nodes:uuid SET
  content = {"type": "note", "text": "Meeting notes..."},
  metadata = {"category": "planning", "priority": "high"},
  embedding = [0.234, -0.567, 0.890, ...], -- 384 dimensions
  created_at = "2025-06-25T14:30:45.123Z",
  updated_at = "2025-06-25T14:30:45.123Z",
  next_sibling = "nodes:other-uuid",
  previous_sibling = null;
```

**Fields:**
- `id`: Record ID format `nodes:{uuid}`
- `content`: serde_json::Value - flexible content structure
- `metadata`: Optional serde_json::Value for additional data
- `embedding`: Vector<float, 384> - BAAI/bge-small-en-v1.5 embeddings  
- `created_at`: ISO 8601 timestamp
- `updated_at`: ISO 8601 timestamp
- `next_sibling`: Optional NodeId for linked list structure
- `previous_sibling`: Optional NodeId for linked list structure

## Relationships

### Hierarchical Organization (`contains`)

Date nodes contain text nodes using SurrealDB relationships:

```sql
-- Create relationship: date contains text
RELATE date:`2025-05-01`->contains->text:{uuid};

-- Query: Find all text for a date
SELECT * FROM date:`2025-05-01`->contains->text;

-- Query: Find date for a text node  
SELECT * FROM <-contains<-date WHERE ->contains->text:{uuid};
```

**Relationship Properties:**
- Type: `contains`
- Direction: `date` → `text`
- Cardinality: One date to many text nodes
- Purpose: Hierarchical organization by date

### Sibling Relationships

Nodes can be linked in sequences using sibling pointers:

```sql
-- Update sibling relationships
UPDATE nodes:{uuid1} SET next_sibling = "nodes:{uuid2}";
UPDATE nodes:{uuid2} SET previous_sibling = "nodes:{uuid1}";
```

**Use Cases:**
- Ordered sequences of related content
- Thread-like organization
- Chronological ordering within dates

## Vector Embeddings

### Model Specification

**Current Model**: BAAI/bge-small-en-v1.5  
**Dimensions**: 384  
**Range**: [-1.0, 1.0] (normalized)  
**Generation**: fastembed-rs with ONNX Runtime  

### Embedding Field

```sql
-- Vector field specification
embedding: Vector<float, 384>

-- Example embedding (truncated)
embedding: [0.123, -0.456, 0.789, 0.234, -0.567, ...]
```

### Similarity Search

SurrealDB native vector search using cosine similarity:

```sql
-- Semantic search query
SELECT *, vector::similarity::cosine(embedding, $query_vector) AS score 
FROM text 
WHERE embedding IS NOT NULL 
ORDER BY score DESC 
LIMIT 10;
```

**Performance:**
- Index: Automatic vector indexing by SurrealDB
- Algorithm: Cosine similarity (range 0.0 to 1.0)
- Speed: Optimized for <50ms search time (NS-43 requirement)

### Migration Compatibility

**Previous Model**: all-MiniLM-L6-v2 (384 dimensions)  
**Current Model**: BAAI/bge-small-en-v1.5 (384 dimensions)  

**Migration Strategy:**
1. Preserve content without embeddings
2. Clear old embedding vectors  
3. Regenerate with new model
4. Maintain schema compatibility

## Data Types and Conversion

### NodeId Mapping

NodeSpace `NodeId` ↔ SurrealDB `Thing` conversion:

```rust
// NodeSpace: "123e4567-e89b-12d3-a456-426614174000"
// SurrealDB: "nodes:123e4567_e89b_12d3_a456_426614174000"

// Conversion rules:
// 1. Hyphens (-) → Underscores (_) 
// 2. Prefix with table name
// 3. Maintain UUID format integrity
```

### Content Storage

Flexible content storage using serde_json::Value:

```rust
// String content
content: "Simple text content"

// Structured content  
content: {
  "type": "meeting_notes",
  "title": "Q3 Strategy Session", 
  "participants": ["Alice", "Bob"],
  "action_items": ["Task 1", "Task 2"]
}

// Rich content
content: {
  "html": "<h1>Title</h1><p>Content...</p>",
  "markdown": "# Title\n\nContent...",
  "plain_text": "Title\n\nContent..."
}
```

## Query Patterns

### Common Queries

**1. Date-based Content Retrieval:**
```sql
-- Get all content for a specific date
SELECT * FROM date:`2025-05-01`->contains->text;

-- Get content for date range
SELECT * FROM text WHERE parent_date >= "2025-05-01" AND parent_date <= "2025-05-31";
```

**2. Semantic Search:**
```sql  
-- Similarity search with threshold
SELECT *, vector::similarity::cosine(embedding, $vector) AS score
FROM text
WHERE embedding IS NOT NULL AND vector::similarity::cosine(embedding, $vector) > 0.7
ORDER BY score DESC
LIMIT 5;
```

**3. Content Discovery:**
```sql
-- Recent content
SELECT * FROM text ORDER BY created_at DESC LIMIT 10;

-- Content by date with embeddings
SELECT * FROM text WHERE parent_date = "2025-05-01" AND embedding IS NOT NULL;
```

### Performance Optimizations

**Indexing Strategy:**
- Vector index on `embedding` fields (automatic)
- Date index on `parent_date` and `date_value` fields
- Text index on `content` fields for full-text search

**Query Optimization:**
- Use specific date lookups: `date:YYYY-MM-DD` vs scanning
- Limit vector search results to reasonable sizes (≤50)
- Combine semantic and date filtering for precision

## Schema Evolution

### Version History

**v1.0**: Initial schema with Candle embeddings
**v1.1**: Migration to fastembed-rs + bge-small-en-v1.5
**v1.2**: Added sibling pointer support for node ordering

### Future Considerations

**Potential Changes:**
- Additional embedding models (domain-specific)
- Multi-lingual embedding support
- Embedding compression for storage efficiency
- Advanced relationship types beyond `contains`

**Backward Compatibility:**
- Schema designed for additive changes
- Embedding field supports dimension changes
- Content field supports any JSON structure
- Relationship patterns extensible

## Development Guidelines

### Adding New Tables

Follow the established patterns:
1. Use UUID-based record IDs
2. Include `created_at` and `updated_at` timestamps
3. Consider embedding requirements for searchable content
4. Define clear relationship patterns with existing tables

### Embedding Integration

For new content types requiring semantic search:
1. Add `embedding: Vector<float, 384>` field
2. Implement content extraction for embedding generation
3. Update search queries to include new table
4. Test vector similarity search performance

### Migration Planning

For future embedding model changes:
1. Document current model specifications
2. Create backup/restore procedures
3. Plan regeneration strategy for large datasets
4. Test migration process on development data
5. Monitor search quality before/after changes