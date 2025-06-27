//! Load Diverse Sample Data - Multiple Business Documents
//! 
//! Creates various business documents across different dates with hierarchical structure,
//! markdown formatting, and emojis for realistic testing scenarios.

use nodespace_data_store::{LanceDataStore, DataStore};
use nodespace_core_types::{Node, NodeId};
use std::error::Error;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("📚 Loading Diverse Sample Data - Multiple Business Documents\n");

    let data_store = LanceDataStore::new("data/diverse_sample.db").await?;
    println!("✅ LanceDB initialized");

    // TODAY (2025-06-27) - Multiple documents on same date
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    
    // Create today's DateNode
    let today_node = Node::with_id(
        NodeId::from_string(today.clone()),
        serde_json::Value::String(format!("📅 {} - Strategic Planning & Policy Updates", today))
    ).with_metadata(serde_json::json!({
        "node_type": "date",
        "date": today,
        "content_type": "date_container"
    }));

    let today_id = data_store.store_node_with_embedding(
        today_node,
        create_embedding("strategic planning policy updates")
    ).await?;
    println!("✅ Created DateNode: {}", today);

    // 🏢 HR Policy Update (same date as Product Launch)
    create_hr_policy_document(&data_store, &today).await?;
    
    // 👥 Weekly Team Standup (same date)
    create_team_standup_document(&data_store, &today).await?;

    // 💰 Q3 Budget Review (same date) 
    create_budget_review_document(&data_store, &today).await?;

    // YESTERDAY (2025-06-26) - Client meetings
    let yesterday = "2025-06-26";
    let yesterday_node = Node::with_id(
        NodeId::from_string(yesterday.to_string()),
        serde_json::Value::String("📅 2025-06-26 - Client Engagement & Partnerships".to_string())
    ).with_metadata(serde_json::json!({
        "node_type": "date",
        "date": yesterday,
        "content_type": "date_container"
    }));

    data_store.store_node_with_embedding(
        yesterday_node,
        create_embedding("client engagement partnerships")
    ).await?;
    println!("✅ Created DateNode: {}", yesterday);

    // 🤝 Client Partnership Meeting
    create_client_meeting_document(&data_store, yesterday).await?;

    // TOMORROW (2025-06-28) - Project retrospectives
    let tomorrow = "2025-06-28";
    let tomorrow_node = Node::with_id(
        NodeId::from_string(tomorrow.to_string()),
        serde_json::Value::String("📅 2025-06-28 - Project Reviews & Team Development".to_string())
    ).with_metadata(serde_json::json!({
        "node_type": "date",
        "date": tomorrow,
        "content_type": "date_container"
    }));

    data_store.store_node_with_embedding(
        tomorrow_node,
        create_embedding("project reviews team development")
    ).await?;
    println!("✅ Created DateNode: {}", tomorrow);

    // 🔄 Project Retrospective
    create_project_retrospective_document(&data_store, tomorrow).await?;

    // NEXT WEEK (2025-07-01) - Planning sessions
    let next_week = "2025-07-01";
    let next_week_node = Node::with_id(
        NodeId::from_string(next_week.to_string()),
        serde_json::Value::String("📅 2025-07-01 - Quarterly Planning & Goal Setting".to_string())
    ).with_metadata(serde_json::json!({
        "node_type": "date",
        "date": next_week,
        "content_type": "date_container"
    }));

    data_store.store_node_with_embedding(
        next_week_node,
        create_embedding("quarterly planning goal setting")
    ).await?;
    println!("✅ Created DateNode: {}", next_week);

    // 🎯 Quarterly Planning Session
    create_quarterly_planning_document(&data_store, next_week).await?;

    // Test diverse search scenarios
    println!("\n🔍 Testing Diverse Search Scenarios...");
    
    use nodespace_data_store::NodeType;
    
    // Search for HR-related content
    let hr_results = data_store.search_multimodal(
        create_embedding("remote work policy guidelines"),
        vec![NodeType::Text]
    ).await?;
    println!("   🏢 HR/Policy search: {} results", hr_results.len());

    // Search for team/meeting content
    let team_results = data_store.search_multimodal(
        create_embedding("team standup sprint planning"),
        vec![NodeType::Text]
    ).await?;
    println!("   👥 Team/Meeting search: {} results", team_results.len());

    // Search for financial content
    let budget_results = data_store.search_multimodal(
        create_embedding("budget revenue expenses quarterly"),
        vec![NodeType::Text]
    ).await?;
    println!("   💰 Budget/Financial search: {} results", budget_results.len());

    println!("\n🎉 Diverse Sample Data Loaded Successfully!");
    println!("📈 Dataset Summary:");
    println!("   📅 4 dates with varied business content");
    println!("   📄 6 different document types");
    println!("   🏢 HR policies with compliance guidelines");
    println!("   👥 Team meetings with action items");
    println!("   💰 Financial reviews with metrics");
    println!("   🤝 Client partnership discussions");
    println!("   🔄 Project retrospectives with lessons learned");
    println!("   🎯 Strategic planning with quarterly goals");
    println!("   ✨ Rich markdown formatting with emojis");

    Ok(())
}

async fn create_hr_policy_document(data_store: &LanceDataStore, date: &str) -> Result<(), Box<dyn Error>> {
    let doc_id = Uuid::new_v4().to_string();
    let main_doc = Node::with_id(
        NodeId::from_string(doc_id.clone()),
        serde_json::Value::String(
            "# 🏢 Remote Work Policy Update\n\nComprehensive update to our remote work guidelines, effective immediately, addressing hybrid work arrangements and digital collaboration standards.".to_string()
        )
    ).with_metadata(serde_json::json!({
        "node_type": "text",
        "title": "Remote Work Policy Update",
        "parent_date": date,
        "depth": 1,
        "document_type": "hr_policy"
    }));

    data_store.store_node_with_embedding(
        main_doc,
        create_embedding("remote work policy hybrid collaboration guidelines")
    ).await?;

    // Policy sections
    let sections = vec![
        ("Eligibility Criteria", "## 📋 Eligibility Criteria\n\n- **Role Requirements**: Position must be suitable for remote work 🏠\n- **Performance Standards**: Meets or exceeds performance expectations ⭐\n- **Equipment Access**: Has reliable internet and necessary tech tools 💻\n- **Communication Skills**: Demonstrates strong written and verbal communication 📞"),
        
        ("Work Arrangements", "## ⏰ Work Arrangements\n\n### Hybrid Options\n- **Flexible Hybrid**: 2-3 days in office, remainder remote 🔄\n- **Remote-First**: Primary remote with monthly office visits 🌐\n- **Project-Based**: In-office during collaborative phases 🤝\n\n### Core Hours\n- **Team Overlap**: 10:00 AM - 3:00 PM local time ⏰\n- **Meeting Windows**: Tuesday/Thursday 2:00-4:00 PM for all-hands 📅"),
        
        ("Technology Requirements", "## 💻 Technology Requirements\n\n- **Secure VPN**: Mandatory for all remote connections 🔒\n- **Communication Tools**: Slack, Zoom, Google Workspace 📱\n- **Time Tracking**: Clockify for project time management ⏱️\n- **Security Training**: Quarterly cybersecurity certification 🛡️"),
    ];

    for (title, content) in sections {
        let section_id = Uuid::new_v4().to_string();
        let section_node = Node::with_id(
            NodeId::from_string(section_id),
            serde_json::Value::String(content.to_string())
        ).with_metadata(serde_json::json!({
            "node_type": "text",
            "title": title,
            "parent_id": doc_id,
            "depth": 2,
            "section_type": "policy_section"
        }));

        data_store.store_node_with_embedding(
            section_node,
            create_embedding(&format!("{} {}", title, content))
        ).await?;
    }

    println!("   🏢 Created HR Policy Update with 3 sections");
    Ok(())
}

async fn create_team_standup_document(data_store: &LanceDataStore, date: &str) -> Result<(), Box<dyn Error>> {
    let doc_id = Uuid::new_v4().to_string();
    let main_doc = Node::with_id(
        NodeId::from_string(doc_id.clone()),
        serde_json::Value::String(
            "# 👥 Weekly Team Standup\n\n**Sprint 23 Progress Review** 🚀\nTeam velocity looking strong this week! Key blockers addressed and new features moving to QA.".to_string()
        )
    ).with_metadata(serde_json::json!({
        "node_type": "text",
        "title": "Weekly Team Standup",
        "parent_date": date,
        "depth": 1,
        "document_type": "meeting_notes"
    }));

    data_store.store_node_with_embedding(
        main_doc,
        create_embedding("team standup sprint progress velocity")
    ).await?;

    let sections = vec![
        ("Sprint Progress", "## 📊 Sprint Progress\n\n✅ **Completed This Week**:\n- User authentication refactor (Sarah) 🔐\n- API rate limiting implementation (Mike) ⚡\n- Mobile responsive fixes (Jessica) 📱\n\n🔄 **In Progress**:\n- Payment gateway integration (David) 💳\n- Search functionality optimization (Lisa) 🔍"),
        
        ("Blockers & Challenges", "## 🚧 Blockers & Challenges\n\n❌ **Current Blockers**:\n- Third-party API documentation incomplete 📚\n- Staging environment deployment issues 🔧\n\n💡 **Solutions Identified**:\n- DevOps team contacted for staging fix ⚙️\n- Alternative API vendor being evaluated 🔄"),
        
        ("Action Items", "## ✅ Action Items\n\n**This Week's Focus**:\n- [ ] Complete payment integration testing by Friday 🧪\n- [ ] Schedule architecture review meeting 🏗️\n- [ ] Update project timeline in Jira 📋\n- [ ] Prepare demo for stakeholder review 🎯"),
    ];

    for (title, content) in sections {
        let section_id = Uuid::new_v4().to_string();
        let section_node = Node::with_id(
            NodeId::from_string(section_id),
            serde_json::Value::String(content.to_string())
        ).with_metadata(serde_json::json!({
            "node_type": "text",
            "title": title,
            "parent_id": doc_id,
            "depth": 2,
            "section_type": "meeting_section"
        }));

        data_store.store_node_with_embedding(
            section_node,
            create_embedding(&format!("{} {}", title, content))
        ).await?;
    }

    println!("   👥 Created Team Standup with 3 sections");
    Ok(())
}

async fn create_budget_review_document(data_store: &LanceDataStore, date: &str) -> Result<(), Box<dyn Error>> {
    let doc_id = Uuid::new_v4().to_string();
    let main_doc = Node::with_id(
        NodeId::from_string(doc_id.clone()),
        serde_json::Value::String(
            "# 💰 Q3 Budget Review\n\n**Financial Performance Analysis** 📈\nQuarterly budget review showing strong performance against targets with key insights for Q4 planning.".to_string()
        )
    ).with_metadata(serde_json::json!({
        "node_type": "text",
        "title": "Q3 Budget Review",
        "parent_date": date,
        "depth": 1,
        "document_type": "financial_review"
    }));

    data_store.store_node_with_embedding(
        main_doc,
        create_embedding("budget review quarterly financial performance")
    ).await?;

    let sections = vec![
        ("Revenue Performance", "## 📈 Revenue Performance\n\n**Q3 Results**: $1.2M (Target: $1.1M) ✅ +9% vs target\n\n**Breakdown by Channel**:\n- **Direct Sales**: $720K 💼\n- **Partner Channel**: $320K 🤝\n- **Online Revenue**: $160K 🌐\n\n**Growth Trends**: 15% YoY growth, strongest Q3 performance in company history! 🚀"),
        
        ("Expense Analysis", "## 💸 Expense Analysis\n\n**Total Expenses**: $940K (Budget: $980K) ✅ Under budget by $40K\n\n**Category Breakdown**:\n- **Personnel Costs**: $620K (66%) 👥\n- **Technology & Tools**: $180K (19%) 💻\n- **Marketing & Sales**: $140K (15%) 📢\n\n**Cost Efficiency**: 22% improvement in cost-per-acquisition 📊"),
    ];

    for (title, content) in sections {
        let section_id = Uuid::new_v4().to_string();
        let section_node = Node::with_id(
            NodeId::from_string(section_id),
            serde_json::Value::String(content.to_string())
        ).with_metadata(serde_json::json!({
            "node_type": "text",
            "title": title,
            "parent_id": doc_id,
            "depth": 2,
            "section_type": "financial_section"
        }));

        data_store.store_node_with_embedding(
            section_node,
            create_embedding(&format!("{} {}", title, content))
        ).await?;
    }

    println!("   💰 Created Q3 Budget Review with 2 sections");
    Ok(())
}

async fn create_client_meeting_document(data_store: &LanceDataStore, date: &str) -> Result<(), Box<dyn Error>> {
    let doc_id = Uuid::new_v4().to_string();
    let main_doc = Node::with_id(
        NodeId::from_string(doc_id.clone()),
        serde_json::Value::String(
            "# 🤝 TechFlow Industries Partnership Meeting\n\n**Strategic Partnership Discussion** 🎯\nExploring integration opportunities and joint go-to-market strategies for Q4 expansion.".to_string()
        )
    ).with_metadata(serde_json::json!({
        "node_type": "text",
        "title": "TechFlow Industries Partnership Meeting",
        "parent_date": date,
        "depth": 1,
        "document_type": "client_meeting"
    }));

    data_store.store_node_with_embedding(
        main_doc,
        create_embedding("client partnership techflow integration collaboration")
    ).await?;

    let sections = vec![
        ("Partnership Opportunities", "## 🚀 Partnership Opportunities\n\n**Technical Integration**:\n- API connectivity for seamless data flow 🔗\n- White-label solution for their enterprise clients 🏷️\n- Joint product development roadmap 🛣️\n\n**Market Expansion**:\n- Co-marketing campaigns in Q4 📢\n- Shared booth at TechExpo 2025 🏢\n- Customer referral program 👥"),
        
        ("Next Steps", "## ✅ Next Steps\n\n**Immediate Actions**:\n- [ ] Technical feasibility assessment (Due: July 5) 🔧\n- [ ] Legal review of partnership terms (Due: July 10) ⚖️\n- [ ] Pilot customer identification (Due: July 15) 🎯\n\n**Follow-up Meeting**: July 20, 2:00 PM PST 📅"),
    ];

    for (title, content) in sections {
        let section_id = Uuid::new_v4().to_string();
        let section_node = Node::with_id(
            NodeId::from_string(section_id),
            serde_json::Value::String(content.to_string())
        ).with_metadata(serde_json::json!({
            "node_type": "text",
            "title": title,
            "parent_id": doc_id,
            "depth": 2,
            "section_type": "meeting_section"
        }));

        data_store.store_node_with_embedding(
            section_node,
            create_embedding(&format!("{} {}", title, content))
        ).await?;
    }

    println!("   🤝 Created Client Partnership Meeting with 2 sections");
    Ok(())
}

async fn create_project_retrospective_document(data_store: &LanceDataStore, date: &str) -> Result<(), Box<dyn Error>> {
    let doc_id = Uuid::new_v4().to_string();
    let main_doc = Node::with_id(
        NodeId::from_string(doc_id.clone()),
        serde_json::Value::String(
            "# 🔄 E-Commerce Platform Retrospective\n\n**Project Post-Mortem Analysis** 📋\nReflecting on our Q2 e-commerce platform launch: successes, challenges, and lessons for future projects.".to_string()
        )
    ).with_metadata(serde_json::json!({
        "node_type": "text",
        "title": "E-Commerce Platform Retrospective", 
        "parent_date": date,
        "depth": 1,
        "document_type": "retrospective"
    }));

    data_store.store_node_with_embedding(
        main_doc,
        create_embedding("project retrospective ecommerce platform lessons learned")
    ).await?;

    let sections = vec![
        ("What Went Well", "## ✅ What Went Well\n\n- **Team Collaboration**: Excellent cross-functional communication 🤝\n- **Technical Delivery**: Platform launched on schedule with 99.9% uptime 🚀\n- **User Feedback**: 4.7/5 average rating from beta users ⭐\n- **Performance**: 40% faster load times than previous platform ⚡"),
        
        ("Challenges & Learnings", "## 🎓 Challenges & Learnings\n\n**Areas for Improvement**:\n- **Testing Coverage**: Need more automated integration tests 🧪\n- **Documentation**: API docs were incomplete at launch 📚\n- **Monitoring**: Better alerting needed for performance issues 📊\n\n**Key Learnings**:\n- Start security review earlier in the process 🔒\n- Involve customer success team in beta planning 📞"),
    ];

    for (title, content) in sections {
        let section_id = Uuid::new_v4().to_string();
        let section_node = Node::with_id(
            NodeId::from_string(section_id),
            serde_json::Value::String(content.to_string())
        ).with_metadata(serde_json::json!({
            "node_type": "text",
            "title": title,
            "parent_id": doc_id,
            "depth": 2,
            "section_type": "retrospective_section"
        }));

        data_store.store_node_with_embedding(
            section_node,
            create_embedding(&format!("{} {}", title, content))
        ).await?;
    }

    println!("   🔄 Created Project Retrospective with 2 sections");
    Ok(())
}

async fn create_quarterly_planning_document(data_store: &LanceDataStore, date: &str) -> Result<(), Box<dyn Error>> {
    let doc_id = Uuid::new_v4().to_string();
    let main_doc = Node::with_id(
        NodeId::from_string(doc_id.clone()),
        serde_json::Value::String(
            "# 🎯 Q4 Strategic Planning Session\n\n**Quarterly Goals & Initiatives** 📈\nSetting ambitious but achievable goals for Q4, focusing on market expansion and product innovation.".to_string()
        )
    ).with_metadata(serde_json::json!({
        "node_type": "text",
        "title": "Q4 Strategic Planning Session",
        "parent_date": date,
        "depth": 1,
        "document_type": "strategic_planning"
    }));

    data_store.store_node_with_embedding(
        main_doc,
        create_embedding("quarterly planning strategic goals market expansion")
    ).await?;

    let sections = vec![
        ("Revenue Goals", "## 💰 Revenue Goals\n\n**Q4 Targets**:\n- **Total Revenue**: $1.8M (50% growth) 📈\n- **New Customer Acquisition**: 200 enterprises 🏢\n- **Upsell Revenue**: $400K from existing clients ⬆️\n\n**Key Initiatives**:\n- Launch premium tier pricing 💎\n- Expand to European markets 🌍\n- Partner channel development 🤝"),
        
        ("Product Roadmap", "## 🛣️ Product Roadmap\n\n**Q4 Feature Releases**:\n- **AI-Powered Analytics** (October) 🤖\n- **Mobile App 2.0** (November) 📱\n- **Enterprise SSO Integration** (December) 🔐\n\n**Innovation Focus**:\n- Machine learning capabilities 🧠\n- Real-time collaboration tools 🔄\n- Advanced security features 🛡️"),
    ];

    for (title, content) in sections {
        let section_id = Uuid::new_v4().to_string();
        let section_node = Node::with_id(
            NodeId::from_string(section_id),
            serde_json::Value::String(content.to_string())
        ).with_metadata(serde_json::json!({
            "node_type": "text",
            "title": title,
            "parent_id": doc_id,
            "depth": 2,
            "section_type": "planning_section"
        }));

        data_store.store_node_with_embedding(
            section_node,
            create_embedding(&format!("{} {}", title, content))
        ).await?;
    }

    println!("   🎯 Created Quarterly Planning with 2 sections");
    Ok(())
}

fn create_embedding(text: &str) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use rand::{SeedableRng, Rng};
    
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let seed = hasher.finish();
    
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    (0..384).map(|_| rng.gen_range(-1.0..1.0)).collect()
}