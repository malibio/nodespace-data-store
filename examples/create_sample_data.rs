use chrono::{DateTime, Datelike, Duration, Utc};
use nodespace_data_store::SurrealDataStore;
use rand::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating comprehensive sample marketing data for NodeSpace Data Store...");
    println!("Generating approximately 300 entries across 100 days...");

    // Initialize the data store
    let store = SurrealDataStore::new("./data/sample.db").await?;

    // Generate sample data across ~100 days (March 15 - June 23, 2025) with ~3 entries per day
    let start_date = DateTime::parse_from_rfc3339("2025-03-15T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let end_date = DateTime::parse_from_rfc3339("2025-06-23T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    let mut rng = thread_rng();
    let mut current_date = start_date;
    let mut total_entries = 0;

    while current_date <= end_date {
        let date_str = current_date.format("%Y-%m-%d").to_string();

        // Skip some days randomly to make it more realistic (weekends, etc.)
        if rng.gen_bool(0.15) {
            current_date = current_date + Duration::days(1);
            continue;
        }

        // Generate 2-4 entries per active day
        let entries_count = rng.gen_range(2..=4);

        // Create date node with contextual description
        let _date_node = store
            .create_or_get_date_node(&date_str, Some(&get_date_context(&current_date, &mut rng)))
            .await?;

        // Generate diverse content for this date
        for _ in 0..entries_count {
            let content = generate_marketing_content(&current_date, &mut rng);
            store.create_text_node(&content, Some(&date_str)).await?;
            total_entries += 1;
        }

        current_date = current_date + Duration::days(1);
    }

    // Test queries to demonstrate date-based filtering
    println!("\nTesting date-based queries:");

    let sample_date_nodes = store.get_nodes_for_date("2025-04-15").await?;
    println!("April 15th nodes: {}", sample_date_nodes.len());

    let june_nodes = store.get_nodes_for_date("2025-06-01").await?;
    println!("June 1st nodes: {}", june_nodes.len());

    println!("\nComprehensive sample marketing data created successfully!");
    println!(
        "Generated {} entries across ~100 days with hierarchical relationships",
        total_entries
    );
    println!("Data spans from March 15, 2025 to June 23, 2025");

    Ok(())
}

fn get_date_context(date: &DateTime<Utc>, rng: &mut ThreadRng) -> String {
    let contexts = vec![
        "Campaign Planning",
        "Client Meetings",
        "Team Strategy",
        "Content Sprint",
        "Market Research",
        "Analytics Review",
        "Creative Sessions",
        "Budget Planning",
        "Competitive Analysis",
        "Product Launch",
        "Webinar Prep",
        "Social Media Planning",
        "Email Campaigns",
        "Lead Generation",
        "Brand Strategy",
        "Partnership Meetings",
        "Industry Events",
        "Content Creation",
        "Data Analysis",
        "Strategy Review",
        "Customer Research",
        "Performance Review",
        "Innovation Workshop",
        "Team Sync",
        "Stakeholder Updates",
        "Creative Review",
        "Campaign Optimization",
        "Trend Analysis",
    ];

    let weekday = date.weekday();
    let is_monday = weekday.number_from_monday() == 1;
    let is_friday = weekday.number_from_monday() == 5;

    if is_monday {
        "Weekly Planning".to_string()
    } else if is_friday {
        "Week Wrap-up".to_string()
    } else {
        contexts.choose(rng).unwrap().to_string()
    }
}

fn generate_marketing_content(date: &DateTime<Utc>, rng: &mut ThreadRng) -> String {
    let content_types = vec![
        generate_strategy_content,
        generate_campaign_content,
        generate_analytics_content,
        generate_client_content,
        generate_team_content,
        generate_creative_content,
        generate_research_content,
        generate_task_list_content,
        generate_meeting_notes,
        generate_performance_metrics,
    ];

    let generator = content_types.choose(rng).unwrap();
    generator(date, rng)
}

fn generate_strategy_content(_date: &DateTime<Utc>, rng: &mut ThreadRng) -> String {
    let strategies = vec![
        "Quarterly planning session revealed key insights: market showing 34% growth in B2B SaaS segment. Competitors focusing on AI integration while we maintain human-centered approach. Recommendation: double down on personalization while highlighting our unique positioning.",
        "Strategic pivot discussion: moving from feature-first to outcome-first messaging. Early testing shows 23% improvement in conversion rates. Next steps: roll out across all campaigns by month-end.",
        "Market positioning workshop outcomes: identified three key differentiators that resonate with target audience. Primary message testing scheduled for next week with focus groups.",
        "Competitive landscape analysis complete. Main threat: Competitor X's aggressive pricing strategy. Counter-strategy: emphasize value and long-term ROI in all communications.",
        "Brand strategy evolution: shifting from 'technology leader' to 'business transformation partner'. Internal alignment achieved with leadership team. External rollout planned for Q3.",
        "Go-to-market strategy refinement: identified new vertical opportunity in healthcare sector. Initial research shows 45% untapped market potential.",
        "Strategic partnership evaluation: three potential collaborations identified. Each offers different market access benefits. Detailed analysis and recommendations prepared for leadership review.",
    ];
    strategies.choose(rng).unwrap().to_string()
}

fn generate_campaign_content(_date: &DateTime<Utc>, rng: &mut ThreadRng) -> String {
    let campaigns = vec![
        "Launched multi-channel campaign 'Future-Ready Business'. Week 1 results: 1,247 leads generated, 23% above target. LinkedIn performing best with 4.2% CTR, followed by email at 2.8%.",
        "Campaign optimization review: A/B testing landing page headlines. Version B ('Transform Your Business Today') outperforming Version A by 31%. Rolling out winning version across all channels.",
        "Product launch campaign timeline finalized. Media blitz scheduled for week 3, influencer partnerships confirmed, PR strategy approved. Budget allocation: 40% digital, 30% events, 30% content.",
        "Email nurture sequence performance strong: Open rate 28.4% (industry avg 22%), click rate 3.7% (industry avg 2.4%). Top performing subject line: 'Your competitors are already ahead'.",
        "Social media campaign momentum building: 2.3M impressions this month, 45K engagements. User-generated content strategy driving 23% of total engagement. Scaling for next quarter.",
        "Webinar series 'Industry Insights' exceeded expectations: 847 registrations, 34% attendance rate, 67 qualified leads generated. Planning follow-up series for Q3.",
        "Retargeting campaign optimizations complete: cost per acquisition down 22%, conversion rate up 18%. Expanding successful segments and pausing underperforming creative.",
    ];
    campaigns.choose(rng).unwrap().to_string()
}

fn generate_analytics_content(_date: &DateTime<Utc>, rng: &mut ThreadRng) -> String {
    let analytics = vec![
        "Monthly performance dashboard review: MQLs up 34%, SQLs up 28%, pipeline value increased by $2.3M. Marketing attribution showing strong contribution across all channels.",
        "Conversion funnel analysis reveals optimization opportunity at awareness-to-consideration stage. Implementing content strategy to address 23% drop-off point.",
        "Website analytics deep dive: organic traffic up 45% YoY, direct traffic up 23%. Blog content driving 34% of all conversions. Doubling content production budget.",
        "Campaign ROI analysis complete: paid search delivering 4.2x return, social advertising at 2.8x, content marketing showing long-term value with 6-month attribution window.",
        "Customer acquisition cost trending down 18% this quarter due to improved targeting and creative optimization. Expanding successful channel mix for next quarter.",
        "Attribution modeling update: multi-touch attribution shows content marketing influences 67% of closed deals, even when not last-touch. Adjusting budget allocation accordingly.",
        "Performance benchmarking against industry standards: email marketing performing 23% above average, social engagement 34% above, conversion rates meeting top quartile.",
    ];
    analytics.choose(rng).unwrap().to_string()
}

fn generate_client_content(_date: &DateTime<Utc>, rng: &mut ThreadRng) -> String {
    let clients = vec![
        "Client meeting with TechCorp exceeded expectations. Approved additional $150K budget for Q3 expansion. Key requirement: focus on ROI measurement and business impact metrics.",
        "Presentation to MegaCorp board went well. Three key stakeholders aligned on messaging strategy. Next steps: develop executive briefing materials and schedule follow-up in two weeks.",
        "Customer success story interview with GlobalTech revealed unexpected use case. Their efficiency gains were 40% higher than anticipated. Developing case study for broader marketing use.",
        "Client feedback session highlighted need for more technical content. Engineering team collaboration increasing. Developing technical blog series to address knowledge gap.",
        "Account review with Enterprise Solutions showed 23% growth opportunity. Upsell campaign planned for next month focusing on advanced features and premium support.",
        "New client onboarding session for InnovateCorp. Strong enthusiasm for partnership approach. Customized campaign strategy developed to address their unique market position.",
        "Quarterly business review with key accounts revealed satisfaction score of 8.7/10. Two accounts expressing interest in case study participation. Planning video testimonial series.",
    ];
    clients.choose(rng).unwrap().to_string()
}

fn generate_team_content(_date: &DateTime<Utc>, rng: &mut ThreadRng) -> String {
    let team_content = vec![
        "Team retrospective identified key process improvements: streamlining approval workflow reduced campaign launch time by 3 days. Implementing across all future campaigns.",
        "Hired new creative director with 8 years B2B experience. Team capacity increased 40%. Planning ambitious Q3 creative campaign lineup with enhanced visual storytelling.",
        "Cross-functional workshop with sales team revealed messaging gaps. Developed unified talk track that addresses top 5 customer objections. Sales adoption scheduled for next week.",
        "Training session on new marketing automation platform complete. Team productivity expected to increase 25%. Advanced workflow implementations planned for next month.",
        "Brainstorming session generated 23 content ideas for thought leadership campaign. Selected top 8 for development. Executive interviews scheduled to support content creation.",
        "Team building exercise revealed collaboration strengths and communication opportunities. Implementing weekly sync meetings and shared project dashboard for better alignment.",
        "Performance review cycle complete. Team morale high, skill development goals identified. Budget approved for 3 team members to attend industry conference next quarter.",
    ];
    team_content.choose(rng).unwrap().to_string()
}

fn generate_creative_content(_date: &DateTime<Utc>, rng: &mut ThreadRng) -> String {
    let creative = vec![
        "Creative review session produced 5 strong concept directions. Client preference for minimalist approach with bold typography. Design phase begins Monday with 2-week timeline.",
        "Brand refresh project milestone: logo variations complete, color palette finalized, typography system established. Brand guidelines document 80% complete.",
        "Video production wrap-up: 3 customer testimonials in post-production, 1 product demo edited and approved. Social media cuts being prepared for multi-platform distribution.",
        "Photoshoot for Q3 campaign assets delivered exceptional results. 47 approved images covering all use cases. Creative library updated with new brand-compliant photography.",
        "Interactive content experiment: calculator tool development 60% complete. Early user testing shows high engagement. Launch planned for mid-month with email campaign support.",
        "Creative A/B testing results: emotional storytelling approach outperforming feature-focused creative by 34%. Adjusting creative strategy for upcoming campaigns.",
        "Design system documentation update complete. Component library expanded with 12 new elements. Development team integration scheduled to streamline future campaign production.",
    ];
    creative.choose(rng).unwrap().to_string()
}

fn generate_research_content(_date: &DateTime<Utc>, rng: &mut ThreadRng) -> String {
    let research = vec![
        "Market research findings: target audience spending 34% more time on LinkedIn vs. other professional platforms. Reallocating social media budget to maximize reach.",
        "Competitive intelligence update: 3 major competitors launching similar products Q3. Differentiation strategy urgent priority. Unique value proposition workshop scheduled.",
        "Customer persona research revealed new segment: technical evaluators with different content preferences. Developing specialized content track for this audience.",
        "Industry trend analysis: remote work driving 45% increase in demand for collaboration tools. Adjusting messaging to emphasize distributed team capabilities.",
        "Voice of customer survey results: satisfaction with onboarding process identified as key differentiator. Developing customer success stories highlighting implementation experience.",
        "Buyer journey mapping exercise uncovered 2 previously unknown touchpoints. Attribution model updates required to properly track multi-channel customer paths.",
        "Thought leadership research: identified 5 emerging topics with low competition but high audience interest. Content calendar updated to capitalize on opportunity.",
    ];
    research.choose(rng).unwrap().to_string()
}

fn generate_task_list_content(_date: &DateTime<Utc>, rng: &mut ThreadRng) -> String {
    let tasks = vec![
        "• Finalize Q3 budget allocation ✓\n• Review creative assets\n• Update stakeholder presentation\n• Schedule client check-in calls\n• Competitive analysis deep-dive",
        "• Email campaign segmentation\n• Landing page optimization\n• Social media content calendar ✓\n• Partnership agreement review\n• Team one-on-one meetings ✓",
        "• Website analytics audit\n• Customer interview scheduling ✓\n• Brand guideline updates\n• Campaign performance report\n• Industry event planning",
        "• Content production timeline\n• Lead scoring model review ✓\n• Sales collateral updates\n• Marketing automation setup\n• Vendor contract negotiations",
        "• Product launch checklist\n• PR strategy development ✓\n• Influencer outreach program\n• Event logistics coordination\n• Budget reconciliation ✓",
        "• Customer success metrics\n• Creative brief development\n• Channel partner enablement ✓\n• Conversion optimization\n• Team training schedule",
        "• Market research synthesis ✓\n• Campaign attribution analysis\n• Content gap assessment\n• Technology stack evaluation\n• Performance dashboard update",
    ];
    tasks.choose(rng).unwrap().to_string()
}

fn generate_meeting_notes(_date: &DateTime<Utc>, rng: &mut ThreadRng) -> String {
    let meetings = vec![
        "Leadership sync meeting: Approved expansion into European markets with $500K initial budget. Timeline: soft launch Q4, full rollout Q1 next year. Market research phase begins immediately.",
        "Customer advisory board session revealed 3 key product enhancement requests. Development roadmap adjusted to prioritize integration capabilities. Customer co-development opportunity identified.",
        "Sales and marketing alignment meeting: lead quality scoring system updated, handoff process streamlined. Weekly pipeline review meetings implemented starting next Monday.",
        "Agency partnership evaluation: 2 finalists selected for creative partnership. Final presentations scheduled for next week. Decision criteria: creativity, strategic thinking, cultural fit.",
        "Budget planning workshop: Q4 allocation approved with 15% increase over previous quarter. Investment priorities: content production, marketing technology, team expansion.",
        "Crisis communications planning session: response protocols established, spokesperson training scheduled. Brand protection strategies developed for potential market scenarios.",
        "Innovation lab brainstorming: identified 5 experimental marketing channels worth testing. Pilot budget approved for 2 initiatives. Success metrics and timelines established.",
    ];
    meetings.choose(rng).unwrap().to_string()
}

fn generate_performance_metrics(_date: &DateTime<Utc>, rng: &mut ThreadRng) -> String {
    let metrics = vec![
        "Monthly KPI review:\n• Lead generation: 2,347 (23% above target)\n• Cost per lead: $47 (18% below target)\n• Conversion rate: 3.4% (industry avg: 2.8%)\n• Pipeline value: $1.2M\n• Marketing ROI: 4.2x",
        "Campaign performance dashboard:\n• Email open rate: 26.8% ↗\n• Social engagement rate: 4.1% ↗\n• Website conversion rate: 2.9% ↗\n• Organic traffic growth: 34% YoY\n• Brand awareness lift: 12%",
        "Channel attribution analysis:\n• Organic search: 34% of pipeline\n• Direct traffic: 28% of pipeline\n• Social media: 18% of pipeline\n• Paid advertising: 12% of pipeline\n• Referrals: 8% of pipeline",
        "Customer acquisition metrics:\n• Average deal size: $23,400 ↗\n• Sales cycle length: 87 days ↘\n• Customer lifetime value: $89,200\n• Churn rate: 3.2% ↘\n• Net promoter score: 67",
        "Content performance analysis:\n• Blog traffic: 45,678 monthly visitors\n• Average session duration: 3:42\n• Content engagement rate: 8.7%\n• Download conversion rate: 12.3%\n• Video completion rate: 76%",
        "Lead quality assessment:\n• Marketing qualified leads: 856\n• Sales qualified leads: 234\n• MQL to SQL conversion: 27.3%\n• SQL to customer conversion: 18.9%\n• Average lead score: 73/100",
        "Digital marketing metrics:\n• Cost per click: $2.34 ↘\n• Click-through rate: 3.8% ↗\n• Landing page conversion: 11.2%\n• Email list growth: 234 new subscribers\n• Social media followers: +1,247",
    ];
    metrics.choose(rng).unwrap().to_string()
}
