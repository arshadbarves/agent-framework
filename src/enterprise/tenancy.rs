// Multi-tenancy support for AgentGraph
// Provides tenant isolation and management capabilities

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use thiserror::Error;

/// Tenant information and configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tenant {
    /// Unique tenant identifier
    pub id: String,
    /// Human-readable tenant name
    pub name: String,
    /// Tenant status
    pub status: TenantStatus,
    /// Tenant configuration
    pub config: TenantConfig,
    /// Resource limits for this tenant
    pub resource_limits: super::resources::ResourceLimits,
    /// Tenant metadata
    pub metadata: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: SystemTime,
    /// Last updated timestamp
    pub updated_at: SystemTime,
}

impl Tenant {
    /// Create a new tenant
    pub fn new(id: String, name: String) -> Self {
        let now = SystemTime::now();
        Self {
            id,
            name,
            status: TenantStatus::Active,
            config: TenantConfig::default(),
            resource_limits: super::resources::ResourceLimits::default(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Check if tenant is active
    pub fn is_active(&self) -> bool {
        self.status == TenantStatus::Active
    }
    
    /// Update tenant configuration
    pub fn update_config(&mut self, config: TenantConfig) {
        self.config = config;
        self.updated_at = SystemTime::now();
    }
    
    /// Update resource limits
    pub fn update_resource_limits(&mut self, limits: super::resources::ResourceLimits) {
        self.resource_limits = limits;
        self.updated_at = SystemTime::now();
    }
    
    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
        self.updated_at = SystemTime::now();
    }
    
    /// Suspend tenant
    pub fn suspend(&mut self, reason: String) {
        self.status = TenantStatus::Suspended;
        self.add_metadata("suspension_reason".to_string(), reason);
    }
    
    /// Activate tenant
    pub fn activate(&mut self) {
        self.status = TenantStatus::Active;
        self.metadata.remove("suspension_reason");
    }
}

/// Tenant status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantStatus {
    /// Tenant is active and can execute workflows
    Active,
    /// Tenant is suspended (temporarily disabled)
    Suspended,
    /// Tenant is archived (permanently disabled)
    Archived,
    /// Tenant is being created
    Provisioning,
}

/// Configuration for a specific tenant
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TenantConfig {
    /// Maximum concurrent executions
    pub max_concurrent_executions: u32,
    /// Default execution timeout
    pub default_timeout: Duration,
    /// Enable audit logging for this tenant
    pub audit_enabled: bool,
    /// Enable monitoring for this tenant
    pub monitoring_enabled: bool,
    /// Custom configuration parameters
    pub custom_config: HashMap<String, serde_json::Value>,
    /// Allowed node types for this tenant
    pub allowed_node_types: Option<Vec<String>>,
    /// Allowed tool categories for this tenant
    pub allowed_tool_categories: Option<Vec<String>>,
}

impl Default for TenantConfig {
    fn default() -> Self {
        Self {
            max_concurrent_executions: 10,
            default_timeout: Duration::from_secs(300), // 5 minutes
            audit_enabled: true,
            monitoring_enabled: true,
            custom_config: HashMap::new(),
            allowed_node_types: None, // None means all allowed
            allowed_tool_categories: None, // None means all allowed
        }
    }
}

/// Global tenancy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenancyConfig {
    /// Enable multi-tenancy
    pub enabled: bool,
    /// Default tenant for non-tenant requests
    pub default_tenant_id: Option<String>,
    /// Maximum number of tenants
    pub max_tenants: Option<u32>,
    /// Tenant isolation level
    pub isolation_level: IsolationLevel,
    /// Tenant storage configuration
    pub storage: TenantStorageConfig,
}

impl Default for TenancyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_tenant_id: None,
            max_tenants: Some(1000),
            isolation_level: IsolationLevel::Strong,
            storage: TenantStorageConfig::default(),
        }
    }
}

/// Tenant isolation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationLevel {
    /// No isolation (single tenant mode)
    None,
    /// Basic isolation (separate contexts)
    Basic,
    /// Strong isolation (separate resources)
    Strong,
    /// Complete isolation (separate processes)
    Complete,
}

/// Tenant storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantStorageConfig {
    /// Storage backend type
    pub backend: StorageBackend,
    /// Connection configuration
    pub connection: HashMap<String, String>,
    /// Enable encryption at rest
    pub encryption_enabled: bool,
}

impl Default for TenantStorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::Memory,
            connection: HashMap::new(),
            encryption_enabled: false,
        }
    }
}

/// Storage backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageBackend {
    /// In-memory storage (for testing)
    Memory,
    /// File-based storage
    File,
    /// Database storage
    Database,
    /// Redis storage
    Redis,
}

/// Tenant execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    /// Tenant information
    pub tenant: Tenant,
    /// Execution namespace
    pub namespace: String,
    /// Isolation boundaries
    pub isolation: IsolationBoundaries,
    /// Context metadata
    pub metadata: HashMap<String, String>,
}

impl TenantContext {
    /// Create a new tenant context
    pub fn new(tenant: Tenant) -> Self {
        let namespace = format!("tenant_{}", tenant.id);
        Self {
            tenant,
            namespace,
            isolation: IsolationBoundaries::default(),
            metadata: HashMap::new(),
        }
    }
    
    /// Get tenant ID
    pub fn tenant_id(&self) -> &str {
        &self.tenant.id
    }
    
    /// Check if tenant can execute
    pub fn can_execute(&self) -> bool {
        self.tenant.is_active()
    }
    
    /// Add context metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

/// Isolation boundaries for tenant execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationBoundaries {
    /// Memory isolation
    pub memory_isolated: bool,
    /// Network isolation
    pub network_isolated: bool,
    /// File system isolation
    pub filesystem_isolated: bool,
    /// Process isolation
    pub process_isolated: bool,
}

impl Default for IsolationBoundaries {
    fn default() -> Self {
        Self {
            memory_isolated: true,
            network_isolated: false,
            filesystem_isolated: false,
            process_isolated: false,
        }
    }
}

/// Tenant manager for handling multi-tenancy
#[derive(Debug)]
pub struct TenantManager {
    /// Configuration
    config: TenancyConfig,
    /// Tenant storage
    tenants: Arc<RwLock<HashMap<String, Tenant>>>,
    /// Tenant statistics
    stats: Arc<RwLock<TenantStats>>,
}

impl TenantManager {
    /// Create a new tenant manager
    pub fn new(config: TenancyConfig) -> Result<Self, TenantError> {
        Ok(Self {
            config,
            tenants: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(TenantStats::default())),
        })
    }
    
    /// Create a new tenant
    pub async fn create_tenant(&self, id: String, name: String) -> Result<Tenant, TenantError> {
        // Check if tenancy is enabled
        if !self.config.enabled {
            return Err(TenantError::TenancyDisabled);
        }
        
        // Check tenant limits
        if let Some(max_tenants) = self.config.max_tenants {
            let current_count = self.tenants.read().unwrap().len() as u32;
            if current_count >= max_tenants {
                return Err(TenantError::TenantLimitExceeded { 
                    limit: max_tenants,
                    current: current_count,
                });
            }
        }
        
        let mut tenants = self.tenants.write().unwrap();
        
        // Check if tenant already exists
        if tenants.contains_key(&id) {
            return Err(TenantError::TenantAlreadyExists { tenant_id: id });
        }
        
        let tenant = Tenant::new(id.clone(), name);
        tenants.insert(id, tenant.clone());
        
        // Update statistics
        let mut stats = self.stats.write().unwrap();
        stats.total_tenants += 1;
        stats.active_tenants += 1;
        
        Ok(tenant)
    }
    
    /// Get a tenant by ID
    pub async fn get_tenant(&self, tenant_id: &str) -> Result<Tenant, TenantError> {
        let tenants = self.tenants.read().unwrap();
        tenants.get(tenant_id)
            .cloned()
            .ok_or_else(|| TenantError::TenantNotFound { 
                tenant_id: tenant_id.to_string() 
            })
    }
    
    /// Update a tenant
    pub async fn update_tenant(&self, tenant: Tenant) -> Result<(), TenantError> {
        let mut tenants = self.tenants.write().unwrap();
        tenants.insert(tenant.id.clone(), tenant);
        Ok(())
    }
    
    /// Delete a tenant
    pub async fn delete_tenant(&self, tenant_id: &str) -> Result<(), TenantError> {
        let mut tenants = self.tenants.write().unwrap();
        
        if let Some(tenant) = tenants.remove(tenant_id) {
            // Update statistics
            let mut stats = self.stats.write().unwrap();
            stats.total_tenants = stats.total_tenants.saturating_sub(1);
            if tenant.is_active() {
                stats.active_tenants = stats.active_tenants.saturating_sub(1);
            }
            Ok(())
        } else {
            Err(TenantError::TenantNotFound { 
                tenant_id: tenant_id.to_string() 
            })
        }
    }
    
    /// List all tenants
    pub async fn list_tenants(&self) -> Result<Vec<Tenant>, TenantError> {
        let tenants = self.tenants.read().unwrap();
        Ok(tenants.values().cloned().collect())
    }
    
    /// Validate tenant for execution
    pub async fn validate_tenant(&self, tenant: &Tenant) -> Result<(), TenantError> {
        if !tenant.is_active() {
            return Err(TenantError::TenantInactive { 
                tenant_id: tenant.id.clone(),
                status: tenant.status,
            });
        }
        
        Ok(())
    }
    
    /// Create tenant context
    pub async fn create_context(&self, tenant_id: &str) -> Result<TenantContext, TenantError> {
        let tenant = self.get_tenant(tenant_id).await?;
        Ok(TenantContext::new(tenant))
    }
    
    /// Get tenant statistics
    pub fn get_stats(&self) -> TenantStats {
        self.stats.read().unwrap().clone()
    }
    
    /// Get configuration
    pub fn config(&self) -> &TenancyConfig {
        &self.config
    }
}

/// Tenant statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantStats {
    /// Total number of tenants
    pub total_tenants: u64,
    /// Number of active tenants
    pub active_tenants: u64,
    /// Number of suspended tenants
    pub suspended_tenants: u64,
    /// Number of archived tenants
    pub archived_tenants: u64,
    /// Total executions across all tenants
    pub total_executions: u64,
    /// Last updated timestamp
    pub last_updated: SystemTime,
}

impl Default for TenantStats {
    fn default() -> Self {
        Self {
            total_tenants: 0,
            active_tenants: 0,
            suspended_tenants: 0,
            archived_tenants: 0,
            total_executions: 0,
            last_updated: SystemTime::now(),
        }
    }
}

/// Errors that can occur in tenant operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum TenantError {
    /// Tenancy is disabled
    #[error("Multi-tenancy is disabled")]
    TenancyDisabled,
    
    /// Tenant not found
    #[error("Tenant not found: {tenant_id}")]
    TenantNotFound { tenant_id: String },
    
    /// Tenant already exists
    #[error("Tenant already exists: {tenant_id}")]
    TenantAlreadyExists { tenant_id: String },
    
    /// Tenant is inactive
    #[error("Tenant is inactive: {tenant_id} (status: {status:?})")]
    TenantInactive { 
        tenant_id: String,
        status: TenantStatus,
    },
    
    /// Tenant limit exceeded
    #[error("Tenant limit exceeded: {current}/{limit}")]
    TenantLimitExceeded { 
        limit: u32,
        current: u32,
    },
    
    /// Configuration error
    #[error("Tenant configuration error: {message}")]
    ConfigurationError { message: String },
    
    /// Storage error
    #[error("Tenant storage error: {message}")]
    StorageError { message: String },
    
    /// System error
    #[error("Tenant system error: {message}")]
    SystemError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tenant_creation() {
        let tenant = Tenant::new("tenant_1".to_string(), "Test Tenant".to_string());
        
        assert_eq!(tenant.id, "tenant_1");
        assert_eq!(tenant.name, "Test Tenant");
        assert_eq!(tenant.status, TenantStatus::Active);
        assert!(tenant.is_active());
    }

    #[tokio::test]
    async fn test_tenant_manager() {
        let config = TenancyConfig::default();
        let manager = TenantManager::new(config).unwrap();
        
        // Create tenant
        let tenant = manager.create_tenant("test_tenant".to_string(), "Test Tenant".to_string()).await.unwrap();
        assert_eq!(tenant.id, "test_tenant");
        
        // Get tenant
        let retrieved = manager.get_tenant("test_tenant").await.unwrap();
        assert_eq!(retrieved.id, tenant.id);
        
        // List tenants
        let tenants = manager.list_tenants().await.unwrap();
        assert_eq!(tenants.len(), 1);
        
        // Delete tenant
        manager.delete_tenant("test_tenant").await.unwrap();
        
        // Verify deletion
        let result = manager.get_tenant("test_tenant").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tenant_limits() {
        let mut config = TenancyConfig::default();
        config.max_tenants = Some(1);
        
        let manager = TenantManager::new(config).unwrap();
        
        // Create first tenant (should succeed)
        manager.create_tenant("tenant_1".to_string(), "Tenant 1".to_string()).await.unwrap();
        
        // Create second tenant (should fail)
        let result = manager.create_tenant("tenant_2".to_string(), "Tenant 2".to_string()).await;
        assert!(matches!(result, Err(TenantError::TenantLimitExceeded { .. })));
    }

    #[test]
    fn test_tenant_config_serialization() {
        let config = TenantConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: TenantConfig = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_tenant_status_transitions() {
        let mut tenant = Tenant::new("test".to_string(), "Test".to_string());
        assert_eq!(tenant.status, TenantStatus::Active);
        
        tenant.suspend("Testing suspension".to_string());
        assert_eq!(tenant.status, TenantStatus::Suspended);
        assert!(!tenant.is_active());
        
        tenant.activate();
        assert_eq!(tenant.status, TenantStatus::Active);
        assert!(tenant.is_active());
    }
}
