// Debug node metadata to see if parent_date is properly set
use nodespace_data_store::{DataStore, SurrealDataStore};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 Debugging node metadata for hierarchical processing...\n");

    let data_store = SurrealDataStore::new("data/sample.db").await?;
    println!("✅ Connected to database");

    // Get nodes for a date and examine their metadata
    let date_str = "2025-06-25";
    println!("\n🔎 Examining nodes for date: {}", date_str);

    match data_store.get_nodes_for_date(date_str).await {
        Ok(nodes) => {
            println!("   ✅ Found {} nodes", nodes.len());
            for (i, node) in nodes.iter().enumerate() {
                let content_preview = node
                    .content
                    .as_str()
                    .map(|s| {
                        if s.len() > 30 {
                            format!("{}...", &s[..27])
                        } else {
                            s.to_string()
                        }
                    })
                    .unwrap_or("NULL".to_string());

                println!("\n   📄 Node {}: {}", i + 1, node.id);
                println!("      Content: '{}'", content_preview);
                println!("      Metadata: {:?}", node.metadata);

                // Check specifically for parent_date in metadata
                if let Some(metadata) = &node.metadata {
                    if let Some(parent_date) = metadata.get("parent_date") {
                        println!("      ✅ Parent date found: {:?}", parent_date);
                    } else {
                        println!("      ❌ No parent_date in metadata");
                        println!(
                            "         Available keys: {:?}",
                            metadata
                                .as_object()
                                .map(|obj| obj.keys().collect::<Vec<_>>())
                        );
                    }
                } else {
                    println!("      ❌ No metadata at all");
                }
            }
        }
        Err(e) => {
            println!("   ❌ Error: {:?}", e);
        }
    }

    Ok(())
}
