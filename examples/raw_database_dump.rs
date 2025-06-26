use surrealdb::engine::local::RocksDb;
use surrealdb::Surreal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Raw database dump to examine relationship storage...");

    // Connect directly to SurrealDB without the wrapper - match DataStore exactly
    let db = Surreal::new::<RocksDb>("./data/sample.db").await?;
    db.use_ns("nodespace").use_db("nodes").await?;

    println!("\n📊 Skipping table info due to parsing complexity");

    // Try multiple table names to see what exists
    let table_names = vec!["text", "nodes", "contains", "date"];

    for table_name in table_names {
        println!("\n📋 {} table:", table_name);
        let count_query = format!("SELECT COUNT() FROM {}", table_name);
        match db.query(&count_query).await {
            Ok(mut result) => {
                if let Ok(count) = result.take::<Vec<surrealdb::sql::Value>>(0) {
                    println!("  Count: {:?}", count);

                    // Get a few sample records
                    let sample_query = format!("SELECT * FROM {} LIMIT 3", table_name);
                    if let Ok(mut sample_result) = db.query(&sample_query).await {
                        if let Ok(samples) = sample_result.take::<Vec<surrealdb::sql::Value>>(0) {
                            for (i, sample) in samples.iter().enumerate() {
                                println!("  Sample {}: {:#?}", i + 1, sample);
                            }
                        }
                    }
                }
            }
            Err(e) => println!("  Error: {}", e),
        }
    }

    println!("\n✅ Raw database examination complete");

    Ok(())
}
