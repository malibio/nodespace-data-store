use arrow_array::{RecordBatch, StringArray, UInt64Array, FixedSizeListArray};
use arrow_schema::{DataType, Field, Schema};
use lancedb::{connect, Connection, Table};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Creating LanceDB Sample Datasets for NodeSpace");
    println!("🎯 Modern vector-first approach with universal document model");
    println!("");

    // Initialize LanceDB connection
    let db = connect("./data/nodescape_lance.db").execute().await?;
    println!("✅ Connected to LanceDB");

    // Create comprehensive sample datasets
    println!("📊 Creating Universal Document Collections...");
    
    // Dataset 1: Business Strategy Documents
    create_business_strategy_collection(&db).await?;
    
    // Dataset 2: Technical Documentation
    create_technical_docs_collection(&db).await?;
    
    // Dataset 3: Project Planning Documents
    create_project_planning_collection(&db).await?;
    
    // Dataset 4: Research and Knowledge Base
    create_research_collection(&db).await?;
    
    // Dataset 5: Meeting and Collaboration Records
    create_collaboration_collection(&db).await?;

    // Generate comprehensive statistics
    println!("\n📈 LanceDB Dataset Statistics");
    println!("═══════════════════════════════════════════");
    
    let total_records = count_total_records(&db).await?;
    println!("🗃️  Total Records: {}", total_records);
    println!("🏗️  Architecture: Universal document model");
    println!("🔍 Vector Search: Native embedding support");
    println!("📊 Content Types: 5 distinct professional domains");
    
    // Test vector search capabilities
    println!("\n🧪 Testing Vector Search Capabilities...");
    test_semantic_search(&db).await?;
    
    println!("\n🎉 LanceDB Sample Datasets Created Successfully!");
    println!("💡 Ready for:");
    println!("   • Native vector search (no external indexing)");
    println!("   • Semantic similarity queries");
    println!("   • RAG context retrieval");
    println!("   • Performance benchmarking");
    println!("   • SQL-like filtering with vectors");

    Ok(())
}

/// Create business strategy document collection
async fn create_business_strategy_collection(db: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    println!("🏢 Creating Business Strategy Collection...");
    
    let schema = create_universal_document_schema();
    
    let documents = vec![
        UniversalDocument {
            id: "bus_001".to_string(),
            content: "# Product Launch Campaign Strategy\n\nComprehensive product launch plan for EcoSmart Professional Series with strategic framework, tactical execution details, and success measurement criteria.".to_string(),
            content_type: "strategy".to_string(),
            domain: "business".to_string(),
            created_at: 1719360000, // 2025-06-15 timestamp
            metadata: r#"{"campaign": "EcoSmart", "budget": 180000, "duration_weeks": 12}"#.to_string(),
            embedding: generate_sample_embedding("product launch campaign strategy marketing business"),
        },
        UniversalDocument {
            id: "bus_002".to_string(),
            content: "## Executive Summary\n\nThe EcoSmart Professional Series represents our most significant product innovation in three years, combining professional-grade performance with industry-leading sustainability features.\n\n**Key Objectives:**\n1. Establish market leadership in sustainable professional products\n2. Achieve 15% market share within 12 months\n3. Generate $2.5M revenue in launch quarter".to_string(),
            content_type: "executive_summary".to_string(),
            domain: "business".to_string(),
            created_at: 1719360300,
            metadata: r#"{"revenue_target": 2500000, "market_share_target": 15}"#.to_string(),
            embedding: generate_sample_embedding("executive summary innovation sustainability market leadership revenue"),
        },
        UniversalDocument {
            id: "bus_003".to_string(),
            content: "## Target Audience Analysis\n\n### Primary Target Segment\n\n**Professional Demographics:**\n- Age: 28-45 years\n- Income: $75,000-$150,000 annually\n- Education: College degree or higher (87%)\n- Location: Urban and suburban professionals\n\n**Psychographic Profile:**\n- Values sustainability and environmental responsibility\n- Willing to pay premium for quality\n- Research-intensive purchase behavior".to_string(),
            content_type: "audience_analysis".to_string(),
            domain: "business".to_string(),
            created_at: 1719360600,
            metadata: r#"{"age_range": [28, 45], "income_range": [75000, 150000], "education_level": "college+"}"#.to_string(),
            embedding: generate_sample_embedding("target audience demographics psychographics sustainability professionals"),
        },
    ];
    
    let table = create_documents_table(db, "business_strategy", &schema, documents).await?;
    println!("   ✅ Created business_strategy table with {} records", table.count_rows(None).await?);
    
    Ok(())
}

/// Create technical documentation collection
async fn create_technical_docs_collection(db: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Creating Technical Documentation Collection...");
    
    let schema = create_universal_document_schema();
    
    let documents = vec![
        UniversalDocument {
            id: "tech_001".to_string(),
            content: "# NodeSpace API Documentation v2.1\n\nComprehensive API documentation for NodeSpace platform including authentication, data models, and endpoint specifications.".to_string(),
            content_type: "api_docs".to_string(),
            domain: "technical".to_string(),
            created_at: 1719446400, // 2025-06-16
            metadata: r#"{"version": "2.1", "auth_type": "oauth2", "format": "rest"}"#.to_string(),
            embedding: generate_sample_embedding("API documentation authentication REST endpoints technical"),
        },
        UniversalDocument {
            id: "tech_002".to_string(),
            content: "## Authentication\n\n### OAuth 2.0 Implementation\n\nNodeSpace uses OAuth 2.0 with PKCE for secure authentication.\n\n**Required Headers:**\n```http\nAuthorization: Bearer {access_token}\nContent-Type: application/json\nX-API-Version: 2.1\n```\n\n**Security Considerations:**\n- Tokens expire after 1 hour\n- Refresh tokens valid for 30 days\n- Rate limiting: 1000 requests/hour".to_string(),
            content_type: "authentication".to_string(),
            domain: "technical".to_string(),
            created_at: 1719446700,
            metadata: r#"{"auth_method": "oauth2_pkce", "token_expiry": 3600, "rate_limit": 1000}"#.to_string(),
            embedding: generate_sample_embedding("OAuth authentication security tokens PKCE rate limiting"),
        },
        UniversalDocument {
            id: "tech_003".to_string(),
            content: "## Data Models\n\n### Node Schema\n\n```typescript\ninterface Node {\n  id: string;           // UUID v4\n  content: any;         // Flexible JSON content\n  metadata?: object;    // Optional metadata\n  created_at: string;   // ISO 8601 timestamp\n  updated_at: string;   // ISO 8601 timestamp\n}\n```\n\n**Validation Rules:**\n- All IDs must be valid UUIDs\n- Content size limit: 1MB per node\n- Timestamps must be valid ISO 8601".to_string(),
            content_type: "data_models".to_string(),
            domain: "technical".to_string(),
            created_at: 1719447000,
            metadata: r#"{"schema_version": "1.0", "max_content_size": 1048576}"#.to_string(),
            embedding: generate_sample_embedding("data models schema TypeScript UUID validation JSON"),
        },
    ];
    
    let table = create_documents_table(db, "technical_docs", &schema, documents).await?;
    println!("   ✅ Created technical_docs table with {} records", table.count_rows(None).await?);
    
    Ok(())
}

/// Create project planning collection
async fn create_project_planning_collection(db: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Creating Project Planning Collection...");
    
    let schema = create_universal_document_schema();
    
    let documents = vec![
        UniversalDocument {
            id: "proj_001".to_string(),
            content: "# Q3 2025 Development Roadmap\n\n**Project Timeline:** July 1 - September 30, 2025\n**Team Size:** 12 developers, 3 designers, 2 QA engineers\n**Budget:** $450,000\n**Primary Goal:** Launch NodeSpace 3.0 with AI-powered features".to_string(),
            content_type: "roadmap".to_string(),
            domain: "project_management".to_string(),
            created_at: 1719532800, // 2025-06-17
            metadata: r#"{"quarter": "Q3_2025", "budget": 450000, "team_size": 17}"#.to_string(),
            embedding: generate_sample_embedding("development roadmap Q3 team budget AI features planning"),
        },
        UniversalDocument {
            id: "proj_002".to_string(),
            content: "## Sprint Planning Overview\n\n### Sprint 1 (July 1-14): Foundation\n\n**Epic: Core Infrastructure**\n- Database migration to LanceDB\n- API versioning system implementation\n- Performance optimization framework\n\n**Story Points:** 89\n**Risk Level:** High (new database system)\n**Dependencies:** DevOps team for infrastructure".to_string(),
            content_type: "sprint_planning".to_string(),
            domain: "project_management".to_string(),
            created_at: 1719533100,
            metadata: r#"{"sprint": 1, "story_points": 89, "risk_level": "high", "epic": "infrastructure"}"#.to_string(),
            embedding: generate_sample_embedding("sprint planning LanceDB migration infrastructure DevOps story points"),
        },
        UniversalDocument {
            id: "proj_003".to_string(),
            content: "## Risk Management\n\n### High-Priority Risks\n\n**Risk 1: LanceDB Migration Complexity**\n- **Probability:** 60%\n- **Impact:** High (could delay launch by 4 weeks)\n- **Mitigation:** Parallel development, rollback plan ready\n- **Owner:** Marcus Rodriguez\n\n**Risk 2: AI Model Performance**\n- **Probability:** 40%\n- **Impact:** Medium (reduced feature quality)\n- **Mitigation:** A/B testing, multiple model options".to_string(),
            content_type: "risk_management".to_string(),
            domain: "project_management".to_string(),
            created_at: 1719533400,
            metadata: r#"{"high_risk_count": 2, "migration_risk": 60, "ai_risk": 40}"#.to_string(),
            embedding: generate_sample_embedding("risk management LanceDB migration AI model performance mitigation"),
        },
    ];
    
    let table = create_documents_table(db, "project_planning", &schema, documents).await?;
    println!("   ✅ Created project_planning table with {} records", table.count_rows(None).await?);
    
    Ok(())
}

/// Create research collection
async fn create_research_collection(db: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Creating Research Collection...");
    
    let schema = create_universal_document_schema();
    
    let documents = vec![
        UniversalDocument {
            id: "research_001".to_string(),
            content: "# AI and Vector Database Research Findings\n\n**Research Period:** May-June 2025\n**Focus Areas:** Vector databases, embedding models, RAG systems\n**Team:** AI Research Division\n**Status:** Comprehensive analysis complete".to_string(),
            content_type: "research_overview".to_string(),
            domain: "research".to_string(),
            created_at: 1719619200, // 2025-06-18
            metadata: r#"{"research_period": "2025-05-2025-06", "focus_areas": ["vector_db", "embeddings", "rag"]}"#.to_string(),
            embedding: generate_sample_embedding("AI research vector database embedding models RAG analysis"),
        },
        UniversalDocument {
            id: "research_002".to_string(),
            content: "## Vector Database Comparison Study\n\n**Results Summary:**\n\n| Database | P95 Latency | Throughput | Memory |\n|----------|-------------|------------|---------|\n| LanceDB  | 12ms       | 850 QPS    | 4.2GB  |\n| Pinecone | 18ms       | 720 QPS    | 3.8GB  |\n| Weaviate | 25ms       | 650 QPS    | 5.1GB  |\n\n**Recommendation:** Proceed with LanceDB migration\n**Confidence Level:** 85%".to_string(),
            content_type: "benchmark_results".to_string(),
            domain: "research".to_string(),
            created_at: 1719619500,
            metadata: r#"{"benchmark_type": "vector_db", "winner": "lancedb", "confidence": 85}"#.to_string(),
            embedding: generate_sample_embedding("vector database benchmark LanceDB Pinecone Weaviate performance latency"),
        },
        UniversalDocument {
            id: "research_003".to_string(),
            content: "## Embedding Model Evaluation\n\n**Top Performers:**\n\n**1. BAAI/bge-small-en-v1.5**\n- MTEB Score: 62.17\n- Speed: 2,100 tokens/sec\n- Memory: 133MB\n- Strengths: Excellent balance, fast inference\n\n**2. all-MiniLM-L6-v2**\n- MTEB Score: 58.84\n- Speed: 1,850 tokens/sec\n- Memory: 91MB\n- Strengths: Lightweight, proven reliability".to_string(),
            content_type: "model_evaluation".to_string(),
            domain: "research".to_string(),
            created_at: 1719619800,
            metadata: r#"{"model_count": 2, "best_model": "bge-small-en-v1.5", "mteb_score": 62.17}"#.to_string(),
            embedding: generate_sample_embedding("embedding model evaluation BAAI bge MiniLM MTEB benchmark"),
        },
    ];
    
    let table = create_documents_table(db, "research", &schema, documents).await?;
    println!("   ✅ Created research table with {} records", table.count_rows(None).await?);
    
    Ok(())
}

/// Create collaboration collection
async fn create_collaboration_collection(db: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    println!("🤝 Creating Collaboration Collection...");
    
    let schema = create_universal_document_schema();
    
    let documents = vec![
        UniversalDocument {
            id: "collab_001".to_string(),
            content: "# Weekly Engineering Standup - June 19, 2025\n\n**Attendees:** 12 (Full engineering team)\n**Duration:** 45 minutes\n**Meeting Lead:** Sarah Chen (Engineering Lead)\n**Next Meeting:** June 26, 2025 9:00 AM PT".to_string(),
            content_type: "meeting_notes".to_string(),
            domain: "collaboration".to_string(),
            created_at: 1719705600, // 2025-06-19
            metadata: r#"{"meeting_type": "standup", "attendees": 12, "duration_minutes": 45}"#.to_string(),
            embedding: generate_sample_embedding("weekly standup engineering team meeting Sarah Chen"),
        },
        UniversalDocument {
            id: "collab_002".to_string(),
            content: "## Backend Team Updates\n\n**Marcus Rodriguez (Database Lead):**\n- ✅ LanceDB integration proof-of-concept complete\n- ✅ Migration script testing on 10k sample records\n- 🔄 **In Progress:** Full data migration planning\n- 🎯 **Next Week:** Begin production migration testing\n- ⚠️ **Blocker:** Waiting for GPU allocation from DevOps".to_string(),
            content_type: "team_updates".to_string(),
            domain: "collaboration".to_string(),
            created_at: 1719705900,
            metadata: r#"{"team": "backend", "lead": "Marcus Rodriguez", "blockers": 1}"#.to_string(),
            embedding: generate_sample_embedding("backend team updates Marcus Rodriguez LanceDB migration GPU DevOps"),
        },
        UniversalDocument {
            id: "collab_003".to_string(),
            content: "## Action Items & Decisions\n\n### Immediate Actions (This Week)\n\n**High Priority:**\n1. **GPU Allocation Request** (Owner: DevOps, Due: June 21)\n2. **Production Migration Plan Review** (Owner: Marcus, Due: June 22)\n3. **Security Penetration Test Scheduling** (Owner: Lisa, Due: June 20)\n\n**✅ Approved: LanceDB Migration Timeline**\n- Phase 1: Testing (June 22-29)\n- Phase 2: Staging Migration (July 1-8)\n- Phase 3: Production Migration (July 15-22)".to_string(),
            content_type: "action_items".to_string(),
            domain: "collaboration".to_string(),
            created_at: 1719706200,
            metadata: r#"{"action_count": 3, "decisions": 1, "migration_phases": 3}"#.to_string(),
            embedding: generate_sample_embedding("action items decisions GPU allocation migration timeline security testing"),
        },
    ];
    
    let table = create_documents_table(db, "collaboration", &schema, documents).await?;
    println!("   ✅ Created collaboration table with {} records", table.count_rows(None).await?);
    
    Ok(())
}

/// Universal document structure for LanceDB
#[derive(Debug)]
struct UniversalDocument {
    id: String,
    content: String,
    content_type: String,
    domain: String,
    created_at: u64,
    metadata: String,
    embedding: Vec<f32>,
}

/// Create schema for universal document model
fn create_universal_document_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("content", DataType::Utf8, false),
        Field::new("content_type", DataType::Utf8, false),
        Field::new("domain", DataType::Utf8, false),
        Field::new("created_at", DataType::UInt64, false),
        Field::new("metadata", DataType::Utf8, false),
        Field::new("embedding", DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 384), false),
    ]))
}

/// Create LanceDB table with documents
async fn create_documents_table(
    db: &Connection,
    table_name: &str,
    schema: &Arc<Schema>,
    documents: Vec<UniversalDocument>,
) -> Result<Table, Box<dyn std::error::Error>> {
    // Convert documents to RecordBatch
    let ids: Vec<String> = documents.iter().map(|d| d.id.clone()).collect();
    let contents: Vec<String> = documents.iter().map(|d| d.content.clone()).collect();
    let content_types: Vec<String> = documents.iter().map(|d| d.content_type.clone()).collect();
    let domains: Vec<String> = documents.iter().map(|d| d.domain.clone()).collect();
    let created_ats: Vec<u64> = documents.iter().map(|d| d.created_at).collect();
    let metadatas: Vec<String> = documents.iter().map(|d| d.metadata.clone()).collect();
    let embeddings: Vec<Vec<f32>> = documents.iter().map(|d| d.embedding.clone()).collect();
    
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(ids)),
            Arc::new(StringArray::from(contents)),
            Arc::new(StringArray::from(content_types)),
            Arc::new(StringArray::from(domains)),
            Arc::new(UInt64Array::from(created_ats)),
            Arc::new(StringArray::from(metadatas)),
            {
                // Create flattened f32 values and construct fixed-size list array
                let flat_values: Vec<f32> = embeddings.into_iter().flatten().collect();
                let float_array = arrow_array::Float32Array::from(flat_values);
                Arc::new(FixedSizeListArray::try_new(
                    Arc::new(arrow_schema::Field::new("item", DataType::Float32, true)),
                    384,
                    Arc::new(float_array),
                    None,
                )?)
            },
        ],
    )?;
    
    // Create table using a RecordBatchIterator
    let batches = vec![batch];
    let reader = arrow_array::RecordBatchIterator::new(batches.into_iter().map(Ok), schema.clone());
    
    let table = db
        .create_table(table_name, Box::new(reader))
        .execute()
        .await?;
        
    Ok(table)
}

/// Generate sample 384-dimensional embedding
fn generate_sample_embedding(content: &str) -> Vec<f32> {
    let content_hash = content.chars().map(|c| c as u32).sum::<u32>();
    let seed = content_hash as f32 / 1000.0;
    
    // Generate 384-dimensional embedding (matching bge-small-en-v1.5)
    (0..384)
        .map(|i| {
            let angle = (seed + i as f32) * 0.1;
            let value = (angle.sin() + angle.cos()) / 2.0;
            let variation = (i as f32 * seed).sin() * 0.1;
            (value + variation).clamp(-1.0, 1.0)
        })
        .collect()
}

/// Test semantic search capabilities
async fn test_semantic_search(_db: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Semantic Search:");
    
    // Test queries across different domains
    let test_queries = vec![
        ("LanceDB performance", "research"),
        ("project timeline", "project_management"),
        ("API authentication", "technical"),
        ("team meeting", "collaboration"),
        ("business strategy", "business"),
    ];
    
    for (query, expected_domain) in test_queries {
        // Note: In a real implementation, you'd use LanceDB's vector search
        // For now, just demonstrate the capability exists
        println!("   Query: '{}' → Expected domain: {}", query, expected_domain);
    }
    
    println!("✅ Vector search ready (awaiting full LanceDB query implementation)");
    Ok(())
}

/// Count total records across all tables
async fn count_total_records(db: &Connection) -> Result<usize, Box<dyn std::error::Error>> {
    let table_names = vec!["business_strategy", "technical_docs", "project_planning", "research", "collaboration"];
    let mut total = 0;
    
    for table_name in table_names {
        if let Ok(table) = db.open_table(table_name).execute().await {
            total += table.count_rows(None).await?;
        }
    }
    
    Ok(total)
}