use nodespace_data_store::{DataStore, SurrealDataStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Working data dump using DataStore wrapper...");

    let store = SurrealDataStore::new("./data/sample.db").await?;

    // Use the DataStore methods that we know work
    println!("\n📋 Nodes for 2025-06-19:");
    let nodes_2025_06_19 = store.get_nodes_for_date("2025-06-19").await?;
    println!("Found {} nodes", nodes_2025_06_19.len());

    for (i, node) in nodes_2025_06_19.iter().enumerate() {
        println!("Node {}: ID={}", i + 1, node.id);
        if let Some(content) = node.content.as_str() {
            println!("  Content: {}", &content[..content.len().min(50)]);
        }
        if let Some(metadata) = &node.metadata {
            println!("  Metadata: {:?}", metadata);
        }
        println!("  Created: {}", node.created_at);
        println!("  Updated: {}", node.updated_at);
        println!("  Next sibling: {:?}", node.next_sibling);
        println!("  Previous sibling: {:?}", node.previous_sibling);
        println!("");
    }

    // Find the parent of our 2025-06-19 child node
    if let Some(child_node) = nodes_2025_06_19.first() {
        println!("\n🔍 Looking for parent of child node: {}", child_node.id);

        // Query to find relationships where this node is the child (out)
        let parent_query = format!(
            "SELECT in FROM contains WHERE out = text:{}",
            child_node.id.as_str().replace("-", "_")
        );
        println!("Parent query: {}", parent_query);

        match store.query_nodes(&parent_query).await {
            Ok(parent_refs) => {
                println!("Found {} parent references", parent_refs.len());
                for (i, parent_ref) in parent_refs.iter().enumerate() {
                    println!("Parent ref {}: {:?}", i + 1, parent_ref);

                    // Now get the actual parent node
                    let parent_id = &parent_ref.id;
                    match store.get_node(parent_id).await {
                        Ok(Some(parent_node)) => {
                            if let Some(content) = parent_node.content.as_str() {
                                println!("  PARENT FOUND: {}", content);

                                // Check if this parent has other children
                                let siblings_query = format!(
                                    "SELECT out FROM contains WHERE in = text:{}",
                                    parent_id.as_str().replace("-", "_")
                                );
                                match store.query_nodes(&siblings_query).await {
                                    Ok(siblings) => {
                                        println!("    Parent has {} children:", siblings.len());
                                        for sibling in siblings.iter().take(5) {
                                            if let Some(sibling_content) = sibling.content.as_str()
                                            {
                                                println!(
                                                    "      - {}",
                                                    &sibling_content
                                                        [..sibling_content.len().min(40)]
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => println!("    Error getting siblings: {}", e),
                                }
                            }
                        }
                        Ok(None) => println!("  Parent node not found in database"),
                        Err(e) => println!("  Error getting parent node: {}", e),
                    }
                }
            }
            Err(e) => println!("Error finding parent: {}", e),
        }
    }

    // Let's examine the actual contains relationships to understand their structure
    println!("\n🔗 Examining actual contains relationships:");
    let contains_query = "SELECT * FROM contains LIMIT 5";
    match store.query_nodes(contains_query).await {
        Ok(relationships) => {
            println!("Found {} relationships", relationships.len());
            for (i, rel) in relationships.iter().enumerate() {
                println!("\nRelationship {}:", i + 1);
                println!("  ID: {}", rel.id);
                println!("  Content: {:?}", rel.content);
                println!("  Created: {}", rel.created_at);
                if let Some(metadata) = &rel.metadata {
                    println!("  Metadata: {:?}", metadata);
                }

                // The relationship record itself might contain the in/out references
                // Let's try to understand the relationship structure by querying its ID
                let rel_detail_query = format!(
                    "SELECT * FROM contains:{}",
                    rel.id.as_str().replace("-", "_")
                );
                println!("  Detail query: {}", rel_detail_query);

                match store.query_nodes(&rel_detail_query).await {
                    Ok(details) => {
                        if !details.is_empty() {
                            println!("    Detail: {:?}", details[0]);
                        }
                    }
                    Err(e) => println!("    Detail error: {}", e),
                }

                if i >= 2 {
                    break;
                } // Limit to first 3 for readability
            }
        }
        Err(e) => println!("Error querying contains: {}", e),
    }

    // Also check if there are any working parent-child pairs in the database
    println!("\n🔍 Looking for ANY working parent-child relationships:");
    let any_parent_query = "SELECT in FROM contains LIMIT 3";
    match store.query_nodes(any_parent_query).await {
        Ok(any_parents) => {
            println!(
                "Found {} parent references in contains table",
                any_parents.len()
            );
            for (i, parent_ref) in any_parents.iter().enumerate() {
                println!(
                    "Parent ref {}: ID={}, Content={:?}",
                    i + 1,
                    parent_ref.id,
                    parent_ref.content
                );
            }
        }
        Err(e) => println!("Error finding any parents: {}", e),
    }

    Ok(())
}
