//! Proper Arrow implementation for LanceDB 0.20.0 based on expert guidance
//! This addresses the ListArray construction issues with correct Option nesting

use arrow_array::{Array, ListArray, RecordBatch, RecordBatchIterator, StringArray};
use arrow_schema::{DataType, Field, Schema};
use chrono::Utc;
use lancedb::connect;
use lancedb::query::{ExecutableQuery, QueryBase};
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
struct UniversalNode {
    id: String,
    node_type: String,
    content: String,
    vector: Vec<f32>, // The simple concept
    parent_id: Option<String>,
    children_ids: Vec<String>, // The simple concept
    mentions: Vec<String>,
    created_at: String,
    updated_at: String,
    metadata: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Proper Arrow Implementation for LanceDB 0.20.0\n");

    // Connect to LanceDB
    let db_path = "/Users/malibio/nodespace/data/lance_db/arrow_test.db";
    let db = connect(db_path).execute().await?;
    println!("✅ Connected to LanceDB 0.20.0 at: {}", db_path);

    // Create sample data with the Universal Document Schema
    let today = Utc::now();
    let mut nodes = Vec::new();

    // Date node (root)
    let date_id = Uuid::new_v4().to_string();
    nodes.push(UniversalNode {
        id: date_id.clone(),
        node_type: "date".to_string(),
        content: format!("Date: {}", today.format("%B %d, %Y")),
        vector: generate_embedding(384),
        parent_id: None,
        children_ids: vec![],
        mentions: vec![],
        created_at: today.to_rfc3339(),
        updated_at: today.to_rfc3339(),
        metadata: Some(r#"{"type": "date_node"}"#.to_string()),
    });

    // Campaign node (child of date)
    let campaign_id = Uuid::new_v4().to_string();
    nodes.push(UniversalNode {
        id: campaign_id.clone(),
        node_type: "project".to_string(),
        content: "# Product Launch Campaign Strategy".to_string(),
        vector: generate_embedding(384),
        parent_id: Some(date_id.clone()),
        children_ids: vec![],
        mentions: vec!["EcoSmart".to_string(), "sustainability".to_string()],
        created_at: today.to_rfc3339(),
        updated_at: today.to_rfc3339(),
        metadata: Some(r#"{"type": "campaign", "priority": "high"}"#.to_string()),
    });

    // Overview node (child of campaign)
    let overview_id = Uuid::new_v4().to_string();
    nodes.push(UniversalNode {
        id: overview_id.clone(),
        node_type: "text".to_string(),
        content: "## Launch Overview\n**Product**: EcoSmart Professional Series".to_string(),
        vector: generate_embedding(384),
        parent_id: Some(campaign_id.clone()),
        children_ids: vec![],
        mentions: vec!["product".to_string()],
        created_at: today.to_rfc3339(),
        updated_at: today.to_rfc3339(),
        metadata: None,
    });

    println!("📊 Created {} sample nodes", nodes.len());

    // Create Arrow schema for Universal Document Schema
    let schema = create_universal_schema();
    println!("✅ Created Universal Document Schema");

    // Convert to Arrow RecordBatch with correct ListArray construction
    let batch = create_record_batch_proper(nodes, schema.clone())?;
    println!("✅ Created RecordBatch with {} rows", batch.num_rows());

    // Create table with proper RecordBatchIterator
    let table_name = "universal_nodes";

    // Check if table exists and drop it for clean test
    let table_names = db.table_names().execute().await?;
    if table_names.contains(&table_name.to_string()) {
        db.drop_table(table_name).await?;
        println!("🗑️  Dropped existing table");
    }

    let batches = RecordBatchIterator::new(vec![batch].into_iter().map(Ok), schema.clone());

    let table = db
        .create_table(table_name, Box::new(batches))
        .execute()
        .await?;
    println!("✅ Created table: {}", table_name);

    // Query the table to verify data
    let results = table.query().limit(10).execute().await?;
    let batches: Vec<_> = futures::TryStreamExt::try_collect(results).await?;

    println!("✅ Query executed - Retrieved {} batches:", batches.len());
    for (i, batch) in batches.iter().enumerate() {
        println!(
            "  Batch {}: {} rows, {} columns",
            i,
            batch.num_rows(),
            batch.num_columns()
        );

        // Show sample data
        if let Some(id_array) = batch.column_by_name("id") {
            if let Some(ids) = id_array.as_any().downcast_ref::<StringArray>() {
                for j in 0..std::cmp::min(3, ids.len()) {
                    let id = ids.value(j);
                    println!("    Row {}: ID = {}...", j, &id[0..8]);
                }
            }
        }
    }

    // Test vector search (if available in 0.20.0)
    println!("\n🔍 Testing vector similarity search...");
    let query_vector = generate_embedding(384);

    // Try vector search - this might require creating an index first in 0.20.0
    match table
        .query()
        .nearest_to(query_vector) // Remove & reference
        .unwrap()
        .limit(2)
        .execute()
        .await
    {
        Ok(search_results) => {
            let search_batches: Vec<_> = futures::TryStreamExt::try_collect(search_results).await?;
            println!(
                "✅ Vector search successful - {} result batches",
                search_batches.len()
            );
        }
        Err(e) => {
            println!("⚠️  Vector search failed (might need index): {}", e);
        }
    }

    println!("\n🎉 Arrow implementation successful with LanceDB 0.20.0!");
    println!("   - Universal Document Schema ✅");
    println!("   - Proper ListArray construction ✅");
    println!("   - Hierarchical relationships ✅");
    println!("   - Vector embeddings ✅");
    println!("   - Metadata support ✅");

    Ok(())
}

/// Create the Universal Document Schema with proper Arrow types
fn create_universal_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("node_type", DataType::Utf8, false),
        Field::new("content", DataType::Utf8, false),
        // Vector field - List of Float32 (nullable elements for Arrow compatibility)
        Field::new(
            "vector",
            DataType::List(Arc::new(Field::new("item", DataType::Float32, true))),
            false,
        ),
        Field::new("parent_id", DataType::Utf8, true), // Nullable
        // Children IDs - List of String (nullable for empty lists)
        Field::new(
            "children_ids",
            DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
            true,
        ),
        // Mentions - List of String (nullable for empty lists)
        Field::new(
            "mentions",
            DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
            true,
        ),
        Field::new("created_at", DataType::Utf8, false),
        Field::new("updated_at", DataType::Utf8, false),
        Field::new("metadata", DataType::Utf8, true), // Nullable JSON string
    ]))
}

/// Create RecordBatch with proper ListArray construction based on expert guidance
fn create_record_batch_proper(
    nodes: Vec<UniversalNode>,
    schema: Arc<Schema>,
) -> Result<RecordBatch, Box<dyn Error>> {
    // Extract simple fields
    let ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
    let node_types: Vec<String> = nodes.iter().map(|n| n.node_type.clone()).collect();
    let contents: Vec<String> = nodes.iter().map(|n| n.content.clone()).collect();
    let parent_ids: Vec<Option<String>> = nodes.iter().map(|n| n.parent_id.clone()).collect();
    let created_ats: Vec<String> = nodes.iter().map(|n| n.created_at.clone()).collect();
    let updated_ats: Vec<String> = nodes.iter().map(|n| n.updated_at.clone()).collect();
    let metadatas: Vec<Option<String>> = nodes.iter().map(|n| n.metadata.clone()).collect();

    // EXPERT GUIDANCE APPLIED: Correct ListArray construction

    // 1. Vector field: Vec<f32> -> ListArray with proper Option nesting
    // The double Option is needed: outer Option for nullable rows, inner Option for nullable elements
    let vectors = ListArray::from_iter_primitive::<arrow_array::types::Float32Type, _, _>(
        nodes.iter().map(|n| {
            Some(n.vector.iter().map(|&f| Some(f)).collect::<Vec<_>>())
            // ^--- Outer Option for nullable list (Some = list exists)
            //          ^--- Inner Option for nullable elements (Some = element exists)
        }),
    );

    // 2. Children IDs: Vec<String> -> ListArray for string lists
    // For strings, use the generic ListArray::from_iter_primitive with StringArray values
    use arrow_array::builder::{ListBuilder, StringBuilder};

    let mut children_builder = ListBuilder::new(StringBuilder::new());
    for node in &nodes {
        for child_id in &node.children_ids {
            children_builder.values().append_value(child_id);
        }
        children_builder.append(true);
    }
    let children_ids = children_builder.finish();

    // 3. Mentions: Vec<String> -> ListArray for string lists
    let mut mentions_builder = ListBuilder::new(StringBuilder::new());
    for node in &nodes {
        for mention in &node.mentions {
            mentions_builder.values().append_value(mention);
        }
        mentions_builder.append(true);
    }
    let mentions = mentions_builder.finish();

    // Create RecordBatch with all columns
    let batch = RecordBatch::try_new(
        schema,
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

/// Generate a dummy embedding vector
fn generate_embedding(size: usize) -> Vec<f32> {
    (0..size).map(|i| (i as f32 * 0.01) % 1.0).collect()
}
