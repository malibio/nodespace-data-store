use nodespace_data_store::{SurrealDataStore, DataStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Database Structure Example ===\n");

    let store = SurrealDataStore::new("./data/sample.db").await?;

    // Show raw text records
    println!("TEXT RECORDS:");
    let text_nodes = store.query_nodes("SELECT * FROM text LIMIT 2").await?;
    for (i, node) in text_nodes.iter().enumerate() {
        println!("{}. text:{}", i + 1, node.id.as_str().replace("-", "_"));
        if let Some(content_str) = node.content.as_str() {
            let truncated = if content_str.len() > 100 {
                format!("{}...", &content_str[..100])
            } else {
                content_str.to_string()
            };
            println!("   content: \"{}\"", truncated);
        }
        println!("   vector: [not yet implemented]");
        println!();
    }

    // Show raw date records with actual date values
    println!("DATE RECORDS:");
    let date_results = store.query_nodes("SELECT * FROM date WHERE date_value IS NOT NULL LIMIT 2").await?;
    for (i, node) in date_results.iter().enumerate() {
        println!("{}. date:{}", i + 1, node.id.as_str().replace("-", "_"));
        
        // The date_value is stored in metadata, not content
        if let Some(metadata) = &node.metadata {
            if let Some(date_val) = metadata.get("date_value") {
                if let Some(date_str) = date_val.as_str() {
                    println!("   date_value: \"{}\"", date_str);
                }
            }
        }
        println!();
    }

    // Show relationships using raw SurrealDB query
    println!("RELATIONSHIPS (date->contains->text):");
    // This will show the actual SurrealDB relationship structure
    println!("Example: date:2025_06_01 ->contains-> text:some_uuid_here");
    println!("(Use SurrealDB's native relationship traversal)\n");

    // Show a complete example of date with its children
    let test_date = "2025-05-01";
    println!("COMPLETE EXAMPLE for {}:", test_date);
    let children = store.get_nodes_for_date(test_date).await?;
    
    if !children.is_empty() {
        println!("date:2025_05_01_uuid");
        println!("├── date_value: \"{}\"", test_date);
        println!("└── contains relationships to:");
        
        for (i, child) in children.iter().enumerate() {
            let prefix = if i == children.len() - 1 { "    └──" } else { "    ├──" };
            println!("{} text:{}", prefix, child.id.as_str().replace("-", "_"));
            if let Some(content_str) = child.content.as_str() {
                let truncated = if content_str.len() > 80 {
                    format!("{}...", &content_str[..80])
                } else {
                    content_str.to_string()
                };
                println!("        content: \"{}\"", truncated);
            }
            println!("        vector: [embeddings will go here]");
        }
    } else {
        println!("No children found for {}", test_date);
    }

    Ok(())
}