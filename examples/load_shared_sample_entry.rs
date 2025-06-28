//! Load Sample Node Entry into Shared Directory for E2E Testing
//!
//! Creates the Product Launch Campaign Strategy from sample-node-entry.md
//! in the shared /Users/malibio/nodespace/data/lance_db/ directory with proper
//! hierarchical structure based on markdown hyphen depth.

use nodespace_core_types::{Node, NodeId};
use nodespace_data_store::{DataStore, LanceDataStore};
use std::error::Error;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("📋 Loading Sample Node Entry to Shared Directory for E2E Testing\n");

    // Use shared data directory for e2e testing
    let shared_db_path = "/Users/malibio/nodespace/data/lance_db/sample_entry.db";
    let data_store = LanceDataStore::new(shared_db_path).await?;
    println!("✅ LanceDB initialized at: {}", shared_db_path);

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

    // Create the main Product Launch Campaign Strategy node (child of DateNode)
    let main_strategy_id = Uuid::new_v4().to_string();
    let main_strategy = Node::with_id(
        NodeId::from_string(main_strategy_id.clone()),
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
            main_strategy,
            create_embedding("product launch campaign strategy comprehensive plan"),
        )
        .await?;
    println!("✅ Created main strategy node");

    // Level 2: Main sections (## headers)
    let mut section_ids = Vec::new();

    // Launch Overview
    let launch_overview_id = create_section(
        &data_store,
        "Launch Overview",
        "## Launch Overview\n\n- **Product**: EcoSmart Professional Series\n- **Launch Date**: July 15, 2025\n- **Campaign Duration**: 12 weeks (4 weeks pre-launch, 4 weeks launch, 4 weeks post-launch)\n- **Total Budget**: $180,000\n- **Primary Objective**: Establish market leadership in sustainable professional products",
        strategy_id.as_str(),
        2
    ).await?;
    section_ids.push(("Launch Overview".to_string(), launch_overview_id));

    // Executive Summary
    let exec_summary_id = create_section(
        &data_store,
        "Executive Summary",
        "## Executive Summary\n\nThe EcoSmart Professional Series represents our most significant product innovation in three years, combining professional-grade performance with industry-leading sustainability features. This launch campaign will position us as the premium choice for environmentally conscious professionals while maintaining our quality and performance reputation.",
        strategy_id.as_str(),
        2
    ).await?;
    section_ids.push(("Executive Summary".to_string(), exec_summary_id));

    // Target Audience Analysis (will have subsections)
    let target_audience_id = create_section(
        &data_store,
        "Target Audience Analysis",
        "## Target Audience Analysis\n\nComprehensive analysis of primary and secondary target segments for the EcoSmart Professional Series launch.",
        strategy_id.as_str(),
        2
    ).await?;
    section_ids.push((
        "Target Audience Analysis".to_string(),
        target_audience_id.clone(),
    ));

    // Product Positioning Strategy (will have subsections)
    let positioning_id = create_section(
        &data_store,
        "Product Positioning Strategy",
        "## Product Positioning Strategy\n\nStrategic positioning framework for market differentiation and competitive advantage.",
        strategy_id.as_str(),
        2
    ).await?;
    section_ids.push((
        "Product Positioning Strategy".to_string(),
        positioning_id.clone(),
    ));

    // Marketing Channel Strategy (will have subsections)
    let channel_strategy_id = create_section(
        &data_store,
        "Marketing Channel Strategy",
        "## Marketing Channel Strategy\n\nThree-phase marketing approach across pre-launch, launch, and post-launch phases.",
        strategy_id.as_str(),
        2
    ).await?;
    section_ids.push((
        "Marketing Channel Strategy".to_string(),
        channel_strategy_id.clone(),
    ));

    // Success Metrics and KPIs (will have subsections)
    let metrics_id = create_section(
        &data_store,
        "Success Metrics and KPIs",
        "## Success Metrics and KPIs\n\nComprehensive measurement framework for launch success and long-term performance.",
        strategy_id.as_str(),
        2
    ).await?;
    section_ids.push(("Success Metrics and KPIs".to_string(), metrics_id.clone()));

    // Budget Allocation and Resource Planning (will have subsections)
    let budget_id = create_section(
        &data_store,
        "Budget Allocation and Resource Planning",
        "## Budget Allocation and Resource Planning\n\nDetailed budget distribution and team resource allocation for campaign execution.",
        strategy_id.as_str(),
        2
    ).await?;
    section_ids.push((
        "Budget Allocation and Resource Planning".to_string(),
        budget_id.clone(),
    ));

    println!("✅ Created 7 main sections");

    // Level 3: Subsections under Target Audience Analysis
    let mut target_subsection_ids = Vec::new();

    let primary_target_id = create_section(
        &data_store,
        "Primary Target Segment",
        "### Primary Target Segment\n\nPrimary target segment consisting of urban professionals aged 28-45 with strong sustainability values and premium purchasing power.",
        &target_audience_id,
        3
    ).await?;
    target_subsection_ids.push((
        "Primary Target Segment".to_string(),
        primary_target_id.clone(),
    ));

    let secondary_target_id = create_section(
        &data_store,
        "Secondary Target Segments",
        "### Secondary Target Segments\n\n- **Segment 2: Sustainability-Focused Organizations**\n  - Corporate buyers implementing sustainability initiatives\n  - Government agencies with environmental mandates\n  - Non-profit organizations with mission alignment\n- **Segment 3: Early Adopter Enthusiasts**\n  - Technology and innovation enthusiasts\n  - Sustainability advocates and influencers\n  - Professional reviewers and industry experts",
        &target_audience_id,
        3
    ).await?;
    target_subsection_ids.push(("Secondary Target Segments".to_string(), secondary_target_id));

    println!("   📋 Created 2 subsections under Target Audience Analysis");

    // Level 4: Details under Primary Target Segment
    let demographics_id = create_section(
        &data_store,
        "Professional Demographics",
        "**Professional Demographics**:\n- Age: 28-45 years\n- Income: $75,000-$150,000 annually\n- Education: College degree or higher (87%)\n- Location: Urban and suburban professionals in major metropolitan areas\n- Industry Focus: Design, consulting, technology, finance, healthcare",
        &primary_target_id,
        4
    ).await?;

    let psychographic_id = create_section(
        &data_store,
        "Psychographic Profile",
        "**Psychographic Profile**:\n- Values sustainability and environmental responsibility\n- Willing to pay premium for quality and environmental benefits\n- Influences others in professional networks\n- Active on LinkedIn and Instagram\n- Research-intensive purchase behavior",
        &primary_target_id,
        4
    ).await?;

    println!("      🎯 Created 2 detail sections under Primary Target Segment");

    // Level 3: Subsections under Product Positioning Strategy
    let core_value_id = create_section(
        &data_store,
        "Core Value Proposition",
        "### Core Value Proposition\n\n\"Professional performance without environmental compromise\" - positioning EcoSmart Professional Series as the only product line that delivers superior professional results while achieving industry-leading sustainability standards.",
        &positioning_id,
        3
    ).await?;

    let differentiators_id = create_section(
        &data_store,
        "Key Differentiators",
        "### Key Differentiators\n\n- **Performance Excellence**: 15% performance improvement over previous generation\n- **Sustainability Leadership**: 75% reduction in environmental impact across lifecycle\n- **Professional Grade**: Meets all professional industry standards and certifications\n- **Innovation Recognition**: Featured in leading industry publications and awards",
        &positioning_id,
        3
    ).await?;

    let competitive_id = create_section(
        &data_store,
        "Competitive Positioning",
        "### Competitive Positioning\n\n- **Versus Premium Competitors**: Superior sustainability without performance sacrifice\n- **Versus Sustainable Alternatives**: Professional-grade performance they cannot match\n- **Versus Mass Market**: Premium quality and environmental leadership justify price difference",
        &positioning_id,
        3
    ).await?;

    println!("   📋 Created 3 subsections under Product Positioning Strategy");

    // Level 3: Subsections under Marketing Channel Strategy
    let prelaunch_id = create_section(
        &data_store,
        "Pre-Launch Phase (Weeks 1-4)",
        "### Pre-Launch Phase (Weeks 1-4)\n\n**Content Marketing and Education**:\n- Educational blog series on sustainability in professional environments\n- Webinar series featuring industry experts and environmental scientists\n- Behind-the-scenes content showing product development and testing\n\n**Influencer and Partnership Strategy**:\n- Partner with 15 industry professionals for authentic product testing\n- Collaborate with sustainability experts for credibility and education\n- Engage professional associations and industry organizations",
        &channel_strategy_id,
        3
    ).await?;

    let launch_phase_id = create_section(
        &data_store,
        "Launch Phase (Weeks 5-8)",
        "### Launch Phase (Weeks 5-8)\n\n**Integrated Campaign Launch**:\n- Coordinated announcement across all digital and traditional channels\n- Press release distribution to industry and sustainability publications\n- Social media campaign with hashtag #ProfessionalWithoutCompromise\n\n**Performance Marketing Acceleration**:\n- Paid search campaigns targeting professional and sustainability keywords\n- Social media advertising with video demonstrations and customer testimonials\n- Display advertising on professional and industry websites",
        &channel_strategy_id,
        3
    ).await?;

    let postlaunch_id = create_section(
        &data_store,
        "Post-Launch Phase (Weeks 9-12)",
        "### Post-Launch Phase (Weeks 9-12)\n\n**Customer Success and Advocacy**:\n- Customer success stories and case study development\n- User-generated content campaigns encouraging professional usage sharing\n- Customer testimonial collection and amplification across channels\n\n**Performance Optimization and Scale**:\n- Campaign performance analysis and budget optimization toward highest-performing channels\n- Creative testing and optimization based on engagement and conversion data",
        &channel_strategy_id,
        3
    ).await?;

    println!("   📋 Created 3 subsections under Marketing Channel Strategy");

    // Test hierarchical data retrieval
    println!("\n🔍 Testing Hierarchical Data Access...");

    // Test search functionality
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

    println!("\n🎉 Sample Node Entry Loaded Successfully!");
    println!("📍 Location: {}", shared_db_path);
    println!("📈 Hierarchical Structure Created:");
    println!("   📅 DateNode: {} (today)", today);
    println!("   📄 Main Document: Product Launch Campaign Strategy (depth 1)");
    println!("   📑 7 Main Sections (depth 2)");
    println!("   📋 Subsections (depth 3):");
    println!("     • Target Audience Analysis: 2 subsections");
    println!("     • Product Positioning Strategy: 3 subsections");
    println!("     • Marketing Channel Strategy: 3 subsections");
    println!("   🎯 Detail sections (depth 4):");
    println!("     • Primary Target Segment: 2 detail sections");
    println!("   🔗 Full parent-child relationships preserved");

    println!("\n📋 Ready for E2E Testing:");
    println!("   • Cross-component hierarchical navigation");
    println!("   • Multi-level content search and discovery");
    println!("   • Professional marketing document structure");
    println!("   • Markdown formatting preserved at all levels");

    Ok(())
}

async fn create_section(
    data_store: &LanceDataStore,
    title: &str,
    content: &str,
    parent_id: &str,
    depth: u32,
) -> Result<String, Box<dyn Error>> {
    let section_id = Uuid::new_v4().to_string();
    let section_node = Node::with_id(
        NodeId::from_string(section_id.clone()),
        serde_json::Value::String(content.to_string()),
    )
    .with_metadata(serde_json::json!({
        "node_type": "text",
        "title": title,
        "parent_id": parent_id,
        "depth": depth,
        "section_type": match depth {
            2 => "main_section",
            3 => "subsection",
            4 => "detail",
            _ => "section"
        }
    }));

    let stored_id = data_store
        .store_node_with_embedding(
            section_node,
            create_embedding(&format!("{} {}", title, content)),
        )
        .await?;

    Ok(stored_id.to_string())
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
