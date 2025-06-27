//! Test basic LanceDB operations to understand the API

use arrow_array::{ListArray, RecordBatch, RecordBatchIterator, StringArray};
use arrow_schema::{DataType, Field, Schema};
use futures::TryStreamExt;
use lancedb::{
    connect,
    query::{ExecutableQuery, QueryBase},
};
use std::error::Error;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 Testing Basic LanceDB Operations\n");

    // Connect to LanceDB
    let uri = "/tmp/test_lancedb";
    let db = connect(uri).execute().await?;
    println!("✅ Connected to LanceDB at: {}", uri);

    // Define a simple schema
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("content", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::List(Arc::new(Field::new("item", DataType::Float32, true))),
            false,
        ),
    ]));

    // Create some test data
    let ids = StringArray::from(vec!["1", "2", "3"]);
    let contents = StringArray::from(vec!["Hello world", "LanceDB test", "Vector database"]);

    // Create vector data - need to wrap float values in Some()
    let vectors = ListArray::from_iter_primitive::<arrow_array::types::Float32Type, _, _>(vec![
        Some(vec![Some(0.1), Some(0.2), Some(0.3)]),
        Some(vec![Some(0.4), Some(0.5), Some(0.6)]),
        Some(vec![Some(0.7), Some(0.8), Some(0.9)]),
    ]);

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(ids), Arc::new(contents), Arc::new(vectors)],
    )?;

    println!("✅ Created RecordBatch with {} rows", batch.num_rows());

    // Create table using RecordBatchIterator
    let batches = vec![batch];
    let reader = RecordBatchIterator::new(batches.into_iter().map(Ok), schema.clone());
    let table = db.create_table("test_table", reader).execute().await?;
    println!("✅ Created table: test_table");

    // Query the table
    let results = table.query().limit(10).execute().await?;
    println!("✅ Query executed");

    // Convert to RecordBatch
    let batches: Vec<RecordBatch> = results.try_collect().await?;
    println!("✅ Retrieved {} batches", batches.len());

    for (i, batch) in batches.iter().enumerate() {
        println!("  Batch {}: {} rows", i, batch.num_rows());
    }

    Ok(())
}
