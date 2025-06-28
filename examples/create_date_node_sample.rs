//! Create a date node with hierarchical Product Launch Campaign Strategy
//! This example demonstrates proper LanceDB persistence with Universal Document Schema

use arrow_array::builder::{ListBuilder, StringBuilder};
use arrow_array::{Array, ListArray, RecordBatch, RecordBatchIterator, StringArray};
use arrow_schema::{DataType, Field, Schema};
use chrono::Utc;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{connect, Connection};
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("📅 Creating Date Node with Product Launch Campaign Strategy\n");

    // Connect to shared LanceDB location for e2e testing
    let db_path = "/Users/malibio/nodespace/data/lance_db/sample_entry.db";
    let db = connect(db_path).execute().await?;
    println!("✅ Connected to LanceDB at: {}", db_path);

    // Ensure nodes table exists
    ensure_nodes_table(&db).await?;
    let table = db.open_table("nodes").execute().await?;

    // Create today's date node (parent)
    let today = Utc::now();
    let date_node_id = Uuid::new_v4().to_string();
    let date_content = format!("Date: {}", today.format("%B %d, %Y"));

    println!("📝 Creating date node: {}", date_content);

    // Create sample data with hierarchical structure
    let mut nodes = Vec::new();

    // 1. Today's Date Node (root)
    nodes.push(create_node_data(
        date_node_id.clone(),
        "date".to_string(),
        date_content,
        None, // No parent
        today.to_rfc3339(),
    ));

    // 2. Product Launch Campaign Strategy (level 1 - child of date)
    let campaign_id = Uuid::new_v4().to_string();
    nodes.push(create_node_data(
        campaign_id.clone(),
        "project".to_string(),
        "# Product Launch Campaign Strategy".to_string(),
        Some(date_node_id.clone()),
        today.to_rfc3339(),
    ));

    // 3. Campaign description (level 2)
    let desc_id = Uuid::new_v4().to_string();
    nodes.push(create_node_data(
        desc_id.clone(),
        "text".to_string(),
        "This comprehensive product launch plan provides the strategic framework, tactical execution details, and success measurement criteria necessary for achieving market leadership in the sustainable professional products category.".to_string(),
        Some(campaign_id.clone()),
        today.to_rfc3339(),
    ));

    // 4. Launch Overview (level 2)
    let overview_id = Uuid::new_v4().to_string();
    nodes.push(create_node_data(
        overview_id.clone(),
        "text".to_string(),
        "## Launch Overview".to_string(),
        Some(campaign_id.clone()),
        today.to_rfc3339(),
    ));

    // 5. Launch Overview details (level 3)
    let details = vec![
        "**Product**: EcoSmart Professional Series",
        "**Launch Date**: July 15, 2025",
        "**Campaign Duration**: 12 weeks (4 weeks pre-launch, 4 weeks launch, 4 weeks post-launch)",
        "**Total Budget**: $180,000",
        "**Primary Objective**: Establish market leadership in sustainable professional products",
    ];

    for detail in details {
        let detail_id = Uuid::new_v4().to_string();
        nodes.push(create_node_data(
            detail_id,
            "text".to_string(),
            detail.to_string(),
            Some(overview_id.clone()),
            today.to_rfc3339(),
        ));
    }

    // 6. Executive Summary (level 2)
    let exec_summary_id = Uuid::new_v4().to_string();
    nodes.push(create_node_data(
        exec_summary_id.clone(),
        "text".to_string(),
        "## Executive Summary".to_string(),
        Some(campaign_id.clone()),
        today.to_rfc3339(),
    ));

    let exec_content_id = Uuid::new_v4().to_string();
    nodes.push(create_node_data(
        exec_content_id,
        "text".to_string(),
        "The EcoSmart Professional Series represents our most significant product innovation in three years, combining professional-grade performance with industry-leading sustainability features. This launch campaign will position us as the premium choice for environmentally conscious professionals while maintaining our quality and performance reputation.".to_string(),
        Some(exec_summary_id.clone()),
        today.to_rfc3339(),
    ));

    // Convert to Arrow format and insert
    let batch = create_record_batch(nodes)?;
    println!("✅ Created RecordBatch with {} rows", batch.num_rows());

    // Insert data into table
    let schema = batch.schema();
    let batches = RecordBatchIterator::new(vec![batch].into_iter().map(Ok), schema);
    table.add(Box::new(batches)).execute().await?;
    println!("✅ Inserted hierarchical data into LanceDB");

    // Query to verify data
    let results = table.query().limit(20).execute().await?;
    let batches: Vec<_> = futures::TryStreamExt::try_collect(results).await?;

    println!("✅ Verification - Retrieved {} batches:", batches.len());
    for (i, batch) in batches.iter().enumerate() {
        println!("  Batch {}: {} rows", i, batch.num_rows());

        // Show sample data
        if let Some(id_array) = batch.column_by_name("id") {
            if let Some(ids) = id_array.as_any().downcast_ref::<StringArray>() {
                for j in 0..std::cmp::min(5, ids.len()) {
                    if let Some(id) = ids.value(j).get(0..8) {
                        println!("    Node {}: {}...", j, id);
                    }
                }
            }
        }
    }

    println!("\n🎉 Successfully created hierarchical date node structure in LanceDB!");
    println!("   - Date Node (today: {})", today.format("%Y-%m-%d"));
    println!("   - Product Launch Campaign Strategy (child)");
    println!("   - Launch Overview with 5 detail items (grandchildren)");
    println!("   - Executive Summary with content (grandchildren)");

    Ok(())
}

/// Ensure the nodes table exists with proper Universal Document Schema
async fn ensure_nodes_table(db: &Connection) -> Result<(), Box<dyn Error>> {
    // Check if table already exists
    let table_names = db.table_names().execute().await?;

    if table_names.contains(&"nodes".to_string()) {
        println!("✅ Nodes table already exists");
        return Ok(());
    }

    // Create schema for Universal Document Schema
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("node_type", DataType::Utf8, false),
        Field::new("content", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::List(Arc::new(Field::new("item", DataType::Float32, true))),
            false,
        ),
        Field::new("parent_id", DataType::Utf8, true),
        Field::new(
            "children_ids",
            DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
            true,
        ),
        Field::new(
            "mentions",
            DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
            true,
        ),
        Field::new("created_at", DataType::Utf8, false),
        Field::new("updated_at", DataType::Utf8, false),
        Field::new("metadata", DataType::Utf8, true), // JSON string
    ]));

    // Create empty batch to initialize table
    let empty_batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(Vec::<String>::new())), // id
            Arc::new(StringArray::from(Vec::<String>::new())), // node_type
            Arc::new(StringArray::from(Vec::<String>::new())), // content
            Arc::new(ListArray::from_iter_primitive::<
                arrow_array::types::Float32Type,
                _,
                _,
            >(
                Vec::<Option<Vec<Option<f32>>>>::new(), // vector
            )),
            Arc::new(StringArray::from(Vec::<Option<String>>::new())), // parent_id
            Arc::new(ListBuilder::new(StringBuilder::new()).finish()), // children_ids
            Arc::new(ListBuilder::new(StringBuilder::new()).finish()), // mentions
            Arc::new(StringArray::from(Vec::<String>::new())),         // created_at
            Arc::new(StringArray::from(Vec::<String>::new())),         // updated_at
            Arc::new(StringArray::from(Vec::<Option<String>>::new())), // metadata
        ],
    )?;

    let batches = RecordBatchIterator::new(vec![empty_batch].into_iter().map(Ok), schema.clone());

    db.create_table("nodes", Box::new(batches))
        .execute()
        .await?;
    println!("✅ Created LanceDB nodes table with Universal Document Schema");

    Ok(())
}

/// Helper to create node data structure
fn create_node_data(
    id: String,
    node_type: String,
    content: String,
    parent_id: Option<String>,
    timestamp: String,
) -> NodeData {
    // Generate dummy 384-dimensional embedding
    let vector: Vec<f32> = (0..384).map(|i| (i as f32 * 0.01) % 1.0).collect();

    NodeData {
        id,
        node_type,
        content,
        vector,
        parent_id,
        children_ids: vec![], // Will be computed during retrieval
        mentions: vec![],
        created_at: timestamp.clone(),
        updated_at: timestamp,
        metadata: None,
    }
}

/// Node data structure
struct NodeData {
    id: String,
    node_type: String,
    content: String,
    vector: Vec<f32>,
    parent_id: Option<String>,
    children_ids: Vec<String>,
    mentions: Vec<String>,
    created_at: String,
    updated_at: String,
    metadata: Option<String>,
}

/// Convert nodes to Arrow RecordBatch
fn create_record_batch(nodes: Vec<NodeData>) -> Result<RecordBatch, Box<dyn Error>> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("node_type", DataType::Utf8, false),
        Field::new("content", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::List(Arc::new(Field::new("item", DataType::Float32, true))),
            false,
        ),
        Field::new("parent_id", DataType::Utf8, true),
        Field::new(
            "children_ids",
            DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
            true,
        ),
        Field::new(
            "mentions",
            DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
            true,
        ),
        Field::new("created_at", DataType::Utf8, false),
        Field::new("updated_at", DataType::Utf8, false),
        Field::new("metadata", DataType::Utf8, true),
    ]));

    let ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
    let node_types: Vec<String> = nodes.iter().map(|n| n.node_type.clone()).collect();
    let contents: Vec<String> = nodes.iter().map(|n| n.content.clone()).collect();
    let parent_ids: Vec<Option<String>> = nodes.iter().map(|n| n.parent_id.clone()).collect();
    let created_ats: Vec<String> = nodes.iter().map(|n| n.created_at.clone()).collect();
    let updated_ats: Vec<String> = nodes.iter().map(|n| n.updated_at.clone()).collect();
    let metadatas: Vec<Option<String>> = nodes.iter().map(|n| n.metadata.clone()).collect();

    // Convert vectors
    let vectors = ListArray::from_iter_primitive::<arrow_array::types::Float32Type, _, _>(
        nodes
            .iter()
            .map(|n| Some(n.vector.iter().map(|&f| Some(f)).collect::<Vec<_>>())),
    );

    // Convert children_ids (empty for now)
    // Convert children_ids using ListBuilder
    let mut children_builder = ListBuilder::new(StringBuilder::new());
    for node in &nodes {
        for child_id in &node.children_ids {
            children_builder.values().append_value(child_id);
        }
        children_builder.append(true);
    }
    let children_ids = children_builder.finish();

    // Convert mentions using ListBuilder
    let mut mentions_builder = ListBuilder::new(StringBuilder::new());
    for node in &nodes {
        for mention in &node.mentions {
            mentions_builder.values().append_value(mention);
        }
        mentions_builder.append(true);
    }
    let mentions = mentions_builder.finish();

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(ids)),
            Arc::new(StringArray::from(node_types)),
            Arc::new(StringArray::from(contents)),
            Arc::new(vectors),
            Arc::new(StringArray::from(parent_ids)),
            Arc::new(children_ids),
            Arc::new(mentions),
            Arc::new(StringArray::from(created_ats)),
            Arc::new(StringArray::from(updated_ats)),
            Arc::new(StringArray::from(metadatas)),
        ],
    )?;

    Ok(batch)
}
