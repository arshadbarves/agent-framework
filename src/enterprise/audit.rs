// Audit logging and compliance system for AgentGraph
// Provides comprehensive audit trails and compliance reporting

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use thiserror::Error;

/// Audit event for tracking system activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub event_id: String,
    /// Event timestamp
    pub timestamp: SystemTime,
    /// Event type
    pub event_type: AuditEventType,
    /// Event level/severity
    pub level: AuditLevel,
    /// User who triggered the event
    pub user_id: Option<String>,
    /// Tenant context
    pub tenant_id: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Resource affected
    pub resource: Option<String>,
    /// Action performed
    pub action: String,
    /// Event description
    pub description: String,
    /// Additional event data
    pub data: HashMap<String, serde_json::Value>,
    /// Source IP address
    pub source_ip: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Request ID for correlation
    pub request_id: Option<String>,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(event_type: AuditEventType, action: String, description: String) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            event_type,
            level: AuditLevel::Info,
            user_id: None,
            tenant_id: None,
            session_id: None,
            resource: None,
            action,
            description,
            data: HashMap::new(),
            source_ip: None,
            user_agent: None,
            request_id: None,
        }
    }
    
    /// Set event level
    pub fn with_level(mut self, level: AuditLevel) -> Self {
        self.level = level;
        self
    }
    
    /// Set user context
    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    /// Set tenant context
    pub fn with_tenant(mut self, tenant_id: String) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }
    
    /// Set session context
    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
    
    /// Set resource
    pub fn with_resource(mut self, resource: String) -> Self {
        self.resource = Some(resource);
        self
    }
    
    /// Add data field
    pub fn with_data<T: Serialize>(mut self, key: String, value: T) -> Self {
        self.data.insert(
            key,
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
        self
    }
    
    /// Set network context
    pub fn with_network(mut self, source_ip: String, user_agent: Option<String>) -> Self {
        self.source_ip = Some(source_ip);
        self.user_agent = user_agent;
        self
    }
    
    /// Set request ID for correlation
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

/// Types of audit events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditEventType {
    /// Authentication events
    Authentication,
    /// Authorization events
    Authorization,
    /// Graph execution events
    GraphExecution,
    /// Tool execution events
    ToolExecution,
    /// Human interaction events
    HumanInteraction,
    /// Resource management events
    ResourceManagement,
    /// Configuration changes
    Configuration,
    /// System events
    System,
    /// Security events
    Security,
    /// Compliance events
    Compliance,
}

/// Audit event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AuditLevel {
    /// Debug information
    Debug,
    /// General information
    Info,
    /// Warning events
    Warning,
    /// Error events
    Error,
    /// Critical security events
    Critical,
}

/// Audit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging
    pub enabled: bool,
    /// Minimum audit level to log
    pub min_level: AuditLevel,
    /// Event types to audit
    pub event_types: Vec<AuditEventType>,
    /// Storage backend configuration
    pub storage: AuditStorageConfig,
    /// Retention policy
    pub retention: RetentionPolicy,
    /// Enable real-time alerts
    pub alerts_enabled: bool,
    /// Compliance standards to follow
    pub compliance_standards: Vec<ComplianceStandard>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_level: AuditLevel::Info,
            event_types: vec![
                AuditEventType::Authentication,
                AuditEventType::Authorization,
                AuditEventType::GraphExecution,
                AuditEventType::Security,
            ],
            storage: AuditStorageConfig::default(),
            retention: RetentionPolicy::default(),
            alerts_enabled: true,
            compliance_standards: vec![ComplianceStandard::SOC2],
        }
    }
}

/// Audit storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStorageConfig {
    /// Storage backend type
    pub backend: AuditStorageBackend,
    /// Storage location/connection
    pub location: String,
    /// Enable encryption
    pub encryption_enabled: bool,
    /// Compression settings
    pub compression_enabled: bool,
    /// Batch size for bulk operations
    pub batch_size: u32,
    /// Flush interval
    pub flush_interval_seconds: u32,
}

impl Default for AuditStorageConfig {
    fn default() -> Self {
        Self {
            backend: AuditStorageBackend::File,
            location: "./audit.log".to_string(),
            encryption_enabled: false,
            compression_enabled: true,
            batch_size: 100,
            flush_interval_seconds: 60,
        }
    }
}

/// Audit storage backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditStorageBackend {
    /// File-based storage
    File,
    /// Database storage
    Database,
    /// Syslog
    Syslog,
    /// Cloud storage (S3, etc.)
    Cloud,
    /// Memory (for testing)
    Memory,
}

/// Retention policy for audit logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Retention period in days
    pub retention_days: u32,
    /// Archive old logs instead of deleting
    pub archive_enabled: bool,
    /// Archive location
    pub archive_location: Option<String>,
    /// Automatic cleanup enabled
    pub auto_cleanup: bool,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            retention_days: 365, // 1 year
            archive_enabled: true,
            archive_location: Some("./audit_archive/".to_string()),
            auto_cleanup: true,
        }
    }
}

/// Compliance standards
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStandard {
    /// SOC 2 compliance
    SOC2,
    /// GDPR compliance
    GDPR,
    /// HIPAA compliance
    HIPAA,
    /// PCI DSS compliance
    PCIDSS,
    /// ISO 27001 compliance
    ISO27001,
}

/// Audit logger for recording events
#[derive(Debug)]
pub struct AuditLogger {
    /// Configuration
    config: AuditConfig,
    /// Event buffer for batching
    event_buffer: Arc<Mutex<Vec<AuditEvent>>>,
    /// Statistics
    stats: Arc<Mutex<AuditStats>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(config: AuditConfig) -> Result<Self, AuditError> {
        Ok(Self {
            config,
            event_buffer: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(AuditStats::default())),
        })
    }
    
    /// Log an audit event
    pub async fn log_event(&self, event: AuditEvent) -> Result<(), AuditError> {
        if !self.config.enabled {
            return Ok(());
        }
        
        // Check if event should be logged
        if !self.should_log_event(&event) {
            return Ok(());
        }
        
        // Add to buffer
        {
            let mut buffer = self.event_buffer.lock().unwrap();
            buffer.push(event.clone());
            
            // Flush if buffer is full
            if buffer.len() >= self.config.storage.batch_size as usize {
                let events = buffer.drain(..).collect();
                drop(buffer); // Release lock before async operation
                self.flush_events(events).await?;
            }
        }
        
        // Update statistics
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_events += 1;
            stats.events_by_type.entry(event.event_type)
                .and_modify(|count| *count += 1)
                .or_insert(1);
            stats.events_by_level.entry(event.level)
                .and_modify(|count| *count += 1)
                .or_insert(1);
            stats.last_event_time = Some(event.timestamp);
        }
        
        Ok(())
    }
    
    /// Check if event should be logged based on configuration
    fn should_log_event(&self, event: &AuditEvent) -> bool {
        // Check minimum level
        if event.level < self.config.min_level {
            return false;
        }
        
        // Check event type filter
        if !self.config.event_types.is_empty() && 
           !self.config.event_types.contains(&event.event_type) {
            return false;
        }
        
        true
    }
    
    /// Flush events to storage
    async fn flush_events(&self, events: Vec<AuditEvent>) -> Result<(), AuditError> {
        match self.config.storage.backend {
            AuditStorageBackend::File => self.flush_to_file(events).await,
            AuditStorageBackend::Memory => Ok(()), // No-op for memory backend
            _ => Err(AuditError::StorageError {
                message: "Storage backend not implemented".to_string(),
            }),
        }
    }
    
    /// Flush events to file
    async fn flush_to_file(&self, events: Vec<AuditEvent>) -> Result<(), AuditError> {
        use std::io::Write;
        
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.storage.location)
            .map_err(|e| AuditError::StorageError {
                message: format!("Failed to open audit log file: {}", e),
            })?;
        
        for event in events {
            let json_line = serde_json::to_string(&event)
                .map_err(|e| AuditError::SerializationError {
                    message: format!("Failed to serialize audit event: {}", e),
                })?;
            
            writeln!(file, "{}", json_line)
                .map_err(|e| AuditError::StorageError {
                    message: format!("Failed to write audit event: {}", e),
                })?;
        }
        
        file.flush()
            .map_err(|e| AuditError::StorageError {
                message: format!("Failed to flush audit log: {}", e),
            })?;
        
        Ok(())
    }
    
    /// Force flush all buffered events
    pub async fn flush(&self) -> Result<(), AuditError> {
        let events: Vec<AuditEvent> = {
            let mut buffer = self.event_buffer.lock().unwrap();
            buffer.drain(..).collect()
        };
        
        if !events.is_empty() {
            self.flush_events(events).await?;
        }
        
        Ok(())
    }
    
    /// Get audit statistics
    pub fn get_stats(&self) -> AuditStats {
        self.stats.lock().unwrap().clone()
    }
    
    /// Generate compliance report
    pub async fn generate_compliance_report(
        &self,
        standard: ComplianceStandard,
        start_time: SystemTime,
        end_time: SystemTime,
    ) -> Result<ComplianceReport, AuditError> {
        // This would read from storage and generate a compliance report
        // For now, return a basic report
        Ok(ComplianceReport {
            standard,
            period_start: start_time,
            period_end: end_time,
            total_events: 0,
            compliance_score: 100.0,
            findings: Vec::new(),
            recommendations: Vec::new(),
            generated_at: SystemTime::now(),
        })
    }
    
    /// Get configuration
    pub fn config(&self) -> &AuditConfig {
        &self.config
    }
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    /// Total number of events logged
    pub total_events: u64,
    /// Events by type
    pub events_by_type: HashMap<AuditEventType, u64>,
    /// Events by level
    pub events_by_level: HashMap<AuditLevel, u64>,
    /// Last event timestamp
    pub last_event_time: Option<SystemTime>,
    /// Storage size in bytes
    pub storage_size_bytes: u64,
}

impl Default for AuditStats {
    fn default() -> Self {
        Self {
            total_events: 0,
            events_by_type: HashMap::new(),
            events_by_level: HashMap::new(),
            last_event_time: None,
            storage_size_bytes: 0,
        }
    }
}

/// Compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    /// Compliance standard
    pub standard: ComplianceStandard,
    /// Report period start
    pub period_start: SystemTime,
    /// Report period end
    pub period_end: SystemTime,
    /// Total events in period
    pub total_events: u64,
    /// Compliance score (0-100)
    pub compliance_score: f64,
    /// Compliance findings
    pub findings: Vec<ComplianceFinding>,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Report generation timestamp
    pub generated_at: SystemTime,
}

/// Compliance finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFinding {
    /// Finding severity
    pub severity: FindingSeverity,
    /// Finding category
    pub category: String,
    /// Finding description
    pub description: String,
    /// Affected events count
    pub affected_events: u64,
    /// Remediation steps
    pub remediation: Vec<String>,
}

/// Finding severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Errors that can occur in audit operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum AuditError {
    /// Storage error
    #[error("Audit storage error: {message}")]
    StorageError { message: String },
    
    /// Serialization error
    #[error("Audit serialization error: {message}")]
    SerializationError { message: String },
    
    /// Configuration error
    #[error("Audit configuration error: {message}")]
    ConfigurationError { message: String },
    
    /// Compliance error
    #[error("Compliance error: {message}")]
    ComplianceError { message: String },
    
    /// System error
    #[error("Audit system error: {message}")]
    SystemError { message: String },
}

/// Predefined audit events
impl AuditEvent {
    /// User login event
    pub fn user_login(user_id: String, success: bool) -> Self {
        Self::new(
            AuditEventType::Authentication,
            "login".to_string(),
            format!("User {} login {}", user_id, if success { "successful" } else { "failed" }),
        )
        .with_user(user_id)
        .with_level(if success { AuditLevel::Info } else { AuditLevel::Warning })
    }
    
    /// Graph execution event
    pub fn graph_execution(user_id: String, graph_id: String, success: bool) -> Self {
        Self::new(
            AuditEventType::GraphExecution,
            "execute".to_string(),
            format!("Graph {} execution {}", graph_id, if success { "completed" } else { "failed" }),
        )
        .with_user(user_id)
        .with_resource(graph_id)
        .with_level(if success { AuditLevel::Info } else { AuditLevel::Error })
    }
    
    /// Permission denied event
    pub fn permission_denied(user_id: String, resource: String, action: String) -> Self {
        Self::new(
            AuditEventType::Authorization,
            "access_denied".to_string(),
            format!("Access denied to {} for action {}", resource, action),
        )
        .with_user(user_id)
        .with_resource(resource)
        .with_level(AuditLevel::Warning)
    }
    
    /// Security event
    pub fn security_event(event_type: String, description: String) -> Self {
        Self::new(
            AuditEventType::Security,
            event_type,
            description,
        )
        .with_level(AuditLevel::Critical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::user_login("user123".to_string(), true)
            .with_session("session456".to_string())
            .with_network("192.168.1.1".to_string(), Some("Mozilla/5.0".to_string()));
        
        assert_eq!(event.event_type, AuditEventType::Authentication);
        assert_eq!(event.action, "login");
        assert_eq!(event.user_id, Some("user123".to_string()));
        assert_eq!(event.session_id, Some("session456".to_string()));
        assert_eq!(event.source_ip, Some("192.168.1.1".to_string()));
        assert_eq!(event.level, AuditLevel::Info);
    }

    #[tokio::test]
    async fn test_audit_logger() {
        let config = AuditConfig {
            storage: AuditStorageConfig {
                backend: AuditStorageBackend::Memory,
                ..Default::default()
            },
            ..Default::default()
        };
        
        let logger = AuditLogger::new(config).unwrap();
        
        let event = AuditEvent::user_login("test_user".to_string(), true);
        logger.log_event(event).await.unwrap();
        
        let stats = logger.get_stats();
        assert_eq!(stats.total_events, 1);
        assert_eq!(stats.events_by_type.get(&AuditEventType::Authentication), Some(&1));
    }

    #[test]
    fn test_audit_config_serialization() {
        let config = AuditConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: AuditConfig = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.min_level, deserialized.min_level);
    }

    #[test]
    fn test_compliance_standards() {
        let standards = vec![
            ComplianceStandard::SOC2,
            ComplianceStandard::GDPR,
            ComplianceStandard::HIPAA,
        ];
        
        for standard in standards {
            let serialized = serde_json::to_string(&standard).unwrap();
            let deserialized: ComplianceStandard = serde_json::from_str(&serialized).unwrap();
            assert_eq!(standard, deserialized);
        }
    }
}
