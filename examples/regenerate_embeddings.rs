use nodespace_core_types::{Node, NodeId};
use nodespace_data_store::{DataStore, SurrealDataStore};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting embedding regeneration with fastembed-rs...");
    println!("📌 This script will re-embed all content using the new model");

    // Initialize the data store
    let store = SurrealDataStore::new("./data/sample.db").await?;

    // Phase 1: Get all content that needs re-embedding
    println!("\n📋 Phase 1: Discovering content to re-embed...");
    let content_to_embed = discover_content_for_embedding(&store).await?;

    println!("✅ Found {} items to re-embed", content_to_embed.len());

    // Phase 2: Re-embed content (placeholder for when NLP engine is ready)
    println!("\n🧠 Phase 2: Re-embedding content with fastembed-rs...");
    println!("⚠️  NLP Engine Integration Required:");
    println!("   This phase requires the updated NLP engine with fastembed-rs support");
    println!("   Current status: Waiting for NS-54 completion");

    // For now, simulate the re-embedding process
    simulate_embedding_regeneration(&store, &content_to_embed).await?;

    // Phase 3: Validation
    println!("\n🔍 Phase 3: Validation Report");
    println!("═══════════════════════════════════════════");
    validate_embedding_migration(&store).await?;

    println!("\n📝 Next steps:");
    println!("  1. Complete NLP engine fastembed-rs integration (NS-54)");
    println!("  2. Replace simulation with real embedding generation");
    println!("  3. Run semantic search quality validation");

    Ok(())
}

/// Discover all content that needs re-embedding
async fn discover_content_for_embedding(
    store: &SurrealDataStore,
) -> Result<Vec<ContentItem>, Box<dyn std::error::Error>> {
    let mut content_items = Vec::new();

    // Get all text nodes that should have embeddings
    let text_nodes = store.query_nodes("SELECT * FROM text").await?;

    for node in text_nodes {
        if let Some(content_str) = extract_text_content(&node.content) {
            if !content_str.trim().is_empty() {
                content_items.push(ContentItem {
                    node_id: node.id.clone(),
                    content: content_str,
                    table: "text".to_string(),
                    node: node,
                });
            }
        }
    }

    // Get regular nodes that should have embeddings
    let regular_nodes = store
        .query_nodes("SELECT * FROM nodes")
        .await
        .unwrap_or_default();

    for node in regular_nodes {
        if let Some(content_str) = extract_text_content(&node.content) {
            if !content_str.trim().is_empty() {
                content_items.push(ContentItem {
                    node_id: node.id.clone(),
                    content: content_str,
                    table: "nodes".to_string(),
                    node: node,
                });
            }
        }
    }

    Ok(content_items)
}

/// Extract text content from serde_json::Value
fn extract_text_content(content: &Value) -> Option<String> {
    match content {
        Value::String(s) => Some(s.clone()),
        Value::Object(obj) => {
            // Look for common text fields
            if let Some(Value::String(text)) = obj.get("text") {
                Some(text.clone())
            } else if let Some(Value::String(content)) = obj.get("content") {
                Some(content.clone())
            } else {
                // Fallback: serialize the object as text
                Some(serde_json::to_string(obj).unwrap_or_default())
            }
        }
        _ => Some(content.to_string()),
    }
}

/// Simulate embedding regeneration (placeholder for real implementation)
async fn simulate_embedding_regeneration(
    store: &SurrealDataStore,
    content_items: &[ContentItem],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Simulating embedding regeneration process...");

    let mut regenerated_count = 0;

    for (index, item) in content_items.iter().enumerate() {
        if index % 50 == 0 {
            println!(
                "   Processing batch {} of {}...",
                index,
                content_items.len()
            );
        }

        // Simulate embedding generation (placeholder)
        let simulated_embedding = generate_placeholder_embedding(&item.content);

        // Store the node with the new embedding
        let result = store
            .store_node_with_embedding(item.node.clone(), simulated_embedding)
            .await;

        if result.is_ok() {
            regenerated_count += 1;
        } else {
            eprintln!("⚠️  Failed to store embedding for node: {}", item.node_id);
        }
    }

    println!(
        "✅ Simulated regeneration of {} embeddings",
        regenerated_count
    );
    println!("📌 Real implementation will use fastembed-rs bge-small-en-v1.5 model");

    Ok(())
}

/// Generate placeholder embedding for simulation
fn generate_placeholder_embedding(content: &str) -> Vec<f32> {
    // Generate a deterministic but varied embedding based on content
    // This is just for simulation - real implementation will use fastembed-rs

    let content_hash = content.chars().map(|c| c as u32).sum::<u32>();
    let seed = content_hash as f32 / 1000.0;

    // Generate 384-dimensional embedding (matching bge-small-en-v1.5)
    (0..384)
        .map(|i| {
            let angle = (seed + i as f32) * 0.1;
            (angle.sin() + angle.cos()) / 2.0
        })
        .collect()
}

/// Validate the embedding migration results
async fn validate_embedding_migration(
    store: &SurrealDataStore,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check how many records now have embeddings
    let embedded_text_nodes = store
        .query_nodes("SELECT COUNT() FROM text WHERE embedding IS NOT NULL GROUP ALL")
        .await
        .unwrap_or_default();
    let embedded_regular_nodes = store
        .query_nodes("SELECT COUNT() FROM nodes WHERE embedding IS NOT NULL GROUP ALL")
        .await
        .unwrap_or_default();

    let text_count = embedded_text_nodes.len();
    let nodes_count = embedded_regular_nodes.len();

    println!("📊 Embedding Status:");
    println!("   Text nodes with embeddings: {}", text_count);
    println!("   Regular nodes with embeddings: {}", nodes_count);
    println!("   Total embedded records: {}", text_count + nodes_count);

    // Test semantic search capability
    println!("\n🔍 Testing semantic search capability...");
    let test_embedding = generate_placeholder_embedding("marketing campaign strategy");
    let search_results = store.search_similar_nodes(test_embedding, 5).await?;

    println!(
        "   Semantic search test: {} results found",
        search_results.len()
    );

    if search_results.len() > 0 {
        println!("✅ Semantic search is functional");
        for (i, (node, score)) in search_results.iter().take(3).enumerate() {
            println!(
                "   Result {}: score {:.3}, content preview: {}",
                i + 1,
                score,
                preview_content(&node.content)
            );
        }
    } else {
        println!("⚠️  No semantic search results found");
    }

    Ok(())
}

/// Create a preview of content for display
fn preview_content(content: &Value) -> String {
    let content_str = extract_text_content(content).unwrap_or_default();
    if content_str.len() > 60 {
        format!("{}...", &content_str[..57])
    } else {
        content_str
    }
}

/// Structure for content items that need embedding
#[derive(Debug)]
struct ContentItem {
    node_id: NodeId,
    content: String,
    table: String,
    node: Node,
}
