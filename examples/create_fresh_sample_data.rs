// Create fresh sample data for SurrealDB 2.x testing
use nodespace_data_store::{DataStore, SurrealDataStore};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔄 Creating fresh sample database for SurrealDB 2.x...\n");

    // Create fresh database with explicit path (matching desktop app expectations)
    let data_store = SurrealDataStore::new("data/sample.db").await?;
    println!("✅ Database initialized");

    // Create a parent date node for 2025-06-19
    let date_id = data_store
        .create_or_get_date_node("2025-06-19", Some("Team meeting notes"))
        .await?;
    println!("✅ Created date node: {}", date_id);

    // Create some text nodes with content
    let text1_id = data_store
        .create_text_node(
            "Discussed the new project requirements and timeline",
            Some("2025-06-19"),
        )
        .await?;
    println!("✅ Created text node 1: {}", text1_id);

    let text2_id = data_store
        .create_text_node(
            "Action items: finalize design specs by Friday",
            Some("2025-06-19"),
        )
        .await?;
    println!("✅ Created text node 2: {}", text2_id);

    let text3_id = data_store
        .create_text_node("Follow up with stakeholders next week", Some("2025-06-19"))
        .await?;
    println!("✅ Created text node 3: {}", text3_id);

    // Create another date for testing
    let date2_id = data_store
        .create_or_get_date_node("2025-06-20", Some("Project planning session"))
        .await?;
    println!("✅ Created second date node: {}", date2_id);

    let text4_id = data_store
        .create_text_node(
            "Reviewed technical architecture options",
            Some("2025-06-20"),
        )
        .await?;
    println!("✅ Created text node 4: {}", text4_id);

    let text5_id = data_store
        .create_text_node(
            "Decided on microservices approach with Rust backend",
            Some("2025-06-20"),
        )
        .await?;
    println!("✅ Created text node 5: {}", text5_id);

    // Verify the database
    println!("\n🔍 Verifying created data...");

    // Test query functionality with our fixed deserialization
    let query = "SELECT COUNT() FROM text GROUP ALL";
    match data_store.query_nodes(query).await {
        Ok(results) => {
            println!("✅ Query test successful: {} results", results.len());
        }
        Err(e) => {
            println!("❌ Query test failed: {:?}", e);
        }
    }

    // Test date node retrieval
    match data_store.get_nodes_for_date("2025-06-19").await {
        Ok(nodes) => {
            println!(
                "✅ Date retrieval test: found {} nodes for 2025-06-19",
                nodes.len()
            );
            for (i, node) in nodes.iter().enumerate() {
                let preview = node
                    .content
                    .as_str()
                    .map(|s| {
                        if s.len() > 50 {
                            format!("{}...", &s[..47])
                        } else {
                            s.to_string()
                        }
                    })
                    .unwrap_or("NULL".to_string());
                println!("   {}. {}", i + 1, preview);
            }
        }
        Err(e) => {
            println!("❌ Date retrieval test failed: {:?}", e);
        }
    }

    println!("\n🎉 Fresh sample database created successfully!");
    println!("   📂 Location: data/sample.db");
    println!("   📊 Contains: 2 date nodes, 5 text nodes");
    println!("   🔗 With proper parent-child relationships");
    println!("   ✅ Ready for SurrealDB 2.x testing");
    println!("   🖥️  Compatible with desktop app at:");
    println!("       /Users/malibio/nodespace/nodespace-data-store/data/sample.db");

    Ok(())
}
