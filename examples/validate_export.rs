//! Example: Validate exported SurrealDB data for migration integrity
//!
//! This example demonstrates how to validate an export manifest and verify
//! data integrity before proceeding with LanceDB migration.

use nodespace_data_store::migration::surrealdb_export::ExportManifest;
use serde_json;
use std::fs;
use std::path::PathBuf;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Validating SurrealDB export for migration integrity...");

    let export_path = PathBuf::from("migration_export");
    let manifest_path = export_path.join("export_manifest.json");

    // Load and validate manifest
    if !manifest_path.exists() {
        eprintln!(
            "❌ Error: export_manifest.json not found in {}",
            export_path.display()
        );
        std::process::exit(1);
    }

    let manifest_content = fs::read_to_string(&manifest_path)?;
    let manifest: ExportManifest = serde_json::from_str(&manifest_content)?;

    println!("📋 Manifest loaded successfully");
    println!("   • Export timestamp: {}", manifest.export_timestamp);
    println!("   • Total records: {}", manifest.total_records);
    println!("   • Schema version: {}", manifest.schema_version);

    // Validate all export files exist
    println!("\n🗂️  Validating export files...");
    let mut total_validated_records = 0;
    let mut missing_files = Vec::new();
    let mut size_mismatches = Vec::new();

    for file_info in &manifest.export_files {
        let file_path = export_path.join(&file_info.file_name);

        if !file_path.exists() {
            missing_files.push(&file_info.file_name);
            continue;
        }

        // Validate file size
        let metadata = fs::metadata(&file_path)?;
        if metadata.len() != file_info.file_size_bytes {
            size_mismatches.push((
                &file_info.file_name,
                file_info.file_size_bytes,
                metadata.len(),
            ));
        }

        // Validate file contents can be parsed
        let content = fs::read_to_string(&file_path)?;
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(data) => {
                if let Some(record_count) = data.get("record_count").and_then(|v| v.as_u64()) {
                    total_validated_records += record_count as usize;
                    println!(
                        "   ✅ {} - {} records validated",
                        file_info.file_name, record_count
                    );
                } else {
                    println!(
                        "   ⚠️  {} - Could not validate record count",
                        file_info.file_name
                    );
                }
            }
            Err(e) => {
                println!("   ❌ {} - JSON parsing failed: {}", file_info.file_name, e);
            }
        }
    }

    // Report validation results
    println!("\n📊 Validation Results:");

    if missing_files.is_empty() {
        println!("   ✅ All export files present");
    } else {
        println!("   ❌ Missing files: {}", missing_files.len());
        for file in &missing_files {
            println!("      • {}", file);
        }
    }

    if size_mismatches.is_empty() {
        println!("   ✅ All file sizes match manifest");
    } else {
        println!("   ⚠️  File size mismatches: {}", size_mismatches.len());
        for (file, expected, actual) in &size_mismatches {
            println!(
                "      • {}: expected {} bytes, got {} bytes",
                file, expected, actual
            );
        }
    }

    if total_validated_records == manifest.total_records {
        println!(
            "   ✅ Record count validation passed: {}",
            total_validated_records
        );
    } else {
        println!(
            "   ❌ Record count mismatch: manifest claims {}, validated {}",
            manifest.total_records, total_validated_records
        );
    }

    // Validate database info
    println!("\n🗄️  Database Information Validation:");
    println!(
        "   • Database path: {}",
        manifest.database_info.database_path
    );
    println!("   • Total tables: {}", manifest.database_info.total_tables);

    for (table, stats) in &manifest.database_info.table_statistics {
        println!("   • {} table:", table);
        println!("     - Records: {}", stats.record_count);
        println!("     - Has embeddings: {}", stats.has_embeddings);
        if let Some(dim) = stats.embedding_dimension {
            println!("     - Embedding dimension: {}", dim);
        }
        if let Some(avg_len) = stats.avg_content_length {
            println!("     - Avg content length: {:.1} chars", avg_len);
        }
    }

    // Check for data completeness
    println!("\n🎯 Migration Readiness Check:");

    let expected_tables = vec!["text", "date", "task", "nodes"];
    let mut tables_with_data = 0;
    let mut tables_with_embeddings = 0;

    for table in &expected_tables {
        if let Some(stats) = manifest.database_info.table_statistics.get(*table) {
            if stats.record_count > 0 {
                tables_with_data += 1;
                println!("   ✅ {} table has {} records", table, stats.record_count);
            } else {
                println!("   ⚠️  {} table is empty", table);
            }

            if stats.has_embeddings {
                tables_with_embeddings += 1;
                println!("   ✅ {} table has embeddings ready for migration", table);
            }
        } else {
            println!("   ❌ {} table statistics missing", table);
        }
    }

    let expected_relationships = vec!["contains_relationships.json", "sibling_relationships.json"];
    let mut relationship_files_found = 0;

    for rel_file in &expected_relationships {
        if manifest
            .export_files
            .iter()
            .any(|f| f.file_name == *rel_file)
        {
            relationship_files_found += 1;
            println!("   ✅ {} exported", rel_file);
        } else {
            println!("   ❌ {} missing", rel_file);
        }
    }

    // Final migration readiness assessment
    println!("\n🏁 Migration Readiness Summary:");

    let readiness_score = calculate_readiness_score(
        missing_files.is_empty(),
        size_mismatches.is_empty(),
        total_validated_records == manifest.total_records,
        tables_with_data >= 2,         // At least 2 tables should have data
        tables_with_embeddings >= 1,   // At least 1 table should have embeddings
        relationship_files_found >= 1, // At least 1 relationship type should exist
    );

    match readiness_score {
        6 => {
            println!("   ✅ READY FOR MIGRATION - All validation checks passed");
            println!("   🚀 Proceed with LanceDB import process");
        }
        4..=5 => {
            println!("   ⚠️  MOSTLY READY - Minor issues detected");
            println!("   🔧 Review warnings above before proceeding");
        }
        _ => {
            println!("   ❌ NOT READY - Critical issues detected");
            println!("   🛠️  Fix errors above before migration");
        }
    }

    println!("\n📈 Readiness Score: {}/6", readiness_score);

    Ok(())
}

fn calculate_readiness_score(
    files_complete: bool,
    sizes_match: bool,
    records_match: bool,
    has_data: bool,
    has_embeddings: bool,
    has_relationships: bool,
) -> u8 {
    let mut score = 0;
    if files_complete {
        score += 1;
    }
    if sizes_match {
        score += 1;
    }
    if records_match {
        score += 1;
    }
    if has_data {
        score += 1;
    }
    if has_embeddings {
        score += 1;
    }
    if has_relationships {
        score += 1;
    }
    score
}
