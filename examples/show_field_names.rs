use nodespace_data_store::{DataStore, SurrealDataStore};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Database Field Names ===\n");

    let store = SurrealDataStore::new("./data/sample.db").await?;

    // Query raw SurrealDB to see exact field structure
    println!("RAW TEXT RECORD FIELDS:");
    let text_query = "SELECT * FROM text LIMIT 1";
    let text_nodes = store.query_nodes(text_query).await?;

    if let Some(node) = text_nodes.first() {
        println!("Node ID: {}", node.id.as_str());
        println!("Content field: {:?}", node.content);
        println!("Metadata field: {:?}", node.metadata);
        println!("Created_at: {}", node.created_at);
        println!("Updated_at: {}", node.updated_at);
    }

    println!("\nRAW DATE RECORD FIELDS:");
    let date_query = "SELECT * FROM date LIMIT 1";
    let date_nodes = store.query_nodes(date_query).await?;

    if let Some(node) = date_nodes.first() {
        println!("Node ID: {}", node.id.as_str());
        println!("Content field: {:?}", node.content);
        println!("Metadata field: {:?}", node.metadata);
        println!("Created_at: {}", node.created_at);
        println!("Updated_at: {}", node.updated_at);
    }

    println!("\n=== Database File Location ===");
    println!("This example uses: ./data/sample.db");
    println!("Full path: /Users/malibio/nodespace/nodespace-data-store/data/sample.db");

    // Check if the desktop app might be using a different path
    println!("\nPossible desktop app database locations:");
    println!("- Same path: ./data/sample.db");
    println!("- Different relative path from desktop app directory");
    println!("- Absolute path specified in desktop app config");

    Ok(())
}
