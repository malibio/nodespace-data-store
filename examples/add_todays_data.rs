// Add sample data for today's date
use nodespace_data_store::{DataStore, SurrealDataStore};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    println!("🔄 Adding sample data for today: {}", today);

    let data_store = SurrealDataStore::new("data/sample.db").await?;
    println!("✅ Connected to existing database");

    // Create today's date node
    let date_id = data_store
        .create_or_get_date_node(&today, Some("Today's notes"))
        .await?;
    println!("✅ Created/found date node: {}", date_id);

    // Add some sample content for today
    let text1_id = data_store
        .create_text_node(
            "Welcome to NodeSpace! This is a sample note for today.",
            Some(&today),
        )
        .await?;

    let text2_id = data_store
        .create_text_node(
            "Try creating your own notes and exploring the features.",
            Some(&today),
        )
        .await?;

    let text3_id = data_store
        .create_text_node(
            "The SurrealDB 2.x upgrade is working perfectly!",
            Some(&today),
        )
        .await?;

    println!("✅ Created 3 text nodes for today");

    // Verify the data
    println!("\n🔍 Verifying today's data...");
    match data_store.get_nodes_for_date(&today).await {
        Ok(nodes) => {
            println!("✅ Found {} nodes for {}", nodes.len(), today);
            for (i, node) in nodes.iter().enumerate() {
                let content_preview = node
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
                println!("   {}. {}", i + 1, content_preview);
            }
        }
        Err(e) => {
            println!("❌ Verification failed: {:?}", e);
        }
    }

    println!("\n🎉 Today's sample data added successfully!");
    println!("   📅 Date: {}", today);
    println!("   📊 Added: 3 sample text nodes");
    println!("   🖥️  App should now show content for today!");

    Ok(())
}
