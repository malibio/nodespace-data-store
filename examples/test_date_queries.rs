use nodespace_data_store::{DataStore, SurrealDataStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing DateNode query patterns...");

    let store = SurrealDataStore::new("./data/sample.db").await?;

    // Test direct SurrealQL queries for different patterns
    let test_date = "2025-04-15"; // Use a date that had data generated
    let queries = vec![
        format!("SELECT * FROM date:`{}`", test_date),
        format!("SELECT * FROM date:`{}`->contains", test_date),
        format!("SELECT * FROM date:`{}`->contains->text", test_date),
        format!(
            "SELECT * FROM date:`{}`->contains->(text,task,note)",
            test_date
        ),
    ];

    for query in queries {
        println!("\n--- Query: {} ---", query);
        match store.query_nodes(&query).await {
            Ok(nodes) => {
                println!("Results: {} items", nodes.len());
                for (i, node) in nodes.iter().enumerate() {
                    if i < 2 {
                        // Show first 2 results
                        println!("  [{}] ID: {:?}", i, node.id);
                        if let Some(content) = node.content.as_str() {
                            let preview = if content.len() > 60 {
                                &content[..60]
                            } else {
                                content
                            };
                            println!("      Content: {}...", preview);
                        }
                    }
                }
                if nodes.len() > 2 {
                    println!("  ... and {} more", nodes.len() - 2);
                }
            }
            Err(e) => println!("Error: {:?}", e),
        }
    }

    // Test the helper methods
    println!("\n--- Testing helper methods ---");

    let date_children = store.get_date_children(test_date).await?;
    println!(
        "get_date_children('{}'): {} items",
        test_date,
        date_children.len()
    );

    let text_nodes = store.get_nodes_for_date(test_date).await?;
    println!(
        "get_nodes_for_date('{}'): {} text nodes",
        test_date,
        text_nodes.len()
    );

    Ok(())
}
