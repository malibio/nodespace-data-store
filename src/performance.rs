//! Performance monitoring infrastructure for LanceDB operations
//!
//! This module provides comprehensive performance tracking, metrics collection,
//! and alerting capabilities for all database operations with configurable
//! thresholds and real-time monitoring.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Performance monitoring configuration with configurable thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum allowed time for image operations (default: 5000ms)
    pub max_image_operation_ms: u64,
    /// Maximum allowed time for search operations (default: 2000ms)
    pub max_search_operation_ms: u64,
    /// Maximum allowed time for node creation (default: 1000ms)
    pub max_create_operation_ms: u64,
    /// Maximum allowed time for node retrieval (default: 500ms)
    pub max_get_operation_ms: u64,
    /// Enable real-time alerting
    pub enable_alerting: bool,
    /// Metrics collection interval in seconds
    pub metrics_interval_seconds: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_image_operation_ms: 5000,
            max_search_operation_ms: 2000,
            max_create_operation_ms: 1000,
            max_get_operation_ms: 500,
            enable_alerting: true,
            metrics_interval_seconds: 60,
        }
    }
}

/// Operation types for performance tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationType {
    CreateNode,
    GetNode,
    DeleteNode,
    QueryNodes,
    SearchSimilar,
    CreateRelationship,
    ImageOperation,
    VectorSearch,
    SchemaValidation,
    DataMigration,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::CreateNode => write!(f, "create_node"),
            OperationType::GetNode => write!(f, "get_node"),
            OperationType::DeleteNode => write!(f, "delete_node"),
            OperationType::QueryNodes => write!(f, "query_nodes"),
            OperationType::SearchSimilar => write!(f, "search_similar"),
            OperationType::CreateRelationship => write!(f, "create_relationship"),
            OperationType::ImageOperation => write!(f, "image_operation"),
            OperationType::VectorSearch => write!(f, "vector_search"),
            OperationType::SchemaValidation => write!(f, "schema_validation"),
            OperationType::DataMigration => write!(f, "data_migration"),
        }
    }
}

/// Individual operation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetric {
    pub operation_type: OperationType,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Aggregated metrics for an operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub operation_type: OperationType,
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub p95_duration_ms: u64,
    pub p99_duration_ms: u64,
    pub operations_per_second: f64,
    pub error_rate: f64,
    pub last_updated: DateTime<Utc>,
}

/// Performance alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    ThresholdExceeded {
        operation_type: OperationType,
        threshold_ms: u64,
        actual_ms: u64,
    },
    HighErrorRate {
        operation_type: OperationType,
        error_rate: f64,
        threshold: f64,
    },
    SystemOverload {
        operations_per_second: f64,
        threshold: f64,
    },
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub alert_type: AlertType,
    pub timestamp: DateTime<Utc>,
    pub severity: AlertSeverity,
    pub description: String,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Performance monitor for tracking and analyzing database operations
#[derive(Debug)]
pub struct PerformanceMonitor {
    config: PerformanceConfig,
    metrics: Arc<Mutex<Vec<OperationMetric>>>,
    aggregated: Arc<Mutex<HashMap<OperationType, AggregatedMetrics>>>,
    alerts: Arc<Mutex<Vec<PerformanceAlert>>>,
}

impl PerformanceMonitor {
    /// Create new performance monitor with configuration
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(Mutex::new(Vec::new())),
            aggregated: Arc::new(Mutex::new(HashMap::new())),
            alerts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create performance monitor with default configuration
    pub fn with_defaults() -> Self {
        Self::new(PerformanceConfig::default())
    }

    /// Start timing an operation
    pub fn start_operation(&self, operation_type: OperationType) -> OperationTimer {
        OperationTimer::new(operation_type, Arc::clone(&self.metrics), &self.config)
    }

    /// Record a completed operation manually
    pub fn record_operation(
        &self,
        operation_type: OperationType,
        duration: Duration,
        success: bool,
        error_message: Option<String>,
        metadata: HashMap<String, String>,
    ) {
        let metric = OperationMetric {
            operation_type,
            duration_ms: duration.as_millis() as u64,
            timestamp: Utc::now(),
            success,
            error_message,
            metadata,
        };

        // Check thresholds and generate alerts
        self.check_thresholds(&metric);

        // Store the metric
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.push(metric);
        }

        // Update aggregated metrics
        self.update_aggregated_metrics(operation_type);
    }

    /// Get aggregated metrics for all operation types
    pub fn get_aggregated_metrics(&self) -> HashMap<OperationType, AggregatedMetrics> {
        self.aggregated.lock().unwrap().clone()
    }

    /// Get aggregated metrics for specific operation type
    pub fn get_operation_metrics(
        &self,
        operation_type: OperationType,
    ) -> Option<AggregatedMetrics> {
        self.aggregated
            .lock()
            .unwrap()
            .get(&operation_type)
            .cloned()
    }

    /// Get recent performance alerts
    pub fn get_recent_alerts(&self, limit: usize) -> Vec<PerformanceAlert> {
        let alerts = self.alerts.lock().unwrap();
        alerts.iter().rev().take(limit).cloned().collect()
    }

    /// Get all alerts since timestamp
    pub fn get_alerts_since(&self, since: DateTime<Utc>) -> Vec<PerformanceAlert> {
        let alerts = self.alerts.lock().unwrap();
        alerts
            .iter()
            .filter(|alert| alert.timestamp > since)
            .cloned()
            .collect()
    }

    /// Clear old metrics to prevent memory growth
    pub fn cleanup_old_metrics(&self, max_age: Duration) {
        let cutoff = Utc::now() - chrono::Duration::from_std(max_age).unwrap();

        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.retain(|m| m.timestamp > cutoff);
        }

        if let Ok(mut alerts) = self.alerts.lock() {
            alerts.retain(|a| a.timestamp > cutoff);
        }
    }

    /// Generate performance summary report
    pub fn generate_summary_report(&self) -> PerformanceSummary {
        let aggregated = self.get_aggregated_metrics();
        let total_operations: u64 = aggregated.values().map(|m| m.total_operations).sum();
        let total_errors: u64 = aggregated.values().map(|m| m.failed_operations).sum();
        let overall_error_rate = if total_operations > 0 {
            (total_errors as f64 / total_operations as f64) * 100.0
        } else {
            0.0
        };

        let avg_response_time = if !aggregated.is_empty() {
            aggregated.values().map(|m| m.avg_duration_ms).sum::<f64>() / aggregated.len() as f64
        } else {
            0.0
        };

        PerformanceSummary {
            total_operations,
            total_errors,
            overall_error_rate,
            avg_response_time_ms: avg_response_time,
            operations_by_type: aggregated,
            recent_alerts: self.get_recent_alerts(10),
            generated_at: Utc::now(),
        }
    }

    /// Check operation against thresholds and generate alerts
    fn check_thresholds(&self, metric: &OperationMetric) {
        if !self.config.enable_alerting {
            return;
        }

        let threshold_ms = match metric.operation_type {
            OperationType::ImageOperation => self.config.max_image_operation_ms,
            OperationType::SearchSimilar | OperationType::VectorSearch => {
                self.config.max_search_operation_ms
            }
            OperationType::CreateNode => self.config.max_create_operation_ms,
            OperationType::GetNode => self.config.max_get_operation_ms,
            _ => self.config.max_search_operation_ms, // Default threshold
        };

        if metric.duration_ms > threshold_ms {
            let alert = PerformanceAlert {
                alert_type: AlertType::ThresholdExceeded {
                    operation_type: metric.operation_type,
                    threshold_ms,
                    actual_ms: metric.duration_ms,
                },
                timestamp: Utc::now(),
                severity: if metric.duration_ms > threshold_ms * 2 {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::Warning
                },
                description: format!(
                    "{} operation took {}ms, exceeding threshold of {}ms",
                    metric.operation_type, metric.duration_ms, threshold_ms
                ),
            };

            if let Ok(mut alerts) = self.alerts.lock() {
                alerts.push(alert);
            }
        }
    }

    /// Update aggregated metrics for operation type
    fn update_aggregated_metrics(&self, operation_type: OperationType) {
        let metrics = self.metrics.lock().unwrap();
        let operation_metrics: Vec<&OperationMetric> = metrics
            .iter()
            .filter(|m| m.operation_type == operation_type)
            .collect();

        if operation_metrics.is_empty() {
            return;
        }

        let total_operations = operation_metrics.len() as u64;
        let successful_operations = operation_metrics.iter().filter(|m| m.success).count() as u64;
        let failed_operations = total_operations - successful_operations;
        let error_rate = (failed_operations as f64 / total_operations as f64) * 100.0;

        let durations: Vec<u64> = operation_metrics.iter().map(|m| m.duration_ms).collect();
        let avg_duration_ms = durations.iter().sum::<u64>() as f64 / durations.len() as f64;
        let min_duration_ms = *durations.iter().min().unwrap_or(&0);
        let max_duration_ms = *durations.iter().max().unwrap_or(&0);

        // Calculate percentiles
        let mut sorted_durations = durations.clone();
        sorted_durations.sort_unstable();
        let p95_index = (sorted_durations.len() as f64 * 0.95) as usize;
        let p99_index = (sorted_durations.len() as f64 * 0.99) as usize;
        let p95_duration_ms = sorted_durations.get(p95_index).copied().unwrap_or(0);
        let p99_duration_ms = sorted_durations.get(p99_index).copied().unwrap_or(0);

        // Calculate operations per second (last minute)
        let one_minute_ago = Utc::now() - chrono::Duration::minutes(1);
        let recent_operations = operation_metrics
            .iter()
            .filter(|m| m.timestamp > one_minute_ago)
            .count() as f64;
        let operations_per_second = recent_operations / 60.0;

        let aggregated_metric = AggregatedMetrics {
            operation_type,
            total_operations,
            successful_operations,
            failed_operations,
            avg_duration_ms,
            min_duration_ms,
            max_duration_ms,
            p95_duration_ms,
            p99_duration_ms,
            operations_per_second,
            error_rate,
            last_updated: Utc::now(),
        };

        if let Ok(mut aggregated) = self.aggregated.lock() {
            aggregated.insert(operation_type, aggregated_metric);
        }
    }
}

/// Timer for measuring operation duration
pub struct OperationTimer {
    operation_type: OperationType,
    start_time: Instant,
    metrics: Arc<Mutex<Vec<OperationMetric>>>,
    _config: PerformanceConfig,
    metadata: HashMap<String, String>,
}

impl OperationTimer {
    fn new(
        operation_type: OperationType,
        metrics: Arc<Mutex<Vec<OperationMetric>>>,
        config: &PerformanceConfig,
    ) -> Self {
        Self {
            operation_type,
            start_time: Instant::now(),
            metrics,
            _config: config.clone(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the operation
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Complete the operation with success
    pub fn complete_success(self) {
        self.complete_with_result(true, None);
    }

    /// Complete the operation with error
    pub fn complete_error(self, error_message: String) {
        self.complete_with_result(false, Some(error_message));
    }

    /// Complete the operation with custom result
    pub fn complete_with_result(self, success: bool, error_message: Option<String>) {
        let duration = self.start_time.elapsed();
        let metric = OperationMetric {
            operation_type: self.operation_type,
            duration_ms: duration.as_millis() as u64,
            timestamp: Utc::now(),
            success,
            error_message,
            metadata: self.metadata,
        };

        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.push(metric);
        }
    }
}

/// Performance summary report
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub total_operations: u64,
    pub total_errors: u64,
    pub overall_error_rate: f64,
    pub avg_response_time_ms: f64,
    pub operations_by_type: HashMap<OperationType, AggregatedMetrics>,
    pub recent_alerts: Vec<PerformanceAlert>,
    pub generated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_performance_monitor_basic() {
        let monitor = PerformanceMonitor::with_defaults();

        let timer = monitor.start_operation(OperationType::CreateNode);
        thread::sleep(Duration::from_millis(10));
        timer.complete_success();

        let metrics = monitor.get_aggregated_metrics();
        assert!(metrics.contains_key(&OperationType::CreateNode));
    }

    #[test]
    fn test_threshold_alerting() {
        let config = PerformanceConfig {
            max_create_operation_ms: 5,
            enable_alerting: true,
            ..Default::default()
        };

        let monitor = PerformanceMonitor::new(config);

        // Simulate slow operation
        monitor.record_operation(
            OperationType::CreateNode,
            Duration::from_millis(100),
            true,
            None,
            HashMap::new(),
        );

        let alerts = monitor.get_recent_alerts(10);
        assert!(!alerts.is_empty());

        if let AlertType::ThresholdExceeded { actual_ms, .. } = &alerts[0].alert_type {
            assert!(*actual_ms >= 100);
        } else {
            panic!("Expected ThresholdExceeded alert");
        }
    }
}
