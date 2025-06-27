# ⚠️ BEFORE STARTING ANY WORK
👉 **STEP 1**: Read development workflow: `../nodespace-system-design/docs/development-workflow.md`
👉 **STEP 2**: Check Linear for assigned tasks
👉 **STEP 3**: Repository-specific patterns below

**This README.md only contains**: Repository-specific LanceDB and database patterns

# NodeSpace Data Store

**Database, persistence, and vector storage for NodeSpace**

This repository implements the complete data layer for NodeSpace, providing persistent storage for nodes, embeddings, and search capabilities. It serves as the **single source of persistence** across the distributed system.

## 🎯 Purpose

- **Node storage** - Persistent storage for text content, metadata, and relationships
- **Vector database** - Embedding storage for semantic search capabilities
- **Search engine** - Fast full-text and semantic search implementation
- **Data integrity** - ACID transactions and data validation

## 📦 Key Features

- **LanceDB backend** - High-performance vector database with Universal Document Schema
- **Cross-modal search** - Text and image embeddings (384/512-dim) with hybrid scoring
- **Hierarchical relationships** - JSON-based parent-child connections for document structure
- **Dynamic schemas** - Schema evolution with flexible metadata "ghost properties"
- **Performance optimized** - <2s search times with native vector operations

## 🔗 Dependencies

- **`nodespace-core-types`** - Shared data structures (NodeId, Node, NodeSpaceResult)
- **LanceDB** - High-performance vector database engine
- **serde_json** - For flexible content serialization and metadata

## 🏗️ Interface Ownership

This repository **owns the `DataStore` trait** as part of NodeSpace's distributed contract architecture:
- **Exports**: `DataStore` trait interface for other services to import
- **Implements**: Complete LanceDB-based data persistence layer with cross-modal search
- **Distributed pattern**: Other repositories import `use nodespace_data_store::DataStore;`

## 🚀 Getting Started

### **New to NodeSpace? Start Here:**
1. **📖 System Context**: Read [NodeSpace System Design](../nodespace-system-design) for complete architecture
2. **📋 Current Work**: Check [Linear workspace](https://linear.app/nodespace) for tasks (filter: `nodespace-data-store`)
3. **🤖 Development**: See [CLAUDE.md](./CLAUDE.md) for autonomous development workflow
4. **🎯 MVP Goal**: Enable text node storage and RAG query context retrieval

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

## 🏗️ Architecture Context

Part of the [NodeSpace system architecture](../nodespace-system-design/README.md):

1. `nodespace-core-types` - Shared data structures and interfaces
2. **`nodespace-data-store`** ← **You are here**
3. `nodespace-nlp-engine` - AI/ML processing and LLM integration  
4. `nodespace-workflow-engine` - Automation and event processing
5. `nodespace-core-logic` - Business logic orchestration
6. `nodespace-core-ui` - React components and UI
7. `nodespace-desktop-app` - Tauri application shell

## 🔄 MVP Implementation

The implementation focuses on LanceDB Universal Document Schema with cross-modal capabilities:

1. **Store nodes** - Universal Document Schema with flexible serde_json::Value content
2. **Hierarchical relationships** - JSON-based parent-child connections for document structure  
3. **Vector search** - Native LanceDB vector similarity with 384/512-dimensional embeddings
4. **Cross-modal search** - Text and image search with hybrid scoring algorithms
5. **Node retrieval** - Fast lookup by NodeId with metadata filtering and relationship traversal

## 🧪 Testing

```bash
# Run all tests including integration tests
cargo test

# Load hierarchical sample data for testing
cargo run --example load_shared_sample_entry

# Benchmark search performance
cargo bench

# Embedding migration (fastembed-rs)
cargo run --example migrate_embeddings
cargo run --example regenerate_embeddings
```

## 🔄 Embedding Migration

The repository supports migration from legacy Candle + all-MiniLM-L6-v2 to modern fastembed-rs + BAAI/bge-small-en-v1.5 embeddings.

### Quick Migration
```bash
# 1. Backup and clear old embeddings
cargo run --example migrate_embeddings

# 2. Regenerate with new model (requires NS-54 completion)
cargo run --example regenerate_embeddings

# 3. Validate migration
cargo run --example fastembed_integration
```

### Benefits
- **Performance**: ONNX Runtime + Rayon parallelization
- **Quality**: BAAI/bge-small-en-v1.5 ranks higher on MTEB leaderboard  
- **Architecture**: Unified stack with text generation
- **Cross-platform**: Better Windows/macOS support

See [MIGRATION.md](./MIGRATION.md) for complete migration guide.

---

**Project Management:** All development tasks tracked in [Linear workspace](https://linear.app/nodespace)