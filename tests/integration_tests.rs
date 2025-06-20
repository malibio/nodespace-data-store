use nodespace_core_types::{Node, NodeId};
use nodespace_data_store::{DataStore, SurrealDataStore};

#[tokio::test]
async fn test_store_and_retrieve_node() {
    let store = SurrealDataStore::new("memory").await.unwrap();
    
    let node_id = NodeId::new();
    let node = Node::with_id(
        node_id.clone(),
        serde_json::json!("Test content"),
    );
    
    // Store the node
    let stored_id = store.store_node(node.clone()).await.unwrap();
    assert_eq!(stored_id, node_id);
    
    // Retrieve the node
    let retrieved = store.get_node(&node_id).await.unwrap();
    assert!(retrieved.is_some());
    
    let retrieved_node = retrieved.unwrap();
    assert_eq!(retrieved_node.id, node.id);
    assert_eq!(retrieved_node.content, node.content);
}

#[tokio::test]
async fn test_delete_node() {
    let store = SurrealDataStore::new("memory").await.unwrap();
    
    let node_id = NodeId::new();
    let node = Node::with_id(
        node_id.clone(),
        serde_json::json!("Test content for deletion"),
    );
    
    // Store the node
    store.store_node(node).await.unwrap();
    
    // Delete the node
    store.delete_node(&node_id).await.unwrap();
    
    // Verify it's gone
    let retrieved = store.get_node(&node_id).await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_query_nodes() {
    let store = SurrealDataStore::new("memory").await.unwrap();
    
    // Test querying nodes
    let result = store.query_nodes("SELECT * FROM nodes").await.unwrap();
    assert!(result.is_empty()); // Should be empty initially
}

#[tokio::test]
async fn test_create_relationship() {
    let store = SurrealDataStore::new("memory").await.unwrap();
    
    let node1_id = NodeId::new();
    let node2_id = NodeId::new();
    
    let node1 = Node::with_id(
        node1_id.clone(),
        serde_json::json!("First node"),
    );
    
    let node2 = Node::with_id(
        node2_id.clone(),
        serde_json::json!("Second node"),
    );
    
    // Store both nodes
    store.store_node(node1).await.unwrap();
    store.store_node(node2).await.unwrap();
    
    // Create a relationship
    store.create_relationship(
        &node1_id,
        &node2_id,
        "connects_to",
    ).await.unwrap();
    
    // Get relationships for node1
    let relationships = store.get_node_relationships(&node1_id).await.unwrap();
    assert!(relationships.is_array() || relationships.is_object());
}