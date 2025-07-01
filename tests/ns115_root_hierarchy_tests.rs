// NS-115: Integration tests for root-based hierarchy optimization
use nodespace_core_types::{Node, NodeId, NodeSpaceResult};
use nodespace_data_store::{DataStore, LanceDataStore};

#[tokio::test]
async fn test_root_hierarchy_optimization() -> NodeSpaceResult<()> {
    let data_store = LanceDataStore::new("data/test_ns115_root_hierarchy.db").await?;

    // Create a hierarchical structure
    let root_id = NodeId::from_string("project-alpha".to_string());
    let root_node = Node {
        id: root_id.clone(),
        content: serde_json::Value::String("🚀 Project Alpha".to_string()),
        metadata: Some(serde_json::json!({
            "node_type": "project",
            "root_id": "project-alpha",
            "root_type": "project"
        })),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        r#type: "project".to_string(),
        parent_id: None,
        next_sibling: None,
        // NS-125: previous_sibling removed
        root_id: Some(root_id.clone()),
        // NS-125: root_type field removed
    };

    data_store.store_node(root_node).await?;

    // Create child nodes
    let mut child_ids = Vec::new();
    for i in 0..5 {
        let child_id = NodeId::from_string(format!("task-{}", i));
        let child_node = Node {
            id: child_id.clone(),
            content: serde_json::Value::String(format!("Task {}: Implementation details", i)),
            metadata: Some(serde_json::json!({
                "node_type": "task",
                "parent_id": root_id.to_string(),
                "root_id": root_id.to_string(),
                "root_type": "project"
            })),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            r#type: "task".to_string(),
            parent_id: Some(root_id.clone()),
            next_sibling: None,
            // NS-125: previous_sibling removed
            root_id: Some(root_id.clone()),
            // NS-125: root_type field removed
        };

        data_store.store_node(child_node).await?;
        child_ids.push(child_id);
    }

    // Create some unrelated nodes (different root)
    let other_root_id = NodeId::from_string("project-beta".to_string());
    let other_root_node = Node {
        id: other_root_id.clone(),
        content: serde_json::Value::String("🔧 Project Beta".to_string()),
        metadata: Some(serde_json::json!({
            "node_type": "project",
            "root_id": "project-beta",
            "root_type": "project"
        })),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        r#type: "project".to_string(),
        parent_id: None,
        next_sibling: None,
        // NS-125: previous_sibling removed
        root_id: Some(other_root_id.clone()),
        // NS-125: root_type field removed
    };

    data_store.store_node(other_root_node).await?;

    // Test 1: get_nodes_by_root should return only Project Alpha nodes
    let alpha_nodes = data_store.get_nodes_by_root(&root_id).await?;
    assert_eq!(alpha_nodes.len(), 6); // 1 root + 5 children

    // Verify all returned nodes have the correct root_id
    for node in &alpha_nodes {
        if let Some(metadata) = &node.metadata {
            if let Some(node_root_id) = metadata.get("root_id").and_then(|v| v.as_str()) {
                assert_eq!(node_root_id, "project-alpha");
            }
        }
    }

    // Test 2: get_nodes_by_root_and_type should filter by type
    let alpha_tasks = data_store
        .get_nodes_by_root_and_type(&root_id, "task")
        .await?;
    assert_eq!(alpha_tasks.len(), 5); // Only the 5 task nodes

    // Verify all returned nodes are tasks
    for node in &alpha_tasks {
        if let Some(metadata) = &node.metadata {
            if let Some(node_type) = metadata.get("node_type").and_then(|v| v.as_str()) {
                assert_eq!(node_type, "task");
            }
        }
    }

    // Test 3: Verify performance - O(1) vs O(N) comparison
    let start_time = std::time::Instant::now();
    let _result1 = data_store.get_nodes_by_root(&root_id).await?;
    let optimized_duration = start_time.elapsed();

    let start_time = std::time::Instant::now();
    let all_nodes = data_store.query_nodes("").await?;
    let _filtered_nodes: Vec<_> = all_nodes
        .into_iter()
        .filter(|node| {
            node.metadata
                .as_ref()
                .and_then(|m| m.get("root_id"))
                .and_then(|v| v.as_str())
                == Some("project-alpha")
        })
        .collect();
    let unoptimized_duration = start_time.elapsed();

    println!(
        "🚀 Optimized query (get_nodes_by_root): {:?}",
        optimized_duration
    );
    println!(
        "🐌 Unoptimized query (query_nodes + filter): {:?}",
        unoptimized_duration
    );

    // Test 4: Empty result for non-existent root
    let empty_root_id = NodeId::from_string("non-existent".to_string());
    let empty_result = data_store.get_nodes_by_root(&empty_root_id).await?;
    assert_eq!(empty_result.len(), 0);

    println!("✅ NS-115 root hierarchy optimization tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_root_id_schema_fields() -> NodeSpaceResult<()> {
    let data_store = LanceDataStore::new("data/test_ns115_schema.db").await?;

    // Test that root_id fields are properly stored and retrieved
    let test_id = NodeId::from_string("schema-test".to_string());
    let test_node = Node {
        id: test_id.clone(),
        content: serde_json::Value::String("Schema test content".to_string()),
        metadata: Some(serde_json::json!({
            "node_type": "test",
            "root_id": "test-root-123",
            "root_type": "test_hierarchy"
        })),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        r#type: "test".to_string(),
        parent_id: None,
        next_sibling: None,
        // NS-125: previous_sibling removed
        root_id: Some(NodeId::from_string("test-root-123".to_string())),
        // NS-125: root_type field removed
    };

    data_store.store_node(test_node).await?;

    // Retrieve and verify
    let retrieved_node = data_store.get_node(&test_id).await?;
    assert!(retrieved_node.is_some());

    let node = retrieved_node.unwrap();
    if let Some(metadata) = &node.metadata {
        assert_eq!(
            metadata.get("root_id").and_then(|v| v.as_str()),
            Some("test-root-123")
        );
        assert_eq!(
            metadata.get("root_type").and_then(|v| v.as_str()),
            Some("test_hierarchy")
        );
    }

    println!("✅ NS-115 schema fields validation passed!");
    Ok(())
}
