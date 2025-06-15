// Security and RBAC system for AgentGraph
// Provides authentication, authorization, and access control

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use thiserror::Error;

/// Authentication context for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    /// User ID
    pub user_id: String,
    /// User roles
    pub roles: Vec<Role>,
    /// Session ID
    pub session_id: String,
    /// Authentication timestamp
    pub authenticated_at: SystemTime,
    /// Token expiration time
    pub expires_at: Option<SystemTime>,
    /// Additional claims
    pub claims: HashMap<String, serde_json::Value>,
}

impl AuthContext {
    /// Create a new authentication context
    pub fn new(user_id: String, roles: Vec<Role>, session_id: String) -> Self {
        Self {
            user_id,
            roles,
            session_id,
            authenticated_at: SystemTime::now(),
            expires_at: None,
            claims: HashMap::new(),
        }
    }
    
    /// Set expiration time
    pub fn with_expiration(mut self, duration: Duration) -> Self {
        self.expires_at = Some(self.authenticated_at + duration);
        self
    }
    
    /// Add claim
    pub fn with_claim<T: Serialize>(mut self, key: String, value: T) -> Self {
        self.claims.insert(
            key,
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
        self
    }
    
    /// Check if authentication has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            SystemTime::now() > expires_at
        } else {
            false
        }
    }
    
    /// Check if user has a specific role
    pub fn has_role(&self, role_name: &str) -> bool {
        self.roles.iter().any(|r| r.name == role_name)
    }
    
    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.roles.iter().any(|role| role.has_permission(permission))
    }
    
    /// Get all permissions for this user
    pub fn get_all_permissions(&self) -> HashSet<Permission> {
        let mut permissions = HashSet::new();
        for role in &self.roles {
            permissions.extend(role.permissions.iter().cloned());
        }
        permissions
    }
}

/// User role with permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Role {
    /// Role name
    pub name: String,
    /// Role description
    pub description: String,
    /// Permissions granted by this role
    pub permissions: Vec<Permission>,
    /// Role metadata
    pub metadata: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: SystemTime,
}

impl Role {
    /// Create a new role
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            permissions: Vec::new(),
            metadata: HashMap::new(),
            created_at: SystemTime::now(),
        }
    }
    
    /// Add permission to role
    pub fn with_permission(mut self, permission: Permission) -> Self {
        if !self.permissions.contains(&permission) {
            self.permissions.push(permission);
        }
        self
    }
    
    /// Add multiple permissions to role
    pub fn with_permissions(mut self, permissions: Vec<Permission>) -> Self {
        for permission in permissions {
            if !self.permissions.contains(&permission) {
                self.permissions.push(permission);
            }
        }
        self
    }
    
    /// Check if role has permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.iter().any(|p| p.matches(permission))
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Permission for specific operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// Resource type (e.g., "graph", "node", "tool")
    pub resource: String,
    /// Action (e.g., "read", "write", "execute", "delete")
    pub action: String,
    /// Optional scope/context
    pub scope: Option<String>,
}

impl Permission {
    /// Create a new permission
    pub fn new(resource: String, action: String) -> Self {
        Self {
            resource,
            action,
            scope: None,
        }
    }
    
    /// Create permission with scope
    pub fn with_scope(mut self, scope: String) -> Self {
        self.scope = Some(scope);
        self
    }
    
    /// Check if this permission matches another (considering wildcards)
    pub fn matches(&self, other: &Permission) -> bool {
        let resource_match = self.resource == "*" || self.resource == other.resource;
        let action_match = self.action == "*" || self.action == other.action;
        let scope_match = match (&self.scope, &other.scope) {
            (None, _) => true, // No scope restriction
            (Some(s1), Some(s2)) => s1 == "*" || s1 == s2,
            (Some(_), None) => false, // Scope required but not provided
        };

        resource_match && action_match && scope_match
    }
}

/// Predefined permissions
impl Permission {
    /// Graph execution permission
    pub fn graph_execute() -> Self {
        Self::new("graph".to_string(), "execute".to_string())
    }
    
    /// Graph read permission
    pub fn graph_read() -> Self {
        Self::new("graph".to_string(), "read".to_string())
    }
    
    /// Graph write permission
    pub fn graph_write() -> Self {
        Self::new("graph".to_string(), "write".to_string())
    }
    
    /// Tool execute permission
    pub fn tool_execute() -> Self {
        Self::new("tool".to_string(), "execute".to_string())
    }
    
    /// Admin permission (all resources, all actions)
    pub fn admin() -> Self {
        Self::new("*".to_string(), "*".to_string())
    }
    
    /// Tenant admin permission
    pub fn tenant_admin(tenant_id: String) -> Self {
        Self::new("*".to_string(), "*".to_string()).with_scope(tenant_id)
    }
}

/// Predefined roles
impl Role {
    /// Administrator role with all permissions
    pub fn admin() -> Self {
        Self::new("admin".to_string(), "System administrator".to_string())
            .with_permission(Permission::admin())
    }
    
    /// User role with basic permissions
    pub fn user() -> Self {
        Self::new("user".to_string(), "Regular user".to_string())
            .with_permissions(vec![
                Permission::graph_read(),
                Permission::graph_execute(),
                Permission::tool_execute(),
            ])
    }
    
    /// Read-only role
    pub fn readonly() -> Self {
        Self::new("readonly".to_string(), "Read-only access".to_string())
            .with_permission(Permission::graph_read())
    }
    
    /// Tenant administrator role
    pub fn tenant_admin(tenant_id: String) -> Self {
        Self::new(
            format!("tenant_admin_{}", tenant_id),
            format!("Administrator for tenant {}", tenant_id),
        )
        .with_permission(Permission::tenant_admin(tenant_id))
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable authentication
    pub authentication_enabled: bool,
    /// Enable authorization
    pub authorization_enabled: bool,
    /// Default session timeout
    pub session_timeout: Duration,
    /// JWT secret key (for JWT authentication)
    pub jwt_secret: Option<String>,
    /// Allowed authentication methods
    pub auth_methods: Vec<AuthMethod>,
    /// Password policy
    pub password_policy: PasswordPolicy,
    /// Rate limiting configuration
    pub rate_limiting: RateLimitConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            authentication_enabled: true,
            authorization_enabled: true,
            session_timeout: Duration::from_secs(3600), // 1 hour
            jwt_secret: None,
            auth_methods: vec![AuthMethod::ApiKey, AuthMethod::JWT],
            password_policy: PasswordPolicy::default(),
            rate_limiting: RateLimitConfig::default(),
        }
    }
}

/// Authentication methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMethod {
    /// API key authentication
    ApiKey,
    /// JWT token authentication
    JWT,
    /// OAuth2 authentication
    OAuth2,
    /// Basic authentication
    Basic,
}

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    /// Minimum password length
    pub min_length: u32,
    /// Require uppercase letters
    pub require_uppercase: bool,
    /// Require lowercase letters
    pub require_lowercase: bool,
    /// Require numbers
    pub require_numbers: bool,
    /// Require special characters
    pub require_special_chars: bool,
    /// Password expiration days
    pub expiration_days: Option<u32>,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: false,
            expiration_days: Some(90),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Requests per minute per user
    pub requests_per_minute: u32,
    /// Requests per hour per user
    pub requests_per_hour: u32,
    /// Burst allowance
    pub burst_allowance: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 60,
            requests_per_hour: 1000,
            burst_allowance: 10,
        }
    }
}

/// Security manager for handling authentication and authorization
#[derive(Debug)]
pub struct SecurityManager {
    /// Configuration
    config: SecurityConfig,
    /// User roles
    user_roles: Arc<RwLock<HashMap<String, Vec<Role>>>>,
    /// Active sessions
    sessions: Arc<RwLock<HashMap<String, AuthContext>>>,
    /// API keys
    api_keys: Arc<RwLock<HashMap<String, String>>>, // key -> user_id
    /// Rate limiting state
    rate_limits: Arc<RwLock<HashMap<String, RateLimitState>>>,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new(config: SecurityConfig) -> Result<Self, SecurityError> {
        Ok(Self {
            config,
            user_roles: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            api_keys: Arc::new(RwLock::new(HashMap::new())),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Authenticate a user with token
    pub async fn authenticate(&self, token: &str) -> Result<AuthContext, SecurityError> {
        if !self.config.authentication_enabled {
            // Create default context when authentication is disabled
            return Ok(AuthContext::new(
                "anonymous".to_string(),
                vec![Role::user()],
                "anonymous_session".to_string(),
            ));
        }
        
        // Try different authentication methods
        for method in &self.config.auth_methods {
            match method {
                AuthMethod::ApiKey => {
                    if let Ok(auth) = self.authenticate_api_key(token).await {
                        return Ok(auth);
                    }
                }
                AuthMethod::JWT => {
                    if let Ok(auth) = self.authenticate_jwt(token).await {
                        return Ok(auth);
                    }
                }
                _ => {} // Other methods not implemented yet
            }
        }
        
        Err(SecurityError::AuthenticationFailed {
            reason: "Invalid token".to_string(),
        })
    }
    
    /// Authenticate with API key
    async fn authenticate_api_key(&self, api_key: &str) -> Result<AuthContext, SecurityError> {
        let api_keys = self.api_keys.read().unwrap();
        
        if let Some(user_id) = api_keys.get(api_key) {
            let roles = self.get_user_roles(user_id).await?;
            let session_id = format!("api_key_{}", uuid::Uuid::new_v4());
            
            Ok(AuthContext::new(user_id.clone(), roles, session_id)
                .with_expiration(self.config.session_timeout))
        } else {
            Err(SecurityError::AuthenticationFailed {
                reason: "Invalid API key".to_string(),
            })
        }
    }
    
    /// Authenticate with JWT token
    async fn authenticate_jwt(&self, _token: &str) -> Result<AuthContext, SecurityError> {
        // JWT authentication would be implemented here
        // For now, return an error
        Err(SecurityError::AuthenticationFailed {
            reason: "JWT authentication not implemented".to_string(),
        })
    }
    
    /// Validate authentication context
    pub async fn validate_auth(&self, auth: &AuthContext) -> Result<(), SecurityError> {
        if auth.is_expired() {
            return Err(SecurityError::SessionExpired {
                session_id: auth.session_id.clone(),
            });
        }
        
        Ok(())
    }
    
    /// Check authorization for a permission
    pub async fn authorize(
        &self,
        auth: &AuthContext,
        permission: &Permission,
    ) -> Result<(), SecurityError> {
        if !self.config.authorization_enabled {
            return Ok(());
        }
        
        if auth.has_permission(permission) {
            Ok(())
        } else {
            Err(SecurityError::PermissionDenied {
                user_id: auth.user_id.clone(),
                permission: permission.clone(),
            })
        }
    }
    
    /// Add user roles
    pub async fn add_user_roles(&self, user_id: String, roles: Vec<Role>) -> Result<(), SecurityError> {
        let mut user_roles = self.user_roles.write().unwrap();
        user_roles.insert(user_id, roles);
        Ok(())
    }
    
    /// Get user roles
    pub async fn get_user_roles(&self, user_id: &str) -> Result<Vec<Role>, SecurityError> {
        let user_roles = self.user_roles.read().unwrap();
        Ok(user_roles.get(user_id).cloned().unwrap_or_else(|| vec![Role::user()]))
    }
    
    /// Add API key
    pub async fn add_api_key(&self, api_key: String, user_id: String) -> Result<(), SecurityError> {
        let mut api_keys = self.api_keys.write().unwrap();
        api_keys.insert(api_key, user_id);
        Ok(())
    }
    
    /// Check rate limits
    pub async fn check_rate_limit(&self, user_id: &str) -> Result<(), SecurityError> {
        if !self.config.rate_limiting.enabled {
            return Ok(());
        }
        
        let mut rate_limits = self.rate_limits.write().unwrap();
        let now = SystemTime::now();
        
        let state = rate_limits.entry(user_id.to_string())
            .or_insert_with(|| RateLimitState::new(now));
        
        state.check_and_update(now, &self.config.rate_limiting)
    }
    
    /// Get configuration
    pub fn config(&self) -> &SecurityConfig {
        &self.config
    }
}

/// Rate limiting state for a user
#[derive(Debug, Clone)]
struct RateLimitState {
    /// Requests in current minute
    minute_requests: u32,
    /// Requests in current hour
    hour_requests: u32,
    /// Last minute timestamp
    last_minute: SystemTime,
    /// Last hour timestamp
    last_hour: SystemTime,
    /// Burst tokens available
    burst_tokens: u32,
}

impl RateLimitState {
    fn new(now: SystemTime) -> Self {
        Self {
            minute_requests: 0,
            hour_requests: 0,
            last_minute: now,
            last_hour: now,
            burst_tokens: 0,
        }
    }
    
    fn check_and_update(&mut self, now: SystemTime, config: &RateLimitConfig) -> Result<(), SecurityError> {
        // Reset counters if time periods have passed
        if now.duration_since(self.last_minute).unwrap_or(Duration::ZERO) >= Duration::from_secs(60) {
            self.minute_requests = 0;
            self.last_minute = now;
            self.burst_tokens = config.burst_allowance;
        }
        
        if now.duration_since(self.last_hour).unwrap_or(Duration::ZERO) >= Duration::from_secs(3600) {
            self.hour_requests = 0;
            self.last_hour = now;
        }
        
        // Check limits
        if self.minute_requests >= config.requests_per_minute && self.burst_tokens == 0 {
            return Err(SecurityError::RateLimitExceeded {
                limit_type: "per_minute".to_string(),
                limit: config.requests_per_minute,
                current: self.minute_requests,
            });
        }
        
        if self.hour_requests >= config.requests_per_hour {
            return Err(SecurityError::RateLimitExceeded {
                limit_type: "per_hour".to_string(),
                limit: config.requests_per_hour,
                current: self.hour_requests,
            });
        }
        
        // Update counters
        if self.burst_tokens > 0 {
            self.burst_tokens -= 1;
        } else {
            self.minute_requests += 1;
        }
        self.hour_requests += 1;
        
        Ok(())
    }
}

/// Errors that can occur in security operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum SecurityError {
    /// Authentication failed
    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },
    
    /// Session expired
    #[error("Session expired: {session_id}")]
    SessionExpired { session_id: String },
    
    /// Permission denied
    #[error("Permission denied for user {user_id}: {permission:?}")]
    PermissionDenied { 
        user_id: String,
        permission: Permission,
    },
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded ({limit_type}): {current}/{limit}")]
    RateLimitExceeded {
        limit_type: String,
        limit: u32,
        current: u32,
    },
    
    /// Configuration error
    #[error("Security configuration error: {message}")]
    ConfigurationError { message: String },
    
    /// System error
    #[error("Security system error: {message}")]
    SystemError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_matching() {
        let admin_perm = Permission::admin();
        let graph_exec = Permission::graph_execute();
        
        assert!(admin_perm.matches(&graph_exec));
        assert!(!graph_exec.matches(&admin_perm));
    }

    #[test]
    fn test_role_permissions() {
        let role = Role::user();
        
        assert!(role.has_permission(&Permission::graph_read()));
        assert!(role.has_permission(&Permission::graph_execute()));
        assert!(!role.has_permission(&Permission::graph_write()));
    }

    #[test]
    fn test_auth_context() {
        let roles = vec![Role::user()];
        let auth = AuthContext::new("user1".to_string(), roles, "session1".to_string())
            .with_expiration(Duration::from_secs(3600));
        
        assert!(auth.has_permission(&Permission::graph_read()));
        assert!(!auth.is_expired());
    }

    #[tokio::test]
    async fn test_security_manager() {
        let config = SecurityConfig::default();
        let manager = SecurityManager::new(config).unwrap();
        
        // Add API key
        manager.add_api_key("test_key".to_string(), "user1".to_string()).await.unwrap();
        
        // Add user roles
        let roles = vec![Role::admin()];
        manager.add_user_roles("user1".to_string(), roles).await.unwrap();
        
        // Authenticate
        let auth = manager.authenticate("test_key").await.unwrap();
        assert_eq!(auth.user_id, "user1");
        assert!(auth.has_permission(&Permission::admin()));
    }
}
