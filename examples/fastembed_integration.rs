use nodespace_core_types::{Node, NodeId};
use nodespace_data_store::{DataStore, SurrealDataStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 FastEmbed-rs Integration Example");
    println!("📌 Demonstrates the new BAAI/bge-small-en-v1.5 embedding model");
    println!("");

    // Initialize the data store
    let store = SurrealDataStore::new("/Users/malibio/nodespace/data/sample.db").await?;

    // Phase 1: Demonstrate embedding storage
    println!("📋 Phase 1: Embedding Storage with fastembed-rs");
    println!("═══════════════════════════════════════════════");

    // Create sample content for embedding
    let sample_contents = vec![
        "Strategic planning session revealed key market opportunities in AI automation",
        "Client feedback indicates strong preference for intuitive user interface design",
        "Competitive analysis shows gaps in mid-market segment pricing strategies",
        "Team collaboration tools adoption increased 40% this quarter across all departments",
    ];

    println!("Sample content prepared for embedding:");
    for (i, content) in sample_contents.iter().enumerate() {
        println!("  {}. {}", i + 1, content);
    }

    // Phase 2: Simulate embedding generation (awaiting real fastembed-rs)
    println!("\n🧠 Phase 2: Embedding Generation");
    println!("═══════════════════════════════════════");
    println!("⚠️  Integration Status: Awaiting fastembed-rs completion (NS-54)");
    println!("");
    println!("When fastembed-rs is ready, this will:");
    println!("  • Use BAAI/bge-small-en-v1.5 model (384 dimensions)");
    println!("  • ONNX Runtime with Rayon parallelization");
    println!("  • Improved semantic similarity vs previous all-MiniLM-L6-v2");
    println!("  • Cross-platform compatibility (Windows/macOS/Linux)");

    // For now, demonstrate with placeholder embeddings
    println!("\n🔄 Placeholder demonstration:");
    for (i, content) in sample_contents.iter().enumerate() {
        let node_id = NodeId::new();
        let node = Node::with_id(node_id.clone(), serde_json::json!(content));

        // Generate placeholder 384-dimensional embedding
        let placeholder_embedding = generate_placeholder_fastembed_embedding(content);

        // Store node with embedding
        let result = store
            .store_node_with_embedding(node, placeholder_embedding)
            .await;

        match result {
            Ok(_) => println!("  ✅ Stored: Content {} with 384-dim embedding", i + 1),
            Err(e) => println!("  ❌ Failed: Content {} - {}", i + 1, e),
        }
    }

    // Phase 3: Semantic search demonstration
    println!("\n🔍 Phase 3: Semantic Search with New Model");
    println!("═══════════════════════════════════════════");

    let search_query = "team collaboration and productivity tools";
    println!("Search query: \"{}\"", search_query);

    let query_embedding = generate_placeholder_fastembed_embedding(search_query);
    let search_results = store.search_similar_nodes(query_embedding, 3).await?;

    println!("Results found: {}", search_results.len());
    for (i, (node, score)) in search_results.iter().enumerate() {
        println!("  {}. Score: {:.3}", i + 1, score);
        if let Some(content) = node.content.as_str() {
            let preview = if content.len() > 60 {
                format!("{}...", &content[..57])
            } else {
                content.to_string()
            };
            println!("     Content: {}", preview);
        }
    }

    // Phase 4: Migration status report
    println!("\n📊 Phase 4: Migration Status Report");
    println!("═══════════════════════════════════════");

    let total_embedded = count_embedded_nodes(&store).await?;
    println!("Database status:");
    println!("  • Total nodes with embeddings: {}", total_embedded);
    println!("  • Embedding model: Transitioning to BAAI/bge-small-en-v1.5");
    println!("  • Vector dimensions: 384 (compatible with new model)");
    println!("  • Migration status: Ready for fastembed-rs integration");

    println!("\n📝 Next Steps:");
    println!("  1. Complete fastembed-rs implementation (NS-54)");
    println!("  2. Replace placeholder embeddings with real fastembed-rs");
    println!("  3. Validate improved semantic search quality");
    println!("  4. Performance benchmark vs previous Candle implementation");

    Ok(())
}

/// Generate placeholder embedding for demonstration
/// Real implementation will use fastembed-rs BAAI/bge-small-en-v1.5
fn generate_placeholder_fastembed_embedding(content: &str) -> Vec<f32> {
    let content_hash = content.chars().map(|c| c as u32).sum::<u32>();
    let seed = content_hash as f32 / 1000.0;

    // Generate 384-dimensional embedding (bge-small-en-v1.5 dimensions)
    (0..384)
        .map(|i| {
            let angle = (seed + i as f32) * 0.1;
            // Simulate more realistic embedding distribution
            let value = (angle.sin() + angle.cos()) / 2.0;
            // Add some variation to make embeddings more realistic
            let variation = ((i as f32 * seed).sin() * 0.1);
            (value + variation).clamp(-1.0, 1.0)
        })
        .collect()
}

/// Count total nodes with embeddings
async fn count_embedded_nodes(
    store: &SurrealDataStore,
) -> Result<usize, Box<dyn std::error::Error>> {
    let text_embedded = store
        .query_nodes("SELECT * FROM text WHERE embedding IS NOT NULL")
        .await
        .unwrap_or_default();
    let nodes_embedded = store
        .query_nodes("SELECT * FROM nodes WHERE embedding IS NOT NULL")
        .await
        .unwrap_or_default();

    Ok(text_embedded.len() + nodes_embedded.len())
}
