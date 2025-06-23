# ⚠️ BEFORE STARTING ANY WORK
👉 **STEP 1**: Read development workflow: `../nodespace-system-design/docs/development-workflow.md`
👉 **STEP 2**: Check Linear for assigned tasks
👉 **STEP 3**: Repository-specific patterns below

**This README.md only contains**: Repository-specific SurrealDB and database patterns

# NodeSpace Data Store

**Database, persistence, and vector storage for NodeSpace**

This repository implements the complete data layer for NodeSpace, providing persistent storage for nodes, embeddings, and search capabilities. It serves as the **single source of persistence** across the distributed system.

## 🎯 Purpose

- **Node storage** - Persistent storage for text content, metadata, and relationships
- **Vector database** - Embedding storage for semantic search capabilities
- **Search engine** - Fast full-text and semantic search implementation
- **Data integrity** - ACID transactions and data validation

## 📦 Key Features

- **SurrealDB backend** - Multi-model database (Document + Graph + Vector)
- **Vector search** - Native vector<float, DIM> embedding similarity search
- **Graph relationships** - Native RELATE statements for entity connections
- **Dynamic schemas** - Schema evolution with "ghost properties"
- **SurrealQL support** - LLM-friendly query language

## 🔗 Dependencies

- **`nodespace-core-types`** - Shared data structures (NodeId, Node, NodeSpaceResult)
- **SurrealDB** - Multi-model database engine
- **serde_json** - For flexible content serialization

## 🏗️ Interface Ownership

This repository **owns the `DataStore` trait** as part of NodeSpace's distributed contract architecture:
- **Exports**: `DataStore` trait interface for other services to import
- **Implements**: Complete SurrealDB-based data persistence layer
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
use nodespace_data_store::SurrealDataStore;
use nodespace_core_types::{DataStore, Node};

let store = SurrealDataStore::new("./data/nodes.db").await?;
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

The initial implementation focuses on SurrealDB-native operations:

1. **Store nodes** - CREATE statements with serde_json::Value content
2. **Graph relationships** - RELATE statements between entities
3. **Vector search** - Native vector<float, DIM> similarity operations
4. **SurrealQL queries** - Custom query execution
5. **Node retrieval** - SELECT statements by NodeId

## 🧪 Testing

```bash
# Run all tests including integration tests
cargo test

# Test with sample data
cargo run --example create_sample_data

# Benchmark search performance
cargo bench
```

---

**Project Management:** All development tasks tracked in [Linear workspace](https://linear.app/nodespace)