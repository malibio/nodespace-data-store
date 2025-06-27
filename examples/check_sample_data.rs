use nodespace_data_store::{DataStore, SurrealDataStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking sample data content in database...");

    // Initialize the data store
    let store = SurrealDataStore::new("/Users/malibio/nodespace/data/sample.db").await?;

    // First, let's do a raw query to see what's actually in the database
    println!("\n=== Raw query for text table ===");
    let text_nodes = store.query_nodes("SELECT * FROM text LIMIT 5").await?;
    println!("Found {} text nodes via direct query:", text_nodes.len());
    for (i, node) in text_nodes.iter().enumerate() {
        println!("{}. Content: {:?}", i + 1, node.content);
        println!("   Metadata: {:?}", node.metadata);
        println!();
    }

    // Check several dates to see the variety of meaningful content
    let test_dates = vec!["2025-04-15", "2025-05-01", "2025-05-15", "2025-06-01"];

    for date in test_dates {
        println!("\n=== Content for {} via get_nodes_for_date ===", date);
        let nodes = store.get_nodes_for_date(date).await?;

        if nodes.is_empty() {
            println!("No nodes found for this date");
        } else {
            println!("Found {} nodes:", nodes.len());
            for (i, node) in nodes.iter().enumerate() {
                if let Some(content_str) = node.content.as_str() {
                    println!("{}. {}", i + 1, content_str);
                } else {
                    println!("{}. {:?}", i + 1, node.content);
                }
                println!();
            }
        }
    }

    Ok(())
}
