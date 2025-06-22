use nodespace_core_types::{Node, NodeId};
use nodespace_data_store::{DataStore, SurrealDataStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating sample data for NodeSpace Data Store...");

    // Initialize the data store
    let store = SurrealDataStore::new("./data/sample.db").await?;

    // Create some sample nodes
    let doc1_id = NodeId::new();
    let doc1 = Node::with_id(
        doc1_id.clone(),
        serde_json::json!("This is a sample document about Rust programming."),
    )
    .with_metadata(serde_json::json!({
        "title": "Rust Programming Guide",
        "tags": ["programming", "rust", "systems"],
        "created_at": "2024-01-01T00:00:00Z"
    }));

    let doc2_id = NodeId::new();
    let doc2 = Node::with_id(
        doc2_id.clone(),
        serde_json::json!("A comprehensive guide to async programming in Rust."),
    )
    .with_metadata(serde_json::json!({
        "title": "Async Rust",
        "tags": ["programming", "rust", "async", "tokio"],
        "created_at": "2024-01-02T00:00:00Z"
    }));

    let concept_id = NodeId::new();
    let concept = Node::with_id(
        concept_id.clone(),
        serde_json::json!("Ownership is Rust's most unique feature."),
    )
    .with_metadata(serde_json::json!({
        "title": "Rust Ownership",
        "tags": ["rust", "concepts", "memory-safety"],
        "type": "concept",
        "created_at": "2024-01-03T00:00:00Z"
    }));

    // Store the nodes
    println!("Storing nodes...");
    store.store_node(doc1.clone()).await?;
    store.store_node(doc2.clone()).await?;
    store.store_node(concept.clone()).await?;

    // Create relationships between the nodes
    println!("Creating relationships...");
    store
        .create_relationship(&doc1_id, &concept_id, "mentions")
        .await?;

    store
        .create_relationship(&doc2_id, &concept_id, "explains")
        .await?;

    store
        .create_relationship(&doc1_id, &doc2_id, "related_to")
        .await?;

    // Query the data
    println!("Querying stored data...");

    // Get all nodes
    let all_nodes = store.query_nodes("SELECT * FROM nodes").await?;
    println!("All nodes count: {}", all_nodes.len());

    // Get relationships for doc1
    let doc1_relationships = store.get_node_relationships(&doc1_id).await?;
    println!(
        "Doc1 relationships: {}",
        serde_json::to_string_pretty(&doc1_relationships)?
    );

    // Custom query - find nodes with "rust" tag
    let rust_nodes = store
        .query_nodes("SELECT * FROM nodes WHERE tags CONTAINS 'rust'")
        .await?;
    println!("Nodes with 'rust' tag count: {}", rust_nodes.len());

    println!("Sample data creation completed successfully!");

    Ok(())
}
