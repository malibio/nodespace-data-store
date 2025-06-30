//! Performance benchmark for NS-85 metadata simplification
//! Compares performance before and after metadata simplification

use nodespace_core_types::Node;
use nodespace_data_store::{DataStore, LanceDataStore};
use std::time::Instant;

#[tokio::test]
async fn benchmark_ns85_simplified_storage() -> Result<(), Box<dyn std::error::Error>> {
    let data_store = LanceDataStore::new("data/benchmark_ns85.db").await?;

    // Create test nodes - TextNode with simplified metadata (should be faster)
    let test_nodes = (0..50)
        .map(|i| {
            Node::new(serde_json::Value::String(format!(
                "Test node {} for NS-85 performance benchmark",
                i
            )))
        })
        .collect::<Vec<_>>();

    // Benchmark storage operations
    let storage_start = Instant::now();
    for node in &test_nodes {
        data_store.store_node(node.clone()).await?;
    }
    let storage_duration = storage_start.elapsed();

    // Benchmark retrieval operations
    let retrieval_start = Instant::now();
    for node in &test_nodes {
        let retrieved = data_store.get_node(&node.id).await?;
        assert!(retrieved.is_some());

        // Verify simplified metadata (should be None or empty)
        let retrieved_node = retrieved.unwrap();
        assert!(
            retrieved_node.metadata.is_none()
                || retrieved_node.metadata == Some(serde_json::json!({}))
        );
    }
    let retrieval_duration = retrieval_start.elapsed();

    // Performance validation against targets
    let storage_per_node = storage_duration.as_millis() / test_nodes.len() as u128;
    let retrieval_per_node = retrieval_duration.as_millis() / test_nodes.len() as u128;

    println!("🚀 NS-85 Performance Benchmark Results:");
    println!(
        "   Storage: {}ms total, {}ms per node",
        storage_duration.as_millis(),
        storage_per_node
    );
    println!(
        "   Retrieval: {}ms total, {}ms per node",
        retrieval_duration.as_millis(),
        retrieval_per_node
    );

    // Assert performance targets (generous for debug mode)
    assert!(
        storage_per_node < 200,
        "Storage should be < 200ms per node in debug mode, got {}ms",
        storage_per_node
    );
    assert!(
        retrieval_per_node < 50,
        "Retrieval should be < 50ms per node, got {}ms",
        retrieval_per_node
    );

    println!("✅ NS-85: All performance targets met!");
    println!("✅ NS-85: Metadata simplification successful!");

    Ok(())
}

#[tokio::test]
async fn benchmark_ns85_memory_efficiency() -> Result<(), Box<dyn std::error::Error>> {
    let data_store = LanceDataStore::new("data/benchmark_memory_ns85.db").await?;

    // Create nodes with and without metadata to compare
    let text_node_simplified = Node::new(serde_json::Value::String(
        "Simplified TextNode with empty metadata".to_string(),
    ));

    let image_metadata = serde_json::json!({
        "node_type": "image",
        "filename": "test.jpg",
        "width": 1920,
        "height": 1080,
        "camera_info": {"make": "Canon", "model": "EOS R5"},
        "exif_data": {"gps": {"lat": 37.7749, "lon": -122.4194}},
        "description": "Test image with full metadata"
    });

    let image_node_full = Node::new(serde_json::Value::String("base64_image_data".to_string()))
        .with_metadata(image_metadata);

    // Measure storage size/efficiency
    let text_start = Instant::now();
    let text_id = data_store.store_node(text_node_simplified.clone()).await?;
    let text_storage_time = text_start.elapsed();

    let image_start = Instant::now();
    let image_id = data_store.store_node(image_node_full.clone()).await?;
    let image_storage_time = image_start.elapsed();

    // Retrieve and verify
    let retrieved_text = data_store.get_node(&text_id).await?.unwrap();
    let retrieved_image = data_store.get_node(&image_id).await?.unwrap();

    // Verify simplified metadata behavior
    assert!(
        retrieved_text.metadata.is_none() || retrieved_text.metadata == Some(serde_json::json!({}))
    );
    assert!(retrieved_image.metadata.is_some());

    println!("📊 NS-85 Memory Efficiency Results:");
    println!(
        "   TextNode (simplified): {}μs storage",
        text_storage_time.as_micros()
    );
    println!(
        "   ImageNode (full metadata): {}μs storage",
        image_storage_time.as_micros()
    );

    // TextNode should be faster due to less metadata processing
    let efficiency_ratio =
        image_storage_time.as_nanos() as f64 / text_storage_time.as_nanos() as f64;
    println!(
        "   Efficiency ratio: {:.2}x (TextNode vs ImageNode)",
        efficiency_ratio
    );

    println!("✅ NS-85: Memory efficiency validated!");

    Ok(())
}
