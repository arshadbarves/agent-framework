// Monitoring and observability system for AgentGraph
// Provides metrics collection, health checks, and alerting

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use thiserror::Error;

/// Performance metrics for system monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Timestamp when metrics were collected
    pub timestamp: SystemTime,
    /// CPU usage percentage (0-100)
    pub cpu_usage_percent: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Total memory available in bytes
    pub memory_total_bytes: u64,
    /// Network bytes sent
    pub network_bytes_sent: u64,
    /// Network bytes received
    pub network_bytes_received: u64,
    /// Disk usage in bytes
    pub disk_usage_bytes: u64,
    /// Total disk space in bytes
    pub disk_total_bytes: u64,
    /// Number of active connections
    pub active_connections: u32,
    /// Request rate (requests per second)
    pub request_rate: f64,
    /// Error rate (errors per second)
    pub error_rate: f64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            timestamp: SystemTime::now(),
            cpu_usage_percent: 0.0,
            memory_usage_bytes: 0,
            memory_total_bytes: 0,
            network_bytes_sent: 0,
            network_bytes_received: 0,
            disk_usage_bytes: 0,
            disk_total_bytes: 0,
            active_connections: 0,
            request_rate: 0.0,
            error_rate: 0.0,
            avg_response_time_ms: 0.0,
            custom_metrics: HashMap::new(),
        }
    }
}

impl PerformanceMetrics {
    /// Get memory usage percentage
    pub fn memory_usage_percent(&self) -> f64 {
        if self.memory_total_bytes == 0 {
            0.0
        } else {
            (self.memory_usage_bytes as f64 / self.memory_total_bytes as f64) * 100.0
        }
    }
    
    /// Get disk usage percentage
    pub fn disk_usage_percent(&self) -> f64 {
        if self.disk_total_bytes == 0 {
            0.0
        } else {
            (self.disk_usage_bytes as f64 / self.disk_total_bytes as f64) * 100.0
        }
    }
    
    /// Add custom metric
    pub fn add_custom_metric(&mut self, name: String, value: f64) {
        self.custom_metrics.insert(name, value);
    }
}

/// Health check status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System has warnings but is operational
    Warning,
    /// System is unhealthy
    Unhealthy,
    /// System status is unknown
    Unknown,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Component name
    pub component: String,
    /// Health status
    pub status: HealthStatus,
    /// Status message
    pub message: String,
    /// Check timestamp
    pub timestamp: SystemTime,
    /// Response time for the check
    pub response_time_ms: u64,
    /// Additional details
    pub details: HashMap<String, serde_json::Value>,
}

impl HealthCheck {
    /// Create a new health check
    pub fn new(component: String, status: HealthStatus, message: String) -> Self {
        Self {
            component,
            status,
            message,
            timestamp: SystemTime::now(),
            response_time_ms: 0,
            details: HashMap::new(),
        }
    }
    
    /// Add detail
    pub fn with_detail<T: Serialize>(mut self, key: String, value: T) -> Self {
        self.details.insert(
            key,
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
        self
    }
    
    /// Set response time
    pub fn with_response_time(mut self, response_time_ms: u64) -> Self {
        self.response_time_ms = response_time_ms;
        self
    }
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    /// Warning alert
    Warning,
    /// Error alert
    Error,
    /// Critical alert
    Critical,
}

/// System alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert title
    pub title: String,
    /// Alert description
    pub description: String,
    /// Component that triggered the alert
    pub component: String,
    /// Metric that triggered the alert
    pub metric: Option<String>,
    /// Current value
    pub current_value: Option<f64>,
    /// Threshold value
    pub threshold_value: Option<f64>,
    /// Alert timestamp
    pub timestamp: SystemTime,
    /// Alert metadata
    pub metadata: HashMap<String, String>,
}

impl Alert {
    /// Create a new alert
    pub fn new(
        severity: AlertSeverity,
        title: String,
        description: String,
        component: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            severity,
            title,
            description,
            component,
            metric: None,
            current_value: None,
            threshold_value: None,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set metric information
    pub fn with_metric(mut self, metric: String, current_value: f64, threshold_value: f64) -> Self {
        self.metric = Some(metric);
        self.current_value = Some(current_value);
        self.threshold_value = Some(threshold_value);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Alert rule for monitoring thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Rule name
    pub name: String,
    /// Metric to monitor
    pub metric: String,
    /// Threshold value
    pub threshold: f64,
    /// Comparison operator
    pub operator: ComparisonOperator,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Rule enabled
    pub enabled: bool,
    /// Evaluation interval
    pub evaluation_interval: Duration,
    /// Last evaluation time
    pub last_evaluation: Option<SystemTime>,
}

/// Comparison operators for alert rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    /// Greater than
    GreaterThan,
    /// Greater than or equal
    GreaterThanOrEqual,
    /// Less than
    LessThan,
    /// Less than or equal
    LessThanOrEqual,
    /// Equal
    Equal,
    /// Not equal
    NotEqual,
}

impl ComparisonOperator {
    /// Evaluate the comparison
    pub fn evaluate(&self, value: f64, threshold: f64) -> bool {
        match self {
            ComparisonOperator::GreaterThan => value > threshold,
            ComparisonOperator::GreaterThanOrEqual => value >= threshold,
            ComparisonOperator::LessThan => value < threshold,
            ComparisonOperator::LessThanOrEqual => value <= threshold,
            ComparisonOperator::Equal => (value - threshold).abs() < f64::EPSILON,
            ComparisonOperator::NotEqual => (value - threshold).abs() >= f64::EPSILON,
        }
    }
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable monitoring
    pub enabled: bool,
    /// Metrics collection interval
    pub collection_interval: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Enable alerting
    pub alerting_enabled: bool,
    /// Alert evaluation interval
    pub alert_evaluation_interval: Duration,
    /// Metrics retention period
    pub metrics_retention: Duration,
    /// Enable detailed metrics
    pub detailed_metrics: bool,
    /// Custom health check endpoints
    pub health_check_endpoints: Vec<String>,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval: Duration::from_secs(60), // 1 minute
            health_check_interval: Duration::from_secs(30), // 30 seconds
            alerting_enabled: true,
            alert_evaluation_interval: Duration::from_secs(60), // 1 minute
            metrics_retention: Duration::from_secs(86400 * 7), // 7 days
            detailed_metrics: false,
            health_check_endpoints: Vec::new(),
        }
    }
}

/// Metrics collector for system monitoring
#[derive(Debug)]
pub struct MetricsCollector {
    /// Configuration
    config: MonitoringConfig,
    /// Metrics history
    metrics_history: Arc<RwLock<Vec<PerformanceMetrics>>>,
    /// Current metrics
    current_metrics: Arc<RwLock<PerformanceMetrics>>,
    /// Health checks
    health_checks: Arc<RwLock<HashMap<String, HealthCheck>>>,
    /// Alert rules
    alert_rules: Arc<RwLock<Vec<AlertRule>>>,
    /// Active alerts
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: MonitoringConfig) -> Result<Self, MonitoringError> {
        Ok(Self {
            config,
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            current_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            health_checks: Arc::new(RwLock::new(HashMap::new())),
            alert_rules: Arc::new(RwLock::new(Vec::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Record resource usage
    pub async fn record_resource_usage(
        &self,
        usage: &super::resources::ResourceUsage,
    ) -> Result<(), MonitoringError> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let mut metrics = self.current_metrics.write().unwrap();
        metrics.memory_usage_bytes = usage.memory_bytes;
        metrics.network_bytes_sent = usage.network_bytes_sent;
        metrics.network_bytes_received = usage.network_bytes_received;
        metrics.disk_usage_bytes = usage.storage_bytes;
        
        // Add custom metrics from resource usage
        for (name, value) in &usage.custom_resources {
            metrics.add_custom_metric(name.clone(), *value);
        }
        
        Ok(())
    }
    
    /// Collect system metrics
    pub async fn collect_metrics(&self) -> Result<PerformanceMetrics, MonitoringError> {
        if !self.config.enabled {
            return Ok(PerformanceMetrics::default());
        }
        
        let mut metrics = PerformanceMetrics::default();
        
        // Collect basic system metrics (simplified for demo)
        metrics.cpu_usage_percent = self.get_cpu_usage().await?;
        metrics.memory_usage_bytes = self.get_memory_usage().await?;
        metrics.memory_total_bytes = self.get_total_memory().await?;
        
        // Update current metrics
        {
            let mut current = self.current_metrics.write().unwrap();
            *current = metrics.clone();
        }
        
        // Add to history
        {
            let mut history = self.metrics_history.write().unwrap();
            history.push(metrics.clone());
            
            // Clean up old metrics based on retention policy
            let cutoff_time = SystemTime::now() - self.config.metrics_retention;
            history.retain(|m| m.timestamp > cutoff_time);
        }
        
        Ok(metrics)
    }
    
    /// Perform health checks
    pub async fn perform_health_checks(&self) -> Result<HashMap<String, HealthCheck>, MonitoringError> {
        let mut checks = HashMap::new();
        
        // Basic system health check
        let system_check = self.check_system_health().await?;
        checks.insert("system".to_string(), system_check);
        
        // Database health check (if configured)
        let db_check = self.check_database_health().await?;
        checks.insert("database".to_string(), db_check);
        
        // Custom endpoint health checks
        for endpoint in &self.config.health_check_endpoints {
            let check = self.check_endpoint_health(endpoint).await?;
            checks.insert(endpoint.clone(), check);
        }
        
        // Update stored health checks
        {
            let mut stored_checks = self.health_checks.write().unwrap();
            *stored_checks = checks.clone();
        }
        
        Ok(checks)
    }
    
    /// Evaluate alert rules
    pub async fn evaluate_alerts(&self) -> Result<Vec<Alert>, MonitoringError> {
        if !self.config.alerting_enabled {
            return Ok(Vec::new());
        }
        
        let current_metrics = self.current_metrics.read().unwrap().clone();
        let mut new_alerts = Vec::new();
        
        let alert_rules = self.alert_rules.read().unwrap().clone();
        for rule in alert_rules {
            if !rule.enabled {
                continue;
            }
            
            // Get metric value
            let metric_value = self.get_metric_value(&current_metrics, &rule.metric);
            
            if let Some(value) = metric_value {
                if rule.operator.evaluate(value, rule.threshold) {
                    let alert = Alert::new(
                        rule.severity,
                        format!("Alert: {}", rule.name),
                        format!("Metric {} {} threshold", rule.metric, 
                               if rule.operator.evaluate(value, rule.threshold) { "exceeded" } else { "below" }),
                        "system".to_string(),
                    )
                    .with_metric(rule.metric.clone(), value, rule.threshold);
                    
                    new_alerts.push(alert);
                }
            }
        }
        
        // Update active alerts
        {
            let mut active = self.active_alerts.write().unwrap();
            for alert in &new_alerts {
                active.insert(alert.id.clone(), alert.clone());
            }
        }
        
        Ok(new_alerts)
    }
    
    /// Add alert rule
    pub fn add_alert_rule(&self, rule: AlertRule) {
        let mut rules = self.alert_rules.write().unwrap();
        rules.push(rule);
    }
    
    /// Get current metrics
    pub fn get_current_metrics(&self) -> PerformanceMetrics {
        self.current_metrics.read().unwrap().clone()
    }
    
    /// Get metrics history
    pub fn get_metrics_history(&self) -> Vec<PerformanceMetrics> {
        self.metrics_history.read().unwrap().clone()
    }
    
    /// Get health checks
    pub fn get_health_checks(&self) -> HashMap<String, HealthCheck> {
        self.health_checks.read().unwrap().clone()
    }
    
    /// Get active alerts
    pub fn get_active_alerts(&self) -> HashMap<String, Alert> {
        self.active_alerts.read().unwrap().clone()
    }
    
    // Helper methods for collecting system metrics
    async fn get_cpu_usage(&self) -> Result<f64, MonitoringError> {
        // Simplified CPU usage calculation
        Ok(25.0) // Mock value
    }
    
    async fn get_memory_usage(&self) -> Result<u64, MonitoringError> {
        // Simplified memory usage calculation
        Ok(1024 * 1024 * 512) // 512 MB mock value
    }
    
    async fn get_total_memory(&self) -> Result<u64, MonitoringError> {
        // Simplified total memory calculation
        Ok(1024 * 1024 * 1024 * 8) // 8 GB mock value
    }
    
    async fn check_system_health(&self) -> Result<HealthCheck, MonitoringError> {
        let start_time = std::time::Instant::now();
        
        // Perform basic system checks
        let cpu_usage = self.get_cpu_usage().await?;
        let memory_usage = self.get_memory_usage().await?;
        let total_memory = self.get_total_memory().await?;
        
        let memory_percent = (memory_usage as f64 / total_memory as f64) * 100.0;
        
        let status = if cpu_usage > 90.0 || memory_percent > 90.0 {
            HealthStatus::Unhealthy
        } else if cpu_usage > 70.0 || memory_percent > 70.0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };
        
        let response_time = start_time.elapsed().as_millis() as u64;
        
        Ok(HealthCheck::new(
            "system".to_string(),
            status,
            format!("CPU: {:.1}%, Memory: {:.1}%", cpu_usage, memory_percent),
        )
        .with_response_time(response_time)
        .with_detail("cpu_usage".to_string(), cpu_usage)
        .with_detail("memory_usage_percent".to_string(), memory_percent))
    }
    
    async fn check_database_health(&self) -> Result<HealthCheck, MonitoringError> {
        // Mock database health check
        Ok(HealthCheck::new(
            "database".to_string(),
            HealthStatus::Healthy,
            "Database connection healthy".to_string(),
        )
        .with_response_time(5))
    }
    
    async fn check_endpoint_health(&self, _endpoint: &str) -> Result<HealthCheck, MonitoringError> {
        // Mock endpoint health check
        Ok(HealthCheck::new(
            "endpoint".to_string(),
            HealthStatus::Healthy,
            "Endpoint responding".to_string(),
        )
        .with_response_time(100))
    }
    
    fn get_metric_value(&self, metrics: &PerformanceMetrics, metric_name: &str) -> Option<f64> {
        match metric_name {
            "cpu_usage_percent" => Some(metrics.cpu_usage_percent),
            "memory_usage_percent" => Some(metrics.memory_usage_percent()),
            "disk_usage_percent" => Some(metrics.disk_usage_percent()),
            "request_rate" => Some(metrics.request_rate),
            "error_rate" => Some(metrics.error_rate),
            "avg_response_time_ms" => Some(metrics.avg_response_time_ms),
            _ => metrics.custom_metrics.get(metric_name).copied(),
        }
    }
    
    /// Get configuration
    pub fn config(&self) -> &MonitoringConfig {
        &self.config
    }
}

/// Alert manager for handling system alerts
#[derive(Debug)]
pub struct AlertManager {
    /// Active alerts
    alerts: Arc<RwLock<HashMap<String, Alert>>>,
    /// Alert handlers
    handlers: Arc<RwLock<Vec<Box<dyn AlertHandler>>>>,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        Self {
            alerts: Arc::new(RwLock::new(HashMap::new())),
            handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Send alert
    pub async fn send_alert(&self, alert: Alert) -> Result<(), MonitoringError> {
        // Store alert
        {
            let mut alerts = self.alerts.write().unwrap();
            alerts.insert(alert.id.clone(), alert.clone());
        }
        
        // Send to handlers
        let handlers = self.handlers.read().unwrap();
        for handler in handlers.iter() {
            handler.handle_alert(&alert).await?;
        }
        
        Ok(())
    }
    
    /// Get active alerts
    pub fn get_active_alerts(&self) -> HashMap<String, Alert> {
        self.alerts.read().unwrap().clone()
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for alert handlers
#[async_trait::async_trait]
pub trait AlertHandler: Send + Sync + std::fmt::Debug {
    /// Handle an alert
    async fn handle_alert(&self, alert: &Alert) -> Result<(), MonitoringError>;
}

/// Console alert handler (for testing)
#[derive(Debug)]
pub struct ConsoleAlertHandler;

#[async_trait::async_trait]
impl AlertHandler for ConsoleAlertHandler {
    async fn handle_alert(&self, alert: &Alert) -> Result<(), MonitoringError> {
        println!("🚨 ALERT [{:?}]: {} - {}", alert.severity, alert.title, alert.description);
        Ok(())
    }
}

/// Errors that can occur in monitoring operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum MonitoringError {
    /// Metrics collection error
    #[error("Metrics collection error: {message}")]
    MetricsError { message: String },
    
    /// Health check error
    #[error("Health check error: {message}")]
    HealthCheckError { message: String },
    
    /// Alert error
    #[error("Alert error: {message}")]
    AlertError { message: String },
    
    /// Configuration error
    #[error("Monitoring configuration error: {message}")]
    ConfigurationError { message: String },
    
    /// System error
    #[error("Monitoring system error: {message}")]
    SystemError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::default();
        metrics.memory_usage_bytes = 1024 * 1024 * 1024; // 1 GB
        metrics.memory_total_bytes = 1024 * 1024 * 1024 * 4; // 4 GB
        
        assert_eq!(metrics.memory_usage_percent(), 25.0);
        
        metrics.add_custom_metric("custom_metric".to_string(), 42.0);
        assert_eq!(metrics.custom_metrics.get("custom_metric"), Some(&42.0));
    }

    #[test]
    fn test_health_check() {
        let check = HealthCheck::new(
            "test_component".to_string(),
            HealthStatus::Healthy,
            "All systems operational".to_string(),
        )
        .with_response_time(100)
        .with_detail("version".to_string(), "1.0.0");
        
        assert_eq!(check.component, "test_component");
        assert_eq!(check.status, HealthStatus::Healthy);
        assert_eq!(check.response_time_ms, 100);
        assert_eq!(check.details.get("version"), Some(&serde_json::json!("1.0.0")));
    }

    #[test]
    fn test_comparison_operators() {
        assert!(ComparisonOperator::GreaterThan.evaluate(10.0, 5.0));
        assert!(!ComparisonOperator::GreaterThan.evaluate(5.0, 10.0));
        
        assert!(ComparisonOperator::LessThan.evaluate(5.0, 10.0));
        assert!(!ComparisonOperator::LessThan.evaluate(10.0, 5.0));
        
        assert!(ComparisonOperator::Equal.evaluate(5.0, 5.0));
        assert!(!ComparisonOperator::Equal.evaluate(5.0, 6.0));
    }

    #[tokio::test]
    async fn test_metrics_collector() {
        let config = MonitoringConfig::default();
        let collector = MetricsCollector::new(config).unwrap();
        
        let metrics = collector.collect_metrics().await.unwrap();
        assert!(metrics.cpu_usage_percent >= 0.0);
        assert!(metrics.memory_usage_bytes > 0);
        
        let current = collector.get_current_metrics();
        assert_eq!(current.cpu_usage_percent, metrics.cpu_usage_percent);
    }

    #[tokio::test]
    async fn test_alert_manager() {
        let manager = AlertManager::new();
        
        let alert = Alert::new(
            AlertSeverity::Warning,
            "Test Alert".to_string(),
            "This is a test alert".to_string(),
            "test_component".to_string(),
        );
        
        manager.send_alert(alert.clone()).await.unwrap();
        
        let active_alerts = manager.get_active_alerts();
        assert_eq!(active_alerts.len(), 1);
        assert!(active_alerts.contains_key(&alert.id));
    }
}
