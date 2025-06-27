//! Load Hierarchical Sample Data from sample-node-entry.md
//!
//! Creates a DateNode for today with "Product Launch Campaign Strategy" as child,
//! then builds the hierarchical structure based on markdown hyphen depth levels.

use nodespace_core_types::{Node, NodeId};
use nodespace_data_store::{DataStore, LanceDataStore};
use std::error::Error;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🗂️  Loading Hierarchical Sample Data - Product Launch Campaign Strategy\n");

    let data_store = LanceDataStore::new("data/hierarchical_sample.db").await?;
    println!("✅ LanceDB initialized");

    // Create today's DateNode
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let date_node = Node::with_id(
        NodeId::from_string(today.clone()),
        serde_json::Value::String(format!("📅 {} - Product Launch Planning", today)),
    )
    .with_metadata(serde_json::json!({
        "node_type": "date",
        "date": today,
        "content_type": "date_container"
    }));

    let date_id = data_store
        .store_node_with_embedding(date_node, create_embedding("date product launch planning"))
        .await?;
    println!("✅ Created DateNode: {}", date_id);

    // Create the main campaign strategy node as child of DateNode
    let campaign_strategy_id = Uuid::new_v4().to_string();
    let campaign_strategy = Node::with_id(
        NodeId::from_string(campaign_strategy_id.clone()),
        serde_json::Value::String("# Product Launch Campaign Strategy\n\nThis comprehensive product launch plan provides the strategic framework, tactical execution details, and success measurement criteria necessary for achieving market leadership in the sustainable professional products category.".to_string())
    ).with_metadata(serde_json::json!({
        "node_type": "text",
        "title": "Product Launch Campaign Strategy",
        "parent_date": today,
        "depth": 1,
        "content_type": "strategy_document"
    }));

    let strategy_id = data_store
        .store_node_with_embedding(
            campaign_strategy,
            create_embedding("product launch campaign strategy comprehensive plan"),
        )
        .await?;
    println!("✅ Created main strategy node: {}", strategy_id);

    // Level 2: Main sections (## headers)
    let sections = vec![
        ("Launch Overview", create_launch_overview_content()),
        ("Executive Summary", create_executive_summary_content()),
        ("Target Audience Analysis", create_target_audience_content()),
        ("Product Positioning Strategy", create_positioning_content()),
        (
            "Marketing Channel Strategy",
            create_channel_strategy_content(),
        ),
        ("Success Metrics and KPIs", create_metrics_content()),
        (
            "Budget Allocation and Resource Planning",
            create_budget_content(),
        ),
    ];

    let mut section_ids = Vec::new();
    for (title, content) in sections {
        let section_id = Uuid::new_v4().to_string();
        let section_node = Node::with_id(
            NodeId::from_string(section_id.clone()),
            serde_json::Value::String(format!("## {}\n\n{}", title, content)),
        )
        .with_metadata(serde_json::json!({
            "node_type": "text",
            "title": title,
            "parent_id": strategy_id,
            "depth": 2,
            "section_type": "main_section"
        }));

        let stored_id = data_store
            .store_node_with_embedding(
                section_node,
                create_embedding(&format!("{} {}", title, content)),
            )
            .await?;

        section_ids.push((title.to_string(), stored_id));
        println!("   📄 Created section: {}", title);
    }

    // Level 3: Subsections for Target Audience Analysis (### headers)
    let target_audience_id = section_ids
        .iter()
        .find(|(title, _)| title == "Target Audience Analysis")
        .map(|(_, id)| id.clone())
        .unwrap();

    let subsections = vec![
        ("Primary Target Segment", create_primary_target_content()),
        (
            "Secondary Target Segments",
            create_secondary_target_content(),
        ),
    ];

    let mut subsection_ids = Vec::new();
    for (title, content) in subsections {
        let subsection_id = Uuid::new_v4().to_string();
        let subsection_node = Node::with_id(
            NodeId::from_string(subsection_id.clone()),
            serde_json::Value::String(format!("### {}\n\n{}", title, content)),
        )
        .with_metadata(serde_json::json!({
            "node_type": "text",
            "title": title,
            "parent_id": target_audience_id,
            "depth": 3,
            "section_type": "subsection"
        }));

        let stored_id = data_store
            .store_node_with_embedding(
                subsection_node,
                create_embedding(&format!("{} {}", title, content)),
            )
            .await?;

        subsection_ids.push((title.to_string(), stored_id));
        println!("      📋 Created subsection: {}", title);
    }

    // Level 4: Detailed breakdowns under Primary Target Segment
    let primary_target_id = subsection_ids
        .iter()
        .find(|(title, _)| title == "Primary Target Segment")
        .map(|(_, id)| id.clone())
        .unwrap();

    let details = vec![
        ("Professional Demographics", create_demographics_content()),
        ("Psychographic Profile", create_psychographic_content()),
    ];

    for (title, content) in details {
        let detail_id = Uuid::new_v4().to_string();
        let detail_node = Node::with_id(
            NodeId::from_string(detail_id.clone()),
            serde_json::Value::String(format!("**{}**:\n{}", title, content)),
        )
        .with_metadata(serde_json::json!({
            "node_type": "text",
            "title": title,
            "parent_id": primary_target_id,
            "depth": 4,
            "section_type": "detail"
        }));

        let stored_id = data_store
            .store_node_with_embedding(
                detail_node,
                create_embedding(&format!("{} {}", title, content)),
            )
            .await?;

        println!("         🎯 Created detail: {}", title);
    }

    // Test hierarchical retrieval
    println!("\n🔍 Testing Hierarchical Data Retrieval...");

    // Get the main strategy node
    if let Some(retrieved_strategy) = data_store.get_node(&strategy_id).await? {
        let preview = retrieved_strategy
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
        println!("   ✅ Retrieved strategy: {}", preview);
    }

    // Test search across hierarchy
    use nodespace_data_store::NodeType;
    let search_results = data_store
        .search_multimodal(
            create_embedding("target audience professional demographics"),
            vec![NodeType::Text],
        )
        .await?;

    println!(
        "   📊 Search for 'target audience': {} results",
        search_results.len()
    );
    for (i, node) in search_results.iter().take(3).enumerate() {
        if let Some(metadata) = &node.metadata {
            if let Some(title) = metadata.get("title").and_then(|v| v.as_str()) {
                println!(
                    "   {}. {} (depth: {})",
                    i + 1,
                    title,
                    metadata.get("depth").and_then(|v| v.as_u64()).unwrap_or(0)
                );
            }
        }
    }

    println!("\n🎉 Hierarchical Sample Data Loaded Successfully!");
    println!("📈 Dataset Structure:");
    println!("   📅 DateNode: {} (today)", today);
    println!("   📄 Main Document: Product Launch Campaign Strategy");
    println!("   📑 7 Main Sections (depth 2)");
    println!("   📋 2 Subsections under Target Audience (depth 3)");
    println!("   🎯 2 Detail sections under Primary Target (depth 4)");
    println!("   🔗 Full parent-child hierarchical relationships");

    println!("\n📋 Ready for Testing:");
    println!("   • Hierarchical navigation through campaign strategy");
    println!("   • Search across different depth levels");
    println!("   • Cross-modal content discovery");
    println!("   • Professional marketing document structure");

    Ok(())
}

// Content creation functions preserving markdown structure

fn create_launch_overview_content() -> String {
    "- **Product**: EcoSmart Professional Series
- **Launch Date**: July 15, 2025
- **Campaign Duration**: 12 weeks (4 weeks pre-launch, 4 weeks launch, 4 weeks post-launch)
- **Total Budget**: $180,000
- **Primary Objective**: Establish market leadership in sustainable professional products"
        .to_string()
}

fn create_executive_summary_content() -> String {
    "The EcoSmart Professional Series represents our most significant product innovation in three years, combining professional-grade performance with industry-leading sustainability features. This launch campaign will position us as the premium choice for environmentally conscious professionals while maintaining our quality and performance reputation.".to_string()
}

fn create_target_audience_content() -> String {
    "Comprehensive analysis of primary and secondary target segments for the EcoSmart Professional Series launch, including demographic profiles, psychographic characteristics, and market positioning insights.".to_string()
}

fn create_primary_target_content() -> String {
    "Primary target segment consisting of urban professionals aged 28-45 with strong sustainability values and premium purchasing power.".to_string()
}

fn create_secondary_target_content() -> String {
    "- **Segment 2: Sustainability-Focused Organizations**
  - Corporate buyers implementing sustainability initiatives
  - Government agencies with environmental mandates
  - Non-profit organizations with mission alignment
- **Segment 3: Early Adopter Enthusiasts**
  - Technology and innovation enthusiasts
  - Sustainability advocates and influencers
  - Professional reviewers and industry experts"
        .to_string()
}

fn create_demographics_content() -> String {
    "- Age: 28-45 years
- Income: $75,000-$150,000 annually
- Education: College degree or higher (87%)
- Location: Urban and suburban professionals in major metropolitan areas
- Industry Focus: Design, consulting, technology, finance, healthcare"
        .to_string()
}

fn create_psychographic_content() -> String {
    "- Values sustainability and environmental responsibility
- Willing to pay premium for quality and environmental benefits
- Influences others in professional networks
- Active on LinkedIn and Instagram
- Research-intensive purchase behavior"
        .to_string()
}

fn create_positioning_content() -> String {
    "**Core Value Proposition**: \"Professional performance without environmental compromise\" - positioning EcoSmart Professional Series as the only product line that delivers superior professional results while achieving industry-leading sustainability standards.

**Key Differentiators**:
- Performance Excellence: 15% performance improvement over previous generation
- Sustainability Leadership: 75% reduction in environmental impact across lifecycle
- Professional Grade: Meets all professional industry standards and certifications".to_string()
}

fn create_channel_strategy_content() -> String {
    "Three-phase marketing approach spanning pre-launch education, coordinated launch activation, and post-launch optimization across digital, traditional, and partnership channels.

**Pre-Launch Phase (Weeks 1-4)**:
- Content marketing and education campaigns
- Influencer partnerships and industry collaboration
- Digital marketing foundation building

**Launch Phase (Weeks 5-8)**:
- Integrated campaign launch across all channels
- Performance marketing acceleration
- Public relations and earned media activation

**Post-Launch Phase (Weeks 9-12)**:
- Customer success stories and advocacy programs
- Performance optimization and geographic expansion".to_string()
}

fn create_metrics_content() -> String {
    "Comprehensive measurement framework tracking awareness, engagement, conversion, and customer satisfaction metrics.

**Launch Success Indicators**:
- Brand awareness increase of 25% in target demographic within 60 days
- 5,000 units sold in first 60 days
- $850,000 revenue generation in launch quarter
- Product satisfaction score above 4.7/5.0

**Long-Term Success Metrics (6-12 months)**:
- Market share increase to 12% in target professional segment
- Customer lifetime value improvement of 20%
- Repeat purchase rate above 35% within 12 months".to_string()
}

fn create_budget_content() -> String {
    "**Total Campaign Budget**: $180,000

**Marketing Budget Distribution**:
- Digital Advertising: $65,000 (36%)
- Content Creation and Production: $45,000 (25%)
- Influencer and Partnership Marketing: $35,000 (19%)
- Public Relations and Events: $25,000 (14%)
- Marketing Technology and Tools: $10,000 (6%)

**Team Resource Allocation**:
- Campaign Management: 40% of marketing team capacity for 12 weeks
- Performance Marketing: Full-time focus from digital specialists
- Analytics and Optimization: Daily monitoring and weekly optimization cycles"
        .to_string()
}

fn create_embedding(text: &str) -> Vec<f32> {
    use rand::{Rng, SeedableRng};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let seed = hasher.finish();

    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    (0..384).map(|_| rng.gen_range(-1.0..1.0)).collect()
}
