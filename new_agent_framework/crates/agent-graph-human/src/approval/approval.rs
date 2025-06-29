//! Human approval system for sensitive operations.

use crate::{CoreError, CoreResult};
use crate::input::{HumanInputCollector, InputRequest, InputResponse, InputType, InputPriority};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Trait for approval systems
#[async_trait]
pub trait ApprovalSystem: Send + Sync + std::fmt::Debug {
    /// Request approval for an operation
    async fn request_approval(&self, request: ApprovalRequest) -> CoreResult<ApprovalResponse>;

    /// Check if the approval system is available
    async fn is_available(&self) -> bool;

    /// Get approval system metadata
    fn metadata(&self) -> &ApprovalSystemMetadata;
}

/// Metadata for approval systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalSystemMetadata {
    /// System ID
    pub id: String,
    /// System name
    pub name: String,
    /// System type
    pub system_type: ApprovalSystemType,
    /// Supported approval types
    pub supported_types: Vec<ApprovalType>,
    /// Whether multi-level approval is supported
    pub multi_level: bool,
    /// Default timeout for approvals
    pub default_timeout: Option<Duration>,
}

/// Type of approval system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApprovalSystemType {
    /// Single approver system
    Single,
    /// Multi-approver system
    Multi,
    /// Hierarchical approval system
    Hierarchical,
    /// Custom approval system
    Custom(String),
}

/// Type of approval being requested
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApprovalType {
    /// General operation approval
    Operation,
    /// Data access approval
    DataAccess,
    /// External API call approval
    ExternalAPI,
    /// File system operation approval
    FileSystem,
    /// Administrative action approval
    Administrative,
    /// Custom approval type
    Custom(String),
}

/// Request for approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Unique request ID
    pub id: String,
    /// Type of approval requested
    pub approval_type: ApprovalType,
    /// Operation being requested
    pub operation: String,
    /// Detailed description of the operation
    pub description: String,
    /// Risk level of the operation
    pub risk_level: RiskLevel,
    /// Requester information
    pub requester: RequesterInfo,
    /// Additional context
    pub context: HashMap<String, serde_json::Value>,
    /// Timeout for the approval
    pub timeout: Option<Duration>,
    /// Whether the operation is reversible
    pub reversible: bool,
    /// Estimated impact of the operation
    pub impact: OperationImpact,
}

/// Risk level of an operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    /// Low risk - minimal impact
    Low,
    /// Medium risk - moderate impact
    Medium,
    /// High risk - significant impact
    High,
    /// Critical risk - severe impact
    Critical,
}

/// Information about the requester
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequesterInfo {
    /// Requester ID
    pub id: String,
    /// Requester name
    pub name: Option<String>,
    /// Requester role
    pub role: Option<String>,
    /// Request timestamp
    pub timestamp: SystemTime,
}

/// Estimated impact of an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationImpact {
    /// Affected systems
    pub affected_systems: Vec<String>,
    /// Estimated duration
    pub estimated_duration: Option<Duration>,
    /// Data sensitivity level
    pub data_sensitivity: DataSensitivity,
    /// Compliance requirements
    pub compliance_requirements: Vec<String>,
}

/// Data sensitivity level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataSensitivity {
    /// Public data
    Public,
    /// Internal data
    Internal,
    /// Confidential data
    Confidential,
    /// Restricted data
    Restricted,
}

/// Response to an approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResponse {
    /// Request ID this response is for
    pub request_id: String,
    /// Whether the request was approved
    pub approved: bool,
    /// Reason for the decision
    pub reason: Option<String>,
    /// Approver information
    pub approver: ApproverInfo,
    /// Response timestamp
    pub timestamp: SystemTime,
    /// Conditions or restrictions
    pub conditions: Vec<String>,
    /// Approval expiration time
    pub expires_at: Option<SystemTime>,
}

/// Information about the approver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproverInfo {
    /// Approver ID
    pub id: String,
    /// Approver name
    pub name: Option<String>,
    /// Approver role
    pub role: Option<String>,
    /// Approval authority level
    pub authority_level: AuthorityLevel,
}

/// Authority level of an approver
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuthorityLevel {
    /// Basic approval authority
    Basic,
    /// Supervisor approval authority
    Supervisor,
    /// Manager approval authority
    Manager,
    /// Executive approval authority
    Executive,
    /// Administrative approval authority
    Administrative,
}

/// Simple approval system using human input
#[derive(Debug)]
pub struct SimpleApprovalSystem {
    metadata: ApprovalSystemMetadata,
    input_collector: Box<dyn HumanInputCollector>,
}

impl SimpleApprovalSystem {
    /// Create a new simple approval system
    pub fn new(input_collector: Box<dyn HumanInputCollector>) -> Self {
        let metadata = ApprovalSystemMetadata {
            id: "simple".to_string(),
            name: "Simple Approval System".to_string(),
            system_type: ApprovalSystemType::Single,
            supported_types: vec![
                ApprovalType::Operation,
                ApprovalType::DataAccess,
                ApprovalType::ExternalAPI,
                ApprovalType::FileSystem,
                ApprovalType::Administrative,
            ],
            multi_level: false,
            default_timeout: Some(Duration::from_secs(300)), // 5 minutes
        };

        Self {
            metadata,
            input_collector,
        }
    }
}

#[async_trait]
impl ApprovalSystem for SimpleApprovalSystem {
    async fn request_approval(&self, request: ApprovalRequest) -> CoreResult<ApprovalResponse> {
        // Create a human input request for approval
        let prompt = format!(
            "APPROVAL REQUIRED\n\
            Operation: {}\n\
            Description: {}\n\
            Risk Level: {:?}\n\
            Requester: {} ({})\n\
            Reversible: {}\n\
            \n\
            Do you approve this operation?",
            request.operation,
            request.description,
            request.risk_level,
            request.requester.name.as_deref().unwrap_or("Unknown"),
            request.requester.id,
            if request.reversible { "Yes" } else { "No" }
        );

        let context = format!(
            "Approval Type: {:?}\n\
            Affected Systems: {}\n\
            Data Sensitivity: {:?}",
            request.approval_type,
            request.impact.affected_systems.join(", "),
            request.impact.data_sensitivity
        );

        let input_request = InputRequest {
            id: Uuid::new_v4().to_string(),
            input_type: InputType::Boolean,
            prompt,
            context: Some(context),
            choices: None,
            default_value: Some(serde_json::Value::Bool(false)),
            required: true,
            timeout: request.timeout.or(self.metadata.default_timeout),
            priority: match request.risk_level {
                RiskLevel::Low => InputPriority::Normal,
                RiskLevel::Medium => InputPriority::Normal,
                RiskLevel::High => InputPriority::High,
                RiskLevel::Critical => InputPriority::Critical,
            },
            metadata: HashMap::new(),
        };

        let input_response = self.input_collector.request_input(input_request).await?;

        let approved = if input_response.cancelled || input_response.timed_out {
            false
        } else {
            input_response.value
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        };

        let reason = if input_response.cancelled {
            Some("Request was cancelled".to_string())
        } else if input_response.timed_out {
            Some("Request timed out".to_string())
        } else if approved {
            Some("Approved by human reviewer".to_string())
        } else {
            Some("Denied by human reviewer".to_string())
        };

        Ok(ApprovalResponse {
            request_id: request.id,
            approved,
            reason,
            approver: ApproverInfo {
                id: "human_approver".to_string(),
                name: Some("Human Approver".to_string()),
                role: Some("Reviewer".to_string()),
                authority_level: AuthorityLevel::Supervisor,
            },
            timestamp: SystemTime::now(),
            conditions: Vec::new(),
            expires_at: None,
        })
    }

    async fn is_available(&self) -> bool {
        self.input_collector.is_available().await
    }

    fn metadata(&self) -> &ApprovalSystemMetadata {
        &self.metadata
    }
}

/// Approval manager for coordinating multiple approval systems
#[derive(Debug)]
pub struct ApprovalManager {
    /// Registered approval systems
    systems: HashMap<String, Box<dyn ApprovalSystem>>,
    /// Default system ID
    default_system: Option<String>,
    /// Approval policies
    policies: Vec<ApprovalPolicy>,
}

/// Approval policy for determining which system to use
#[derive(Debug, Clone)]
pub struct ApprovalPolicy {
    /// Policy name
    pub name: String,
    /// Conditions for applying this policy
    pub conditions: PolicyConditions,
    /// Required approval system
    pub required_system: String,
    /// Whether multiple approvals are required
    pub multiple_approvals: bool,
}

/// Conditions for applying an approval policy
#[derive(Debug, Clone)]
pub struct PolicyConditions {
    /// Approval types this policy applies to
    pub approval_types: Option<Vec<ApprovalType>>,
    /// Minimum risk level
    pub min_risk_level: Option<RiskLevel>,
    /// Required data sensitivity level
    pub data_sensitivity: Option<DataSensitivity>,
    /// Affected systems
    pub affected_systems: Option<Vec<String>>,
}

impl ApprovalManager {
    /// Create a new approval manager
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
            default_system: None,
            policies: Vec::new(),
        }
    }

    /// Register an approval system
    pub fn register_system(&mut self, system: Box<dyn ApprovalSystem>) {
        let id = system.metadata().id.clone();
        self.systems.insert(id.clone(), system);
        
        // Set as default if it's the first system
        if self.default_system.is_none() {
            self.default_system = Some(id);
        }
    }

    /// Add an approval policy
    pub fn add_policy(&mut self, policy: ApprovalPolicy) {
        self.policies.push(policy);
    }

    /// Request approval using the appropriate system
    pub async fn request_approval(&self, request: ApprovalRequest) -> CoreResult<ApprovalResponse> {
        // Find the appropriate system based on policies
        let system_id = self.find_system_for_request(&request)?;
        
        let system = self.systems.get(&system_id)
            .ok_or_else(|| CoreError::configuration_error(format!(
                "Approval system not found: {}",
                system_id
            )))?;

        // Check if system is available
        if !system.is_available().await {
            return Err(CoreError::execution_error(format!(
                "Approval system {} is not available",
                system_id
            )));
        }

        system.request_approval(request).await
    }

    /// Find the appropriate system for a request
    fn find_system_for_request(&self, request: &ApprovalRequest) -> CoreResult<String> {
        // Check policies in order
        for policy in &self.policies {
            if self.policy_matches(policy, request) {
                return Ok(policy.required_system.clone());
            }
        }

        // Fall back to default system
        self.default_system.clone()
            .ok_or_else(|| CoreError::configuration_error("No default approval system set"))
    }

    /// Check if a policy matches a request
    fn policy_matches(&self, policy: &PolicyConditions, request: &ApprovalRequest) -> bool {
        // Check approval types
        if let Some(ref types) = policy.approval_types {
            if !types.contains(&request.approval_type) {
                return false;
            }
        }

        // Check risk level
        if let Some(min_risk) = &policy.min_risk_level {
            if request.risk_level < *min_risk {
                return false;
            }
        }

        // Check data sensitivity
        if let Some(sensitivity) = &policy.data_sensitivity {
            if request.impact.data_sensitivity < *sensitivity {
                return false;
            }
        }

        // Check affected systems
        if let Some(ref systems) = policy.affected_systems {
            if !request.impact.affected_systems.iter().any(|s| systems.contains(s)) {
                return false;
            }
        }

        true
    }

    /// List available approval systems
    pub fn list_systems(&self) -> Vec<String> {
        self.systems.keys().cloned().collect()
    }
}

impl Default for ApprovalManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ApprovalRequest {
    /// Create a new approval request
    pub fn new(
        approval_type: ApprovalType,
        operation: String,
        description: String,
        risk_level: RiskLevel,
        requester_id: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            approval_type,
            operation,
            description,
            risk_level,
            requester: RequesterInfo {
                id: requester_id,
                name: None,
                role: None,
                timestamp: SystemTime::now(),
            },
            context: HashMap::new(),
            timeout: None,
            reversible: true,
            impact: OperationImpact {
                affected_systems: Vec::new(),
                estimated_duration: None,
                data_sensitivity: DataSensitivity::Internal,
                compliance_requirements: Vec::new(),
            },
        }
    }

    /// Set the requester information
    pub fn with_requester(mut self, name: Option<String>, role: Option<String>) -> Self {
        self.requester.name = name;
        self.requester.role = role;
        self
    }

    /// Set the operation impact
    pub fn with_impact(mut self, impact: OperationImpact) -> Self {
        self.impact = impact;
        self
    }

    /// Set whether the operation is reversible
    pub fn with_reversible(mut self, reversible: bool) -> Self {
        self.reversible = reversible;
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}