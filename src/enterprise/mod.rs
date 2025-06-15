// Enterprise features for AgentGraph
// Provides multi-tenancy, security, and resource management capabilities

#![allow(missing_docs)]

/// Multi-tenancy support for isolating execution contexts
pub mod tenancy;
/// Resource management and quotas
pub mod resources;
/// Role-based access control and security
pub mod security;
/// Audit logging and compliance
pub mod audit;
/// Monitoring and observability
pub mod monitoring;

pub use tenancy::{Tenant, TenantManager, TenantConfig, TenantContext, TenantError};
pub use resources::{ResourceManager, ResourceQuota, ResourceUsage, ResourceLimits};
pub use security::{SecurityManager, Role, Permission, AuthContext, SecurityError};
pub use audit::{AuditLogger, AuditEvent, AuditLevel, ComplianceReport};
pub use monitoring::{MetricsCollector, PerformanceMetrics, HealthCheck, AlertManager};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Enterprise configuration for AgentGraph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseConfig {
    /// Multi-tenancy configuration
    pub tenancy: tenancy::TenancyConfig,
    /// Resource management configuration
    pub resources: resources::ResourceConfig,
    /// Security configuration
    pub security: security::SecurityConfig,
    /// Audit configuration
    pub audit: audit::AuditConfig,
    /// Monitoring configuration
    pub monitoring: monitoring::MonitoringConfig,
    /// Feature flags
    pub features: FeatureFlags,
}

impl Default for EnterpriseConfig {
    fn default() -> Self {
        Self {
            tenancy: tenancy::TenancyConfig::default(),
            resources: resources::ResourceConfig::default(),
            security: security::SecurityConfig::default(),
            audit: audit::AuditConfig::default(),
            monitoring: monitoring::MonitoringConfig::default(),
            features: FeatureFlags::default(),
        }
    }
}

/// Feature flags for enterprise capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Enable multi-tenancy
    pub multi_tenancy: bool,
    /// Enable resource quotas
    pub resource_quotas: bool,
    /// Enable RBAC
    pub rbac: bool,
    /// Enable audit logging
    pub audit_logging: bool,
    /// Enable monitoring
    pub monitoring: bool,
    /// Enable encryption
    pub encryption: bool,
    /// Enable compliance features
    pub compliance: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            multi_tenancy: true,
            resource_quotas: true,
            rbac: true,
            audit_logging: true,
            monitoring: true,
            encryption: false, // Disabled by default for performance
            compliance: false, // Disabled by default, enable for regulated industries
        }
    }
}

/// Enterprise execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseContext {
    /// Tenant information
    pub tenant: Option<Tenant>,
    /// Authentication context
    pub auth: Option<AuthContext>,
    /// Resource usage tracking
    pub resource_usage: ResourceUsage,
    /// Audit trail
    pub audit_trail: Vec<AuditEvent>,
    /// Request metadata
    pub metadata: HashMap<String, String>,
    /// Execution start time
    pub started_at: SystemTime,
}

impl EnterpriseContext {
    /// Create a new enterprise context
    pub fn new() -> Self {
        Self {
            tenant: None,
            auth: None,
            resource_usage: ResourceUsage::default(),
            audit_trail: Vec::new(),
            metadata: HashMap::new(),
            started_at: SystemTime::now(),
        }
    }
    
    /// Set tenant context
    pub fn with_tenant(mut self, tenant: Tenant) -> Self {
        self.tenant = Some(tenant);
        self
    }
    
    /// Set authentication context
    pub fn with_auth(mut self, auth: AuthContext) -> Self {
        self.auth = Some(auth);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Get tenant ID
    pub fn tenant_id(&self) -> Option<&str> {
        self.tenant.as_ref().map(|t| t.id.as_str())
    }
    
    /// Get user ID
    pub fn user_id(&self) -> Option<&str> {
        self.auth.as_ref().map(|a| a.user_id.as_str())
    }
    
    /// Check if user has permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.auth.as_ref()
            .map(|auth| auth.has_permission(permission))
            .unwrap_or(false)
    }
    
    /// Add audit event
    pub fn add_audit_event(&mut self, event: AuditEvent) {
        self.audit_trail.push(event);
    }
    
    /// Get execution duration
    pub fn execution_duration(&self) -> Duration {
        self.started_at.elapsed().unwrap_or(Duration::ZERO)
    }
}

impl Default for EnterpriseContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Enterprise manager that coordinates all enterprise features
#[derive(Debug)]
pub struct EnterpriseManager {
    /// Configuration
    config: EnterpriseConfig,
    /// Tenant manager
    tenant_manager: TenantManager,
    /// Resource manager
    resource_manager: ResourceManager,
    /// Security manager
    security_manager: SecurityManager,
    /// Audit logger
    audit_logger: AuditLogger,
    /// Metrics collector
    metrics_collector: MetricsCollector,
}

impl EnterpriseManager {
    /// Create a new enterprise manager
    pub fn new(config: EnterpriseConfig) -> Result<Self, EnterpriseError> {
        let tenant_manager = TenantManager::new(config.tenancy.clone())?;
        let resource_manager = ResourceManager::new(config.resources.clone())?;
        let security_manager = SecurityManager::new(config.security.clone())?;
        let audit_logger = AuditLogger::new(config.audit.clone())?;
        let metrics_collector = MetricsCollector::new(config.monitoring.clone())?;
        
        Ok(Self {
            config,
            tenant_manager,
            resource_manager,
            security_manager,
            audit_logger,
            metrics_collector,
        })
    }
    
    /// Create enterprise context for a request
    pub async fn create_context(
        &self,
        tenant_id: Option<String>,
        auth_token: Option<String>,
    ) -> Result<EnterpriseContext, EnterpriseError> {
        let mut context = EnterpriseContext::new();
        
        // Set tenant if provided
        if let Some(tenant_id) = tenant_id {
            let tenant = self.tenant_manager.get_tenant(&tenant_id).await?;
            context = context.with_tenant(tenant);
        }
        
        // Authenticate if token provided
        if let Some(token) = auth_token {
            let auth = self.security_manager.authenticate(&token).await?;
            context = context.with_auth(auth);
        }
        
        Ok(context)
    }
    
    /// Validate context before execution
    pub async fn validate_context(&self, context: &EnterpriseContext) -> Result<(), EnterpriseError> {
        // Check tenant status
        if let Some(tenant) = &context.tenant {
            self.tenant_manager.validate_tenant(tenant).await?;
        }
        
        // Check authentication
        if let Some(auth) = &context.auth {
            self.security_manager.validate_auth(auth).await?;
        }
        
        // Check resource quotas
        self.resource_manager.check_quotas(&context.resource_usage, context.tenant_id()).await?;
        
        Ok(())
    }
    
    /// Record resource usage
    pub async fn record_usage(
        &self,
        context: &mut EnterpriseContext,
        usage: ResourceUsage,
    ) -> Result<(), EnterpriseError> {
        // Update context
        context.resource_usage.add(&usage);
        
        // Record in resource manager
        self.resource_manager.record_usage(context.tenant_id(), &usage).await?;
        
        // Record metrics
        self.metrics_collector.record_resource_usage(&usage).await?;
        
        Ok(())
    }
    
    /// Log audit event
    pub async fn audit_event(
        &self,
        context: &mut EnterpriseContext,
        event: AuditEvent,
    ) -> Result<(), EnterpriseError> {
        // Add to context
        context.add_audit_event(event.clone());
        
        // Log to audit system
        self.audit_logger.log_event(event).await?;
        
        Ok(())
    }
    
    /// Get tenant manager
    pub fn tenant_manager(&self) -> &TenantManager {
        &self.tenant_manager
    }
    
    /// Get resource manager
    pub fn resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }
    
    /// Get security manager
    pub fn security_manager(&self) -> &SecurityManager {
        &self.security_manager
    }
    
    /// Get audit logger
    pub fn audit_logger(&self) -> &AuditLogger {
        &self.audit_logger
    }
    
    /// Get metrics collector
    pub fn metrics_collector(&self) -> &MetricsCollector {
        &self.metrics_collector
    }
    
    /// Get configuration
    pub fn config(&self) -> &EnterpriseConfig {
        &self.config
    }
}

/// Errors that can occur in enterprise operations
#[derive(Debug, thiserror::Error, Clone, Serialize, Deserialize)]
pub enum EnterpriseError {
    /// Tenant-related error
    #[error("Tenant error: {0}")]
    Tenant(#[from] TenantError),
    
    /// Security-related error
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    
    /// Resource-related error
    #[error("Resource error: {0}")]
    Resource(#[from] resources::ResourceError),

    /// Audit-related error
    #[error("Audit error: {0}")]
    Audit(#[from] audit::AuditError),

    /// Monitoring-related error
    #[error("Monitoring error: {0}")]
    Monitoring(#[from] monitoring::MonitoringError),
    
    /// Configuration error
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    /// Permission denied
    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },
    
    /// System error
    #[error("System error: {message}")]
    System { message: String },
}

/// Result type for enterprise operations
pub type EnterpriseResult<T> = Result<T, EnterpriseError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enterprise_config_default() {
        let config = EnterpriseConfig::default();
        assert!(config.features.multi_tenancy);
        assert!(config.features.resource_quotas);
        assert!(config.features.rbac);
        assert!(config.features.audit_logging);
        assert!(config.features.monitoring);
        assert!(!config.features.encryption); // Disabled by default
        assert!(!config.features.compliance); // Disabled by default
    }

    #[test]
    fn test_enterprise_context_creation() {
        let context = EnterpriseContext::new()
            .with_metadata("request_id".to_string(), "req_123".to_string())
            .with_metadata("source".to_string(), "api".to_string());
        
        assert_eq!(context.metadata.get("request_id"), Some(&"req_123".to_string()));
        assert_eq!(context.metadata.get("source"), Some(&"api".to_string()));
        assert!(context.tenant.is_none());
        assert!(context.auth.is_none());
    }

    #[test]
    fn test_feature_flags_serialization() {
        let flags = FeatureFlags::default();
        let serialized = serde_json::to_string(&flags).unwrap();
        let deserialized: FeatureFlags = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(flags.multi_tenancy, deserialized.multi_tenancy);
        assert_eq!(flags.resource_quotas, deserialized.resource_quotas);
        assert_eq!(flags.rbac, deserialized.rbac);
    }

    #[test]
    fn test_enterprise_error_serialization() {
        let error = EnterpriseError::PermissionDenied {
            message: "Access denied".to_string(),
        };
        
        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: EnterpriseError = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            EnterpriseError::PermissionDenied { message } => {
                assert_eq!(message, "Access denied");
            }
            _ => panic!("Wrong error type"),
        }
    }
}
