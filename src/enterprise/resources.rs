// Resource management and quotas for AgentGraph
// Provides CPU, memory, and execution limits per tenant

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use thiserror::Error;

/// Resource usage tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceUsage {
    /// CPU time used in milliseconds
    pub cpu_time_ms: u64,
    /// Memory used in bytes
    pub memory_bytes: u64,
    /// Number of executions
    pub execution_count: u64,
    /// Network bytes sent
    pub network_bytes_sent: u64,
    /// Network bytes received
    pub network_bytes_received: u64,
    /// Storage bytes used
    pub storage_bytes: u64,
    /// Custom resource usage
    pub custom_resources: HashMap<String, f64>,
    /// Last updated timestamp
    pub last_updated: SystemTime,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_time_ms: 0,
            memory_bytes: 0,
            execution_count: 0,
            network_bytes_sent: 0,
            network_bytes_received: 0,
            storage_bytes: 0,
            custom_resources: HashMap::new(),
            last_updated: SystemTime::now(),
        }
    }
}

impl ResourceUsage {
    /// Create new resource usage
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add another resource usage to this one
    pub fn add(&mut self, other: &ResourceUsage) {
        self.cpu_time_ms += other.cpu_time_ms;
        self.memory_bytes += other.memory_bytes;
        self.execution_count += other.execution_count;
        self.network_bytes_sent += other.network_bytes_sent;
        self.network_bytes_received += other.network_bytes_received;
        self.storage_bytes += other.storage_bytes;
        
        // Add custom resources
        for (key, value) in &other.custom_resources {
            *self.custom_resources.entry(key.clone()).or_insert(0.0) += value;
        }
        
        self.last_updated = SystemTime::now();
    }
    
    /// Reset all usage counters
    pub fn reset(&mut self) {
        self.cpu_time_ms = 0;
        self.memory_bytes = 0;
        self.execution_count = 0;
        self.network_bytes_sent = 0;
        self.network_bytes_received = 0;
        self.storage_bytes = 0;
        self.custom_resources.clear();
        self.last_updated = SystemTime::now();
    }
    
    /// Get total network bytes
    pub fn total_network_bytes(&self) -> u64 {
        self.network_bytes_sent + self.network_bytes_received
    }
    
    /// Add custom resource usage
    pub fn add_custom_resource(&mut self, name: String, value: f64) {
        *self.custom_resources.entry(name).or_insert(0.0) += value;
        self.last_updated = SystemTime::now();
    }
}

/// Resource limits and quotas
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceLimits {
    /// Maximum CPU time per period in milliseconds
    pub max_cpu_time_ms: Option<u64>,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<u64>,
    /// Maximum number of executions per period
    pub max_executions: Option<u64>,
    /// Maximum network bytes per period
    pub max_network_bytes: Option<u64>,
    /// Maximum storage bytes
    pub max_storage_bytes: Option<u64>,
    /// Maximum concurrent executions
    pub max_concurrent_executions: Option<u32>,
    /// Custom resource limits
    pub custom_limits: HashMap<String, f64>,
    /// Quota period (e.g., daily, hourly)
    pub quota_period: QuotaPeriod,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_time_ms: Some(3600000), // 1 hour
            max_memory_bytes: Some(1024 * 1024 * 1024), // 1 GB
            max_executions: Some(1000),
            max_network_bytes: Some(100 * 1024 * 1024), // 100 MB
            max_storage_bytes: Some(10 * 1024 * 1024 * 1024), // 10 GB
            max_concurrent_executions: Some(10),
            custom_limits: HashMap::new(),
            quota_period: QuotaPeriod::Daily,
        }
    }
}

impl ResourceLimits {
    /// Create unlimited resource limits
    pub fn unlimited() -> Self {
        Self {
            max_cpu_time_ms: None,
            max_memory_bytes: None,
            max_executions: None,
            max_network_bytes: None,
            max_storage_bytes: None,
            max_concurrent_executions: None,
            custom_limits: HashMap::new(),
            quota_period: QuotaPeriod::Daily,
        }
    }
    
    /// Create basic resource limits
    pub fn basic() -> Self {
        Self {
            max_cpu_time_ms: Some(600000), // 10 minutes
            max_memory_bytes: Some(256 * 1024 * 1024), // 256 MB
            max_executions: Some(100),
            max_network_bytes: Some(10 * 1024 * 1024), // 10 MB
            max_storage_bytes: Some(1024 * 1024 * 1024), // 1 GB
            max_concurrent_executions: Some(5),
            custom_limits: HashMap::new(),
            quota_period: QuotaPeriod::Daily,
        }
    }
    
    /// Create premium resource limits
    pub fn premium() -> Self {
        Self {
            max_cpu_time_ms: Some(7200000), // 2 hours
            max_memory_bytes: Some(4 * 1024 * 1024 * 1024), // 4 GB
            max_executions: Some(10000),
            max_network_bytes: Some(1024 * 1024 * 1024), // 1 GB
            max_storage_bytes: Some(100 * 1024 * 1024 * 1024), // 100 GB
            max_concurrent_executions: Some(50),
            custom_limits: HashMap::new(),
            quota_period: QuotaPeriod::Daily,
        }
    }
    
    /// Check if usage exceeds limits
    pub fn check_usage(&self, usage: &ResourceUsage) -> Vec<ResourceViolation> {
        let mut violations = Vec::new();
        
        if let Some(limit) = self.max_cpu_time_ms {
            if usage.cpu_time_ms > limit {
                violations.push(ResourceViolation::CpuTimeExceeded {
                    used: usage.cpu_time_ms,
                    limit,
                });
            }
        }
        
        if let Some(limit) = self.max_memory_bytes {
            if usage.memory_bytes > limit {
                violations.push(ResourceViolation::MemoryExceeded {
                    used: usage.memory_bytes,
                    limit,
                });
            }
        }
        
        if let Some(limit) = self.max_executions {
            if usage.execution_count > limit {
                violations.push(ResourceViolation::ExecutionCountExceeded {
                    used: usage.execution_count,
                    limit,
                });
            }
        }
        
        if let Some(limit) = self.max_network_bytes {
            if usage.total_network_bytes() > limit {
                violations.push(ResourceViolation::NetworkBytesExceeded {
                    used: usage.total_network_bytes(),
                    limit,
                });
            }
        }
        
        if let Some(limit) = self.max_storage_bytes {
            if usage.storage_bytes > limit {
                violations.push(ResourceViolation::StorageBytesExceeded {
                    used: usage.storage_bytes,
                    limit,
                });
            }
        }
        
        // Check custom limits
        for (name, limit) in &self.custom_limits {
            if let Some(used) = usage.custom_resources.get(name) {
                if *used > *limit {
                    violations.push(ResourceViolation::CustomResourceExceeded {
                        resource_name: name.clone(),
                        used: *used,
                        limit: *limit,
                    });
                }
            }
        }
        
        violations
    }
}

/// Quota period for resource limits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuotaPeriod {
    /// Per minute quotas
    Minute,
    /// Per hour quotas
    Hourly,
    /// Per day quotas
    Daily,
    /// Per week quotas
    Weekly,
    /// Per month quotas
    Monthly,
}

impl QuotaPeriod {
    /// Get duration for this quota period
    pub fn duration(&self) -> Duration {
        match self {
            QuotaPeriod::Minute => Duration::from_secs(60),
            QuotaPeriod::Hourly => Duration::from_secs(3600),
            QuotaPeriod::Daily => Duration::from_secs(86400),
            QuotaPeriod::Weekly => Duration::from_secs(604800),
            QuotaPeriod::Monthly => Duration::from_secs(2592000), // 30 days
        }
    }
}

/// Resource quota for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    /// Tenant ID
    pub tenant_id: String,
    /// Resource limits
    pub limits: ResourceLimits,
    /// Current usage
    pub current_usage: ResourceUsage,
    /// Usage history
    pub usage_history: Vec<ResourceUsageSnapshot>,
    /// Quota start time
    pub quota_start: SystemTime,
    /// Quota end time
    pub quota_end: SystemTime,
}

impl ResourceQuota {
    /// Create a new resource quota
    pub fn new(tenant_id: String, limits: ResourceLimits) -> Self {
        let now = SystemTime::now();
        let quota_end = now + limits.quota_period.duration();
        
        Self {
            tenant_id,
            limits,
            current_usage: ResourceUsage::default(),
            usage_history: Vec::new(),
            quota_start: now,
            quota_end,
        }
    }
    
    /// Check if quota has expired
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.quota_end
    }
    
    /// Reset quota for new period
    pub fn reset_for_new_period(&mut self) {
        // Save current usage to history
        let snapshot = ResourceUsageSnapshot {
            usage: self.current_usage.clone(),
            timestamp: SystemTime::now(),
            period_start: self.quota_start,
            period_end: self.quota_end,
        };
        self.usage_history.push(snapshot);
        
        // Reset current usage
        self.current_usage.reset();
        
        // Update quota period
        let now = SystemTime::now();
        self.quota_start = now;
        self.quota_end = now + self.limits.quota_period.duration();
        
        // Keep only recent history (last 30 periods)
        if self.usage_history.len() > 30 {
            self.usage_history.drain(0..self.usage_history.len() - 30);
        }
    }
    
    /// Add resource usage
    pub fn add_usage(&mut self, usage: &ResourceUsage) {
        self.current_usage.add(usage);
    }
    
    /// Check for violations
    pub fn check_violations(&self) -> Vec<ResourceViolation> {
        self.limits.check_usage(&self.current_usage)
    }
}

/// Snapshot of resource usage for a specific period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageSnapshot {
    /// Resource usage
    pub usage: ResourceUsage,
    /// Snapshot timestamp
    pub timestamp: SystemTime,
    /// Period start time
    pub period_start: SystemTime,
    /// Period end time
    pub period_end: SystemTime,
}

/// Resource violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceViolation {
    /// CPU time limit exceeded
    CpuTimeExceeded { used: u64, limit: u64 },
    /// Memory limit exceeded
    MemoryExceeded { used: u64, limit: u64 },
    /// Execution count limit exceeded
    ExecutionCountExceeded { used: u64, limit: u64 },
    /// Network bytes limit exceeded
    NetworkBytesExceeded { used: u64, limit: u64 },
    /// Storage bytes limit exceeded
    StorageBytesExceeded { used: u64, limit: u64 },
    /// Concurrent execution limit exceeded
    ConcurrentExecutionExceeded { used: u32, limit: u32 },
    /// Custom resource limit exceeded
    CustomResourceExceeded { resource_name: String, used: f64, limit: f64 },
}

/// Resource configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Enable resource quotas
    pub quotas_enabled: bool,
    /// Default resource limits
    pub default_limits: ResourceLimits,
    /// Resource monitoring interval
    pub monitoring_interval: Duration,
    /// Enable resource alerts
    pub alerts_enabled: bool,
    /// Alert thresholds (percentage of limit)
    pub alert_thresholds: Vec<f64>,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            quotas_enabled: true,
            default_limits: ResourceLimits::default(),
            monitoring_interval: Duration::from_secs(60), // 1 minute
            alerts_enabled: true,
            alert_thresholds: vec![0.8, 0.9, 0.95], // 80%, 90%, 95%
        }
    }
}

/// Resource manager for handling quotas and limits
#[derive(Debug)]
pub struct ResourceManager {
    /// Configuration
    config: ResourceConfig,
    /// Tenant quotas
    quotas: Arc<RwLock<HashMap<String, ResourceQuota>>>,
    /// Concurrent execution tracking
    concurrent_executions: Arc<RwLock<HashMap<String, u32>>>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new(config: ResourceConfig) -> Result<Self, ResourceError> {
        Ok(Self {
            config,
            quotas: Arc::new(RwLock::new(HashMap::new())),
            concurrent_executions: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Set resource limits for a tenant
    pub async fn set_tenant_limits(
        &self,
        tenant_id: String,
        limits: ResourceLimits,
    ) -> Result<(), ResourceError> {
        let mut quotas = self.quotas.write().unwrap();
        
        if let Some(quota) = quotas.get_mut(&tenant_id) {
            quota.limits = limits;
        } else {
            let quota = ResourceQuota::new(tenant_id.clone(), limits);
            quotas.insert(tenant_id, quota);
        }
        
        Ok(())
    }
    
    /// Record resource usage for a tenant
    pub async fn record_usage(
        &self,
        tenant_id: Option<&str>,
        usage: &ResourceUsage,
    ) -> Result<(), ResourceError> {
        if !self.config.quotas_enabled {
            return Ok(());
        }
        
        let tenant_id = tenant_id.unwrap_or("default");
        let mut quotas = self.quotas.write().unwrap();
        
        let quota = quotas.entry(tenant_id.to_string())
            .or_insert_with(|| ResourceQuota::new(tenant_id.to_string(), self.config.default_limits.clone()));
        
        // Reset quota if expired
        if quota.is_expired() {
            quota.reset_for_new_period();
        }
        
        quota.add_usage(&usage);
        
        Ok(())
    }
    
    /// Check resource quotas for a tenant
    pub async fn check_quotas(
        &self,
        usage: &ResourceUsage,
        tenant_id: Option<&str>,
    ) -> Result<(), ResourceError> {
        if !self.config.quotas_enabled {
            return Ok(());
        }
        
        let tenant_id = tenant_id.unwrap_or("default");
        let quotas = self.quotas.read().unwrap();
        
        if let Some(quota) = quotas.get(tenant_id) {
            let violations = quota.check_violations();
            if !violations.is_empty() {
                return Err(ResourceError::QuotaExceeded {
                    tenant_id: tenant_id.to_string(),
                    violations,
                });
            }
        }
        
        Ok(())
    }
    
    /// Start execution (increment concurrent count)
    pub async fn start_execution(&self, tenant_id: Option<&str>) -> Result<(), ResourceError> {
        let tenant_id = tenant_id.unwrap_or("default");
        let mut concurrent = self.concurrent_executions.write().unwrap();
        
        let current_count = concurrent.entry(tenant_id.to_string()).or_insert(0);
        
        // Check concurrent execution limit
        let quotas = self.quotas.read().unwrap();
        if let Some(quota) = quotas.get(tenant_id) {
            if let Some(limit) = quota.limits.max_concurrent_executions {
                if *current_count >= limit {
                    return Err(ResourceError::ConcurrentLimitExceeded {
                        tenant_id: tenant_id.to_string(),
                        current: *current_count,
                        limit,
                    });
                }
            }
        }
        
        *current_count += 1;
        Ok(())
    }
    
    /// End execution (decrement concurrent count)
    pub async fn end_execution(&self, tenant_id: Option<&str>) -> Result<(), ResourceError> {
        let tenant_id = tenant_id.unwrap_or("default");
        let mut concurrent = self.concurrent_executions.write().unwrap();
        
        if let Some(count) = concurrent.get_mut(tenant_id) {
            *count = count.saturating_sub(1);
        }
        
        Ok(())
    }
    
    /// Get resource usage for a tenant
    pub async fn get_usage(&self, tenant_id: &str) -> Result<ResourceUsage, ResourceError> {
        let quotas = self.quotas.read().unwrap();
        
        quotas.get(tenant_id)
            .map(|quota| quota.current_usage.clone())
            .ok_or_else(|| ResourceError::TenantNotFound {
                tenant_id: tenant_id.to_string(),
            })
    }
    
    /// Get configuration
    pub fn config(&self) -> &ResourceConfig {
        &self.config
    }
}

/// Errors that can occur in resource operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ResourceError {
    /// Quota exceeded
    #[error("Resource quota exceeded for tenant {tenant_id}")]
    QuotaExceeded {
        tenant_id: String,
        violations: Vec<ResourceViolation>,
    },
    
    /// Concurrent execution limit exceeded
    #[error("Concurrent execution limit exceeded for tenant {tenant_id}: {current}/{limit}")]
    ConcurrentLimitExceeded {
        tenant_id: String,
        current: u32,
        limit: u32,
    },
    
    /// Tenant not found
    #[error("Tenant not found: {tenant_id}")]
    TenantNotFound { tenant_id: String },
    
    /// Configuration error
    #[error("Resource configuration error: {message}")]
    ConfigurationError { message: String },
    
    /// System error
    #[error("Resource system error: {message}")]
    SystemError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_usage_operations() {
        let mut usage1 = ResourceUsage::new();
        usage1.cpu_time_ms = 1000;
        usage1.memory_bytes = 1024;
        usage1.execution_count = 1;
        
        let mut usage2 = ResourceUsage::new();
        usage2.cpu_time_ms = 500;
        usage2.memory_bytes = 512;
        usage2.execution_count = 1;
        
        usage1.add(&usage2);
        
        assert_eq!(usage1.cpu_time_ms, 1500);
        assert_eq!(usage1.memory_bytes, 1536);
        assert_eq!(usage1.execution_count, 2);
    }

    #[test]
    fn test_resource_limits_check() {
        let limits = ResourceLimits {
            max_cpu_time_ms: Some(1000),
            max_memory_bytes: Some(1024),
            max_executions: Some(10),
            ..Default::default()
        };
        
        let mut usage = ResourceUsage::new();
        usage.cpu_time_ms = 1500; // Exceeds limit
        usage.memory_bytes = 512;  // Within limit
        usage.execution_count = 5; // Within limit
        
        let violations = limits.check_usage(&usage);
        assert_eq!(violations.len(), 1);
        
        match &violations[0] {
            ResourceViolation::CpuTimeExceeded { used, limit } => {
                assert_eq!(*used, 1500);
                assert_eq!(*limit, 1000);
            }
            _ => panic!("Wrong violation type"),
        }
    }

    #[tokio::test]
    async fn test_resource_manager() {
        let config = ResourceConfig::default();
        let manager = ResourceManager::new(config).unwrap();
        
        // Set tenant limits
        let limits = ResourceLimits::basic();
        manager.set_tenant_limits("test_tenant".to_string(), limits).await.unwrap();
        
        // Record usage
        let mut usage = ResourceUsage::new();
        usage.cpu_time_ms = 100;
        usage.memory_bytes = 1024;
        usage.execution_count = 1;
        
        manager.record_usage(Some("test_tenant"), &usage).await.unwrap();
        
        // Get usage
        let recorded_usage = manager.get_usage("test_tenant").await.unwrap();
        assert_eq!(recorded_usage.cpu_time_ms, 100);
        assert_eq!(recorded_usage.memory_bytes, 1024);
        assert_eq!(recorded_usage.execution_count, 1);
    }

    #[test]
    fn test_quota_period_duration() {
        assert_eq!(QuotaPeriod::Minute.duration(), Duration::from_secs(60));
        assert_eq!(QuotaPeriod::Hourly.duration(), Duration::from_secs(3600));
        assert_eq!(QuotaPeriod::Daily.duration(), Duration::from_secs(86400));
    }

    #[test]
    fn test_resource_quota_reset() {
        let limits = ResourceLimits::default();
        let mut quota = ResourceQuota::new("test".to_string(), limits);
        
        // Add some usage
        let mut usage = ResourceUsage::new();
        usage.cpu_time_ms = 1000;
        quota.add_usage(&usage);
        
        assert_eq!(quota.current_usage.cpu_time_ms, 1000);
        
        // Reset quota
        quota.reset_for_new_period();
        
        assert_eq!(quota.current_usage.cpu_time_ms, 0);
        assert_eq!(quota.usage_history.len(), 1);
    }
}
