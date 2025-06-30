// NS-115: Performance benchmarks to validate O(1) vs O(N) hierarchy query claims
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nodespace_core_types::{Node, NodeId, NodeSpaceResult};
use nodespace_data_store::{DataStore, LanceDataStore};
use std::time::Duration;
use tokio::runtime::Runtime;

async fn setup_test_data(data_store: &LanceDataStore, size: usize) -> NodeSpaceResult<Vec<NodeId>> {
    let mut node_ids = Vec::new();

    // Create a root node (date node)
    let root_node = Node {
        id: NodeId::from_string("root-2025-06-30".to_string()),
        content: serde_json::Value::String("📅 2025-06-30 - Test Hierarchy".to_string()),
        metadata: Some(serde_json::json!({
            "node_type": "date",
            "root_id": "root-2025-06-30",
            "root_type": "date"
        })),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        parent_id: None,
        next_sibling: None,
        previous_sibling: None,
    };

    let root_id = data_store.store_node(root_node).await?;
    node_ids.push(root_id.clone());

    // Create child nodes under this root
    for i in 0..size {
        let child_node = Node {
            id: NodeId::from_string(format!("child-{}", i)),
            content: serde_json::Value::String(format!("Test content {}", i)),
            metadata: Some(serde_json::json!({
                "node_type": "text",
                "parent_id": root_id.to_string(),
                "root_id": root_id.to_string(),
                "root_type": "date"
            })),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            parent_id: Some(root_id.clone()),
            next_sibling: None,
            previous_sibling: None,
        };

        let child_id = data_store.store_node(child_node).await?;
        node_ids.push(child_id);
    }

    Ok(node_ids)
}

fn bench_hierarchy_queries(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("hierarchy_operations");
    group.measurement_time(Duration::from_secs(30));

    // Test with different dataset sizes
    for size in [100, 1000, 5000] {
        let data_store = rt.block_on(async {
            LanceDataStore::new(&format!("data/benchmark_ns115_{}.db", size))
                .await
                .expect("Failed to create data store")
        });

        rt.block_on(async {
            setup_test_data(&data_store, size)
                .await
                .expect("Failed to setup test data");
        });

        let root_id = NodeId::from_string("root-2025-06-30".to_string());

        // Benchmark OLD approach: query_nodes("") - O(N) scan
        group.bench_function(format!("old_query_all_filter_memory_{}", size), |b| {
            b.iter(|| {
                rt.block_on(async {
                    let all_nodes = data_store.query_nodes("").await.unwrap();
                    // Filter in memory (simulating old approach)
                    let _filtered: Vec<_> = all_nodes
                        .into_iter()
                        .filter(|node| {
                            node.metadata
                                .as_ref()
                                .and_then(|m| m.get("root_id"))
                                .and_then(|v| v.as_str())
                                == Some("root-2025-06-30")
                        })
                        .collect();
                })
            })
        });

        // Benchmark NEW approach: get_nodes_by_root - O(1) indexed
        group.bench_function(format!("new_root_based_query_{}", size), |b| {
            b.iter(|| {
                rt.block_on(async {
                    let _nodes = data_store
                        .get_nodes_by_root(black_box(&root_id))
                        .await
                        .unwrap();
                })
            })
        });

        // Benchmark NEW approach with type filtering
        group.bench_function(format!("new_root_type_query_{}", size), |b| {
            b.iter(|| {
                rt.block_on(async {
                    let _nodes = data_store
                        .get_nodes_by_root_and_type(black_box(&root_id), black_box("text"))
                        .await
                        .unwrap();
                })
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_hierarchy_queries);
criterion_main!(benches);
