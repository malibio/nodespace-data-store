# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This repository implements the **NodeSpace Data Store** - the complete data persistence layer for the NodeSpace distributed system. It serves as the single source of truth for nodes, embeddings, and search capabilities using LanceDB as the backend.

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

- **Trait ownership**: This repository owns and exports the `DataStore` trait for other services to import
- **Core dependency**: `nodespace-core-types` for shared `Node` structures and `NodeSpaceResult` types
- **Database**: LanceDB vector database with Universal Document Schema  
- **Primary trait**: Implements and exports `DataStore` trait from this repository
- **Architecture**: Entity-centric storage with embedded vector search

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

1. **Node Storage** - Universal Document Schema with flexible serde_json::Value content
2. **Vector Search** - Native LanceDB vector similarity search with 384/512-dim embeddings
3. **Hierarchical Relationships** - JSON-based parent-child connections for entity organization
4. **Cross-Modal Search** - Support for text and image embeddings with hybrid scoring
5. **Node Retrieval** - Fast lookup by NodeId with metadata filtering

## Important Context

- NS-81 cross-modal search implementation complete with comprehensive testing
- Focus on LanceDB Universal Document Schema for multimodal data
- Performance targets: <2s search, <5s image processing 
- Schema flexibility with metadata "ghost properties"
- Task tracking in Linear workspace (filter by `nodespace-data-store`)

## External Resources

- [NodeSpace System Design](../nodespace-system-design/README.md) - Full architecture
- [Linear workspace](https://linear.app/nodespace) - Task management
- [Development Workflow](../nodespace-system-design/docs/development/workflow.md)
- [MVP User Flow](../nodespace-system-design/examples/mvp-user-flow.md)