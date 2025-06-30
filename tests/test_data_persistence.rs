//! Test that verifies actual data persistence between database sessions
//! This test validates that the NS-104 placeholder fixes are working correctly

use nodespace_core_types::Node;
use nodespace_data_store::{LanceDataStore, DataStore};
use std::fs;

#[tokio::test]
async fn test_data_persists_between_sessions() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = "data/test_persistence_unique.db";
    
    // Clean up any existing test database
    if std::path::Path::new(db_path).exists() {
        fs::remove_dir_all(db_path)?;
    }

    // Session 1: Store data
    let test_content = "This content should persist between sessions";
    let node_id = {
        let data_store = LanceDataStore::new(db_path).await?;
        let node = Node::new(serde_json::Value::String(test_content.to_string()));
        let stored_id = node.id.clone();
        data_store.store_node(node).await?;
        stored_id
    }; // data_store goes out of scope, connection closed

    // Session 2: Retrieve data (new connection)
    {
        let data_store = LanceDataStore::new(db_path).await?;
        let retrieved = data_store.get_node(&node_id).await?;
        
        assert!(retrieved.is_some(), "Node should persist between sessions");
        let retrieved_node = retrieved.unwrap();
        assert_eq!(retrieved_node.id, node_id);
        
        let retrieved_content = retrieved_node.content.as_str().unwrap();
        assert_eq!(retrieved_content, test_content, "Content should match exactly");
    }

    // Session 3: Update data
    let updated_content = "Updated content that should also persist";
    {
        let data_store = LanceDataStore::new(db_path).await?;
        let mut node = data_store.get_node(&node_id).await?.unwrap();
        node.content = serde_json::Value::String(updated_content.to_string());
        data_store.update_node(node).await?;
    }

    // Session 4: Verify update persisted
    {
        let data_store = LanceDataStore::new(db_path).await?;
        let retrieved = data_store.get_node(&node_id).await?;
        
        assert!(retrieved.is_some(), "Updated node should persist");
        let retrieved_node = retrieved.unwrap();
        let retrieved_content = retrieved_node.content.as_str().unwrap();
        assert_eq!(retrieved_content, updated_content, "Updated content should persist");
    }

    // Session 5: Delete data
    {
        let data_store = LanceDataStore::new(db_path).await?;
        data_store.delete_node(&node_id).await?;
    }

    // Session 6: Verify deletion persisted
    {
        let data_store = LanceDataStore::new(db_path).await?;
        let retrieved = data_store.get_node(&node_id).await?;
        assert!(retrieved.is_none(), "Deleted node should not exist");
    }

    // Clean up test database
    if std::path::Path::new(db_path).exists() {
        fs::remove_dir_all(db_path)?;
    }

    println!("✅ All data persistence tests passed!");
    println!("✅ NS-104: LanceDB placeholders successfully replaced with real persistence");
    
    Ok(())
}

#[tokio::test]
async fn test_multiple_nodes_persistence() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = "data/test_multi_persistence_clean.db";
    
    // Clean up any existing test database
    if std::path::Path::new(db_path).exists() {
        fs::remove_dir_all(db_path)?;
    }

    let test_nodes = vec![
        "First node content",
        "Second node content", 
        "Third node content"
    ];

    // Session 1: Store multiple nodes
    let node_ids = {
        let data_store = LanceDataStore::new(db_path).await?;
        let mut ids = Vec::new();
        
        for content in &test_nodes {
            let node = Node::new(serde_json::Value::String(content.to_string()));
            let id = node.id.clone();
            data_store.store_node(node).await?;
            ids.push(id);
        }
        ids
    };

    // Session 2: Retrieve all nodes (new connection)
    {
        let data_store = LanceDataStore::new(db_path).await?;
        
        for (i, node_id) in node_ids.iter().enumerate() {
            let retrieved = data_store.get_node(node_id).await?;
            assert!(retrieved.is_some(), "Node {} should persist", i);
            
            let retrieved_node = retrieved.unwrap();
            let retrieved_content = retrieved_node.content.as_str().unwrap();
            assert_eq!(retrieved_content, test_nodes[i], "Content should match for node {}", i);
        }
    }

    // Clean up test database
    if std::path::Path::new(db_path).exists() {
        fs::remove_dir_all(db_path)?;
    }

    println!("✅ Multiple nodes persistence test passed!");
    
    Ok(())
}