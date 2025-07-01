//! Create a fresh e2e sample database with hierarchical node structure
//! Based on sample-node-entry.md with today's date as root

use chrono::Utc;
use nodespace_core_types::{Node, NodeId};
use nodespace_data_store::{DataStore, LanceDataStore};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Creating fresh e2e sample database with hierarchical nodes\n");

    // Create fresh database
    let db_path = "/Users/malibio/nodespace/data/lance_db/e2e_sample.db";

    // Remove existing database if it exists
    if std::path::Path::new(db_path).exists() {
        std::fs::remove_dir_all(db_path)?;
        println!("🗑️  Removed existing database");
    }

    let data_store = LanceDataStore::new(db_path).await?;
    println!("✅ Created fresh LanceDB at: {}", db_path);

    // Create today's date node as root
    let today = Utc::now();
    let date_id = Uuid::new_v4().to_string();

    let date_node = Node {
        id: NodeId::from_string(date_id.clone()),
        content: serde_json::Value::String(format!("# {}", today.format("%B %d, %Y"))),
        metadata: Some(serde_json::json!({
            "node_type": "date",
            "date": today.format("%Y-%m-%d").to_string(),
            "created_from": "sample-node-entry.md"
        })),
        created_at: today.to_rfc3339(),
        updated_at: today.to_rfc3339(),
        node_type: "date".to_string(),
        parent_id: None,
        next_sibling: None,
        previous_sibling: None,
        root_id: Some(NodeId::from_string(date_id.clone())), // Points to itself as root
        root_type: Some("date".to_string()),
    };

    // Generate a dummy 384-dimensional embedding
    let embedding = generate_embedding(384);
    let _date_node_id = data_store
        .store_node_with_embedding(date_node, embedding)
        .await?;
    println!("📅 Created date node: {}", today.format("%B %d, %Y"));

    // Track node relationships for hierarchy
    let mut parent_stack: Vec<(String, usize)> = vec![(date_id.clone(), 0)]; // (node_id, depth)
    let mut node_counter = 0;

    // Parse and create hierarchical nodes from the markdown content
    let markdown_lines = vec![
        "- # Product Launch Campaign Strategy",
        "\t- This comprehensive product launch plan provides the strategic framework, tactical execution details, and success measurement criteria necessary for achieving market leadership in the sustainable professional products category.",
        "\t- ## Launch Overview",
        "\t\t- **Product**: EcoSmart Professional Series",
        "\t\t- **Launch Date**: July 15, 2025",
        "\t\t- **Campaign Duration**: 12 weeks (4 weeks pre-launch, 4 weeks launch, 4 weeks post-launch)",
        "\t\t- **Total Budget**: $180,000",
        "\t\t- **Primary Objective**: Establish market leadership in sustainable professional products",
        "\t- ## Executive Summary",
        "\t\t- The EcoSmart Professional Series represents our most significant product innovation in three years, combining professional-grade performance with industry-leading sustainability features. This launch campaign will position us as the premium choice for environmentally conscious professionals while maintaining our quality and performance reputation.",
        "\t- ## Target Audience Analysis",
        "\t\t- ### Primary Target Segment",
        "\t\t\t- **Professional Demographics**:",
        "\t\t\t\t- Age: 28-45 years",
        "\t\t\t\t- Income: $75,000−$150,000 annually",
        "\t\t\t\t- Education: College degree or higher (87%)",
        "\t\t\t\t- Location: Urban and suburban professionals in major metropolitan areas",
        "\t\t\t\t- Industry Focus: Design, consulting, technology, finance, healthcare",
        "\t\t\t- **Psychographic Profile**:",
        "\t\t\t\t- Values sustainability and environmental responsibility",
        "\t\t\t\t- Willing to pay premium for quality and environmental benefits",
        "\t\t\t\t- Influences others in professional networks",
        "\t\t\t\t- Active on LinkedIn and Instagram",
        "\t\t\t\t- Research-intensive purchase behavior",
        "\t\t- ### Secondary Target Segments",
        "\t\t\t- **Segment 2: Sustainability-Focused Organizations**",
        "\t\t\t\t- Corporate buyers implementing sustainability initiatives",
        "\t\t\t\t- Government agencies with environmental mandates",
        "\t\t\t\t- Non-profit organizations with mission alignment",
        "\t\t\t\t- Educational institutions with sustainability programs",
        "\t\t\t- **Segment 3: Early Adopter Enthusiasts**",
        "\t\t\t\t- Technology and innovation enthusiasts",
        "\t\t\t\t- Sustainability advocates and influencers",
        "\t\t\t\t- Professional reviewers and industry experts",
        "\t\t\t\t- Brand advocates and loyal customers",
    ];

    for line in markdown_lines {
        if line.trim().is_empty() {
            continue;
        }

        // Count depth by tabs and hyphens
        let depth = count_depth(line);
        let content = extract_content(line);

        if content.is_empty() {
            continue;
        }

        // Adjust parent stack to current depth
        while parent_stack.len() > depth + 1 {
            parent_stack.pop();
        }

        // Get parent ID
        let parent_id = parent_stack.last().unwrap().0.clone();

        // Create new node
        let node_id = Uuid::new_v4().to_string();
        node_counter += 1;

        let node_type = if content.starts_with("# ") {
            "project"
        } else if content.starts_with("## ") {
            "section"
        } else if content.starts_with("### ") {
            "subsection"
        } else if content.starts_with("**") && content.ends_with("**:") {
            "category"
        } else {
            "text"
        };

        let node = Node {
            id: NodeId::from_string(node_id.clone()),
            content: serde_json::Value::String(content.clone()),
            metadata: Some(serde_json::json!({
                "node_type": node_type,
                "parent_id": parent_id,
                "depth": depth,
                "order": node_counter
            })),
            created_at: today.to_rfc3339(),
            updated_at: today.to_rfc3339(),
            node_type: node_type.to_string(),
            parent_id: parent_id.as_ref().map(|id| NodeId::from_string(id.clone())),
            next_sibling: None,
            previous_sibling: None,
            root_id: Some(NodeId::from_string(date_id.clone())), // All nodes point to date root
            root_type: Some("date".to_string()),
        };

        let embedding = generate_embedding(384);
        let _stored_id = data_store
            .store_node_with_embedding(node, embedding)
            .await?;

        // Create relationship
        data_store
            .create_relationship(
                &NodeId::from_string(parent_id),
                &NodeId::from_string(node_id.clone()),
                "contains",
            )
            .await?;

        // Add to parent stack for potential children
        parent_stack.push((node_id, depth));

        println!(
            "📝 Created {} node: {}",
            node_type,
            if content.len() > 60 {
                format!("{}...", &content[0..60])
            } else {
                content
            }
        );
    }

    println!("\n🎉 Successfully created hierarchical node structure!");
    println!("   📅 Root date node: {}", today.format("%B %d, %Y"));
    println!("   📊 Total child nodes: {}", node_counter);
    println!("   🗄️  Database: {}", db_path);

    // Test retrieval
    println!("\n🔍 Testing node retrieval...");
    let retrieved_date = data_store.get_node(&NodeId::from_string(date_id)).await?;
    if let Some(node) = retrieved_date {
        println!("✅ Successfully retrieved date node: {}", node.content);
    }

    // Test query
    println!("\n🔎 Testing content search...");
    let search_results = data_store.query_nodes("EcoSmart").await?;
    println!(
        "✅ Found {} nodes containing 'EcoSmart'",
        search_results.len()
    );

    Ok(())
}

/// Count the depth of indentation (tabs + leading hyphens)
fn count_depth(line: &str) -> usize {
    let mut depth = 0;
    let mut chars = line.chars();

    // Count leading tabs
    for ch in chars {
        if ch == '\t' {
            depth += 1;
        } else if ch == '-' && depth > 0 {
            // Found the hyphen after tabs, we're done counting
            break;
        } else if ch == '-' && depth == 0 {
            // Root level hyphen
            break;
        } else if !ch.is_whitespace() {
            break;
        }
    }

    depth
}

/// Extract the content after the hyphen, cleaning up formatting
fn extract_content(line: &str) -> String {
    let trimmed = line.trim();
    if let Some(hyphen_pos) = trimmed.find('-') {
        let content = trimmed[hyphen_pos + 1..].trim();
        content.to_string()
    } else {
        trimmed.to_string()
    }
}

/// Generate a dummy embedding vector of specified dimension
fn generate_embedding(size: usize) -> Vec<f32> {
    (0..size).map(|i| (i as f32 * 0.01) % 1.0).collect()
}
