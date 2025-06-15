// Approval workflow system for human-in-the-loop operations

use super::traits::{HumanResult, InteractionError, HumanInteraction};
use super::{HumanContext, HumanStats};
// use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// Request for human approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Unique request ID
    pub request_id: String,
    /// Title of the approval request
    pub title: String,
    /// Detailed description of what needs approval
    pub description: String,
    /// Risk level of the operation
    pub risk_level: RiskLevel,
    /// Data related to the approval
    pub data: HashMap<String, serde_json::Value>,
    /// Required approvers
    pub required_approvers: Vec<String>,
    /// Minimum number of approvals needed
    pub min_approvals: u32,
    /// Expiration time for the request
    pub expires_at: Option<SystemTime>,
    /// Context for the approval
    pub context: HumanContext,
}

impl ApprovalRequest {
    /// Create a new approval request
    pub fn new(
        request_id: String,
        title: String,
        description: String,
        context: HumanContext,
    ) -> Self {
        Self {
            request_id,
            title,
            description,
            risk_level: RiskLevel::Medium,
            data: HashMap::new(),
            required_approvers: Vec::new(),
            min_approvals: 1,
            expires_at: None,
            context,
        }
    }
    
    /// Set risk level
    pub fn with_risk_level(mut self, risk_level: RiskLevel) -> Self {
        self.risk_level = risk_level;
        self
    }
    
    /// Add required approver
    pub fn with_approver(mut self, approver_id: String) -> Self {
        self.required_approvers.push(approver_id);
        self
    }
    
    /// Set minimum approvals needed
    pub fn with_min_approvals(mut self, min_approvals: u32) -> Self {
        self.min_approvals = min_approvals;
        self
    }
    
    /// Set expiration time
    pub fn with_expiration(mut self, duration: Duration) -> Self {
        self.expires_at = Some(SystemTime::now() + duration);
        self
    }
    
    /// Add data to the request
    pub fn with_data(mut self, key: String, value: serde_json::Value) -> Self {
        self.data.insert(key, value);
        self
    }
    
    /// Check if the request has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            SystemTime::now() > expires_at
        } else {
            false
        }
    }
}

/// Response to an approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResponse {
    /// Request ID this response is for
    pub request_id: String,
    /// Approver ID
    pub approver_id: String,
    /// Approval decision
    pub decision: ApprovalDecision,
    /// Comments from the approver
    pub comments: Option<String>,
    /// Timestamp of the response
    pub timestamp: SystemTime,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ApprovalResponse {
    /// Create a new approval response
    pub fn new(
        request_id: String,
        approver_id: String,
        decision: ApprovalDecision,
    ) -> Self {
        Self {
            request_id,
            approver_id,
            decision,
            comments: None,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add comments
    pub fn with_comments(mut self, comments: String) -> Self {
        self.comments = Some(comments);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Approval decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalDecision {
    /// Approved
    Approved,
    /// Rejected
    Rejected,
    /// Needs more information
    NeedsInfo,
    /// Delegated to another approver
    Delegated,
}

/// Risk level for approval requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk operation
    Low,
    /// Medium risk operation
    Medium,
    /// High risk operation
    High,
    /// Critical risk operation
    Critical,
}

impl RiskLevel {
    /// Get the default timeout for this risk level
    pub fn default_timeout(&self) -> Duration {
        match self {
            RiskLevel::Low => Duration::from_secs(300),      // 5 minutes
            RiskLevel::Medium => Duration::from_secs(1800),  // 30 minutes
            RiskLevel::High => Duration::from_secs(3600),    // 1 hour
            RiskLevel::Critical => Duration::from_secs(7200), // 2 hours
        }
    }
    
    /// Get the minimum approvals required for this risk level
    pub fn min_approvals(&self) -> u32 {
        match self {
            RiskLevel::Low => 1,
            RiskLevel::Medium => 1,
            RiskLevel::High => 2,
            RiskLevel::Critical => 3,
        }
    }
}

/// Policy for approval workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalPolicy {
    /// Policy name
    pub name: String,
    /// Risk level this policy applies to
    pub risk_level: RiskLevel,
    /// Required approvers for this policy
    pub required_approvers: Vec<String>,
    /// Minimum number of approvals
    pub min_approvals: u32,
    /// Maximum time to wait for approval
    pub timeout: Duration,
    /// Whether to allow self-approval
    pub allow_self_approval: bool,
    /// Whether to require unanimous approval
    pub require_unanimous: bool,
}

impl ApprovalPolicy {
    /// Create a new approval policy
    pub fn new(name: String, risk_level: RiskLevel) -> Self {
        Self {
            name,
            risk_level,
            required_approvers: Vec::new(),
            min_approvals: risk_level.min_approvals(),
            timeout: risk_level.default_timeout(),
            allow_self_approval: false,
            require_unanimous: false,
        }
    }
    
    /// Add required approver
    pub fn with_approver(mut self, approver_id: String) -> Self {
        self.required_approvers.push(approver_id);
        self
    }
    
    /// Set minimum approvals
    pub fn with_min_approvals(mut self, min_approvals: u32) -> Self {
        self.min_approvals = min_approvals;
        self
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Allow self-approval
    pub fn allow_self_approval(mut self) -> Self {
        self.allow_self_approval = true;
        self
    }
    
    /// Require unanimous approval
    pub fn require_unanimous(mut self) -> Self {
        self.require_unanimous = true;
        self
    }
}

/// State of an approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalState {
    /// The approval request
    pub request: ApprovalRequest,
    /// Responses received so far
    pub responses: Vec<ApprovalResponse>,
    /// Current status
    pub status: ApprovalStatus,
    /// Applied policy
    pub policy: Option<ApprovalPolicy>,
}

impl ApprovalState {
    /// Create a new approval state
    pub fn new(request: ApprovalRequest) -> Self {
        Self {
            request,
            responses: Vec::new(),
            status: ApprovalStatus::Pending,
            policy: None,
        }
    }
    
    /// Add a response
    pub fn add_response(&mut self, response: ApprovalResponse) {
        self.responses.push(response);
        self.update_status();
    }
    
    /// Update the status based on current responses
    fn update_status(&mut self) {
        if self.request.is_expired() {
            self.status = ApprovalStatus::Expired;
            return;
        }
        
        let approvals = self.responses.iter()
            .filter(|r| r.decision == ApprovalDecision::Approved)
            .count() as u32;
        
        let rejections = self.responses.iter()
            .filter(|r| r.decision == ApprovalDecision::Rejected)
            .count();
        
        // Check for rejections
        if rejections > 0 {
            self.status = ApprovalStatus::Rejected;
            return;
        }
        
        // Check if we have enough approvals
        if approvals >= self.request.min_approvals {
            // If unanimous is required, check all required approvers have responded
            if let Some(policy) = &self.policy {
                if policy.require_unanimous {
                    let responded_approvers: std::collections::HashSet<_> = self.responses
                        .iter()
                        .map(|r| &r.approver_id)
                        .collect();
                    
                    let required_approvers: std::collections::HashSet<_> = self.request
                        .required_approvers
                        .iter()
                        .collect();
                    
                    if responded_approvers.len() == required_approvers.len() &&
                       responded_approvers.iter().all(|id| required_approvers.contains(id)) {
                        self.status = ApprovalStatus::Approved;
                    }
                } else {
                    self.status = ApprovalStatus::Approved;
                }
            } else {
                self.status = ApprovalStatus::Approved;
            }
        }
    }
    
    /// Get approval progress
    pub fn progress(&self) -> (u32, u32) {
        let approvals = self.responses.iter()
            .filter(|r| r.decision == ApprovalDecision::Approved)
            .count() as u32;
        (approvals, self.request.min_approvals)
    }
}

/// Status of an approval request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    /// Waiting for approval
    Pending,
    /// Approved
    Approved,
    /// Rejected
    Rejected,
    /// Expired
    Expired,
    /// Cancelled
    Cancelled,
}

/// Manager for approval workflows
#[derive(Debug)]
pub struct ApprovalManager {
    /// Active approval requests
    approvals: Arc<Mutex<HashMap<String, ApprovalState>>>,
    /// Approval policies
    policies: Arc<Mutex<HashMap<RiskLevel, ApprovalPolicy>>>,
    /// Human interaction provider
    interaction_provider: Arc<dyn HumanInteraction>,
    /// Statistics
    stats: Arc<Mutex<HumanStats>>,
}

impl ApprovalManager {
    /// Create a new approval manager
    pub fn new(interaction_provider: Arc<dyn HumanInteraction>) -> Self {
        Self {
            approvals: Arc::new(Mutex::new(HashMap::new())),
            policies: Arc::new(Mutex::new(HashMap::new())),
            interaction_provider,
            stats: Arc::new(Mutex::new(HumanStats::default())),
        }
    }
    
    /// Register an approval policy
    pub fn register_policy(&self, policy: ApprovalPolicy) -> HumanResult<()> {
        let mut policies = self.policies.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire policies lock".to_string(),
            }
        })?;
        
        policies.insert(policy.risk_level, policy);
        Ok(())
    }
    
    /// Submit an approval request
    pub async fn submit_request(&self, mut request: ApprovalRequest) -> HumanResult<String> {
        // Apply policy if available
        let policy = {
            let policies = self.policies.lock().map_err(|_| {
                InteractionError::SystemError {
                    message: "Failed to acquire policies lock".to_string(),
                }
            })?;
            policies.get(&request.risk_level).cloned()
        };
        
        if let Some(policy) = &policy {
            // Apply policy settings
            if request.required_approvers.is_empty() {
                request.required_approvers = policy.required_approvers.clone();
            }
            if request.min_approvals == 1 {
                request.min_approvals = policy.min_approvals;
            }
            if request.expires_at.is_none() {
                request.expires_at = Some(SystemTime::now() + policy.timeout);
            }
        }
        
        let mut state = ApprovalState::new(request.clone());
        state.policy = policy;
        
        // Store the approval request
        let mut approvals = self.approvals.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire approvals lock".to_string(),
            }
        })?;
        
        approvals.insert(request.request_id.clone(), state);
        
        // Update statistics
        let mut stats = self.stats.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire stats lock".to_string(),
            }
        })?;
        stats.total_interactions += 1;
        
        Ok(request.request_id)
    }
    
    /// Submit an approval response
    pub fn submit_response(&self, response: ApprovalResponse) -> HumanResult<ApprovalStatus> {
        let mut approvals = self.approvals.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire approvals lock".to_string(),
            }
        })?;
        
        let state = approvals.get_mut(&response.request_id)
            .ok_or_else(|| InteractionError::ValidationError {
                message: format!("Approval request not found: {}", response.request_id),
            })?;
        
        // Check if request has expired
        if state.request.is_expired() {
            state.status = ApprovalStatus::Expired;
            return Ok(ApprovalStatus::Expired);
        }
        
        // Check if approver is authorized
        if !state.request.required_approvers.is_empty() &&
           !state.request.required_approvers.contains(&response.approver_id) {
            return Err(InteractionError::PermissionError {
                message: format!("User {} is not authorized to approve this request", response.approver_id),
            });
        }
        
        // Check for duplicate response from same approver
        if state.responses.iter().any(|r| r.approver_id == response.approver_id) {
            return Err(InteractionError::ValidationError {
                message: format!("User {} has already responded to this request", response.approver_id),
            });
        }
        
        state.add_response(response);
        
        // Update statistics
        let mut stats = self.stats.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire stats lock".to_string(),
            }
        })?;
        
        match state.status {
            ApprovalStatus::Approved => {
                stats.successful_interactions += 1;
            }
            ApprovalStatus::Rejected => {
                // Count as successful interaction (human responded)
                stats.successful_interactions += 1;
            }
            _ => {} // Still pending
        }
        
        Ok(state.status)
    }
    
    /// Get approval status
    pub fn get_status(&self, request_id: &str) -> HumanResult<ApprovalStatus> {
        let approvals = self.approvals.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire approvals lock".to_string(),
            }
        })?;
        
        let state = approvals.get(request_id)
            .ok_or_else(|| InteractionError::ValidationError {
                message: format!("Approval request not found: {}", request_id),
            })?;
        
        Ok(state.status)
    }
    
    /// List pending approvals for a user
    pub fn list_pending_for_user(&self, user_id: &str) -> HumanResult<Vec<ApprovalRequest>> {
        let approvals = self.approvals.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire approvals lock".to_string(),
            }
        })?;
        
        let pending: Vec<ApprovalRequest> = approvals
            .values()
            .filter(|state| {
                state.status == ApprovalStatus::Pending &&
                (state.request.required_approvers.is_empty() || 
                 state.request.required_approvers.contains(&user_id.to_string()))
            })
            .map(|state| state.request.clone())
            .collect();
        
        Ok(pending)
    }
    
    /// Cancel an approval request
    pub fn cancel_request(&self, request_id: &str) -> HumanResult<()> {
        let mut approvals = self.approvals.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire approvals lock".to_string(),
            }
        })?;
        
        let state = approvals.get_mut(request_id)
            .ok_or_else(|| InteractionError::ValidationError {
                message: format!("Approval request not found: {}", request_id),
            })?;
        
        state.status = ApprovalStatus::Cancelled;
        Ok(())
    }
    
    /// Clean up expired requests
    pub fn cleanup_expired(&self) -> HumanResult<u32> {
        let mut approvals = self.approvals.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire approvals lock".to_string(),
            }
        })?;
        
        let mut expired_count = 0;
        for state in approvals.values_mut() {
            if state.request.is_expired() && state.status == ApprovalStatus::Pending {
                state.status = ApprovalStatus::Expired;
                expired_count += 1;
            }
        }
        
        // Update statistics
        if expired_count > 0 {
            let mut stats = self.stats.lock().map_err(|_| {
                InteractionError::SystemError {
                    message: "Failed to acquire stats lock".to_string(),
                }
            })?;
            stats.timeout_interactions += expired_count as u64;
        }
        
        Ok(expired_count)
    }
    
    /// Get approval statistics
    pub fn get_stats(&self) -> HumanResult<HumanStats> {
        let stats = self.stats.lock().map_err(|_| {
            InteractionError::SystemError {
                message: "Failed to acquire stats lock".to_string(),
            }
        })?;
        
        Ok(stats.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::human::input::ConsoleInteraction;

    #[test]
    fn test_approval_request_creation() {
        let context = HumanContext::new("test_approval".to_string());
        let request = ApprovalRequest::new(
            "req_1".to_string(),
            "Delete Database".to_string(),
            "This will permanently delete the production database".to_string(),
            context,
        )
        .with_risk_level(RiskLevel::Critical)
        .with_approver("admin1".to_string())
        .with_approver("admin2".to_string())
        .with_min_approvals(2);
        
        assert_eq!(request.title, "Delete Database");
        assert_eq!(request.risk_level, RiskLevel::Critical);
        assert_eq!(request.required_approvers.len(), 2);
        assert_eq!(request.min_approvals, 2);
    }

    #[test]
    fn test_approval_policy() {
        let policy = ApprovalPolicy::new("critical_ops".to_string(), RiskLevel::Critical)
            .with_approver("admin1".to_string())
            .with_approver("admin2".to_string())
            .with_min_approvals(2)
            .require_unanimous();
        
        assert_eq!(policy.risk_level, RiskLevel::Critical);
        assert_eq!(policy.min_approvals, 2);
        assert!(policy.require_unanimous);
    }

    #[test]
    fn test_approval_state_updates() {
        let context = HumanContext::new("test_approval".to_string());
        let request = ApprovalRequest::new(
            "req_1".to_string(),
            "Test Operation".to_string(),
            "Test description".to_string(),
            context,
        ).with_min_approvals(2);
        
        let mut state = ApprovalState::new(request);
        assert_eq!(state.status, ApprovalStatus::Pending);
        
        // Add first approval
        let response1 = ApprovalResponse::new(
            "req_1".to_string(),
            "user1".to_string(),
            ApprovalDecision::Approved,
        );
        state.add_response(response1);
        assert_eq!(state.status, ApprovalStatus::Pending); // Still need one more
        
        // Add second approval
        let response2 = ApprovalResponse::new(
            "req_1".to_string(),
            "user2".to_string(),
            ApprovalDecision::Approved,
        );
        state.add_response(response2);
        assert_eq!(state.status, ApprovalStatus::Approved); // Now approved
    }

    #[test]
    fn test_risk_level_defaults() {
        assert_eq!(RiskLevel::Low.min_approvals(), 1);
        assert_eq!(RiskLevel::High.min_approvals(), 2);
        assert_eq!(RiskLevel::Critical.min_approvals(), 3);
        
        assert!(RiskLevel::Low.default_timeout() < RiskLevel::Critical.default_timeout());
    }
}
