# Embedding Migration Guide: Candle to fastembed-rs

This document provides a comprehensive guide for migrating vector embeddings from the legacy Candle + all-MiniLM-L6-v2 implementation to the new fastembed-rs + BAAI/bge-small-en-v1.5 implementation.

## Overview

### Why Migrate?

The migration from Candle to fastembed-rs provides significant improvements:

**Performance Benefits:**
- **ONNX Runtime**: Unified foundation with text generation
- **Parallel Processing**: Built-in Rayon parallelization vs sequential processing
- **Modern Models**: BAAI/bge-small-en-v1.5 (top MTEB leaderboard performance)
- **Cross-platform**: Better Windows/macOS compatibility

**Architectural Benefits:**
- **Unified Stack**: Both embeddings and text generation use ONNX Runtime
- **Simplified Code**: ~180 lines → ~50 lines in embedding implementation
- **Better Maintenance**: Remove complex manual BERT implementation
- **Model Flexibility**: Easy switching between embedding models

### Migration Impact

**Vector Incompatibility:**
```rust
// Old embeddings (all-MiniLM-L6-v2)
[0.123, -0.456, 0.789, ...] // 384 dimensions

// New embeddings (bge-small-en-v1.5) 
[0.891, -0.234, 0.567, ...] // Same dimensions, DIFFERENT VALUES
```

**Critical**: Semantic search will return incorrect results with mixed old/new embeddings.

## Migration Process

### Phase 1: Pre-Migration Backup

**Command:**
```bash
cargo run --example migrate_embeddings
```

**What it does:**
1. Backs up all content without embeddings (preserves text, metadata, timestamps)
2. Clears existing vector embeddings from the database
3. Prepares database for clean regeneration

**Output:**
- Backup completed: 612 content records preserved
- Embeddings cleared: 1024 records cleaned
- Database ready for fastembed-rs regeneration

### Phase 2: NLP Engine Migration

**Dependency**: Wait for NS-54 completion (fastembed-rs implementation in nlp-engine)

**Requirements:**
- NLP engine updated to use fastembed-rs
- BAAI/bge-small-en-v1.5 model available
- ONNX Runtime integration functional

### Phase 3: Embedding Regeneration

**Command (when ready):**
```bash
cargo run --example regenerate_embeddings
```

**Process:**
1. Discover all content requiring embeddings
2. Generate new embeddings using fastembed-rs + bge-small-en-v1.5
3. Store updated nodes with new vector data
4. Validate semantic search functionality

### Phase 4: Validation

**Command:**
```bash
cargo run --example fastembed_integration
```

**Validation Steps:**
1. **Embedding Count**: Verify all content has new embeddings
2. **Semantic Search**: Test similarity search functionality
3. **Quality Assessment**: Compare search relevance vs old model
4. **Performance Check**: Measure embedding generation speed

## Database Schema Impact

### No Breaking Changes

The migration maintains full backward compatibility:

**Node Structure (unchanged):**
```rust
pub struct Node {
    pub id: NodeId,
    pub content: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
    pub next_sibling: Option<NodeId>,
    pub previous_sibling: Option<NodeId>,
}
```

**SurrealDB Schema (unchanged):**
- `text` table: Content with embeddings
- `date` table: Hierarchical organization  
- `nodes` table: Generic node storage
- Relationships: `date->contains->text` patterns

**Vector Field:**
- Field name: `embedding`
- Type: `Vec<f32>` (384 dimensions)
- Only the values change, not the structure

## Testing Updates

### Test Fixtures

All integration tests updated to use 384-dimensional embeddings:

```rust
// Before (5-dimensional)
let embedding1 = vec![0.1, 0.2, 0.3, 0.4, 0.5];

// After (384-dimensional, deterministic for testing)
let embedding1 = generate_test_embedding("rust systems programming", 42);
```

### Test Helper Function

```rust
fn generate_test_embedding(content: &str, seed: u32) -> Vec<f32> {
    // Generates deterministic 384-dimensional embeddings for testing
    // Simulates bge-small-en-v1.5 output dimensions
}
```

## Performance Expectations

### Embedding Generation

**Target Performance:**
- **Speed**: <200ms per text embedding
- **Throughput**: ~5-10 embeddings/second
- **Memory**: <4GB total system usage

**Quality Improvements:**
- **MTEB Score**: bge-small-en-v1.5 ranks higher than all-MiniLM-L6-v2
- **Semantic Accuracy**: Better similarity detection for domain-specific content
- **Cross-lingual**: Improved handling of technical terminology

### Semantic Search

**Expected Improvements:**
- **Relevance**: Better semantic matching for business/marketing content
- **Speed**: Similar or improved search performance
- **Accuracy**: Reduced false positives in similarity scoring

## Rollback Strategy

### Emergency Rollback

If critical issues arise:

1. **Restore Backup**: Re-run sample data generation
2. **Revert NLP Engine**: Switch back to Candle implementation
3. **Regenerate Old Embeddings**: Use all-MiniLM-L6-v2 model

**Command:**
```bash
cargo run --example create_sample_data
```

### Gradual Migration

For production systems, consider:
1. **Dual Column Approach**: Store both old and new embeddings temporarily
2. **A/B Testing**: Compare search quality before full migration
3. **Incremental Update**: Migrate content in batches

## Troubleshooting

### Common Issues

**Issue**: Semantic search returns no results
**Cause**: Embeddings not regenerated after clearing
**Solution**: Run regeneration script with fastembed-rs

**Issue**: Performance degradation
**Cause**: ONNX Runtime not optimized for system
**Solution**: Check Metal/CUDA acceleration settings

**Issue**: Memory usage too high
**Cause**: Embedding generation batch size too large
**Solution**: Reduce batch size in regeneration script

### Validation Commands

```bash
# Check embedding status
cargo run --example simple_regenerate

# Test database structure
cargo run --example show_db_structure

# Validate semantic search
cargo test test_semantic_search_with_embedding
```

## Future Embedding Migrations

### Process Template

This migration establishes the pattern for future embedding model changes:

1. **Backup Content**: Always preserve text without embeddings
2. **Clear Embeddings**: Remove old vectors to prevent corruption
3. **Update Tests**: Adjust test fixtures for new dimensions
4. **Regenerate**: Use new model to create fresh embeddings
5. **Validate**: Confirm search quality and performance
6. **Document**: Update schema documentation

### Model Considerations

When choosing future embedding models:
- **Dimension Compatibility**: SurrealDB vector fields adapt automatically
- **Performance Impact**: Consider generation speed vs quality tradeoffs
- **Domain Specificity**: Match model to content type (business, technical, etc.)
- **Language Support**: Ensure coverage for required languages

## References

- **Linear Issue**: NS-55 (Database Migration for New fastembed-rs Embeddings)
- **Blocking Issue**: NS-54 (Migrate Vector Embeddings from Candle to fastembed-rs)
- **Model Documentation**: [BAAI/bge-small-en-v1.5](https://huggingface.co/BAAI/bge-small-en-v1.5)
- **fastembed-rs**: [GitHub Repository](https://github.com/Anush008/fastembed-rs)