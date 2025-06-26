# NodeSpace Sample Datasets

This directory contains comprehensive sample datasets for testing, validation, and demonstration of NodeSpace functionality.

## 🗃️ Available Datasets

### `create_comprehensive_sample_datasets.rs`

**Purpose:** Complete sample dataset generation for LanceDB migration validation, RAG functionality testing, and performance benchmarking using SurrealDB.

### `create_lancedb_sample_datasets.rs` ⭐ **Recommended**

**Purpose:** Modern LanceDB-based sample dataset generator with native vector search capabilities and Arrow/Parquet storage format.

**Content Coverage:**
- **Business Strategy** (2025-06-15): Marketing campaigns, budgets, KPIs, audience analysis
- **Technical Documentation** (2025-06-16): API docs, schemas, authentication, endpoints
- **Project Planning** (2025-06-17): Sprint planning, resource allocation, risk management
- **Research & Knowledge** (2025-06-18): AI research, vector database analysis, performance benchmarks
- **Meeting & Collaboration** (2025-06-19): Team standups, updates, action items

**SurrealDB Statistics:**
- 47 total nodes (42 text nodes + 5 date nodes)
- 5 distinct content domains
- Hierarchical depth up to 4 levels
- Rich markdown formatting (headings, tables, code blocks, lists)
- Realistic business content suitable for semantic search

**LanceDB Statistics:**
- 15 total records across 5 tables
- Universal document model with 384-dimensional embeddings
- Native vector search (no external indexing required)
- Arrow/Parquet storage format for optimal performance
- Ready for semantic similarity queries and RAG testing

### Usage

```bash
# Generate comprehensive sample datasets
cargo run --example create_comprehensive_sample_datasets  # SurrealDB-based
cargo run --example create_lancedb_sample_datasets        # LanceDB-based (recommended)

# Other available examples
cargo run --example create_sample_data              # Basic sample data
cargo run --example fastembed_integration          # FastEmbed testing
cargo run --example migrate_embeddings             # Migration tools
cargo run --example regenerate_embeddings          # Embedding regeneration
cargo run --example validate_search_quality        # Search quality validation
```

## 📊 Dataset Structure

### Hierarchical Organization

```
DateNode: "2025-06-15"
├── TextNode: "# Product Launch Campaign Strategy"
│   ├── TextNode: "## Executive Summary"
│   ├── TextNode: "## Target Audience Analysis"
│   ├── TextNode: "## Marketing Channel Strategy"
│   └── TextNode: "## Success Metrics and KPIs"
```

### Content Formatting Rules

**✅ Allowed within node content:**
- Headings: `# ## ### #### ##### ######`
- Emphasis: `**bold** *italic* ~~strikethrough~~`
- Ordered lists: `1. First item\n2. Second item`
- Links: `[text](url)`
- Code: `` `inline code` `` and ``` code blocks ```
- Tables: `| Column 1 | Column 2 |`
- Blockquotes: `> quoted text`

**❌ Creates separate nodes:**
- Unordered lists: `- item` (becomes child node)
- Bullet points: `* item` (becomes child node)

## 🎯 Use Cases

### LanceDB Migration Validation
- Test data migration accuracy from SurrealDB to LanceDB
- Validate hierarchical relationships preservation
- Ensure content integrity during migration

### RAG Functionality Testing
- Semantic search quality assessment
- Context retrieval accuracy
- Response relevance validation
- Source attribution testing

### Performance Benchmarking
- Query response time measurement
- Embedding similarity search performance
- Database scaling behavior
- Memory usage optimization

### AIChatNode Demonstration
- Realistic knowledge base for AI interactions
- Complex query scenarios
- Multi-domain expertise simulation
- Professional content examples

## 🔧 Technical Details

### Database Schema
- **Date nodes:** Organized by date strings (e.g., "2025-06-15")
- **Text nodes:** Rich content with markdown formatting
- **Relationships:** Parent-child hierarchies via "contains" relationships
- **Metadata:** Creation timestamps, content types, structural data

### Content Characteristics
- **Variety:** 5 distinct professional domains
- **Depth:** Multi-level hierarchical structure
- **Realism:** Based on actual business scenarios
- **Searchability:** Keyword-rich content for testing
- **Complexity:** Nested information suitable for advanced queries

## 📝 Development Notes

### For Contributors
- Each dash (`-`) in source content becomes a separate Node
- Indentation depth determines parent-child relationships
- Rich markdown content stays within individual nodes
- Sample data should be realistic and professionally relevant

### Testing Guidelines
- Use diverse query types (keyword, semantic, structural)
- Test both shallow and deep hierarchical retrieval
- Validate cross-domain search capabilities
- Measure performance with realistic data volumes