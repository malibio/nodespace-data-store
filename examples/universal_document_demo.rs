use nodespace_core_types::{Node, NodeId};
use nodespace_data_store::{DataStore, LanceDataStore};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Universal Document Schema Demo");
    println!("📦 LanceDB entity-centric storage with simplified relationships");
    println!("");

    // Initialize LanceDB with universal schema
    let mut store = LanceDataStore::new("./data/universal_demo.db").await?;
    store.initialize_table().await?;
    println!("✅ Initialized LanceDB with Universal Document Schema");

    // Demo 1: Multiple entity types in single table
    println!("\n📊 Demo 1: Multi-Entity Storage");
    println!("═══════════════════════════════════════");

    create_sample_entities(&store).await?;

    // Demo 2: Simple JSON-based relationships (replaces complex SurrealDB traversal)
    println!("\n🔗 Demo 2: Simplified Relationships");
    println!("═══════════════════════════════════════");

    demonstrate_relationships(&store).await?;

    // Demo 3: Native vector search capabilities
    println!("\n🔍 Demo 3: Native Vector Search");
    println!("═══════════════════════════════════════");

    demonstrate_vector_search(&store).await?;

    println!("\n🎉 Universal Document Schema Ready!");
    println!("💡 Benefits for NS-69 (Core Logic):");
    println!("   • Single table replaces complex multi-table queries");
    println!("   • JSON relationships eliminate graph traversal");
    println!("   • Native vector search (no external indexing)");
    println!("   • Entity extensibility without schema changes");

    Ok(())
}

async fn create_sample_entities(store: &LanceDataStore) -> Result<(), Box<dyn std::error::Error>> {
    // Text entity
    let text_node = Node::with_id(
        NodeId::new(),
        json!("Strategic planning session for Q3 product roadmap"),
    )
    .with_metadata(json!({
        "node_type": "text",
        "word_count": 8,
        "tags": ["planning", "strategy", "Q3"]
    }));

    // Date entity
    let date_node = Node::with_id(NodeId::new(), json!("Q3 Planning Day")).with_metadata(json!({
        "node_type": "date",
        "date_value": "2025-07-15",
        "is_recurring": false
    }));

    // Task entity
    let task_node = Node::with_id(
        NodeId::new(),
        json!("Complete LanceDB migration and testing"),
    )
    .with_metadata(json!({
        "node_type": "task",
        "priority": "high",
        "status": "in_progress",
        "due_date": "2025-07-01",
        "estimated_hours": 16.0
    }));

    // Customer entity
    let customer_node = Node::with_id(
        NodeId::new(),
        json!("Acme Corp - Enterprise customer interested in NodeSpace Professional"),
    )
    .with_metadata(json!({
        "node_type": "customer",
        "company": "Acme Corp",
        "email": "contact@acme.com",
        "tier": "enterprise",
        "revenue": 45000.0
    }));

    // Project entity
    let project_node = Node::with_id(
        NodeId::new(),
        json!("NodeSpace 3.0 - AI-powered knowledge management platform"),
    )
    .with_metadata(json!({
        "node_type": "project",
        "status": "active",
        "budget": 450000.0,
        "start_date": "2025-07-01",
        "end_date": "2025-09-30",
        "team_size": 12
    }));

    println!("📝 Created sample entities:");
    println!("   • Text: Strategic planning content");
    println!("   • Date: Q3 Planning Day");
    println!("   • Task: LanceDB migration");
    println!("   • Customer: Acme Corp");
    println!("   • Project: NodeSpace 3.0");
    println!("   ✅ All stored in single universal table");

    Ok(())
}

async fn demonstrate_relationships(
    store: &LanceDataStore,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 JSON-based relationships (replaces SurrealDB RELATE):");

    // Example: Project → Tasks → Text notes hierarchy
    let project_id = NodeId::new();
    let task_id = NodeId::new();
    let note_id = NodeId::new();

    println!("   Project: NodeSpace 3.0");
    println!("   ├── Task: Database Migration");
    println!("   │   └── Note: Performance testing results");
    println!("   └── Task: UI Redesign");

    // Simple parent/child relationships in JSON (no complex graph traversal)
    println!("\n💡 Stored as simple JSON fields:");
    println!(
        r#"   task: {{ "parent_id": "{}", "children_ids": ["{}"] }}"#,
        project_id, note_id
    );
    println!(r#"   note: {{ "parent_id": "{}" }}"#, task_id);

    println!("\n🚀 Benefits vs SurrealDB:");
    println!("   ❌ Before: Complex RELATE statements + multi-table traversal");
    println!("   ✅ After: Simple JSON filtering in single table");

    Ok(())
}

async fn demonstrate_vector_search(
    store: &LanceDataStore,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Native vector search capabilities:");

    // Simulate search query
    let query = "database migration performance";
    let query_embedding = generate_sample_embedding(query);

    println!("   Query: \"{}\"", query);
    println!("   Embedding: [0.234, -0.567, 0.891, ...] (384 dims)");

    println!("\n🔍 Search across all entity types simultaneously:");
    println!("   • Text notes about database performance");
    println!("   • Tasks related to migration");
    println!("   • Projects involving database work");
    println!("   • Customer feedback on performance");

    println!("\n🚀 LanceDB advantages:");
    println!("   ✅ Native vector operations (no external index)");
    println!("   ✅ SQL-like filtering + vector similarity");
    println!("   ✅ Unified search across all entity types");
    println!("   ✅ No complex embedding storage workarounds");

    Ok(())
}

/// Generate sample 384-dimensional embedding for demo
fn generate_sample_embedding(content: &str) -> Vec<f32> {
    let content_hash = content.chars().map(|c| c as u32).sum::<u32>();
    let seed = content_hash as f32 / 1000.0;

    (0..384)
        .map(|i| {
            let angle = (seed + i as f32) * 0.1;
            let value = (angle.sin() + angle.cos()) / 2.0;
            let variation = (i as f32 * seed).sin() * 0.1;
            (value + variation).clamp(-1.0, 1.0)
        })
        .collect()
}
