# NodeSpace Data Store

**Database, persistence, and vector storage for NodeSpace**

This repository implements the complete data layer for NodeSpace, providing persistent storage for nodes, embeddings, and search capabilities. It serves as the **single source of persistence** across the distributed system.

## 🎯 Purpose

- **Node storage** - Persistent storage for text content, metadata, and relationships
- **Vector database** - Embedding storage for semantic search capabilities
- **Search engine** - Fast full-text and semantic search implementation
- **Data integrity** - ACID transactions and data validation

## 📦 Key Features

- **SQLite backend** - Local-first storage with excellent performance
- **Vector search** - Optimized embedding similarity search
- **Full-text search** - Traditional keyword-based search capabilities
- **Schema migration** - Automated database schema updates
- **Backup/restore** - Data export and import functionality

## 🔗 Dependencies

- **`nodespace-core-types`** - Data structures and `DataStore` trait interface
- **SQLite** - Primary database engine
- **Vector storage library** - For embedding similarity search

## 🏗️ Architecture Context

Part of the [NodeSpace system architecture](https://github.com/malibio/nodespace-system-design):

1. `nodespace-core-types` - Shared data structures and interfaces
2. **`nodespace-data-store`** ← **You are here**
3. `nodespace-nlp-engine` - AI/ML processing and LLM integration  
4. `nodespace-workflow-engine` - Automation and event processing
5. `nodespace-core-logic` - Business logic orchestration
6. `nodespace-core-ui` - React components and UI
7. `nodespace-desktop-app` - Tauri application shell

## 🚀 Getting Started

```bash
# Add to your Cargo.toml
[dependencies]
nodespace-data-store = { git = "https://github.com/malibio/nodespace-data-store" }

# Use in your code
use nodespace_data_store::SqliteDataStore;
use nodespace_core_types::{DataStore, Node};

let store = SqliteDataStore::new("./data/nodes.db").await?;
let node = store.create_node(create_request).await?;
```

## 🔄 MVP Implementation

The initial implementation focuses on the core RAG workflow:

1. **Create nodes** - Store text content with auto-generated IDs
2. **Store embeddings** - Persist vector representations for search
3. **Semantic search** - Find similar content using vector similarity
4. **Full-text search** - Traditional keyword-based search
5. **Node retrieval** - Fast access to individual nodes and metadata

## 🧪 Testing

```bash
# Run all tests including integration tests
cargo test

# Test with sample data
cargo run --example create_sample_data

# Benchmark search performance
cargo bench
```

## 📋 Development Status

- [ ] Implement `DataStore` trait from core-types
- [ ] Set up SQLite schema and migrations
- [ ] Add vector storage implementation
- [ ] Implement full-text search
- [ ] Add comprehensive test suite
- [ ] Performance optimization and benchmarks

---

**Project Management:** All tasks tracked in [NodeSpace Project](https://github.com/users/malibio/projects/4)