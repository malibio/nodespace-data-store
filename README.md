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

- **`nodespace-core-types`** - Data structures and `DataStore` trait interface
- **SurrealDB** - Multi-model database engine
- **serde_json** - For flexible content serialization

## 🚀 Getting Started

### **New to NodeSpace? Start Here:**
1. **Read [NodeSpace System Design](../nodespace-system-design/README.md)** - Understand the full architecture
2. **Check [Linear workspace](https://linear.app/nodespace)** - Find your current tasks (filter by `nodespace-data-store`)
3. **Review [Development Workflow](../nodespace-system-design/docs/development-workflow.md)** - Process and procedures
4. **Study [Key Contracts](../nodespace-system-design/contracts/)** - Interface definitions you'll implement
5. **See [MVP User Flow](../nodespace-system-design/examples/mvp-user-flow.md)** - What you're building

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