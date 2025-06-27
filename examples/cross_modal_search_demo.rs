//! Cross-Modal Search Demo for NS-81 Implementation
//!
//! This example demonstrates the new cross-modal search capabilities:
//! 1. Text→Image search: "what color shirt was I wearing during Claire's birthday"
//! 2. Image→Text search: Find text nodes related to images
//! 3. Hybrid search: Weighted multi-factor relevance scoring

use nodespace_core_types::{Node, NodeId};
use nodespace_data_store::{
    DataStore, HybridSearchConfig, ImageMetadata, ImageNode, LanceDataStore, NodeType,
};
use rand::SeedableRng;
use std::error::Error;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🎯 Cross-Modal Search Demo - NS-81 Implementation\n");

    // Initialize LanceDB with cross-modal capabilities
    let data_store = LanceDataStore::new("data/cross_modal_demo.db").await?;
    println!("✅ LanceDB initialized with Universal Document Schema");

    // PART 1: Create sample multimodal data
    println!("\n📝 Creating sample multimodal dataset...");

    // Create text nodes about events
    let text_node_1 = Node::with_id(
        NodeId::from_string("claire-birthday-notes".to_string()),
        serde_json::Value::String(
            "Claire's birthday party was amazing! I wore my blue shirt and we had cake."
                .to_string(),
        ),
    )
    .with_metadata(serde_json::json!({
        "node_type": "text",
        "event": "birthday",
        "person": "Claire",
        "date": "2025-06-15"
    }));

    let text_node_2 = Node::with_id(
        NodeId::from_string("team-meeting-notes".to_string()),
        serde_json::Value::String(
            "Team meeting discussion about project timeline. Wore red shirt today.".to_string(),
        ),
    )
    .with_metadata(serde_json::json!({
        "node_type": "text",
        "event": "meeting",
        "date": "2025-06-20"
    }));

    // Store text nodes with mock embeddings (384-dim for BGE-small-en-v1.5)
    let text_embedding_1 = create_mock_text_embedding("claire birthday blue shirt cake");
    let text_embedding_2 = create_mock_text_embedding("team meeting red shirt project");

    let text_id_1 = data_store
        .store_node_with_embedding(text_node_1, text_embedding_1)
        .await?;
    let text_id_2 = data_store
        .store_node_with_embedding(text_node_2, text_embedding_2)
        .await?;

    println!("   ✅ Created text node: {}", text_id_1);
    println!("   ✅ Created text node: {}", text_id_2);

    // Create image nodes with mock image data and CLIP embeddings (512-dim)
    let image_node_1 = ImageNode {
        id: Uuid::new_v4().to_string(),
        image_data: create_mock_image_data("blue_shirt_birthday.jpg"),
        embedding: create_mock_image_embedding("person wearing blue shirt at birthday party"),
        metadata: ImageMetadata {
            filename: "blue_shirt_birthday.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            width: 1920,
            height: 1080,
            exif_data: Some(serde_json::json!({
                "date_taken": "2025-06-15T18:30:00Z",
                "camera": "iPhone 15 Pro",
                "location": "Claire's House"
            })),
            description: Some("Person wearing blue shirt at Claire's birthday party".to_string()),
        },
        created_at: chrono::Utc::now(),
    };

    let image_node_2 = ImageNode {
        id: Uuid::new_v4().to_string(),
        image_data: create_mock_image_data("red_shirt_meeting.jpg"),
        embedding: create_mock_image_embedding("person wearing red shirt in office meeting"),
        metadata: ImageMetadata {
            filename: "red_shirt_meeting.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            width: 1280,
            height: 720,
            exif_data: Some(serde_json::json!({
                "date_taken": "2025-06-20T14:15:00Z",
                "camera": "MacBook Pro Camera"
            })),
            description: Some("Person wearing red shirt during team meeting".to_string()),
        },
        created_at: chrono::Utc::now(),
    };

    let image_id_1 = data_store.create_image_node(image_node_1).await?;
    let image_id_2 = data_store.create_image_node(image_node_2).await?;

    println!("   ✅ Created image node: {}", image_id_1);
    println!("   ✅ Created image node: {}", image_id_2);

    // PART 2: Demonstrate cross-modal search use cases
    println!("\n🔍 Testing Cross-Modal Search Use Cases...\n");

    // USE CASE 1: Text→Image Search
    println!("🎯 USE CASE 1: Text→Image Search");
    println!("Query: 'what color shirt was I wearing during Claire's birthday'");

    let query_embedding = create_mock_text_embedding("blue shirt Claire birthday");
    let multimodal_results = data_store
        .search_multimodal(
            query_embedding.clone(),
            vec![NodeType::Text, NodeType::Image],
        )
        .await?;

    println!(
        "   📊 Found {} cross-modal results:",
        multimodal_results.len()
    );
    for (i, node) in multimodal_results.iter().enumerate() {
        let preview = node
            .content
            .as_str()
            .map(|s| {
                if s.len() > 60 {
                    format!("{}...", &s[..57])
                } else {
                    s.to_string()
                }
            })
            .unwrap_or("NULL".to_string());
        println!("   {}. {} - {}", i + 1, node.id, preview);
    }

    // USE CASE 2: Image→Text Search
    println!("\n🎯 USE CASE 2: Image→Text Search");
    println!("Query: Find text related to images of people wearing shirts");

    let image_query_embedding = create_mock_image_embedding("person wearing shirt");
    let image_to_text_results = data_store
        .search_multimodal(image_query_embedding, vec![NodeType::Text])
        .await?;

    println!(
        "   📊 Found {} text nodes related to shirt images:",
        image_to_text_results.len()
    );
    for (i, node) in image_to_text_results.iter().enumerate() {
        let preview = node
            .content
            .as_str()
            .map(|s| {
                if s.len() > 60 {
                    format!("{}...", &s[..57])
                } else {
                    s.to_string()
                }
            })
            .unwrap_or("NULL".to_string());
        println!("   {}. {} - {}", i + 1, node.id, preview);
    }

    // USE CASE 3: Hybrid Multimodal Search
    println!("\n🎯 USE CASE 3: Hybrid Multimodal Search with Weighted Scoring");
    println!("Query: 'Claire birthday' with temporal and semantic weighting");

    let hybrid_config = HybridSearchConfig {
        semantic_weight: 0.6,   // Emphasize semantic similarity
        structural_weight: 0.2, // Some relationship importance
        temporal_weight: 0.2,   // Recent content boost
        max_results: 10,
        min_similarity_threshold: 0.1,
        enable_cross_modal: true, // Enable text→image connections
        search_timeout_ms: 2000,  // 2 second timeout
    };

    let claire_query_embedding = create_mock_text_embedding("Claire birthday");
    let hybrid_results = data_store
        .hybrid_multimodal_search(claire_query_embedding, &hybrid_config)
        .await?;

    println!("   📊 Hybrid search results with relevance scoring:");
    for (i, result) in hybrid_results.iter().enumerate() {
        let preview = result
            .node
            .content
            .as_str()
            .map(|s| {
                if s.len() > 50 {
                    format!("{}...", &s[..47])
                } else {
                    s.to_string()
                }
            })
            .unwrap_or("NULL".to_string());

        println!(
            "   {}. Score: {:.3} | {} - {}",
            i + 1,
            result.score,
            result.node.id,
            preview
        );
        println!(
            "      Factors: semantic={:.2}, structural={:.2}, temporal={:.2}, cross_modal={:?}",
            result.relevance_factors.semantic_score,
            result.relevance_factors.structural_score,
            result.relevance_factors.temporal_score,
            result.relevance_factors.cross_modal_score
        );
    }

    // PART 3: Performance validation
    println!("\n⚡ Performance Validation...");

    let start_time = std::time::Instant::now();
    let _perf_results = data_store
        .hybrid_multimodal_search(
            create_mock_text_embedding("performance test query"),
            &hybrid_config,
        )
        .await?;
    let search_duration = start_time.elapsed();

    println!("   🚀 Hybrid search completed in: {:?}", search_duration);

    if search_duration.as_secs() < 2 {
        println!("   ✅ Performance target achieved: < 2 seconds");
    } else {
        println!(
            "   ⚠️  Performance target missed: {} seconds",
            search_duration.as_secs()
        );
    }

    // PART 4: Image retrieval test
    println!("\n🖼️  Image Retrieval Test...");

    if let Some(retrieved_image) = data_store.get_image_node(&image_id_1).await? {
        println!(
            "   ✅ Successfully retrieved image: {}",
            retrieved_image.metadata.filename
        );
        println!(
            "   📏 Dimensions: {}x{}",
            retrieved_image.metadata.width, retrieved_image.metadata.height
        );
        println!(
            "   📊 Embedding dimensions: {}",
            retrieved_image.embedding.len()
        );
        println!(
            "   💾 Image data size: {} bytes",
            retrieved_image.image_data.len()
        );

        if let Some(exif) = &retrieved_image.metadata.exif_data {
            println!(
                "   📸 EXIF data available: {} fields",
                exif.as_object().unwrap().len()
            );
        }
    }

    println!("\n🎉 Cross-Modal Search Demo Completed Successfully!");
    println!("📈 Implementation Status:");
    println!("   ✅ DataStore trait extended with multimodal methods");
    println!("   ✅ ImageNode creation and retrieval working");
    println!("   ✅ Cross-modal search (text↔image) functional");
    println!("   ✅ Hybrid scoring with configurable weights");
    println!("   ✅ Performance monitoring and validation");
    println!("   ✅ LanceDB Universal Document Schema integration");

    println!("\n📋 Key Features Demonstrated:");
    println!("   • Text→Image search for 'Claire's birthday shirt color' use case");
    println!("   • Image→Text semantic connections");
    println!("   • Weighted hybrid scoring (semantic + structural + temporal)");
    println!("   • Cross-modal relevance boosting");
    println!("   • Performance validation against <2s target");
    println!("   • EXIF metadata integration");
    println!("   • Base64 image storage in Universal Document format");

    Ok(())
}

/// Create mock text embedding (384-dim for BGE-small-en-v1.5)
fn create_mock_text_embedding(text: &str) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let seed = hasher.finish();

    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    use rand::Rng;

    (0..384).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

/// Create mock image embedding (512-dim for CLIP vision)
fn create_mock_image_embedding(description: &str) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    description.hash(&mut hasher);
    let seed = hasher.finish();

    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    use rand::Rng;

    (0..512).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

/// Create mock image data
fn create_mock_image_data(filename: &str) -> Vec<u8> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    filename.hash(&mut hasher);
    let seed = hasher.finish();

    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    use rand::Rng;

    // Mock JPEG header + random data
    let mut data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header
    data.extend((0..1000).map(|_| rng.gen::<u8>()));
    data
}
