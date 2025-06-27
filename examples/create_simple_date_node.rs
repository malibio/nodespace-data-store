//! Create a simple date node with hierarchical Product Launch Campaign Strategy
//! Uses the current JSON-based persistence for e2e testing

use chrono::Utc;
use nodespace_core_types::{Node, NodeId};
use nodespace_data_store::{DataStore, LanceDataStore};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📅 Creating Date Node with Product Launch Campaign Strategy\n");

    // Connect to shared LanceDB location for e2e testing
    let db_path = "/Users/malibio/nodespace/data/lance_db/sample_entry.db";
    let data_store = LanceDataStore::new(db_path).await?;
    println!("✅ Connected to LanceDB at: {}", db_path);

    // Create today's date node (parent)
    let today = Utc::now();
    let date_node_id = NodeId::from_string(Uuid::new_v4().to_string());
    let date_content = format!("Date: {}", today.format("%B %d, %Y"));

    println!("📝 Creating date node: {}", date_content);

    // 1. Today's Date Node (root)
    let date_node = Node {
        id: date_node_id.clone(),
        content: serde_json::Value::String(date_content),
        metadata: Some(serde_json::json!({
            "node_type": "date"
        })),
        created_at: today.to_rfc3339(),
        updated_at: today.to_rfc3339(),
        next_sibling: None,
        previous_sibling: None,
    };

    let _date_id = data_store.store_node(date_node).await?;
    println!("✅ Stored date node");

    // 2. Product Launch Campaign Strategy (level 1 - child of date)
    let campaign_id = NodeId::from_string(Uuid::new_v4().to_string());
    let campaign_node = Node {
        id: campaign_id.clone(),
        content: serde_json::Value::String("# Product Launch Campaign Strategy".to_string()),
        metadata: Some(serde_json::json!({
            "node_type": "project",
            "parent_id": date_node_id.to_string()
        })),
        created_at: today.to_rfc3339(),
        updated_at: today.to_rfc3339(),
        next_sibling: None,
        previous_sibling: None,
    };

    let _campaign_id_stored = data_store.store_node(campaign_node).await?;
    println!("✅ Stored campaign strategy node");

    // 3. Campaign description (level 2)
    let desc_id = NodeId::from_string(Uuid::new_v4().to_string());
    let desc_node = Node {
        id: desc_id.clone(),
        content: serde_json::Value::String("This comprehensive product launch plan provides the strategic framework, tactical execution details, and success measurement criteria necessary for achieving market leadership in the sustainable professional products category.".to_string()),
        metadata: Some(serde_json::json!({
            "node_type": "text",
            "parent_id": campaign_id.to_string()
        })),
        created_at: today.to_rfc3339(),
        updated_at: today.to_rfc3339(),
        next_sibling: None,
        previous_sibling: None,
    };

    let _desc_id_stored = data_store.store_node(desc_node).await?;
    println!("✅ Stored description node");

    // 4. Launch Overview (level 2)
    let overview_id = NodeId::from_string(Uuid::new_v4().to_string());
    let overview_node = Node {
        id: overview_id.clone(),
        content: serde_json::Value::String("## Launch Overview".to_string()),
        metadata: Some(serde_json::json!({
            "node_type": "text",
            "parent_id": campaign_id.to_string()
        })),
        created_at: today.to_rfc3339(),
        updated_at: today.to_rfc3339(),
        next_sibling: None,
        previous_sibling: None,
    };

    let _overview_id_stored = data_store.store_node(overview_node).await?;
    println!("✅ Stored launch overview node");

    // 5. Launch Overview details (level 3)
    let details = ["**Product**: EcoSmart Professional Series",
        "**Launch Date**: July 15, 2025",
        "**Campaign Duration**: 12 weeks (4 weeks pre-launch, 4 weeks launch, 4 weeks post-launch)",
        "**Total Budget**: $180,000",
        "**Primary Objective**: Establish market leadership in sustainable professional products"];

    for (i, detail) in details.iter().enumerate() {
        let detail_id = NodeId::from_string(Uuid::new_v4().to_string());
        let detail_node = Node {
            id: detail_id.clone(),
            content: serde_json::Value::String(detail.to_string()),
            metadata: Some(serde_json::json!({
                "node_type": "text",
                "parent_id": overview_id.to_string()
            })),
            created_at: today.to_rfc3339(),
            updated_at: today.to_rfc3339(),
            next_sibling: None,
            previous_sibling: None,
        };

        let _detail_id_stored = data_store.store_node(detail_node).await?;
        println!("✅ Stored detail {} of {}", i + 1, details.len());
    }

    // 6. Executive Summary (level 2)
    let exec_summary_id = NodeId::from_string(Uuid::new_v4().to_string());
    let exec_summary_node = Node {
        id: exec_summary_id.clone(),
        content: serde_json::Value::String("## Executive Summary".to_string()),
        metadata: Some(serde_json::json!({
            "node_type": "text",
            "parent_id": campaign_id.to_string()
        })),
        created_at: today.to_rfc3339(),
        updated_at: today.to_rfc3339(),
        next_sibling: None,
        previous_sibling: None,
    };

    let _exec_summary_id_stored = data_store.store_node(exec_summary_node).await?;
    println!("✅ Stored executive summary node");

    let exec_content_id = NodeId::from_string(Uuid::new_v4().to_string());
    let exec_content_node = Node {
        id: exec_content_id.clone(),
        content: serde_json::Value::String("The EcoSmart Professional Series represents our most significant product innovation in three years, combining professional-grade performance with industry-leading sustainability features. This launch campaign will position us as the premium choice for environmentally conscious professionals while maintaining our quality and performance reputation.".to_string()),
        metadata: Some(serde_json::json!({
            "node_type": "text",
            "parent_id": exec_summary_id.to_string()
        })),
        created_at: today.to_rfc3339(),
        updated_at: today.to_rfc3339(),
        next_sibling: None,
        previous_sibling: None,
    };

    let _exec_content_id_stored = data_store.store_node(exec_content_node).await?;
    println!("✅ Stored executive summary content");

    println!("\n🎉 Successfully created hierarchical date node structure!");
    println!("   - Date Node (today: {})", today.format("%Y-%m-%d"));
    println!("   - Product Launch Campaign Strategy (child)");
    println!("   - Launch Overview with 5 detail items (grandchildren)");
    println!("   - Executive Summary with content (grandchildren)");
    println!("   - Data persisted to: {}/nodes.json", db_path);

    Ok(())
}
