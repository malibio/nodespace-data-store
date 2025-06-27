//! LanceDB import functionality for migrating data from SurrealDB exports
//!
//! This module complements the SurrealDB export functionality by providing
//! the import side of the migration pipeline, converting exported SurrealDB
//! data into LanceDB's universal document format.

use crate::error::DataStoreError;
use crate::lance_data_store::{LanceDataStoreFull, LanceDBConfig, UniversalDocument};
use crate::migration::surrealdb_export::{ExportManifest, ExportData, ExportFile};
use crate::performance::{OperationType, PerformanceMonitor};
use crate::schema::lance_schema::{NodeType, ContentType};
use crate::surrealdb_types::{TextRecord, DateRecord, NodeRecord, RelationshipRecord};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufWriter, AsyncWriteExt};

/// Migration statistics for tracking progress and performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStats {
    pub total_records: usize,
    pub migrated_records: usize,
    pub failed_records: usize,
    pub text_nodes: usize,
    pub date_nodes: usize,
    pub task_nodes: usize,
    pub generic_nodes: usize,
    pub relationships: usize,
    pub migration_time_ms: u64,
    pub avg_record_time_ms: f64,
    pub errors: Vec<String>,
}

impl Default for MigrationStats {
    fn default() -> Self {
        Self {
            total_records: 0,
            migrated_records: 0,
            failed_records: 0,
            text_nodes: 0,
            date_nodes: 0,
            task_nodes: 0,
            generic_nodes: 0,
            relationships: 0,
            migration_time_ms: 0,
            avg_record_time_ms: 0.0,
            errors: Vec::new(),
        }
    }
}

/// LanceDB migration importer
pub struct LanceDBImporter {
    lance_store: LanceDataStoreFull,
    performance_monitor: PerformanceMonitor,
    config: ImportConfig,
}

/// Configuration for LanceDB import process
#[derive(Debug, Clone)]
pub struct ImportConfig {
    pub batch_size: usize,
    pub enable_validation: bool,
    pub skip_existing: bool,
    pub include_relationships: bool,
    pub performance_monitoring: bool,
    pub max_retry_attempts: u32,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            enable_validation: true,
            skip_existing: false,
            include_relationships: true,
            performance_monitoring: true,
            max_retry_attempts: 3,
        }
    }
}

impl LanceDBImporter {
    /// Create new LanceDB importer
    pub async fn new(
        lance_db_path: &str,
        lance_config: LanceDBConfig,
        import_config: ImportConfig,
    ) -> Result<Self, DataStoreError> {
        let lance_store = LanceDataStoreFull::new(lance_db_path, lance_config).await?;
        let performance_monitor = PerformanceMonitor::with_defaults();

        Ok(Self {
            lance_store,
            performance_monitor,
            config: import_config,
        })
    }

    /// Import all data from SurrealDB export directory
    pub async fn import_from_export(
        &self,
        export_dir: &Path,
    ) -> Result<MigrationStats, DataStoreError> {
        let timer = self.performance_monitor
            .start_operation(OperationType::DataMigration)
            .with_metadata("export_dir".to_string(), export_dir.to_string_lossy().to_string());

        let mut stats = MigrationStats::default();
        let start_time = std::time::Instant::now();

        // Read the export manifest
        let manifest = self.read_manifest(export_dir).await?;
        println!("📊 Starting migration of {} files", manifest.export_files.len());

        // Process each export file
        for export_file in &manifest.export_files {
            match self.import_export_file(export_dir, export_file, &mut stats).await {
                Ok(_) => {
                    println!("✅ Imported {}: {} records", export_file.file_name, export_file.record_count);
                }
                Err(e) => {
                    let error_msg = format!("Failed to import {}: {}", export_file.file_name, e);
                    println!("❌ {}", error_msg);
                    stats.errors.push(error_msg);
                    stats.failed_records += export_file.record_count;
                }
            }
        }

        // Calculate final statistics
        stats.migration_time_ms = start_time.elapsed().as_millis() as u64;
        stats.avg_record_time_ms = if stats.migrated_records > 0 {
            stats.migration_time_ms as f64 / stats.migrated_records as f64
        } else {
            0.0
        };

        // Generate migration report
        self.generate_migration_report(&stats).await?;

        timer.complete_success();
        Ok(stats)
    }

    /// Import a single export file
    async fn import_export_file(
        &self,
        export_dir: &Path,
        export_file: &ExportFile,
        stats: &mut MigrationStats,
    ) -> Result<(), DataStoreError> {
        let file_path = export_dir.join(&export_file.file_name);
        
        match export_file.table_name.as_str() {
            "text" => {
                let export_data: ExportData<TextRecord> = self.read_export_file(&file_path).await?;
                self.import_text_nodes(export_data, stats).await?;
            }
            "date" => {
                let export_data: ExportData<DateRecord> = self.read_export_file(&file_path).await?;
                self.import_date_nodes(export_data, stats).await?;
            }
            "task" => {
                let export_data: ExportData<NodeRecord> = self.read_export_file(&file_path).await?;
                self.import_task_nodes(export_data, stats).await?;
            }
            "nodes" => {
                let export_data: ExportData<NodeRecord> = self.read_export_file(&file_path).await?;
                self.import_generic_nodes(export_data, stats).await?;
            }
            "contains" | "sibling" => {
                if self.config.include_relationships {
                    let export_data: ExportData<RelationshipRecord> = self.read_export_file(&file_path).await?;
                    self.import_relationships(export_data, stats).await?;
                }
            }
            _ => {
                println!("⚠️  Skipping unknown table: {}", export_file.table_name);
            }
        }

        Ok(())
    }

    /// Import text nodes into LanceDB
    async fn import_text_nodes(
        &self,
        export_data: ExportData<TextRecord>,
        stats: &mut MigrationStats,
    ) -> Result<(), DataStoreError> {
        println!("🔄 Importing {} text nodes...", export_data.records.len());

        for text_record in export_data.records {
            let document = self.text_record_to_universal_document(&text_record)?;
            
            match self.insert_document_with_retry(&document).await {
                Ok(_) => {
                    stats.migrated_records += 1;
                    stats.text_nodes += 1;
                }
                Err(e) => {
                    stats.failed_records += 1;
                    stats.errors.push(format!("Text node {}: {}", text_record.id, e));
                }
            }
        }

        Ok(())
    }

    /// Import date nodes into LanceDB
    async fn import_date_nodes(
        &self,
        export_data: ExportData<DateRecord>,
        stats: &mut MigrationStats,
    ) -> Result<(), DataStoreError> {
        println!("🔄 Importing {} date nodes...", export_data.records.len());

        for date_record in export_data.records {
            let document = self.date_record_to_universal_document(&date_record)?;
            
            match self.insert_document_with_retry(&document).await {
                Ok(_) => {
                    stats.migrated_records += 1;
                    stats.date_nodes += 1;
                }
                Err(e) => {
                    stats.failed_records += 1;
                    stats.errors.push(format!("Date node {}: {}", date_record.id, e));
                }
            }
        }

        Ok(())
    }

    /// Import task nodes into LanceDB
    async fn import_task_nodes(
        &self,
        export_data: ExportData<NodeRecord>,
        stats: &mut MigrationStats,
    ) -> Result<(), DataStoreError> {
        println!("🔄 Importing {} task nodes...", export_data.records.len());

        for node_record in export_data.records {
            let document = self.node_record_to_universal_document(&node_record, NodeType::Task)?;
            
            match self.insert_document_with_retry(&document).await {
                Ok(_) => {
                    stats.migrated_records += 1;
                    stats.task_nodes += 1;
                }
                Err(e) => {
                    stats.failed_records += 1;
                    stats.errors.push(format!("Task node {}: {}", node_record.id, e));
                }
            }
        }

        Ok(())
    }

    /// Import generic nodes into LanceDB
    async fn import_generic_nodes(
        &self,
        export_data: ExportData<NodeRecord>,
        stats: &mut MigrationStats,
    ) -> Result<(), DataStoreError> {
        println!("🔄 Importing {} generic nodes...", export_data.records.len());

        for node_record in export_data.records {
            let document = self.node_record_to_universal_document(&node_record, NodeType::Text)?;
            
            match self.insert_document_with_retry(&document).await {
                Ok(_) => {
                    stats.migrated_records += 1;
                    stats.generic_nodes += 1;
                }
                Err(e) => {
                    stats.failed_records += 1;
                    stats.errors.push(format!("Generic node {}: {}", node_record.id, e));
                }
            }
        }

        Ok(())
    }

    /// Import relationships into LanceDB (stored as document updates)
    async fn import_relationships(
        &self,
        export_data: ExportData<RelationshipRecord>,
        stats: &mut MigrationStats,
    ) -> Result<(), DataStoreError> {
        println!("🔄 Processing {} relationships...", export_data.records.len());

        // Relationships in LanceDB are stored as part of the document structure
        // This is a simplified implementation - in practice, you'd update existing documents
        for relationship in export_data.records {
            // TODO: Implement relationship updates to existing documents
            // For now, just count them
            stats.relationships += 1;
        }

        Ok(())
    }

    /// Convert TextRecord to UniversalDocument
    fn text_record_to_universal_document(&self, record: &TextRecord) -> Result<UniversalDocument, DataStoreError> {
        let now = Utc::now().to_rfc3339();
        
        Ok(UniversalDocument {
            id: record.id.to_string().replace(':', "-"), // Convert SurrealDB ID format
            node_type: NodeType::Text.to_string(),
            content: record.content.clone(),
            content_type: ContentType::TextPlain.to_string(),
            content_size_bytes: Some(record.content.len() as u64),
            metadata: record.metadata.clone().map(|m| serde_json::to_string(&m).unwrap_or_default()),
            vector: record.embedding.clone(),
            vector_model: Some("bge-small-en-v1.5".to_string()), // Default model
            vector_dimensions: record.embedding.as_ref().map(|v| v.len() as u32),
            parent_id: record.parent_id.as_ref().map(|id| id.to_string().replace(':', "-")),
            children_ids: vec![], // TODO: Extract from relationships
            mentions: vec![],
            next_sibling: record.next_sibling.as_ref().map(|id| id.to_string().replace(':', "-")),
            previous_sibling: record.previous_sibling.as_ref().map(|id| id.to_string().replace(':', "-")),
            created_at: record.created_at.to_rfc3339(),
            updated_at: record.updated_at.to_rfc3339(),
            image_alt_text: None,
            image_width: None,
            image_height: None,
            image_format: None,
            search_priority: Some(1.0),
            last_accessed: Some(now),
            extended_properties: None,
        })
    }

    /// Convert DateRecord to UniversalDocument
    fn date_record_to_universal_document(&self, record: &DateRecord) -> Result<UniversalDocument, DataStoreError> {
        let now = Utc::now().to_rfc3339();
        
        Ok(UniversalDocument {
            id: record.id.to_string().replace(':', "-"),
            node_type: NodeType::Date.to_string(),
            content: record.date_value.clone(),
            content_type: ContentType::TextPlain.to_string(),
            content_size_bytes: Some(record.date_value.len() as u64),
            metadata: record.metadata.clone().map(|m| serde_json::to_string(&m).unwrap_or_default()),
            vector: None, // Date nodes typically don't have embeddings
            vector_model: None,
            vector_dimensions: None,
            parent_id: record.parent_id.as_ref().map(|id| id.to_string().replace(':', "-")),
            children_ids: vec![],
            mentions: vec![],
            next_sibling: record.next_sibling.as_ref().map(|id| id.to_string().replace(':', "-")),
            previous_sibling: record.previous_sibling.as_ref().map(|id| id.to_string().replace(':', "-")),
            created_at: record.created_at.to_rfc3339(),
            updated_at: record.updated_at.to_rfc3339(),
            image_alt_text: None,
            image_width: None,
            image_height: None,
            image_format: None,
            search_priority: Some(0.5), // Lower priority for date nodes
            last_accessed: Some(now),
            extended_properties: None,
        })
    }

    /// Convert NodeRecord to UniversalDocument
    fn node_record_to_universal_document(
        &self, 
        record: &NodeRecord, 
        node_type: NodeType
    ) -> Result<UniversalDocument, DataStoreError> {
        let now = Utc::now().to_rfc3339();
        
        Ok(UniversalDocument {
            id: record.id.to_string().replace(':', "-"),
            node_type: node_type.to_string(),
            content: serde_json::to_string(&record.content).unwrap_or_default(),
            content_type: ContentType::ApplicationJson.to_string(),
            content_size_bytes: None,
            metadata: record.metadata.clone().map(|m| serde_json::to_string(&m).unwrap_or_default()),
            vector: record.embedding.clone(),
            vector_model: Some("bge-small-en-v1.5".to_string()),
            vector_dimensions: record.embedding.as_ref().map(|v| v.len() as u32),
            parent_id: record.parent_id.as_ref().map(|id| id.to_string().replace(':', "-")),
            children_ids: vec![],
            mentions: vec![],
            next_sibling: record.next_sibling.as_ref().map(|id| id.to_string().replace(':', "-")),
            previous_sibling: record.previous_sibling.as_ref().map(|id| id.to_string().replace(':', "-")),
            created_at: record.created_at.to_rfc3339(),
            updated_at: record.updated_at.to_rfc3339(),
            image_alt_text: None,
            image_width: None,
            image_height: None,
            image_format: None,
            search_priority: Some(1.0),
            last_accessed: Some(now),
            extended_properties: None,
        })
    }

    /// Insert document with retry logic
    async fn insert_document_with_retry(&self, document: &UniversalDocument) -> Result<(), DataStoreError> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.config.max_retry_attempts {
            match self.lance_store.insert_document(document).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    attempts += 1;
                    last_error = Some(e);
                    
                    if attempts < self.config.max_retry_attempts {
                        // Exponential backoff
                        let delay_ms = 100 * (2_u64.pow(attempts));
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| DataStoreError::Migration("Unknown retry error".to_string())))
    }

    /// Read export manifest file
    async fn read_manifest(&self, export_dir: &Path) -> Result<ExportManifest, DataStoreError> {
        let manifest_path = export_dir.join("export_manifest.json");
        let mut file = File::open(&manifest_path)
            .await
            .map_err(|e| DataStoreError::IoError(format!("Failed to open manifest: {}", e)))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(|e| DataStoreError::IoError(format!("Failed to read manifest: {}", e)))?;

        serde_json::from_str(&contents)
            .map_err(|e| DataStoreError::Serialization(e))
    }

    /// Read individual export file
    async fn read_export_file<T>(&self, file_path: &Path) -> Result<ExportData<T>, DataStoreError>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut file = File::open(file_path)
            .await
            .map_err(|e| DataStoreError::IoError(format!("Failed to open export file: {}", e)))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(|e| DataStoreError::IoError(format!("Failed to read export file: {}", e)))?;

        serde_json::from_str(&contents)
            .map_err(|e| DataStoreError::Serialization(e))
    }

    /// Generate migration report
    async fn generate_migration_report(&self, stats: &MigrationStats) -> Result<(), DataStoreError> {
        let report = format!(
            r#"
# LanceDB Migration Report

## Summary
- **Total Records**: {}
- **Successfully Migrated**: {}
- **Failed Records**: {}
- **Migration Time**: {}ms
- **Average Time per Record**: {:.2}ms

## Breakdown by Type
- **Text Nodes**: {}
- **Date Nodes**: {}
- **Task Nodes**: {}
- **Generic Nodes**: {}
- **Relationships**: {}

## Performance Metrics
{}

## Errors ({})
{}

---
Generated at: {}
"#,
            stats.total_records,
            stats.migrated_records,
            stats.failed_records,
            stats.migration_time_ms,
            stats.avg_record_time_ms,
            stats.text_nodes,
            stats.date_nodes,
            stats.task_nodes,
            stats.generic_nodes,
            stats.relationships,
            serde_json::to_string_pretty(&self.performance_monitor.get_aggregated_metrics())
                .unwrap_or_else(|_| "Performance metrics unavailable".to_string()),
            stats.errors.len(),
            stats.errors.join("\n"),
            Utc::now().to_rfc3339()
        );

        println!("{}", report);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[tokio::test]
    async fn test_migration_stats_default() {
        let stats = MigrationStats::default();
        assert_eq!(stats.total_records, 0);
        assert_eq!(stats.migrated_records, 0);
        assert_eq!(stats.errors.len(), 0);
    }

    #[tokio::test]
    async fn test_import_config_default() {
        let config = ImportConfig::default();
        assert_eq!(config.batch_size, 1000);
        assert!(config.enable_validation);
        assert!(config.performance_monitoring);
    }
}