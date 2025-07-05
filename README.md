
# NodeSpace Data Store

**Database, persistence, and vector storage for NodeSpace**

This repository implements the complete data layer for NodeSpace, providing persistent storage for nodes, embeddings, and search capabilities. It serves as the **single source of persistence** across the distributed system.

## Purpose

- **Node storage** - Persistent storage for text content, metadata, and relationships
- **Vector database** - Embedding storage for semantic search capabilities
- **Search engine** - Fast full-text and semantic search implementation
- **Data integrity** - ACID transactions and data validation

## Key Features

- **LanceDB backend** - High-performance vector database with Universal Document Schema
- **Cross-modal search** - Text and image embeddings (384/512-dim) with hybrid scoring
- **Hierarchical relationships** - JSON-based parent-child connections for document structure
- **Dynamic schemas** - Schema evolution with flexible metadata "ghost properties"
- **Performance optimized** - <2s search times with native vector operations
- **Performance monitoring** - Built-in metrics, alerting, and operation tracking

## Dependencies

- **[nodespace-core-types](https://github.com/malibio/nodespace-core-types)** - Shared data structures (NodeId, Node, NodeSpaceResult)
- **LanceDB** - High-performance vector database engine
- **Arrow** - Columnar in-memory analytics
- **serde_json** - For flexible content serialization and metadata

## Interface Ownership

This repository **owns the `DataStore` trait** as part of NodeSpace's distributed contract architecture:
- **Exports**: `DataStore` trait interface for other services to import
- **Implements**: Complete LanceDB-based data persistence layer with cross-modal search
- **Distributed pattern**: Other repositories import `use nodespace_data_store::DataStore;`

## Getting Started

### **New to NodeSpace? Start Here:**
1. **MVP Goal**: Enable text node storage and RAG query context retrieval

### **Development Setup:**
```bash
# Add to your Cargo.toml
[dependencies]
nodespace-data-store = { git = "https://github.com/malibio/nodespace-data-store" }

# Use in your code
use nodespace_data_store::{DataStore, LanceDataStore};
use nodespace_core_types::Node;

let store = LanceDataStore::new("./data/nodes.db").await?;
let node = store.store_node(node).await?;
```

## Architecture Context

Part of the NodeSpace system architecture:

1. [nodespace-core-types](https://github.com/malibio/nodespace-core-types) - Shared data structures and interfaces
2. **[nodespace-data-store](https://github.com/malibio/nodespace-data-store)** ← **You are here**
3. [nodespace-nlp-engine](https://github.com/malibio/nodespace-nlp-engine) - AI/ML processing and LLM integration  
4. [nodespace-workflow-engine](https://github.com/malibio/nodespace-workflow-engine) - Automation and event processing
5. [nodespace-core-logic](https://github.com/malibio/nodespace-core-logic) - Business logic orchestration
6. [nodespace-core-ui](https://github.com/malibio/nodespace-core-ui) - React components and UI
7. [nodespace-desktop-app](https://github.com/malibio/nodespace-desktop-app) - Tauri application shell

## MVP Implementation

The implementation focuses on LanceDB Universal Document Schema with cross-modal capabilities:

1. **Store nodes** - Universal Document Schema with flexible serde_json::Value content
2. **Hierarchical relationships** - JSON-based parent-child connections for document structure  
3. **Vector search** - Native LanceDB vector similarity with 384/512-dimensional embeddings
4. **Cross-modal search** - Text and image search with hybrid scoring algorithms
5. **Node retrieval** - Fast lookup by NodeId with metadata filtering and relationship traversal

## Testing

```bash
# Run all tests including integration tests
cargo test
```

## Documentation

For detailed schema information and implementation patterns, see [SCHEMA.md](./SCHEMA.md).