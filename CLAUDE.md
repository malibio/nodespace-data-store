# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This repository implements the **NodeSpace Data Store** - the complete data persistence layer for the NodeSpace distributed system. It serves as the single source of truth for nodes, embeddings, and search capabilities using SurrealDB as the backend.

## System Architecture

This is part of the larger NodeSpace system architecture:
1. `nodespace-core-types` - Shared data structures and interfaces
2. **`nodespace-data-store`** ← This repository
3. `nodespace-nlp-engine` - AI/ML processing and LLM integration
4. `nodespace-workflow-engine` - Automation and event processing
5. `nodespace-core-logic` - Business logic orchestration
6. `nodespace-core-ui` - React components and UI
7. `nodespace-desktop-app` - Tauri application shell

## Key Interfaces & Dependencies

- **Core dependency**: `nodespace-core-types` for `DataStore` trait and `Node` structures
- **Database**: SurrealDB multi-model database (Document + Graph + Vector)
- **Primary trait**: Must implement `DataStore` trait from `nodespace-core-types`
- **Query language**: SurrealQL (LLM-friendly)

## Development Commands

```bash
# Run all tests including integration tests
cargo test

# Test with sample data
cargo run --example create_sample_data

# Benchmark search performance
cargo bench
```

## Core Functionality Requirements

1. **Node Storage** - Persistent storage using CREATE statements with serde_json::Value content
2. **Vector Search** - Native vector<float, DIM> embedding similarity search
3. **Graph Relationships** - RELATE statements between entities
4. **SurrealQL Support** - Custom query execution
5. **Node Retrieval** - SELECT statements by NodeId

## Important Context

- Repository is in early development stage - implementation needs to be built
- Focus on SurrealDB-native operations for MVP
- Data integrity with ACID transactions
- Schema evolution support with "ghost properties"
- Task tracking in Linear workspace (filter by `nodespace-data-store`)

## External Resources

- [NodeSpace System Design](../nodespace-system-design/README.md) - Full architecture
- [Linear workspace](https://linear.app/nodespace) - Task management
- [Development Workflow](../nodespace-system-design/docs/development/workflow.md)
- [MVP User Flow](../nodespace-system-design/examples/mvp-user-flow.md)