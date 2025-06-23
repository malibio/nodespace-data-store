use nodespace_data_store::SurrealDataStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating sample marketing data for NodeSpace Data Store...");

    // Initialize the data store
    let store = SurrealDataStore::new("./data/sample.db").await?;

    // Create sample data for a fictitious marketing person across last 2-3 weeks (June 1-23, 2025)

    // June 1, 2025 - Campaign Planning Day
    let june_1_date = store
        .create_or_get_date_node("2025-06-01", Some("Campaign Planning Day"))
        .await?;

    store.create_text_node(
        "Q3 Campaign Strategy Deep Dive: Today I spent 3 hours analyzing our competitor landscape. Key findings: 1) Competitor A is focusing heavily on AI-driven personalization, 2) Competitor B just launched a major rebrand with emphasis on sustainability, 3) Our positioning around 'human-centered technology' is still unique in the market. Need to leverage this more aggressively in Q3 campaigns.",
        Some(&june_1_date)
    ).await?;

    store.create_text_node(
        "• Finalize Q3 budget allocation\n• Schedule creative review with design team\n• Research influencer partnerships for tech sector\n• Competitive analysis deep-dive (DONE)",
        Some(&june_1_date)
    ).await?;

    // June 3, 2025 - Client Meeting Day
    let june_3_date = store
        .create_or_get_date_node("2025-06-03", Some("Client Meetings"))
        .await?;

    store.create_text_node(
        "Client meeting with Acme Corp went well. Key takeaways: they're interested in digital transformation messaging, budget approved for $250K campaign, want to target C-suite executives in manufacturing. Timeline: campaign launch by July 15th. Next steps: create personas, develop messaging framework, present initial concepts by June 10th.",
        Some(&june_3_date)
    ).await?;

    store.create_text_node(
        "• Call Sarah about budget approval ✓\n• Review creative assets\n• Update stakeholder deck\n• Send Acme Corp follow-up email ✓",
        Some(&june_3_date)
    ).await?;

    // June 5, 2025 - Team Strategy Session
    let june_5_date = store
        .create_or_get_date_node("2025-06-05", Some("Team Strategy"))
        .await?;

    store.create_text_node(
        "Team brainstorm session for TechFlow campaign. Generated 12 concept directions. Top 3: 1) 'Future-Ready Manufacturing' - focuses on AI integration, 2) 'Human + Machine Excellence' - emphasizes collaboration, 3) 'Transform Today, Lead Tomorrow' - urgency-driven messaging. Team consensus: concept #2 aligns best with brand values and client objectives.",
        Some(&june_5_date)
    ).await?;

    store
        .create_text_node("Remember to follow up on:", Some(&june_5_date))
        .await?;

    // June 8, 2025 - Content Creation Sprint
    let june_8_date = store
        .create_or_get_date_node("2025-06-08", Some("Content Sprint"))
        .await?;

    store.create_text_node(
        "Content creation day. Wrote 3 blog post outlines, 15 social media posts, and email sequence for nurture campaign. Blog topics: 'The ROI of Manufacturing AI', 'Building Change-Ready Teams', 'Data-Driven Decision Making in Industry 4.0'. Social posts focus on thought leadership and behind-the-scenes content.",
        Some(&june_8_date)
    ).await?;

    store.create_text_node(
        "Performance metrics from last campaign:\n• Email open rate: 24.3% (industry avg: 21%)\n• Click-through rate: 3.1% (industry avg: 2.3%)\n• Lead conversion: 8.7%\n• Customer acquisition cost: $127\n\nStrong performance across all metrics. Scale similar approach for Q3.",
        Some(&june_8_date)
    ).await?;

    // June 12, 2025 - Market Research
    let june_12_date = store
        .create_or_get_date_node("2025-06-12", Some("Market Research"))
        .await?;

    store.create_text_node(
        "Industry report insights: Manufacturing sector showing 23% increased interest in AI solutions compared to last year. Key pain points: 1) Skills gap in digital literacy, 2) Integration complexity with legacy systems, 3) ROI measurement challenges. Our messaging should address these directly. Opportunity: position as the 'practical AI partner' that solves real problems.",
        Some(&june_12_date)
    ).await?;

    // June 15, 2025 - Campaign Execution
    let june_15_date = store
        .create_or_get_date_node("2025-06-15", Some("Campaign Launch"))
        .await?;

    store.create_text_node(
        "Launched 'Smart Manufacturing Series' webinar campaign. Registration page live, promoted across LinkedIn, industry publications, and email list. Initial response strong: 127 registrations in first 24 hours. Target: 500 registrations by June 30th.",
        Some(&june_15_date)
    ).await?;

    store.create_text_node(
        "Daily standup notes:\n• Creative team: finalizing webinar slides\n• Demand gen: email sequence performing at 2.8% CTR\n• Events: confirmed 3 industry speakers\n• PR: secured mention in ManufacturingTech Weekly",
        Some(&june_15_date)
    ).await?;

    // June 18, 2025 - Analytics Review
    let june_18_date = store
        .create_or_get_date_node("2025-06-18", Some("Analytics & Optimization"))
        .await?;

    store.create_text_node(
        "Mid-campaign analytics review. Webinar series performing above expectations: 342 registrations (68% of goal), 12% show-up rate typical for industry. Top-performing content: 'AI ROI Calculator' landing page (4.2% conversion), LinkedIn video series (187 shares). Need to double down on video content - clearly resonating with audience.",
        Some(&june_18_date)
    ).await?;

    // June 21, 2025 - Strategy Planning
    let june_21_date = store
        .create_or_get_date_node("2025-06-21", Some("Q3 Strategy"))
        .await?;

    store.create_text_node(
        "Q3 planning session with leadership team. Approved budget increase to $1.2M based on strong Q2 performance. New priorities: 1) Expand into European markets, 2) Develop partner channel program, 3) Launch customer advocacy initiative. Timeline aggressive but achievable with current team velocity.",
        Some(&june_21_date)
    ).await?;

    store.create_text_node(
        "• Research European market entry requirements\n• Draft partner enablement materials\n• Identify customer advocacy candidates\n• Competitive analysis for EU market\n• Budget allocation for international campaigns",
        Some(&june_21_date)
    ).await?;

    // June 23, 2025 - Today
    let june_23_date = store
        .create_or_get_date_node("2025-06-23", Some("Current Focus"))
        .await?;

    store.create_text_node(
        "Week in review: Webinar series exceeded registration goal (547 total), customer case study interviews completed for 3 major accounts, new brand guidelines approved by executive team. Next week focus: finalize European go-to-market strategy, launch partner recruitment campaign, begin customer advocacy program development.",
        Some(&june_23_date)
    ).await?;

    // Test queries to demonstrate date-based filtering
    println!("\\nTesting date-based queries:");

    let june_1_nodes = store.get_nodes_for_date("2025-06-01").await?;
    println!("June 1st nodes: {}", june_1_nodes.len());

    let june_15_nodes = store.get_nodes_for_date("2025-06-15").await?;
    println!("June 15th nodes: {}", june_15_nodes.len());

    let june_23_children = store.get_date_children(&june_23_date).await?;
    println!("June 23rd children: {}", june_23_children.len());

    println!("\\nSample marketing data created successfully!");
    println!(
        "Generated realistic content across {} dates with hierarchical relationships",
        9
    );

    Ok(())
}
