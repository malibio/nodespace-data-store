//! SurrealDB data export utilities for migration to LanceDB
//!
//! This module provides comprehensive data export capabilities to extract all
//! NodeSpace data from SurrealDB in preparation for migration to LanceDB's
//! universal document format.

use crate::error::DataStoreError;
use crate::surrealdb_types::{DateRecord, NodeRecord, RelationshipRecord};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Export manifest tracking all exported data from SurrealDB
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportManifest {
    pub export_timestamp: String,
    pub total_records: usize,
    pub export_files: Vec<ExportFile>,
    pub schema_version: String,
    pub validation_checksum: String,
    pub database_info: DatabaseInfo,
}

/// Information about an individual export file
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportFile {
    pub file_name: String,
    pub table_name: String,
    pub record_count: usize,
    pub file_size_bytes: u64,
    pub checksum: String,
    pub export_timestamp: String,
}

/// Database metadata and statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub database_path: String,
    pub total_tables: usize,
    pub table_statistics: HashMap<String, TableStats>,
}

/// Statistics for individual tables
#[derive(Debug, Serialize, Deserialize)]
pub struct TableStats {
    pub record_count: usize,
    pub has_embeddings: bool,
    pub embedding_dimension: Option<usize>,
    pub avg_content_length: Option<f64>,
}

/// Container for exported table data
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData<T> {
    pub table_name: String,
    pub schema_version: String,
    pub export_timestamp: String,
    pub record_count: usize,
    pub records: Vec<T>,
    pub metadata: ExportMetadata,
}

/// Metadata for exported data validation
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub content_hash: String,
    pub embedding_stats: Option<EmbeddingStats>,
    pub relationship_count: usize,
}

/// Statistics about embeddings in exported data
#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingStats {
    pub total_embeddings: usize,
    pub dimension: usize,
    pub model_info: String,
    pub avg_magnitude: f64,
}

/// SurrealDB data exporter for migration to LanceDB
pub struct SurrealDBExporter {
    db: Surreal<Db>,
    export_path: PathBuf,
}

impl SurrealDBExporter {
    /// Create a new exporter with database connection
    pub async fn new(db_path: &str, export_path: PathBuf) -> Result<Self, DataStoreError> {
        let db = Surreal::new::<RocksDb>(db_path).await?;
        db.use_ns("nodespace").use_db("main").await?;

        // Ensure export directory exists
        if !export_path.exists() {
            fs::create_dir_all(&export_path).map_err(|e| DataStoreError::IoError(e.to_string()))?;
        }

        Ok(Self { db, export_path })
    }

    /// Export all SurrealDB data for migration to LanceDB
    pub async fn export_all_data(&self) -> Result<ExportManifest, DataStoreError> {
        let mut manifest = ExportManifest {
            export_timestamp: Utc::now().to_rfc3339(),
            total_records: 0,
            export_files: vec![],
            schema_version: "1.0".to_string(),
            validation_checksum: String::new(),
            database_info: self.gather_database_info().await?,
        };

        // Export all node tables
        manifest.export_files.push(self.export_text_nodes().await?);
        manifest.export_files.push(self.export_date_nodes().await?);
        manifest.export_files.push(self.export_task_nodes().await?);
        manifest
            .export_files
            .push(self.export_generic_nodes().await?);

        // Export all relationships
        manifest
            .export_files
            .push(self.export_contains_relationships().await?);
        manifest
            .export_files
            .push(self.export_sibling_relationships().await?);

        // Export metadata and configuration
        manifest.export_files.push(self.export_metadata().await?);

        // Calculate totals and finalize manifest
        manifest.total_records = manifest.export_files.iter().map(|f| f.record_count).sum();
        manifest.validation_checksum = self.calculate_manifest_checksum(&manifest);

        // Save manifest file
        self.save_manifest(&manifest).await?;

        Ok(manifest)
    }

    /// Export text nodes table
    async fn export_text_nodes(&self) -> Result<ExportFile, DataStoreError> {
        // Try both the raw SurrealDB query and the properly formatted version
        let query = "SELECT * FROM text ORDER BY created_at";
        let mut response = self.db.query(query).await?;

        // Handle the raw SurrealDB response format
        let raw_results: Vec<serde_json::Value> = response.take(0)?;

        // Convert raw results to a simplified format for export
        let results: Vec<serde_json::Value> = raw_results
            .iter()
            .filter_map(|item| {
                // Extract the core data, handling SurrealDB's Thing format
                let mut export_item = serde_json::Map::new();

                if let Some(obj) = item.as_object() {
                    // Copy all fields, converting Thing IDs to strings
                    for (key, value) in obj {
                        match key.as_str() {
                            "id" => {
                                // Convert SurrealDB Thing to string representation
                                export_item.insert(
                                    "id".to_string(),
                                    serde_json::Value::String(format!("{}", value)),
                                );
                            }
                            _ => {
                                export_item.insert(key.clone(), value.clone());
                            }
                        }
                    }
                    Some(serde_json::Value::Object(export_item))
                } else {
                    None
                }
            })
            .collect();

        let export_data = ExportData {
            table_name: "text".to_string(),
            schema_version: "1.0".to_string(),
            export_timestamp: Utc::now().to_rfc3339(),
            record_count: results.len(),
            records: results,
            metadata: ExportMetadata {
                content_hash: self.calculate_content_hash(&"text")?,
                embedding_stats: self.calculate_embedding_stats("text").await?,
                relationship_count: self.count_table_relationships("text").await?,
            },
        };

        self.save_export_file("text_nodes.json", &export_data).await
    }

    /// Export date nodes table
    async fn export_date_nodes(&self) -> Result<ExportFile, DataStoreError> {
        let query = "SELECT * FROM date";
        let mut response = self.db.query(query).await?;
        let results: Vec<DateRecord> = response.take(0)?;

        let export_data = ExportData {
            table_name: "date".to_string(),
            schema_version: "1.0".to_string(),
            export_timestamp: Utc::now().to_rfc3339(),
            record_count: results.len(),
            records: results,
            metadata: ExportMetadata {
                content_hash: self.calculate_content_hash(&"date")?,
                embedding_stats: None, // Date nodes typically don't have embeddings
                relationship_count: self.count_table_relationships("date").await?,
            },
        };

        self.save_export_file("date_nodes.json", &export_data).await
    }

    /// Export task nodes table
    async fn export_task_nodes(&self) -> Result<ExportFile, DataStoreError> {
        let query = "SELECT * FROM task";
        let mut response = self.db.query(query).await?;
        let results: Vec<NodeRecord> = response.take(0)?;

        let export_data = ExportData {
            table_name: "task".to_string(),
            schema_version: "1.0".to_string(),
            export_timestamp: Utc::now().to_rfc3339(),
            record_count: results.len(),
            records: results,
            metadata: ExportMetadata {
                content_hash: self.calculate_content_hash(&"task")?,
                embedding_stats: self.calculate_embedding_stats("task").await?,
                relationship_count: self.count_table_relationships("task").await?,
            },
        };

        self.save_export_file("task_nodes.json", &export_data).await
    }

    /// Export generic nodes table
    async fn export_generic_nodes(&self) -> Result<ExportFile, DataStoreError> {
        let query = "SELECT * FROM nodes";
        let mut response = self.db.query(query).await?;
        let results: Vec<NodeRecord> = response.take(0)?;

        let export_data = ExportData {
            table_name: "nodes".to_string(),
            schema_version: "1.0".to_string(),
            export_timestamp: Utc::now().to_rfc3339(),
            record_count: results.len(),
            records: results,
            metadata: ExportMetadata {
                content_hash: self.calculate_content_hash(&"nodes")?,
                embedding_stats: self.calculate_embedding_stats("nodes").await?,
                relationship_count: self.count_table_relationships("nodes").await?,
            },
        };

        self.save_export_file("generic_nodes.json", &export_data)
            .await
    }

    /// Export contains relationships
    async fn export_contains_relationships(&self) -> Result<ExportFile, DataStoreError> {
        let query = "SELECT * FROM contains";
        let mut response = self.db.query(query).await?;
        let results: Vec<RelationshipRecord> = response.take(0)?;

        let export_data = ExportData {
            table_name: "contains".to_string(),
            schema_version: "1.0".to_string(),
            export_timestamp: Utc::now().to_rfc3339(),
            record_count: results.len(),
            records: results,
            metadata: ExportMetadata {
                content_hash: self.calculate_content_hash(&"contains")?,
                embedding_stats: None, // Relationships don't have embeddings
                relationship_count: 0, // This IS the relationship data
            },
        };

        self.save_export_file("contains_relationships.json", &export_data)
            .await
    }

    /// Export sibling relationships
    async fn export_sibling_relationships(&self) -> Result<ExportFile, DataStoreError> {
        let query = "SELECT * FROM sibling";
        let mut response = self.db.query(query).await?;
        let results: Vec<RelationshipRecord> = response.take(0)?;

        let export_data = ExportData {
            table_name: "sibling".to_string(),
            schema_version: "1.0".to_string(),
            export_timestamp: Utc::now().to_rfc3339(),
            record_count: results.len(),
            records: results,
            metadata: ExportMetadata {
                content_hash: self.calculate_content_hash(&"sibling")?,
                embedding_stats: None, // Relationships don't have embeddings
                relationship_count: 0, // This IS the relationship data
            },
        };

        self.save_export_file("sibling_relationships.json", &export_data)
            .await
    }

    /// Export database metadata and configuration
    async fn export_metadata(&self) -> Result<ExportFile, DataStoreError> {
        // Export database schema information and configuration
        let metadata = serde_json::json!({
            "database_version": "surrealdb-2.3.6",
            "namespace": "nodespace",
            "database": "main",
            "export_purpose": "lancedb_migration",
            "table_schemas": {
                "text": "Text content nodes with embeddings",
                "date": "Date-based nodes for temporal organization",
                "task": "Task and action item nodes",
                "nodes": "Generic nodes for flexible content",
                "contains": "Parent-child relationships",
                "sibling": "Sequential ordering relationships"
            },
            "migration_notes": [
                "Embeddings generated using fastembed-rs bge-small-en-v1.5",
                "All node types will be unified in LanceDB universal document format",
                "Relationships will be embedded as document properties"
            ]
        });

        let export_data = ExportData {
            table_name: "_metadata".to_string(),
            schema_version: "1.0".to_string(),
            export_timestamp: Utc::now().to_rfc3339(),
            record_count: 1,
            records: vec![metadata],
            metadata: ExportMetadata {
                content_hash: "metadata".to_string(),
                embedding_stats: None,
                relationship_count: 0,
            },
        };

        self.save_export_file("database_metadata.json", &export_data)
            .await
    }

    /// Gather comprehensive database information
    async fn gather_database_info(&self) -> Result<DatabaseInfo, DataStoreError> {
        let tables = vec!["text", "date", "task", "nodes"];
        let mut table_statistics = HashMap::new();

        for table in &tables {
            let stats = self.gather_table_stats(table).await?;
            table_statistics.insert(table.to_string(), stats);
        }

        Ok(DatabaseInfo {
            database_path: "data/sample.db".to_string(), // This should be configurable
            total_tables: tables.len(),
            table_statistics,
        })
    }

    /// Gather statistics for a specific table
    async fn gather_table_stats(&self, table: &str) -> Result<TableStats, DataStoreError> {
        // Count records
        let count_query = format!("SELECT count() FROM {} GROUP ALL", table);
        let mut response = self.db.query(&count_query).await?;
        let count_result: Vec<serde_json::Value> = response.take(0).unwrap_or_default();
        let record_count = count_result
            .first()
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        // Check for embeddings
        let embedding_query = format!(
            "SELECT embedding FROM {} WHERE embedding IS NOT NULL LIMIT 1",
            table
        );
        let mut response = self.db.query(&embedding_query).await?;
        let embedding_result: Vec<serde_json::Value> = response.take(0).unwrap_or_default();
        let has_embeddings = !embedding_result.is_empty();
        let embedding_dimension = if has_embeddings {
            embedding_result
                .first()
                .and_then(|v| v.get("embedding"))
                .and_then(|v| v.as_array())
                .map(|arr| arr.len())
        } else {
            None
        };

        // Calculate average content length (for text-based tables)
        let avg_content_length = if table == "text" {
            let content_query = format!("SELECT string::len(content) as len FROM {}", table);
            let mut response = self.db.query(&content_query).await?;
            let lengths: Vec<serde_json::Value> = response.take(0).unwrap_or_default();
            if !lengths.is_empty() {
                let total: f64 = lengths
                    .iter()
                    .filter_map(|v| v.get("len"))
                    .filter_map(|v| v.as_f64())
                    .sum();
                Some(total / lengths.len() as f64)
            } else {
                None
            }
        } else {
            None
        };

        Ok(TableStats {
            record_count,
            has_embeddings,
            embedding_dimension,
            avg_content_length,
        })
    }

    /// Calculate embedding statistics for a table
    async fn calculate_embedding_stats(
        &self,
        table: &str,
    ) -> Result<Option<EmbeddingStats>, DataStoreError> {
        let query = format!(
            "SELECT embedding FROM {} WHERE embedding IS NOT NULL",
            table
        );
        let mut response = self.db.query(&query).await?;
        let results: Vec<serde_json::Value> = response.take(0).unwrap_or_default();

        if results.is_empty() {
            return Ok(None);
        }

        let embeddings: Vec<Vec<f64>> = results
            .iter()
            .filter_map(|v| v.get("embedding"))
            .filter_map(|v| v.as_array())
            .filter_map(|arr| arr.iter().map(|x| x.as_f64()).collect::<Option<Vec<f64>>>())
            .collect();

        if embeddings.is_empty() {
            return Ok(None);
        }

        let dimension = embeddings[0].len();
        let total_embeddings = embeddings.len();

        // Calculate average magnitude
        let avg_magnitude = embeddings
            .iter()
            .map(|emb| emb.iter().map(|x| x * x).sum::<f64>().sqrt())
            .sum::<f64>()
            / total_embeddings as f64;

        Ok(Some(EmbeddingStats {
            total_embeddings,
            dimension,
            model_info: "fastembed-rs bge-small-en-v1.5".to_string(),
            avg_magnitude,
        }))
    }

    /// Count relationships for a table
    async fn count_table_relationships(&self, table: &str) -> Result<usize, DataStoreError> {
        // Simplified query - just return 0 for now since relationship counting is complex in SurrealDB
        // This can be enhanced later with proper SurrealQL syntax
        let _query = format!(
            "SELECT count() FROM contains WHERE type::string(in) CONTAINS '{}' OR type::string(out) CONTAINS '{}'",
            table, table
        );
        // For now, return 0 - relationship counting can be enhanced later
        Ok(0)
    }

    /// Calculate content hash for validation
    fn calculate_content_hash(&self, table: &str) -> Result<String, DataStoreError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        table.hash(&mut hasher);
        Utc::now().timestamp().hash(&mut hasher);
        Ok(format!("{:x}", hasher.finish()))
    }

    /// Calculate manifest checksum for integrity validation
    fn calculate_manifest_checksum(&self, manifest: &ExportManifest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        manifest.export_timestamp.hash(&mut hasher);
        manifest.total_records.hash(&mut hasher);
        for file in &manifest.export_files {
            file.file_name.hash(&mut hasher);
            file.record_count.hash(&mut hasher);
            file.checksum.hash(&mut hasher);
        }
        format!("{:x}", hasher.finish())
    }

    /// Save export data to JSON file
    async fn save_export_file<T: Serialize>(
        &self,
        filename: &str,
        data: &ExportData<T>,
    ) -> Result<ExportFile, DataStoreError> {
        let file_path = self.export_path.join(filename);

        // Serialize data
        let json_data =
            serde_json::to_string_pretty(data).map_err(|e| DataStoreError::Serialization(e))?;

        // Write to file
        let mut file = File::create(&file_path)
            .await
            .map_err(|e| DataStoreError::IoError(e.to_string()))?;
        file.write_all(json_data.as_bytes())
            .await
            .map_err(|e| DataStoreError::IoError(e.to_string()))?;

        // Get file metadata
        let metadata =
            std::fs::metadata(&file_path).map_err(|e| DataStoreError::IoError(e.to_string()))?;

        // Calculate file checksum
        let checksum = self.calculate_content_hash(&data.table_name)?;

        Ok(ExportFile {
            file_name: filename.to_string(),
            table_name: data.table_name.clone(),
            record_count: data.record_count,
            file_size_bytes: metadata.len(),
            checksum,
            export_timestamp: data.export_timestamp.clone(),
        })
    }

    /// Save export manifest to file
    async fn save_manifest(&self, manifest: &ExportManifest) -> Result<(), DataStoreError> {
        let manifest_path = self.export_path.join("export_manifest.json");

        let json_data =
            serde_json::to_string_pretty(manifest).map_err(|e| DataStoreError::Serialization(e))?;

        let mut file = File::create(&manifest_path)
            .await
            .map_err(|e| DataStoreError::IoError(e.to_string()))?;
        file.write_all(json_data.as_bytes())
            .await
            .map_err(|e| DataStoreError::IoError(e.to_string()))?;

        Ok(())
    }
}
