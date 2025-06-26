use nodespace_data_store::{DataStore, SurrealDataStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Debugging parent-child relationships...");

    let store = SurrealDataStore::new("./data/sample.db").await?;

    // Get all nodes from 2025-06-19 to test with
    let nodes_2025_06_19 = store.get_nodes_for_date("2025-06-19").await?;
    println!("Found {} nodes for 2025-06-19", nodes_2025_06_19.len());

    for (i, node) in nodes_2025_06_19.iter().enumerate() {
        if let Some(content) = node.content.as_str() {
            println!(
                "Node {}: {} (content: {})",
                i + 1,
                node.id,
                &content[..content.len().min(50)]
            );
        }
    }

    // Look for parent-child relationships involving any of these nodes
    println!("\n🔍 Checking for parent-child relationships:");

    // Find relationships where any of our nodes is the parent
    for node in &nodes_2025_06_19 {
        let clean_id = node.id.as_str().replace("-", "_");
        let query = format!("SELECT out FROM contains WHERE in = nodes:{}", clean_id);

        match store.query_nodes(&query).await {
            Ok(children) if !children.is_empty() => {
                if let Some(content) = node.content.as_str() {
                    println!(
                        "Parent: {} has {} children",
                        &content[..content.len().min(30)],
                        children.len()
                    );
                    for child in children.iter().take(3) {
                        if let Some(child_content) = child.content.as_str() {
                            println!("  Child: {}", &child_content[..child_content.len().min(30)]);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Find relationships where any of our nodes is the child
    println!("\n🔍 Checking if any of these nodes are children:");
    for node in &nodes_2025_06_19 {
        let clean_id = node.id.as_str().replace("-", "_");
        let query = format!("SELECT in FROM contains WHERE out = nodes:{}", clean_id);

        match store.query_nodes(&query).await {
            Ok(parents) if !parents.is_empty() => {
                if let Some(content) = node.content.as_str() {
                    println!(
                        "Child: {} has {} parents",
                        &content[..content.len().min(30)],
                        parents.len()
                    );
                    for parent in parents.iter().take(3) {
                        if let Some(parent_content) = parent.content.as_str() {
                            println!(
                                "  Parent: {}",
                                &parent_content[..parent_content.len().min(30)]
                            );
                        }
                    }
                }
            }
            Ok(_) => {
                if let Some(content) = node.content.as_str() {
                    println!(
                        "Node: {} has no parent relationships",
                        &content[..content.len().min(30)]
                    );
                }
            }
            Err(e) => {
                println!("Error checking parent for node: {}", e);
            }
        }
    }

    // Let's check if there are any parent nodes that should be associated with 2025-06-19
    println!("\n🔍 Looking for parent nodes that have children...");

    // Let's just examine the raw contains table structure
    let raw_query = "SELECT * FROM contains LIMIT 3";
    match store.query_nodes(raw_query).await {
        Ok(relationships) => {
            println!(
                "Raw contains table structure ({} total relationships):",
                relationships.len()
            );
            for (i, rel) in relationships.iter().enumerate() {
                println!(
                    "Relationship {}: ID={}, Content={:?}",
                    i + 1,
                    rel.id,
                    rel.content
                );
                if let Some(metadata) = &rel.metadata {
                    println!("  Metadata: {:?}", metadata);
                }
            }
        }
        Err(e) => println!("Error querying raw contains: {}", e),
    }

    // Now let's see if we can find the actual relationships using the data store's internal query
    println!("\n🔍 Using raw query to understand relationship storage...");

    // Let's check if the issue is in the query_nodes method vs raw SurrealDB queries
    println!("The issue might be that relationships are stored differently than expected.");
    println!("Let's check the actual database schema...");

    // Try to understand what contains relationships actually exist
    let schema_query = "INFO FOR DB";
    match store.query_nodes(schema_query).await {
        Ok(info) => {
            println!("Database info returned {} results", info.len());
        }
        Err(e) => println!("Cannot get DB info: {}", e),
    }

    Ok(())
}
