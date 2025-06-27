//! Load Sample Data from NodeSpace Documentation
//! Sarah Chen Marketing Professional - June 2025 Journal Entries

use nodespace_core_types::{Node, NodeId};
use nodespace_data_store::{DataStore, LanceDataStore};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("📚 Loading Sarah Chen Marketing Professional Sample Data\n");

    let data_store = LanceDataStore::new("data/sarah_chen_sample.db").await?;
    println!("✅ LanceDB initialized");

    // June 23, 2025 - Q3 Strategy & Client Check-ins
    let q3_strategy = Node::with_id(
        NodeId::from_string("q3-strategy-review".to_string()),
        serde_json::Value::String(
            "Q3 Campaign Strategy Review - Leadership Meeting\n\nAction items:\n• Schedule creative brief session with design team\n• Update campaign timeline in Airtable\n• Review competitor analysis from last quarter".to_string()
        )
    ).with_metadata(serde_json::json!({
        "node_type": "text",
        "date": "2025-06-23",
        "meeting_type": "leadership",
        "has_action_items": true
    }));

    let client_notes = Node::with_id(
        NodeId::from_string("client-checkin-notes".to_string()),
        serde_json::Value::String(
            "Client Check-in Notes - Monday Sessions\n\n**Acme Corp** - Enterprise customer ($2M annual contract)\n• Interested in expanding contract for Q4\n• Positive feedback on current campaign performance\n\n**TechStart Inc** - Mid-market prospect\n• Delayed decision until August due to budget cycle\n\n**Global Solutions** - Premium package prospect\n• Ready to move forward with premium package ($500K investment)".to_string()
        )
    ).with_metadata(serde_json::json!({
        "node_type": "text",
        "date": "2025-06-23",
        "clients": ["Acme Corp", "TechStart Inc", "Global Solutions"],
        "revenue_impact": "$2.5M potential"
    }));

    // June 15, 2025 - Conference Insights
    let conference_insights = Node::with_id(
        NodeId::from_string("marketingtech-conference-insights".to_string()),
        serde_json::Value::String(
            "MarketingTech 2025 Conference - Key Insights\n\n**AI-Driven Personalization Trends:**\n• Real-time content adaptation based on user behavior\n• Predictive analytics for customer lifecycle optimization\n• Privacy-first personalization strategies\n\n**Emerging Channel Strategies:**\n• Connected TV advertising showing 40% better ROI\n• Voice search optimization critical for B2B discovery\n• Social commerce integration driving direct sales\n\n**Actionable Takeaways:**\n1. Implement dynamic email content\n2. Explore Connected TV pilot program\n3. Develop sustainability-focused messaging\n4. Invest in video production capabilities".to_string()
        )
    ).with_metadata(serde_json::json!({
        "node_type": "text",
        "date": "2025-06-15",
        "event": "MarketingTech 2025",
        "conference_insights": true,
        "actionable_items": 4
    }));

    // Store nodes with embeddings
    use rand::{Rng, SeedableRng};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn create_embedding(text: &str) -> Vec<f32> {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let seed = hasher.finish();
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        (0..384).map(|_| rng.gen_range(-1.0..1.0)).collect()
    }

    let q3_id = data_store
        .store_node_with_embedding(
            q3_strategy,
            create_embedding("q3 campaign strategy leadership meeting action items"),
        )
        .await?;

    let client_id = data_store
        .store_node_with_embedding(
            client_notes,
            create_embedding("client checkin acme corp techstart global solutions revenue"),
        )
        .await?;

    let conf_id = data_store
        .store_node_with_embedding(
            conference_insights,
            create_embedding(
                "marketingtech conference ai personalization connected tv voice search",
            ),
        )
        .await?;

    println!("✅ Created nodes:");
    println!("   📝 Q3 Strategy Review: {}", q3_id);
    println!("   👥 Client Check-ins: {}", client_id);
    println!("   🎤 Conference Insights: {}", conf_id);

    // Test retrieval
    println!("\n🔍 Testing Sample Data Retrieval...");

    if let Some(retrieved_node) = data_store.get_node(&q3_id).await? {
        let preview = retrieved_node
            .content
            .as_str()
            .map(|s| {
                if s.len() > 100 {
                    format!("{}...", &s[..97])
                } else {
                    s.to_string()
                }
            })
            .unwrap_or("NULL".to_string());
        println!("   ✅ Retrieved Q3 Strategy: {}", preview);
    }

    // Test search functionality
    use nodespace_data_store::NodeType;
    let search_results = data_store
        .search_multimodal(
            create_embedding("marketing conference insights"),
            vec![NodeType::Text],
        )
        .await?;

    println!(
        "   📊 Search for 'marketing conference': {} results",
        search_results.len()
    );

    println!("\n🎉 Sample Data Loading Complete!");
    println!("📈 Dataset includes:");
    println!("   👤 Sarah Chen Marketing Professional persona");
    println!("   📅 June 2025 journal entries");
    println!("   🏢 Enterprise clients: Acme Corp, TechStart Inc, Global Solutions");
    println!("   📊 Campaign strategy, client notes, conference insights");
    println!("   🔍 Ready for cross-modal search testing");

    Ok(())
}
