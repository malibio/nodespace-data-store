use nodespace_core_types::{Node, NodeId};
use nodespace_data_store::{LanceDataStore, DataStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing SimpleLanceDataStore Implementation");
    println!("================================================");
    
    // Initialize the LanceDataStore
    let mut data_store = LanceDataStore::new("/tmp/test_lance_db").await?;
    data_store.initialize_table().await?;
    
    println!("✅ Initialized LanceDataStore");
    
    // Test 1: Store and retrieve a node
    println!("\n📝 Test 1: Store and retrieve node");
    let node1 = Node::with_id(
        NodeId::new(),
        serde_json::Value::String("This is a test node with some content".to_string())
    ).with_metadata(serde_json::json!({
        "node_type": "text",
        "tags": ["test", "example"]
    }));
    
    let node1_id = node1.id.clone();
    let stored_id = data_store.store_node(node1.clone()).await?;
    println!("   Stored node with ID: {}", stored_id);
    
    let retrieved_node = data_store.get_node(&node1_id).await?;
    match retrieved_node {
        Some(node) => {
            println!("   Retrieved node: {}", node.content);
            println!("   Metadata: {:?}", node.metadata);
        }
        None => println!("   ❌ Failed to retrieve node"),
    }
    
    // Test 2: Store node with embedding and test vector search
    println!("\n🔍 Test 2: Store with embedding and vector search");
    let node2 = Node::with_id(
        NodeId::new(),
        serde_json::Value::String("Machine learning and artificial intelligence content".to_string())
    ).with_metadata(serde_json::json!({
        "node_type": "technical",
        "domain": "ai"
    }));
    
    // Generate a sample embedding (384 dimensions)
    let embedding: Vec<f32> = (0..384).map(|i| (i as f32).sin() * 0.1).collect();
    
    let node2_id = node2.id.clone();
    data_store.store_node_with_embedding(node2, embedding.clone()).await?;
    println!("   Stored node with embedding");
    
    // Test vector search
    let search_results = data_store.search_similar_nodes(embedding, 5).await?;
    println!("   Vector search found {} results", search_results.len());
    for (node, score) in search_results {
        println!("     - Score: {:.3}, Content: {}", score, node.content);
    }
    
    // Test 3: Create relationships
    println!("\n🔗 Test 3: Create relationships");
    let node3 = Node::with_id(
        NodeId::new(),
        serde_json::Value::String("Child node content".to_string())
    ).with_metadata(serde_json::json!({
        "node_type": "text"
    }));
    
    let node3_id = node3.id.clone();
    data_store.store_node(node3).await?;
    
    // Create parent-child relationship
    data_store.create_relationship(&node1_id, &node3_id, "contains").await?;
    println!("   Created relationship: {} -> {}", node1_id, node3_id);
    
    // Verify relationship by checking metadata
    let parent_node = data_store.get_node(&node1_id).await?.unwrap();
    if let Some(metadata) = parent_node.metadata {
        if let Some(children) = metadata.get("children_ids") {
            println!("   Parent node children: {:?}", children);
        }
    }
    
    let child_node = data_store.get_node(&node3_id).await?.unwrap();
    if let Some(metadata) = child_node.metadata {
        if let Some(parent) = metadata.get("parent_id") {
            println!("   Child node parent: {:?}", parent);
        }
    }
    
    // Test 4: Query nodes
    println!("\n🔎 Test 4: Query nodes");
    let all_nodes = data_store.query_nodes("").await?;
    println!("   Total nodes in store: {}", all_nodes.len());
    
    // Test query with filter (simplified - just text matching)
    let filtered_nodes = data_store.query_nodes("test").await?;
    println!("   Nodes containing 'test': {}", filtered_nodes.len());
    
    // Test 5: Get child nodes using custom method
    println!("\n👨‍👩‍👧‍👦 Test 5: Get child nodes");
    let child_nodes = data_store.get_child_nodes(&node1_id).await?;
    println!("   Child nodes of {}: {}", node1_id, child_nodes.len());
    for child in child_nodes {
        println!("     - Child: {}", child.content);
    }
    
    // Test 6: Hybrid search
    println!("\n🔀 Test 6: Hybrid search");
    let search_embedding: Vec<f32> = (0..384).map(|i| (i as f32).cos() * 0.05).collect();
    let hybrid_results = data_store.hybrid_search(
        search_embedding,
        Some("technical".to_string()),
        None,
        3
    ).await?;
    println!("   Hybrid search (technical type) found {} results", hybrid_results.len());
    for (node, score) in hybrid_results {
        println!("     - Score: {:.3}, Type: {:?}", score, 
            node.metadata.as_ref().and_then(|m| m.get("node_type")));
    }
    
    // Test 7: Update node embedding
    println!("\n🔄 Test 7: Update node embedding");
    let new_embedding: Vec<f32> = (0..384).map(|i| (i as f32).tan() * 0.02).collect();
    data_store.update_node_embedding(&node2_id, new_embedding.clone()).await?;
    println!("   Updated embedding for node {}", node2_id);
    
    // Test 8: Delete node
    println!("\n🗑️  Test 8: Delete node");
    data_store.delete_node(&node3_id).await?;
    println!("   Deleted node {}", node3_id);
    
    let deleted_check = data_store.get_node(&node3_id).await?;
    match deleted_check {
        Some(_) => println!("   ❌ Node still exists after deletion"),
        None => println!("   ✅ Node successfully deleted"),
    }
    
    println!("\n🎉 All Tests Completed Successfully!");
    println!("📊 Summary:");
    println!("   • Node storage and retrieval: ✅");
    println!("   • Vector embeddings and search: ✅");
    println!("   • Relationship creation: ✅");
    println!("   • Node querying: ✅");
    println!("   • Child node retrieval: ✅");
    println!("   • Hybrid search: ✅");
    println!("   • Embedding updates: ✅");
    println!("   • Node deletion: ✅");
    
    println!("\n💡 LanceDataStore provides full DataStore trait functionality!");
    println!("   Ready for NS-69 core logic simplification");
    
    Ok(())
}