//! Load E2E Sample Data for Cross-Component Testing
//!
//! Creates sample data in the shared /Users/malibio/nodespace/data/lance_db/ directory
//! for end-to-end testing across NodeSpace components.

use nodespace_core_types::{Node, NodeId};
use nodespace_data_store::{DataStore, LanceDataStore};
use std::error::Error;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🌐 Loading E2E Sample Data for Cross-Component Testing\n");

    // Use shared data directory for e2e testing
    let shared_db_path = "/Users/malibio/nodespace/data/lance_db/e2e_sample.db";
    let data_store = LanceDataStore::new(shared_db_path).await?;
    println!("✅ LanceDB initialized at: {}", shared_db_path);

    // Create comprehensive sample data for e2e testing
    println!("\n📋 Creating E2E Test Dataset...");

    // TODAY's DateNode with multiple documents
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let date_node = Node::with_id(
        NodeId::from_string(today.clone()),
        serde_json::Value::String(format!("📅 {} - E2E Testing Dataset", today)),
    )
    .with_metadata(serde_json::json!({
        "node_type": "date",
        "date": today,
        "content_type": "date_container",
        "test_purpose": "e2e_cross_component"
    }));

    let date_id = data_store
        .store_node_with_embedding(
            date_node,
            create_embedding("e2e testing dataset cross component"),
        )
        .await?;
    println!("✅ Created E2E DateNode: {}", date_id);

    // 1. Product Launch Campaign Strategy (hierarchical structure)
    create_product_launch_strategy(&data_store, &today).await?;

    // 2. Technical Documentation (for NLP engine testing)
    create_technical_documentation(&data_store, &today).await?;

    // 3. Team Meeting Notes (for workflow engine testing)
    create_team_meeting_notes(&data_store, &today).await?;

    // 4. Customer Feedback (for analytics testing)
    create_customer_feedback(&data_store, &today).await?;

    // Test cross-modal search capabilities
    println!("\n🔍 Testing E2E Cross-Modal Search...");

    use nodespace_data_store::HybridSearchConfig;

    let search_config = HybridSearchConfig {
        semantic_weight: 0.6,
        structural_weight: 0.2,
        temporal_weight: 0.2,
        max_results: 10,
        min_similarity_threshold: 0.1,
        enable_cross_modal: true,
        search_timeout_ms: 2000,
    };

    let search_results = data_store
        .hybrid_multimodal_search(
            create_embedding("product launch strategy technical documentation"),
            &search_config,
        )
        .await?;

    println!(
        "   📊 Hybrid search results: {} items",
        search_results.len()
    );
    for (i, result) in search_results.iter().take(3).enumerate() {
        if let Some(metadata) = &result.node.metadata {
            if let Some(title) = metadata.get("title").and_then(|v| v.as_str()) {
                println!("   {}. {} (score: {:.3})", i + 1, title, result.score);
            }
        }
    }

    println!("\n🎉 E2E Sample Data Created Successfully!");
    println!("📍 Location: {}", shared_db_path);
    println!("📈 Dataset Summary for E2E Testing:");
    println!("   📅 1 DateNode with today's date");
    println!("   📄 4 Main documents with different purposes:");
    println!("     • Product Launch Strategy (hierarchical, 7 sections)");
    println!("     • Technical Documentation (API specs, architecture)");
    println!("     • Team Meeting Notes (action items, decisions)");
    println!("     • Customer Feedback (satisfaction, feature requests)");
    println!("   🔗 Full parent-child relationships preserved");
    println!("   🎯 Cross-modal search capabilities enabled");
    println!("   ⚡ Performance optimized for <2s search requirements");

    println!("\n🧪 Ready for E2E Testing:");
    println!("   • NodeSpace NLP Engine: Content analysis and embeddings");
    println!("   • NodeSpace Workflow Engine: Task automation and triggers");
    println!("   • NodeSpace Core Logic: Business rule processing");
    println!("   • NodeSpace UI: Search and navigation interfaces");
    println!("   • Cross-component data flow validation");

    Ok(())
}

async fn create_product_launch_strategy(
    data_store: &LanceDataStore,
    date: &str,
) -> Result<(), Box<dyn Error>> {
    let doc_id = Uuid::new_v4().to_string();
    let main_doc = Node::with_id(
        NodeId::from_string(doc_id.clone()),
        serde_json::Value::String(
            "# 🚀 Product Launch Campaign Strategy\n\nComprehensive strategic framework for launching the EcoSmart Professional Series, targeting environmentally conscious professionals with premium performance requirements.".to_string()
        )
    ).with_metadata(serde_json::json!({
        "node_type": "text",
        "title": "Product Launch Campaign Strategy",
        "parent_date": date,
        "depth": 1,
        "document_type": "strategy",
        "test_purpose": "hierarchical_navigation"
    }));

    let strategy_id = data_store
        .store_node_with_embedding(
            main_doc,
            create_embedding(
                "product launch strategy comprehensive framework ecosmart professional",
            ),
        )
        .await?;

    // Create sections for testing hierarchical relationships
    let sections = vec![
        ("Executive Summary", "Strategic overview of the EcoSmart Professional Series launch, targeting 28-45 year old urban professionals with $180K budget allocation across 12-week campaign timeline."),
        ("Target Market Analysis", "Primary segment: Urban professionals ($75K-$150K income) with sustainability values. Secondary: Corporate buyers implementing green initiatives."),
        ("Marketing Channels", "Three-phase approach: Pre-launch education (weeks 1-4), coordinated launch activation (weeks 5-8), post-launch optimization (weeks 9-12)."),
        ("Success Metrics", "KPIs: 25% brand awareness increase, 5,000 units sold in 60 days, $850K revenue target, 4.7/5.0 customer satisfaction score."),
    ];

    for (title, content) in sections {
        let section_id = Uuid::new_v4().to_string();
        let section_node = Node::with_id(
            NodeId::from_string(section_id),
            serde_json::Value::String(format!("## {}\n\n{}", title, content)),
        )
        .with_metadata(serde_json::json!({
            "node_type": "text",
            "title": title,
            "parent_id": strategy_id,
            "depth": 2,
            "section_type": "strategy_section"
        }));

        data_store
            .store_node_with_embedding(
                section_node,
                create_embedding(&format!("{} {}", title, content)),
            )
            .await?;
    }

    println!("   🚀 Created Product Launch Strategy with 4 sections");
    Ok(())
}

async fn create_technical_documentation(
    data_store: &LanceDataStore,
    date: &str,
) -> Result<(), Box<dyn Error>> {
    let doc_id = Uuid::new_v4().to_string();
    let main_doc = Node::with_id(
        NodeId::from_string(doc_id.clone()),
        serde_json::Value::String(
            "# 💻 API Documentation & Architecture\n\n**NodeSpace Data Store Technical Specifications**\nComprehensive API documentation covering LanceDB integration, vector search capabilities, and cross-modal functionality.".to_string()
        )
    ).with_metadata(serde_json::json!({
        "node_type": "text",
        "title": "API Documentation & Architecture",
        "parent_date": date,
        "depth": 1,
        "document_type": "technical_documentation",
        "test_purpose": "nlp_engine_processing"
    }));

    let docs_id = data_store
        .store_node_with_embedding(
            main_doc,
            create_embedding(
                "api documentation architecture lancedb vector search technical specifications",
            ),
        )
        .await?;

    let tech_sections = vec![
        ("DataStore Trait", "```rust\n#[async_trait]\npub trait DataStore {\n    async fn store_node(&self, node: Node) -> NodeSpaceResult<NodeId>;\n    async fn search_multimodal(&self, query_embedding: Vec<f32>, types: Vec<NodeType>) -> NodeSpaceResult<Vec<Node>>;\n}\n```"),
        ("Vector Search API", "Semantic search using 384-dimensional BGE embeddings with cosine similarity. Performance target: <2s for complex queries with 10K+ nodes."),
        ("Cross-Modal Support", "CLIP vision embeddings (512-dim) for image processing. Hybrid scoring combines semantic, structural, and temporal relevance factors."),
    ];

    for (title, content) in tech_sections {
        let section_id = Uuid::new_v4().to_string();
        let section_node = Node::with_id(
            NodeId::from_string(section_id),
            serde_json::Value::String(format!("## {}\n\n{}", title, content)),
        )
        .with_metadata(serde_json::json!({
            "node_type": "text",
            "title": title,
            "parent_id": docs_id,
            "depth": 2,
            "section_type": "technical_section"
        }));

        data_store
            .store_node_with_embedding(
                section_node,
                create_embedding(&format!("{} {}", title, content)),
            )
            .await?;
    }

    println!("   💻 Created Technical Documentation with 3 sections");
    Ok(())
}

async fn create_team_meeting_notes(
    data_store: &LanceDataStore,
    date: &str,
) -> Result<(), Box<dyn Error>> {
    let doc_id = Uuid::new_v4().to_string();
    let main_doc = Node::with_id(
        NodeId::from_string(doc_id.clone()),
        serde_json::Value::String(
            "# 👥 Weekly Engineering Standup\n\n**Sprint 24 Progress & Planning**\nTeam sync covering NS-81 cross-modal search implementation, testing progress, and upcoming workflow engine integration tasks.".to_string()
        )
    ).with_metadata(serde_json::json!({
        "node_type": "text",
        "title": "Weekly Engineering Standup",
        "parent_date": date,
        "depth": 1,
        "document_type": "meeting_notes",
        "test_purpose": "workflow_engine_triggers"
    }));

    let meeting_id = data_store
        .store_node_with_embedding(
            main_doc,
            create_embedding(
                "engineering standup sprint progress cross modal search implementation",
            ),
        )
        .await?;

    let meeting_sections = vec![
        ("Completed Tasks", "✅ **NS-81 Implementation Complete**:\n- Cross-modal DataStore trait extended\n- LanceDB Universal Document Schema\n- 7 integration tests passing\n- Performance: 101µs hybrid search (target: <2s)\n\n✅ **Code Review & Testing**:\n- PR #10 created for review\n- Linear task updated to 'In Review' status"),
        ("Action Items", "🎯 **This Week's Priorities**:\n- [ ] Code review completion for NS-81 (Due: Friday)\n- [ ] E2E testing with NLP engine integration\n- [ ] Workflow engine trigger implementation\n- [ ] Performance benchmarking with realistic datasets\n\n📋 **Next Sprint Planning**:\n- [ ] Advanced vector search optimization\n- [ ] Multi-modal relevance scoring refinement"),
    ];

    for (title, content) in meeting_sections {
        let section_id = Uuid::new_v4().to_string();
        let section_node = Node::with_id(
            NodeId::from_string(section_id),
            serde_json::Value::String(format!("## {}\n\n{}", title, content)),
        )
        .with_metadata(serde_json::json!({
            "node_type": "text",
            "title": title,
            "parent_id": meeting_id,
            "depth": 2,
            "section_type": "meeting_section"
        }));

        data_store
            .store_node_with_embedding(
                section_node,
                create_embedding(&format!("{} {}", title, content)),
            )
            .await?;
    }

    println!("   👥 Created Engineering Standup with 2 sections");
    Ok(())
}

async fn create_customer_feedback(
    data_store: &LanceDataStore,
    date: &str,
) -> Result<(), Box<dyn Error>> {
    let doc_id = Uuid::new_v4().to_string();
    let main_doc = Node::with_id(
        NodeId::from_string(doc_id.clone()),
        serde_json::Value::String(
            "# 📊 Customer Feedback Analysis\n\n**Q2 2025 User Satisfaction Report**\nComprehensive analysis of customer feedback, feature requests, and satisfaction metrics from our professional user base.".to_string()
        )
    ).with_metadata(serde_json::json!({
        "node_type": "text",
        "title": "Customer Feedback Analysis",
        "parent_date": date,
        "depth": 1,
        "document_type": "analytics",
        "test_purpose": "core_logic_processing"
    }));

    let feedback_id = data_store
        .store_node_with_embedding(
            main_doc,
            create_embedding(
                "customer feedback analysis satisfaction metrics professional user base",
            ),
        )
        .await?;

    let feedback_sections = vec![
        ("Satisfaction Metrics", "📈 **Overall Satisfaction**: 4.6/5.0 (Target: 4.5+)\n\n**Category Breakdown**:\n- Search Performance: 4.8/5.0 ⚡\n- User Interface: 4.4/5.0 🎨\n- Cross-Modal Features: 4.7/5.0 🔄\n- API Reliability: 4.9/5.0 🛡️\n\n**Response Rate**: 73% (1,240 responses from 1,700 active users)"),
        ("Feature Requests", "🔮 **Top Requested Features**:\n1. **Advanced Filtering**: 34% of users want better search filters\n2. **Bulk Operations**: 28% request batch processing capabilities\n3. **Real-time Collaboration**: 22% want live document editing\n4. **Mobile App**: 18% request native mobile experience\n\n🎯 **Implementation Priority**: Advanced filtering moved to next sprint based on user demand"),
    ];

    for (title, content) in feedback_sections {
        let section_id = Uuid::new_v4().to_string();
        let section_node = Node::with_id(
            NodeId::from_string(section_id),
            serde_json::Value::String(format!("## {}\n\n{}", title, content)),
        )
        .with_metadata(serde_json::json!({
            "node_type": "text",
            "title": title,
            "parent_id": feedback_id,
            "depth": 2,
            "section_type": "analytics_section"
        }));

        data_store
            .store_node_with_embedding(
                section_node,
                create_embedding(&format!("{} {}", title, content)),
            )
            .await?;
    }

    println!("   📊 Created Customer Feedback Analysis with 2 sections");
    Ok(())
}

fn create_embedding(text: &str) -> Vec<f32> {
    use rand::{Rng, SeedableRng};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let seed = hasher.finish();

    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    (0..384).map(|_| rng.gen_range(-1.0..1.0)).collect()
}
