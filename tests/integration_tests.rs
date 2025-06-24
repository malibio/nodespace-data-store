use nodespace_core_types::{Node, NodeId};
use nodespace_data_store::{DataStore, SurrealDataStore};

#[tokio::test]
async fn test_store_and_retrieve_node() {
    let store = SurrealDataStore::new("memory").await.unwrap();

    let node_id = NodeId::new();
    let node = Node::with_id(node_id.clone(), serde_json::json!("Test content"));

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

    let node1 = Node::with_id(node1_id.clone(), serde_json::json!("First node"));

    let node2 = Node::with_id(node2_id.clone(), serde_json::json!("Second node"));

    // Store both nodes
    store.store_node(node1).await.unwrap();
    store.store_node(node2).await.unwrap();

    // Create a relationship
    store
        .create_relationship(&node1_id, &node2_id, "connects_to")
        .await
        .unwrap();

    // Get relationships for node1
    let relationships = store.get_node_relationships(&node1_id).await.unwrap();
    assert!(relationships.is_array() || relationships.is_object());
}

#[tokio::test]
async fn test_vector_storage_and_search() {
    let store = SurrealDataStore::new("memory").await.unwrap();

    let node1_id = NodeId::new();
    let node1 = Node::with_id(
        node1_id.clone(),
        serde_json::json!("Rust is a systems programming language"),
    );

    let node2_id = NodeId::new();
    let node2 = Node::with_id(
        node2_id.clone(),
        serde_json::json!("Python is a high-level programming language"),
    );

    // Sample embeddings (in reality these would come from an NLP engine)
    let embedding1 = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    let embedding2 = vec![0.2, 0.3, 0.4, 0.5, 0.6];

    // Store nodes with embeddings
    store
        .store_node_with_embedding(node1, embedding1.clone())
        .await
        .unwrap();
    store
        .store_node_with_embedding(node2, embedding2.clone())
        .await
        .unwrap();

    // Search for similar nodes
    let similar_nodes = store
        .search_similar_nodes(embedding1.clone(), 2)
        .await
        .unwrap();

    // Should find at least the exact match
    assert!(!similar_nodes.is_empty());

    // The first result should be the most similar (exact match)
    let (first_node, _score) = &similar_nodes[0];
    assert_eq!(first_node.id, node1_id);
}

#[tokio::test]
async fn test_update_node_embedding() {
    let store = SurrealDataStore::new("memory").await.unwrap();

    let node_id = NodeId::new();
    let node = Node::with_id(
        node_id.clone(),
        serde_json::json!("Test content for embedding update"),
    );

    // Store node without embedding first
    store.store_node(node).await.unwrap();

    // Add embedding to the node
    let embedding = vec![0.7, 0.8, 0.9, 1.0, 1.1];
    store
        .update_node_embedding(&node_id, embedding.clone())
        .await
        .unwrap();

    // Search should now find this node
    let similar_nodes = store
        .search_similar_nodes(embedding.clone(), 1)
        .await
        .unwrap();
    assert!(!similar_nodes.is_empty());

    let (found_node, _score) = &similar_nodes[0];
    assert_eq!(found_node.id, node_id);
}

#[tokio::test]
async fn test_semantic_search_with_embedding() {
    let store = SurrealDataStore::new("memory").await.unwrap();

    let node1_id = NodeId::new();
    let node1 = Node::with_id(
        node1_id.clone(),
        serde_json::json!("AI and machine learning fundamentals"),
    );

    let node2_id = NodeId::new();
    let node2 = Node::with_id(
        node2_id.clone(),
        serde_json::json!("Database management systems"),
    );

    let node3_id = NodeId::new();
    let node3 = Node::with_id(
        node3_id.clone(),
        serde_json::json!("Neural networks and deep learning"),
    );

    // Sample embeddings representing semantic similarity
    let ai_embedding = vec![0.9, 0.8, 0.1, 0.2, 0.1]; // AI-related
    let db_embedding = vec![0.1, 0.2, 0.9, 0.8, 0.1]; // Database-related
    let ml_embedding = vec![0.8, 0.9, 0.1, 0.1, 0.2]; // ML-related (similar to AI)

    // Store nodes with embeddings
    store
        .store_node_with_embedding(node1, ai_embedding.clone())
        .await
        .unwrap();
    store
        .store_node_with_embedding(node2, db_embedding.clone())
        .await
        .unwrap();
    store
        .store_node_with_embedding(node3, ml_embedding.clone())
        .await
        .unwrap();

    // Search using the new semantic_search_with_embedding method
    let query_embedding = vec![0.85, 0.85, 0.1, 0.15, 0.15]; // Should be closest to AI/ML nodes
    let search_results = store
        .semantic_search_with_embedding(query_embedding, 2)
        .await
        .unwrap();

    // Should find 2 results
    assert_eq!(search_results.len(), 2);

    // Results should be ordered by similarity score (descending)
    let (_first_result, first_score) = &search_results[0];
    let (_second_result, second_score) = &search_results[1];

    // First score should be higher than second
    assert!(first_score >= second_score);

    // Should find AI/ML related content, not database content
    let found_ids: Vec<&NodeId> = search_results.iter().map(|(node, _)| &node.id).collect();
    assert!(found_ids.contains(&&node1_id) || found_ids.contains(&&node3_id));
    assert!(!found_ids.contains(&&node2_id)); // Database content should be less similar
}
