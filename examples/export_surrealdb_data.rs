//! Example: Export all SurrealDB data for LanceDB migration
//!
//! This example demonstrates how to use the SurrealDBExporter to extract
//! all NodeSpace data from SurrealDB in preparation for migration to LanceDB.

use nodespace_data_store::migration::surrealdb_export::SurrealDBExporter;
use std::path::PathBuf;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Starting SurrealDB data export for LanceDB migration...");

    // Configure paths
    let db_path = "data/sample.db";
    let export_path = PathBuf::from("migration_export");

    // Create exporter
    let exporter = SurrealDBExporter::new(db_path, export_path.clone()).await?;

    // Perform comprehensive export
    println!("📊 Exporting all data tables and relationships...");
    let manifest = exporter.export_all_data().await?;

    // Display export summary
    println!("\n✅ Export completed successfully!");
    println!("📋 Export Summary:");
    println!("   • Total records exported: {}", manifest.total_records);
    println!("   • Export files created: {}", manifest.export_files.len());
    println!("   • Export timestamp: {}", manifest.export_timestamp);
    println!("   • Validation checksum: {}", manifest.validation_checksum);

    println!("\n📁 Exported files:");
    for file in &manifest.export_files {
        println!(
            "   • {} ({} records, {} bytes)",
            file.file_name, file.record_count, file.file_size_bytes
        );
    }

    println!("\n🗄️  Database Information:");
    println!("   • Total tables: {}", manifest.database_info.total_tables);
    for (table, stats) in &manifest.database_info.table_statistics {
        println!(
            "   • {} table: {} records, embeddings: {}",
            table,
            stats.record_count,
            if stats.has_embeddings { "✓" } else { "✗" }
        );
    }

    println!("\n📄 Files saved to: {}", export_path.display());
    println!("   • export_manifest.json - Complete export metadata");
    println!("   • *_nodes.json - Node data by type");
    println!("   • *_relationships.json - Relationship data");
    println!("   • database_metadata.json - Schema and configuration");

    println!("\n🎯 Next Steps:");
    println!("   1. Verify export completeness using the manifest");
    println!("   2. Run data validation checks");
    println!("   3. Begin LanceDB import process");
    println!("   4. Validate migrated data integrity");

    Ok(())
}
