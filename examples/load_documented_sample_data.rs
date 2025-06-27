//! Load Sample Data from NodeSpace System Design Documentation
//! 
//! This creates the established sample dataset from the system-design docs:
//! - Sarah Chen marketing professional persona data
//! - June 2025 journal entries with hierarchical structure
//! - Cross-modal examples for testing

use nodespace_data_store::{LanceDataStore, DataStore, ImageNode, ImageMetadata};
use nodespace_core_types::{Node, NodeId};
use std::error::Error;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("📚 Loading Documented Sample Data - NodeSpace Marketing Professional Persona\n");

    let data_store = LanceDataStore::new("data/documented_sample.db").await?;
    println!("✅ LanceDB initialized");

    // SARAH CHEN MARKETING PROFESSIONAL - JUNE 2025 JOURNAL DATA
    
    // June 23, 2025 (Today) - Q3 Campaign Strategy Review
    let june_23_date = create_date_node(
        "2025-06-23",
        "Monday, June 23 - Q3 Planning & Client Check-ins"
    ).await?;
    let date_id_23 = data_store.store_node_with_embedding(
        june_23_date,
        create_text_embedding("monday june 23 q3 planning client checkins")
    ).await?;
    
    let q3_strategy = create_text_node(
        "Q3 Campaign Strategy Review - Leadership Meeting",
        "Comprehensive review of Q3 campaign strategies with leadership team. Key focus areas: digital transformation messaging, enterprise client acquisition, and competitive positioning against industry leaders.\n\nAction items:\n• Schedule creative brief session with design team for new visual identity\n• Update campaign timeline in Airtable with revised milestones\n• Review competitor analysis from last quarter for positioning gaps\n• Coordinate with sales team on enterprise lead qualification process",
        Some("2025-06-23"),
        Some(serde_json::json!({
            "meeting_type": "leadership",
            "participants": ["Sarah Chen", "Marketing Director", "Creative Lead"],
            "priority": "high",
            "has_action_items": true
        }))
    ).await?;
    data_store.store_node_with_embedding(
        q3_strategy,
        create_text_embedding("q3 campaign strategy review leadership meeting action items creative brief competitor analysis")
    ).await?;

    let client_checkins = create_text_node(
        "Client Check-in Notes - Monday Sessions",
        "Client relationship management calls completed for three key accounts:\n\n**Acme Corp** - Enterprise customer ($2M annual contract)\n• Interested in expanding contract for Q4 with additional product lines\n• Positive feedback on current campaign performance (+15% engagement)\n• Next steps: Prepare proposal for Q4 expansion by July 1st\n\n**TechStart Inc** - Mid-market prospect\n• Delayed decision timeline until August due to budget cycle constraints\n• Still engaged, requesting case studies from similar tech companies\n• Action: Send relevant case studies by end of week\n\n**Global Solutions** - Premium package prospect\n• Ready to move forward with premium package ($500K investment)\n• Requires board approval, timeline 2-3 weeks\n• Next meeting scheduled for July 2nd to finalize details",
        Some("2025-06-23"),
        Some(serde_json::json!({
            "clients": ["Acme Corp", "TechStart Inc", "Global Solutions"],
            "revenue_impact": "$2.5M potential",
            "follow_up_required": true
        }))
    ).await?;
    data_store.store_node_with_embedding(
        client_checkins,
        create_text_embedding("client checkin notes acme corp techstart global solutions revenue contracts")
    ).await?;

    // June 22, 2025 (Yesterday) - Weekend Planning
    let june_22_date = create_date_node(
        "2025-06-22",
        "Sunday, June 22 - Weekend Campaign Ideas & Industry Research"
    ).await?;
    let date_id_22 = data_store.store_node_with_embedding(
        june_22_date,
        create_text_embedding("sunday june 22 weekend campaign ideas industry research")
    ).await?;

    let weekend_ideas = create_text_node(
        "Weekend Campaign Ideas - Video Content Strategy",
        "Brainstorming session for innovative video content approaches:\n\n**Behind-the-Scenes Content Series**\n• Documentary-style glimpses into product development process\n• Customer success story interviews (authentic, unscripted)\n• Technical team explanations of complex features in simple terms\n\n**Interactive Campaign Elements**\n• Virtual product demonstrations with live Q&A sessions\n• Customer challenge competitions with real prizes\n• Social media polls driving product feature prioritization\n\n**Thought Leadership Content**\n• Industry trend analysis videos (monthly series)\n• Prediction pieces for next quarter's market shifts\n• Collaboration pieces with industry influencers and partners\n\nNext steps: Validate ideas with creative team Monday morning",
        Some("2025-06-22"),
        Some(serde_json::json!({
            "content_type": "video_strategy",
            "brainstorm_session": true,
            "validation_needed": true
        }))
    ).await?;
    data_store.store_node_with_embedding(
        weekend_ideas,
        create_text_embedding("weekend campaign ideas video content strategy behind scenes interactive thought leadership")
    ).await?;

    // June 21, 2025 - Marketing Weekly Review
    let june_21_date = create_date_node(
        "2025-06-21",
        "Friday, June 21 - Marketing Weekly Review & Metrics Analysis"
    ).await?;
    let date_id_21 = data_store.store_node_with_embedding(
        june_21_date,
        create_text_embedding("friday june 21 marketing weekly review metrics analysis")
    ).await?;

    let weekly_review = create_text_node(
        "Marketing Weekly Review - Performance Metrics & Priorities",
        "**Week Ending June 21, 2025 - Performance Summary**\n\n**Campaign Performance Metrics:**\n• Email open rates: 24.3% (↑2.1% from last week)\n• Social media engagement: 156K interactions (↑12% weekly growth)\n• Website traffic: 45K unique visitors (↑8% organic growth)\n• Lead generation: 127 qualified leads (↑23% conversion improvement)\n\n**Campaign Updates:**\n• Product launch campaign entering Phase 2 next week\n• A/B testing results favor messaging variant B (+15% click-through)\n• Video content performing 3x better than static posts\n\n**Next Week Priorities:**\n1. Finalize Q3 budget allocation with finance team\n2. Launch customer feedback survey for product improvements\n3. Coordinate with sales on lead handoff process optimization\n4. Review creative assets for upcoming conference booth design",
        Some("2025-06-21"),
        Some(serde_json::json!({
            "metrics": {
                "email_open_rate": 24.3,
                "social_engagement": 156000,
                "website_traffic": 45000,
                "qualified_leads": 127
            },
            "weekly_review": true,
            "performance_trending": "positive"
        }))
    ).await?;
    data_store.store_node_with_embedding(
        weekly_review,
        create_text_embedding("marketing weekly review performance metrics campaign updates email social media leads")
    ).await?;

    // June 15, 2025 - Conference Learnings
    let june_15_date = create_date_node(
        "2025-06-15",
        "Saturday, June 15 - MarketingTech 2025 Conference Insights"
    ).await?;
    let date_id_15 = data_store.store_node_with_embedding(
        june_15_date,
        create_text_embedding("saturday june 15 marketingtech conference insights")
    ).await?;

    let conference_learnings = create_text_node(
        "Conference Learnings - MarketingTech 2025 Key Insights",
        "**MarketingTech 2025 Conference - Day 2 Keynotes & Sessions**\n\n**AI-Driven Personalization Trends:**\n• Real-time content adaptation based on user behavior patterns\n• Predictive analytics for customer lifecycle optimization\n• Cross-platform identity resolution becoming standard practice\n• Privacy-first personalization strategies gaining momentum\n\n**Emerging Channel Strategies:**\n• Connected TV advertising showing 40% better ROI than traditional\n• Voice search optimization critical for B2B discovery\n• Social commerce integration driving direct sales conversions\n• Podcast advertising reaching new professional demographics\n\n**Industry Predictions for 2026:**\n• Marketing automation will require human creativity oversight\n• Customer data platforms becoming unified decision engines\n• Sustainability messaging mandatory for enterprise sales\n• Video-first content strategies essential for engagement\n\n**Actionable Takeaways for Our Q3 Strategy:**\n1. Implement dynamic email content based on engagement history\n2. Explore Connected TV pilot program for enterprise awareness\n3. Develop sustainability-focused messaging for enterprise clients\n4. Invest in video production capabilities for thought leadership",
        Some("2025-06-15"),
        Some(serde_json::json!({
            "event": "MarketingTech 2025",
            "conference_insights": true,
            "actionable_items": 4,
            "strategic_importance": "high"
        }))
    ).await?;
    data_store.store_node_with_embedding(
        conference_learnings,
        create_text_embedding("marketingtech conference insights ai personalization connected tv voice search sustainability")
    ).await?;

    // Create some cross-modal image examples
    println!("\n📸 Adding Cross-Modal Image Examples...");

    let conference_photo = ImageNode {
        id: Uuid::new_v4().to_string(),
        image_data: create_mock_image_data("marketingtech_conference_2025.jpg"),
        embedding: create_image_embedding("marketing conference presentation slides audience professional event"),
        metadata: ImageMetadata {
            filename: "marketingtech_conference_2025.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            width: 1920,
            height: 1080,
            exif_data: Some(serde_json::json!({
                "date_taken": "2025-06-15T14:30:00Z",
                "camera": "iPhone 15 Pro",
                "location": "San Francisco Conference Center",
                "event": "MarketingTech 2025"
            })),
            description: Some("MarketingTech 2025 keynote presentation on AI-driven personalization trends".to_string()),
        },
        created_at: chrono::DateTime::parse_from_rfc3339("2025-06-15T14:30:00Z")?.with_timezone(&chrono::Utc),
    };

    let conference_image_id = data_store.create_image_node(conference_photo).await?;
    println!("   ✅ Added conference photo: {}", conference_image_id);

    let client_meeting_photo = ImageNode {
        id: Uuid::new_v4().to_string(),
        image_data: create_mock_image_data("acme_corp_meeting_june.jpg"),
        embedding: create_image_embedding("business meeting conference room presentation charts professional discussion"),
        metadata: ImageMetadata {
            filename: "acme_corp_meeting_june.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            width: 1280,
            height: 720,
            exif_data: Some(serde_json::json!({
                "date_taken": "2025-06-23T10:15:00Z",
                "camera": "MacBook Pro Camera",
                "location": "Acme Corp Headquarters",
                "participants": ["Sarah Chen", "Acme Corp Team"]
            })),
            description: Some("Q4 expansion discussion meeting with Acme Corp enterprise team".to_string()),
        },
        created_at: chrono::DateTime::parse_from_rfc3339("2025-06-23T10:15:00Z")?.with_timezone(&chrono::Utc),
    };

    let meeting_image_id = data_store.create_image_node(client_meeting_photo).await?;
    println!("   ✅ Added client meeting photo: {}", meeting_image_id);

    // Verification
    println!("\n🔍 Verifying Sample Data...");
    
    let june_23_nodes = data_store.get_child_nodes(&NodeId::from_string(date_id_23)).await?;
    println!("   📅 June 23, 2025: {} child nodes", june_23_nodes.len());
    
    let june_22_nodes = data_store.get_child_nodes(&NodeId::from_string(date_id_22)).await?;
    println!("   📅 June 22, 2025: {} child nodes", june_22_nodes.len());
    
    let june_21_nodes = data_store.get_child_nodes(&NodeId::from_string(date_id_21)).await?;
    println!("   📅 June 21, 2025: {} child nodes", june_21_nodes.len());
    
    let june_15_nodes = data_store.get_child_nodes(&NodeId::from_string(date_id_15)).await?;
    println!("   📅 June 15, 2025: {} child nodes", june_15_nodes.len());

    // Test cross-modal search with documented content
    println!("\n🔍 Testing Cross-Modal Search with Documented Data...");
    
    use nodespace_data_store::NodeType;
    let conference_query = create_text_embedding("marketing conference insights ai personalization");
    let conference_results = data_store.search_multimodal(
        conference_query,
        vec![NodeType::Text, NodeType::Image]
    ).await?;
    
    println!("   📊 'marketing conference insights' search: {} results", conference_results.len());
    for (i, node) in conference_results.iter().take(3).enumerate() {
        let preview = node.content.as_str()
            .map(|s| if s.len() > 80 { format!("{}...", &s[..77]) } else { s.to_string() })
            .unwrap_or("Image Node".to_string());
        println!("   {}. {} - {}", i + 1, node.id, preview);
    }

    println!("\n🎉 Documented Sample Data Loaded Successfully!");
    println!("📈 Dataset Summary:");
    println!("   👤 Sarah Chen Marketing Professional Persona");
    println!("   📅 June 15-23, 2025 journal entries");
    println!("   📝 {} text nodes with hierarchical relationships", 6);
    println!("   🖼️  {} image nodes with cross-modal connections", 2);
    println!("   🏢 Enterprise clients: Acme Corp, TechStart Inc, Global Solutions");
    println!("   📊 Marketing metrics, campaign data, conference insights");
    println!("   🔗 Established parent-child relationships for date organization");
    
    println!("\n📋 Ready for Testing:");
    println!("   • Cross-modal search: 'marketing conference insights'");
    println!("   • Client search: 'Acme Corp expansion Q4'");
    println!("   • Campaign analysis: 'video content strategy performance'");
    println!("   • Date-based retrieval: June 23 leadership meeting notes");

    Ok(())
}

async fn create_date_node(date: &str, description: &str) -> Result<Node, Box<dyn Error>> {
    let node = Node::with_id(
        NodeId::from_string(date.to_string()),
        serde_json::Value::String(description.to_string())
    ).with_metadata(serde_json::json!({
        "node_type": "date",
        "date": date,
        "content_type": "date_container"
    }));
    Ok(node)
}

async fn create_text_node(
    title: &str, 
    content: &str, 
    date: Option<&str>,
    metadata: Option<serde_json::Value>
) -> Result<Node, Box<dyn Error>> {
    let mut base_metadata = serde_json::json!({
        "node_type": "text",
        "title": title,
        "content_length": content.len()
    });
    
    if let Some(date_str) = date {
        base_metadata["parent_date"] = serde_json::Value::String(date_str.to_string());
    }
    
    if let Some(additional_metadata) = metadata {
        if let (Some(base_obj), Some(add_obj)) = (base_metadata.as_object_mut(), additional_metadata.as_object()) {
            for (key, value) in add_obj {
                base_obj.insert(key.clone(), value.clone());
            }
        }
    }
    
    let node = Node::new(serde_json::Value::String(content.to_string()))
        .with_metadata(base_metadata);
    Ok(node)
}

fn create_text_embedding(text: &str) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use rand::{SeedableRng, Rng};
    
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let seed = hasher.finish();
    
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    (0..384).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

fn create_image_embedding(description: &str) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use rand::{SeedableRng, Rng};
    
    let mut hasher = DefaultHasher::new();
    description.hash(&mut hasher);
    let seed = hasher.finish();
    
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    (0..512).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

fn create_mock_image_data(filename: &str) -> Vec<u8> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use rand::{SeedableRng, Rng};
    
    let mut hasher = DefaultHasher::new();
    filename.hash(&mut hasher);
    let seed = hasher.finish();
    
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    
    // Mock JPEG header + random data
    let mut data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header
    data.extend((0..2000).map(|_| rng.gen::<u8>()));
    data
}