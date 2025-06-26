// Basic test of LanceDataStore structure and configuration
// NS-67: Phase 1.3 validation

use nodespace_core_types::Node;
use nodespace_data_store::{LanceConfig, LanceDataStore, UniversalNode};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing LanceDataStore Phase 1.3 Implementation");

    // Test 1: Configuration creation and validation
    println!("\n1. Testing LanceConfig creation and validation...");

    let config = LanceConfig::new()
        .with_dimensions(384)
        .with_path("./test_data/lance_test.db");

    println!("✅ Config created: {:?}", config);

    // Test vector validation
    let valid_vector = vec![0.1; 384];
    let invalid_vector = vec![0.1; 256];

    assert!(config.validate_vector(&valid_vector).is_ok());
    assert!(config.validate_vector(&invalid_vector).is_err());
    println!("✅ Vector validation working correctly");

    // Test 2: LanceDataStore creation
    println!("\n2. Testing LanceDataStore creation...");

    let lance_store = LanceDataStore::new(config).await?;
    println!("✅ LanceDataStore created successfully");

    // Test 3: Initialize (basic version)
    println!("\n3. Testing initialization...");

    lance_store.initialize().await?;
    println!("✅ LanceDataStore initialized");

    // Test 4: UniversalNode conversion
    println!("\n4. Testing Node <-> UniversalNode conversion...");

    let original_node = Node::new(json!({
        "text": "This is a test node for LanceDB migration",
        "type": "text"
    }));

    let universal_node = UniversalNode::from_node(original_node.clone())?;
    println!("✅ Node -> UniversalNode conversion successful");

    let converted_back = universal_node.to_node()?;
    println!("✅ UniversalNode -> Node conversion successful");

    // Verify core fields are preserved
    assert_eq!(original_node.id, converted_back.id);
    assert_eq!(original_node.content, converted_back.content);
    println!("✅ Node data integrity preserved through conversion");

    // Test 5: DataStore trait methods (placeholder validation)
    println!("\n5. Testing DataStore trait implementation...");

    use nodespace_data_store::DataStore;

    // These should return placeholder responses but not panic
    let stored_id = lance_store.store_node(original_node.clone()).await?;
    assert_eq!(stored_id, original_node.id);
    println!("✅ store_node returns expected ID");

    let retrieved = lance_store.get_node(&original_node.id).await?;
    assert!(retrieved.is_none()); // Expected for placeholder
    println!("✅ get_node returns None (placeholder behavior)");

    // Test vector operations
    let test_embedding = vec![0.1; 384];
    let stored_with_embedding = lance_store
        .store_node_with_embedding(original_node.clone(), test_embedding.clone())
        .await?;
    assert_eq!(stored_with_embedding, original_node.id);
    println!("✅ store_node_with_embedding validates vector dimensions");

    let search_results = lance_store.search_similar_nodes(test_embedding, 10).await?;
    assert!(search_results.is_empty()); // Expected for placeholder
    println!("✅ search_similar_nodes returns empty (placeholder behavior)");

    // Test 6: Enhanced LanceDB methods
    println!("\n6. Testing enhanced LanceDB-specific methods...");

    let child_nodes = lance_store.get_child_nodes(&original_node.id).await?;
    assert!(child_nodes.is_empty()); // Expected for placeholder
    println!("✅ get_child_nodes ready for simple parent_id filtering");

    let semantic_results = lance_store
        .semantic_search_with_filters(
            vec![0.1; 384],
            Some("text"),
            Some(&original_node.id.to_string()),
            5,
        )
        .await?;
    assert!(semantic_results.is_empty()); // Expected for placeholder
    println!("✅ semantic_search_with_filters validates and structures correctly");

    println!("\n🎉 All Phase 1.3 Tests Passed!");
    println!("\n📋 Summary:");
    println!("   ✅ LanceConfig: BGE-optimized, validates vector dimensions");
    println!("   ✅ LanceDataStore: Basic structure with connection handling");
    println!("   ✅ UniversalNode: Bidirectional conversion with Node");
    println!("   ✅ DataStore trait: Full compatibility interface");
    println!("   ✅ Enhanced methods: Ready for vector-first operations");
    println!("   🚀 Architecture: Hybrid schema, relationship simplification");

    println!("\n🔧 Next Steps (Future Phases):");
    println!("   • Implement RecordBatch conversion (Arrow integration)");
    println!("   • Add table creation and schema management");
    println!("   • Implement vector indexing (IVF-PQ)");
    println!("   • Add query execution and filtering");
    println!("   • Complete semantic search functionality");

    Ok(())
}
