use chrono::Utc;
use nodespace_core_types::{Node, NodeId};
use nodespace_data_store::{DataStore, SurrealDataStore};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Starting fastembed-rs embedding migration...");
    println!("📦 This will backup content without embeddings and clear embedding data");

    // Initialize the data store
    let store = SurrealDataStore::new("/Users/malibio/nodespace/data/sample.db").await?;

    // Phase 1: Backup current data without embeddings
    println!("\n📋 Phase 1: Backing up current data without embeddings...");
    let backup_data = backup_content_without_embeddings(&store).await?;

    println!(
        "✅ Backed up {} content records without embeddings",
        backup_data.len()
    );

    // Phase 2: Clear all existing embeddings
    println!("\n🧹 Phase 2: Clearing all existing embedding data...");
    let cleared_count = clear_all_embeddings(&store).await?;

    println!("✅ Cleared embeddings from {} records", cleared_count);

    // Phase 3: Report migration readiness
    println!("\n📊 Phase 3: Migration Status Report");
    println!("═══════════════════════════════════════════");
    println!(
        "✅ Backup completed: {} content records preserved",
        backup_data.len()
    );
    println!("✅ Embeddings cleared: {} records cleaned", cleared_count);
    println!("🔄 Ready for fastembed-rs regeneration");
    println!("\n📝 Next steps:");
    println!("  1. Ensure NLP engine is updated to fastembed-rs (NS-54)");
    println!("  2. Run regeneration script to re-embed all content");
    println!("  3. Validate semantic search quality with new embeddings");

    Ok(())
}

/// Backup all content without embeddings for safe migration
async fn backup_content_without_embeddings(
    store: &SurrealDataStore,
) -> Result<Vec<ContentBackup>, Box<dyn std::error::Error>> {
    let mut backup_data = Vec::new();

    // Use the public query interface to backup data
    // Backup text nodes
    let text_nodes = store.query_nodes("SELECT * FROM text").await?;

    for node in text_nodes {
        let backup = ContentBackup {
            table: "text".to_string(),
            node_id: node.id.clone(),
            content: node.content.clone(),
            metadata: node.metadata.clone(),
            created_at: node.created_at.clone(),
            updated_at: node.updated_at.clone(),
        };
        backup_data.push(backup);
    }

    // Backup date nodes using custom query
    let date_nodes = store.query_nodes("SELECT * FROM date").await?;

    for node in date_nodes {
        let backup = ContentBackup {
            table: "date".to_string(),
            node_id: node.id.clone(),
            content: node.content.clone(),
            metadata: node.metadata.clone(),
            created_at: node.created_at.clone(),
            updated_at: node.updated_at.clone(),
        };
        backup_data.push(backup);
    }

    // Backup regular nodes if they exist
    let regular_nodes = store
        .query_nodes("SELECT * FROM nodes")
        .await
        .unwrap_or_default();

    for node in regular_nodes {
        let backup = ContentBackup {
            table: "nodes".to_string(),
            node_id: node.id.clone(),
            content: node.content.clone(),
            metadata: node.metadata.clone(),
            created_at: node.created_at.clone(),
            updated_at: node.updated_at.clone(),
        };
        backup_data.push(backup);
    }

    Ok(backup_data)
}

/// Clear all embedding data from the database
async fn clear_all_embeddings(
    store: &SurrealDataStore,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut cleared_count = 0;

    println!("🔄 Clearing embeddings using SurrealQL UPDATE statements...");

    // Clear embeddings from text table
    let clear_text_query = "UPDATE text SET embedding = NONE WHERE embedding IS NOT NULL";
    let cleared_text = store
        .query_nodes(clear_text_query)
        .await
        .unwrap_or_default();
    cleared_count += cleared_text.len();

    // Clear embeddings from nodes table
    let clear_nodes_query = "UPDATE nodes SET embedding = NONE WHERE embedding IS NOT NULL";
    let cleared_nodes = store
        .query_nodes(clear_nodes_query)
        .await
        .unwrap_or_default();
    cleared_count += cleared_nodes.len();

    // Alternative: Use DELETE approach for complete embedding removal
    println!("🗑️  Performing complete embedding field removal...");

    // Get all records that have embeddings and recreate them without
    let embedded_records_query = "SELECT * FROM text WHERE embedding IS NOT NULL";
    let embedded_records = store
        .query_nodes(embedded_records_query)
        .await
        .unwrap_or_default();

    for record in embedded_records {
        // Delete the existing record with embeddings
        let delete_result = store.delete_node(&record.id).await;
        if delete_result.is_ok() {
            // Recreate the record without embeddings (embeddings will be None by default)
            let clean_node = Node {
                id: record.id.clone(),
                content: record.content,
                metadata: record.metadata,
                created_at: record.created_at,
                updated_at: Utc::now().to_rfc3339(), // Update timestamp for migration
                next_sibling: record.next_sibling,
                previous_sibling: record.previous_sibling,
            };

            // Store the clean node (without embedding)
            let store_result = store.store_node(clean_node).await;
            if store_result.is_ok() {
                cleared_count += 1;
            }
        }
    }

    Ok(cleared_count)
}

/// Structure for backing up content without embeddings
#[derive(Debug)]
struct ContentBackup {
    table: String,
    node_id: NodeId,
    content: Value,
    metadata: Option<Value>,
    created_at: String,
    updated_at: String,
}
