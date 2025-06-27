// Debug the get_nodes_for_date function
use nodespace_data_store::{DataStore, SurrealDataStore};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 Debugging get_nodes_for_date function...\n");

    let data_store = SurrealDataStore::new("data/sample.db").await?;
    println!("✅ Connected to database");

    // Test the exact function that the frontend is calling
    let test_dates = vec!["2025-06-25", "2025-06-19", "2025-06-20"];

    for date_str in test_dates {
        println!("\n🔎 Testing date: {}", date_str);

        match data_store.get_nodes_for_date(date_str).await {
            Ok(nodes) => {
                println!("   ✅ Found {} nodes", nodes.len());
                for (i, node) in nodes.iter().enumerate() {
                    let content_preview = node
                        .content
                        .as_str()
                        .map(|s| {
                            if s.len() > 40 {
                                format!("{}...", &s[..37])
                            } else {
                                s.to_string()
                            }
                        })
                        .unwrap_or("NULL".to_string());
                    println!("      {}. {} = '{}'", i + 1, node.id, content_preview);
                }
            }
            Err(e) => {
                println!("   ❌ Error: {:?}", e);
            }
        }
    }

    // Let's also test the raw database queries to see what's actually in there
    println!("\n🔍 Raw database queries:");

    println!("\n📊 All text records:");
    let text_query = "SELECT * FROM text";
    match data_store.query_nodes(text_query).await {
        Ok(nodes) => {
            println!("   Found {} text records", nodes.len());
            for (i, node) in nodes.iter().enumerate() {
                println!(
                    "      {}. {} (metadata: {:?})",
                    i + 1,
                    node.id,
                    node.metadata
                );
            }
        }
        Err(e) => {
            println!("   Error in text query: {:?}", e);
        }
    }

    println!("\n📊 All date records:");
    let date_query = "SELECT * FROM date";
    match data_store.query_nodes(date_query).await {
        Ok(nodes) => {
            println!("   Found {} date records", nodes.len());
            for (i, node) in nodes.iter().enumerate() {
                println!("      {}. {}", i + 1, node.id);
            }
        }
        Err(e) => {
            println!("   Error in date query: {:?}", e);
        }
    }

    println!("\n🔗 Relationship traversal test:");
    let traversal_query = "SELECT * FROM date:`2025-06-25`->contains->text";
    match data_store.query_nodes(traversal_query).await {
        Ok(nodes) => {
            println!("   Found {} nodes via relationship traversal", nodes.len());
            for (i, node) in nodes.iter().enumerate() {
                let content_preview = node
                    .content
                    .as_str()
                    .map(|s| {
                        if s.len() > 40 {
                            format!("{}...", &s[..37])
                        } else {
                            s.to_string()
                        }
                    })
                    .unwrap_or("NULL".to_string());
                println!("      {}. {} = '{}'", i + 1, node.id, content_preview);
            }
        }
        Err(e) => {
            println!("   Error in traversal query: {:?}", e);
        }
    }

    Ok(())
}
